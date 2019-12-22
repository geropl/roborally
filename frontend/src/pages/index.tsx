import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { GetGameStateRequest, StartGameRequest, StartGameResponse, GetGameStateResponse, SetProgramInputRequest, SetProgramInputResponse, SetStartPositionRequest, SetStartPositionResponse } from "ts-client/lib/protocol_pb";
import { RoboRallyGameClient } from "ts-client/lib/ProtocolServiceClientPb";
import { BoardView } from "../components/board/board-view";
import { GameState, EGamePhase, ERoundPhase, Position } from "ts-client/lib/gamestate_pb";
import { ProgramSheet } from "../components/program-sheet";
import { ProgramInput, StartPositionInput } from "ts-client/lib/inputs_pb";

interface DashboardState {
    gameState: GameState.AsObject | undefined;
    error: any;
}

export default class Dashboard extends React.Component<{}, DashboardState> {
    protected client: RoboRallyGameClient | undefined;

    constructor() {
        super({});
    }

    render() {
        const state = this.state;
        let programSheets: JSX.Element[] = [];
        let gamePhase = '-';
        let roundPhase = '-';
        let roundId = '-';
        if (state && state.gameState) {
            const rounds = state.gameState.roundsList;
            const round = rounds.length > 0 && rounds[rounds.length - 1];
            const currentState = round && round.state || state.gameState.startState!;

            const board = state.gameState!.initialState!.board!;
            const availableStartPositionIds: number[] = [];
            if (state.gameState!.phase === EGamePhase.PREPARATION) {
                const posToStr = (pos: Position.AsObject): string => `${pos.x}:${pos.y}`;
                const startPositionIds = new Map<string, number>();
                board.tilesList
                    .filter(t => !!t.startPositionId)
                    .forEach(t => startPositionIds.set(posToStr(t.position!), t.startPositionId!.id))
                for (const p of currentState.playersList) {
                    if (!p.inputRequired) {
                        startPositionIds.delete(posToStr(p.robot!.position!));
                    }
                }
                startPositionIds.forEach(id => availableStartPositionIds.push(id));
            }
            
            programSheets = currentState.playersList.map(p => {
                return (
                    <ProgramSheet
                        roundId={round && round.id || -1}
                        gamePhase={state.gameState!.phase}
                        availableStartPositionIds={availableStartPositionIds}
                        player={p}
                        onSendProgramInput={(input) => this.sendProgramInput(input)}
                        onSendStartPosition={(input) => this.sendStartPosition(input)} />
                );
            });

            gamePhase = JSON.stringify(Object.keys(EGamePhase)[state.gameState!.phase]);
            roundPhase = round && JSON.stringify(Object.keys(ERoundPhase)[round.phase]) || roundPhase;
            roundId = round && round.id + "" || roundId;
        }
        const labelStyle = {
            marginLeft: '1em'
        };
        return (
            <div>
                RoboRally!!!
                <div>
                    <BoardView />
                    <input type="button" value="StartGame" onClick={() => this.requestStartGame() } />
                    <input type="button" value="GetGameState" onClick={() => this.requestGameState() } />
                    <label style={labelStyle} id="game-phase">Game Phase: {gamePhase}</label>
                    <label style={labelStyle} id="round-id">Round ID: {roundId}</label>
                    <label style={labelStyle} id="round-phase">Round Phase: {roundPhase}</label>
                    {programSheets}
                    <label id="output">{state && JSON.stringify(state.gameState) || ""}</label>
                </div>
            </div>
        );
    }

    protected onNewGameState(state: GameState | undefined) {
        if (!state) {
            throw new Error("ProtocolError: Expected GameState, got undefined!");
        }

        console.log("received new GameState");
        this.setState({
            error: undefined,
            gameState: state.toObject()
        });
    }

    protected async requestStartGame() {
        const request = new StartGameRequest();

        const client = this.getClient();
        try {
            const response = await new Promise<StartGameResponse>((resolve, reject) => {
                client.startGame(request, null, (err: Error, response: StartGameResponse) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    resolve(response);
                });
                console.log("Sent StartGameRequest");
            });
            this.onNewGameState(response.getState());
        } catch (err) {
            this.onError(err);
        }
    }

    protected async requestGameState() {
        const gameStateRequest = new GetGameStateRequest();

        const client = this.getClient();
        try {
            const response = await new Promise<GetGameStateResponse>((resolve, reject) => {
                client.getGameState(gameStateRequest, null, (err: Error, response: GetGameStateResponse) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    resolve(response);
                });
            });
            this.onNewGameState(response.getState());
        } catch (err) {
            this.onError(err);
        }
    }

    protected async sendProgramInput(input: ProgramInput) {
        const request = new SetProgramInputRequest();
        request.setProgramInput(input);

        const client = this.getClient();
        try {
            const response = await new Promise<SetProgramInputResponse>((resolve, reject) => {
                client.setProgramInput(request, null, (err: Error, response: SetProgramInputResponse) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    resolve(response);
                });
                console.log("Sent SetProgramInputResponse");
            });
            this.onNewGameState(response.getState());
        } catch (err) {
            this.onError(err);
        }
    }

    protected async sendStartPosition(input: StartPositionInput) {
        const request = new SetStartPositionRequest();
        request.setStartPosition(input);

        const client = this.getClient();
        try {
            const response = await new Promise<SetStartPositionResponse>((resolve, reject) => {
                client.setStartPosition(request, null, (err: Error, response: SetStartPositionResponse) => {
                    if (err) {
                        reject(err);
                        return;
                    }
                    resolve(response);
                });
                console.log("Sent SetStartPositionResponse");
            });
            this.onNewGameState(response.getState());
        } catch (err) {
            this.onError(err);
        }
    }

    protected getClient() {
        if (!this.client) {
            const connStr = this.getGitpodConnectionString();
            this.client = new RoboRallyGameClient(connStr);
        }
        return this.client;
    }

    protected onError(err: any) {
        console.error(err);
        this.setState({ error: err });
    }

    protected getGitpodConnectionString(): string {
        const SEPARATOR = "-";
        const url = URL.parse(window.location.href);
        const parts = url.host!.split(SEPARATOR);
        parts[0] = "8080";
        url.pathname = "";
        return `${url.protocol}//${parts.join(SEPARATOR)}`;
    }
}