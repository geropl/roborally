import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { HelloRequest, HelloReply } from "backend-ts-client-lib/lib/proto/protocol_pb";
import { GreeterClient } from "backend-ts-client-lib/lib/proto/ProtocolServiceClientPb";
import { Board } from "../components/board/board";

interface DashboardState {
    response: string | undefined,
}

export default class Dashboard extends React.Component<{}, DashboardState> {
    protected helloService: GreeterClient | undefined;

    constructor() {
        super({});
    }

    render() {
        return (
            <div>
                RoboRally!!!
                <div>
                    <label id="output">{this.state && this.state.response || ""}</label>
                    <input type="button" value="send" onClick={() => {
                        const helloRequest = new HelloRequest();
                        helloRequest.setName("mike");

                        const helloService = this.getClient();
                        helloService.sayHello(helloRequest, null, (err: Error, response: HelloReply) => {
                            if (err) {
                                console.error(err);
                                return
                            }
                            this.setState({ response: response.getMessage() });
                            console.log("received response");
                        });
                        console.log("Sent hello request");
                    }} />
                    <Board />
                </div>
            </div>
        );
    }

    protected getClient(): GreeterClient {
        if (!this.helloService) {
            const connStr = this.getGitpodConnectionString();
            this.helloService = new GreeterClient(connStr);
        }
        return this.helloService;
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