import React from "react";

export class MoveCardDebugInput extends React.Component<{}, {}> {

    render() {
        return (
            <div className="movecardinput">
                <DebugMoveCard />
                <DebugMoveCard />
                <DebugMoveCard />
                <input
                    className="send"
                    type="button"
                    value="Send"
                    />
            </div>
        );
    }
}

interface DebugMoveCardState {
    priority?: number;
    moves?: string[];
}
const DEFAULT_STATE: DebugMoveCardState = {
    priority: 20,
    moves: ["left", "forward", "uturn"],
};

class DebugMoveCard extends React.Component<{}, DebugMoveCardState> {
    constructor() {
        super({});
        this.state = DEFAULT_STATE;
    }

    render() {
        const errorStyle = {
            color: "red",
        };
        return (
            <div className="movecard">
                <input
                    className="priority"
                    style={this.state.priority === undefined ? errorStyle : undefined}
                    type="text"
                    value={!this.state.priority ? "" : this.state.priority + ""}
                    onChange={(e) => this.onPriorityChange(e)}
                    />
                <input
                    className="moves"
                    style={this.state.moves === undefined ? errorStyle : undefined}
                    type="text"
                    value={!!this.state.moves ? this.state.moves.join(",") : ""}
                    onChange={(e) => this.onMovesChange(e)}
                    />
            </div>
        );
    }

    protected onPriorityChange(event: React.ChangeEvent<HTMLInputElement>) {
        let priority = undefined;
        try {
            priority = Number.parseInt(event.target!.value);
            if (priority < 0) {
                priority = undefined;
            }
        } catch (e) {
            // Nothing
        }
        this.setState({ priority });
    }

    protected onMovesChange(event: React.ChangeEvent<HTMLInputElement>) {
        const moves = event.target!.value.split(",");
        if (moves.length === 0) {
            this.setState({ moves: undefined });
            return;
        }
        this.setState({ moves });   // TODO check values!
    }
}