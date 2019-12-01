import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { GetGameStateRequest, StartGameRequest, StartGameResponse, GetGameStateResponse, SetRoundInputRequest, SetRoundInputResponse } from "ts-client/lib/protocol_pb";
import { RoboRallyGameClient } from "ts-client/lib/ProtocolServiceClientPb";
import { BoardView } from "../components/board/board-view";
import { MoveCardDebugInput, DebugMoveCardState } from "../components/move-card-debug-input";
import { GameState } from "ts-client/lib/gamestate_pb";
import { PlayerInput, MoveCard, ESimpleMove, ESimpleMoveMap } from "ts-client/lib/inputs_pb";

interface DashboardState {
    response: string | undefined,
}

export default class Dashboard extends React.Component<{}, DashboardState> {
    protected client: RoboRallyGameClient | undefined;

    constructor() {
        super({});
    }

    render() {
        return (
            <div>
                RoboRally!!!
                <div>
                    <BoardView />
                    <input type="button" value="StartGame" onClick={() => this.requestStartGame() } />
                    <input type="button" value="GetGameState" onClick={() => this.requestGameState() } />
                    <MoveCardDebugInput onNewDebugInput={(rs) => this.setDebugInput(rs)}/>
                    <label id="output">{this.state && this.state.response || ""}</label>
                </div>
            </div>
        );
    }

    protected onNewGameState(state: GameState | undefined) {
        if (!state) {
            throw new Error("ProtocolError: Expected GameState, got undefined!");
        }

        console.log("received new GameState");
        const obj = state.toObject();
        this.setState({ response: JSON.stringify(obj) });
    }

    protected async requestStartGame() {
        const request = new StartGameRequest();

        const client = this.getClient();
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
    }

    protected async requestGameState() {
        const gameStateRequest = new GetGameStateRequest();

        const client = this.getClient();
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
    }

    protected async setDebugInput(registers: DebugMoveCardState[]) {
        const setInputRequest = new SetRoundInputRequest();
        const playerInput = new PlayerInput();
        playerInput.setPlayerId(0);
        const moveCards = registers.map(r => {
            const moveCard = new MoveCard();
            moveCard.setPriority(r.priority || NaN);    // Provokes error on backend
            const moves: ESimpleMoveMap[keyof ESimpleMoveMap][] = (r.moves || []).map(m => {
                let move = -1;
                switch (m) {
                    case "backward":
                        move = ESimpleMove.BACKWARD;
                        break;
                    case "forward":
                        move = ESimpleMove.FORWARD;
                        break;
                    case "stepleft":
                        move = ESimpleMove.STEPLEFT;
                        break;
                    case "stepright":
                        move = ESimpleMove.STEPRIGHT;
                        break;
                    case "turnleft":
                        move = ESimpleMove.TURNLEFT;
                        break;
                    case "turnright":
                        move = ESimpleMove.TURNRIGHT;
                        break;
                    case "uturn":
                        move = ESimpleMove.UTURN;
                        break;
                }
                return move as ESimpleMoveMap[keyof ESimpleMoveMap];
            })
            moveCard.setMovesList(moves);
            return moveCard;
        })
        playerInput.setMoveCardsList(moveCards);
        setInputRequest.setPlayerInput(playerInput);

        const client = this.getClient();
        const response = await new Promise<SetRoundInputResponse>((resolve, reject) => {
            client.setRoundInput(setInputRequest, null, (err: Error, response: SetRoundInputResponse) => {
                if (err) {
                    reject(err);
                    return;
                }
                resolve(response);
            });
        });
        this.onNewGameState(response.getState());
    }

    protected getClient() {
        if (!this.client) {
            const connStr = this.getGitpodConnectionString();
            this.client = new RoboRallyGameClient(connStr);
        }
        return this.client;
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