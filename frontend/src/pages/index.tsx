import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { GetGameStateRequest, GetGameStateResponse } from "backend-ts-client-lib/lib/proto/protocol_pb";
import { RoboRallyGameClient } from "backend-ts-client-lib/lib/proto/ProtocolServiceClientPb";
import { BoardView } from "../components/board/board-view";

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
                    <label id="output">{this.state && this.state.response || ""}</label>
                    <input type="button" value="GetGameState" onClick={() => {
                        const gameStateRequest = new GetGameStateRequest();

                        const client = this.getClient();
                        client.getGameState(gameStateRequest, null, (err: Error, response: GetGameStateResponse) => {
                            if (err) {
                                console.error(err);
                                return
                            }
                            const gameStateStr = response.getState().toString();
                            this.setState({ response: gameStateStr });
                            console.log("received GetGameStateResponse");
                        });
                        console.log("Sent GetGameStateRequest");
                    }} />
                    <BoardView />
                </div>
            </div>
        );
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