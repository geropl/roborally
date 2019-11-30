import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { GetGameStateRequest, GetGameStateResponse, StartGameRequest, StartGameResponse } from "ts-client/lib/protocol_pb";
import { RoboRallyGameClient } from "ts-client/lib/ProtocolServiceClientPb";
import { BoardView } from "../components/board/board-view";
import { MoveCardDebugInput } from "../components/move-card-debug-input";

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
                    <MoveCardDebugInput />
                    <label id="output">{this.state && this.state.response || ""}</label>
                </div>
            </div>
        );
    }

    protected async requestStartGame() {
        const request = new StartGameRequest();

        const client = this.getClient();
        await new Promise<StartGameResponse>((resolve, reject) => {
            client.startGame(request, null, (err: Error, response: StartGameResponse) => {
                if (err) {
                    reject(err);
                    return;
                }
                resolve(response);
            });
            console.log("Sent StartGameRequest");
        });
        console.log("Received StartGameResponse");
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
            console.log("Sent GetGameStateRequest");
        });

        const gameState = response.getState();
        if (!gameState) {
            console.error("No state returned!");
            return;
        }
        const obj = gameState.toObject();
        this.setState({ response: JSON.stringify(obj) });
        console.log("received GetGameStateResponse");
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