import React from "react";
import { Error } from "grpc-web";
import * as URL from "url";
import { HelloRequest, HelloReply } from "backend-ts-client-lib/lib/proto/protocol_pb";
import { GreeterClient } from "backend-ts-client-lib/lib/proto/ProtocolServiceClientPb";

export default class Dashboard extends React.Component<{},{}> {
    protected helloService: GreeterClient | undefined;

    render() {
        return (
            <div>
                RoboRally!!!
                <div>
                <input type="button" value="send" onClick={() => {
                    const helloRequest = new HelloRequest();
                    helloRequest.setName("mike");

                    const helloService = this.getClient();
                    helloService.sayHello(helloRequest, null, (err: Error, response: HelloReply) => {
                        if (err) {
                            console.error(err);
                            return
                        }
                        console.log("Response: " + response.getMessage());
                    });
                    console.log("Sent hello request");
                }} />
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