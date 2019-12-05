import React from "react";

export interface MoveCardDebugInputProps {
    playerId: number;
    onNewDebugInput: (playerId: number, registers: Register[]) => void;
}

export interface MoveCardDebugInputState {
    registers: DebugMoveCardProps[];
}

export interface Register {
    priority?: number;
    moves?: string[];
}

interface DebugMoveCardProps extends Register {
    id: number;
    onPriorityChanged: (priority: number | undefined) => void;
    onMovesChanged: (moves: string[] | undefined) => void;
}

export class MoveCardDebugInput extends React.Component<MoveCardDebugInputProps, MoveCardDebugInputState> {

    constructor(props: MoveCardDebugInputProps) {
        super(props);

        const debugMoveCardProps = (index: number): DebugMoveCardProps => {
            return {
                id: index,
                priority: (index + 1) * 10,
                moves: ["forward"],
                onPriorityChanged: (priority: number | undefined) => {
                    this.setState((os) => {
                        os.registers[index].priority = priority;
                        return os;
                    });
                },
                onMovesChanged: (moves: string[] | undefined) => {
                    this.setState((os) => {
                        os.registers[index].moves = moves;
                        return os;
                    });
                }
            };
        };
        this.state = {
            registers: [
                debugMoveCardProps(0),
                debugMoveCardProps(1),
                debugMoveCardProps(2),
                debugMoveCardProps(3),
                debugMoveCardProps(4),
            ]
        };
    }

    render() {
        return (
            <div className="movecardinput">
                {this.state.registers.map(r => <DebugMoveCard
                    key={r.id}
                    id={r.id}
                    priority={r.priority}
                    moves={r.moves}
                    onPriorityChanged={r.onPriorityChanged}
                    onMovesChanged={r.onMovesChanged} />)}
                <input
                    className="send"
                    type="button"
                    value={`Send player ${this.props.playerId}`}
                    onClick={(e) => this.onSendClicked(e)}
                    />
            </div>
        );
    }

    protected onSendClicked(_event: React.MouseEvent<HTMLInputElement>) {
        this.props.onNewDebugInput(this.props.playerId, this.state.registers);
    }
}

class DebugMoveCard extends React.Component<DebugMoveCardProps, {}> {
    render() {
        const errorStyle = {
            color: "red",
        };
        return (
            <div className="movecard">
                <input
                    className="priority"
                    style={this.props.priority === undefined ? errorStyle : undefined}
                    type="text"
                    value={!this.props.priority ? "" : this.props.priority + ""}
                    onChange={(e) => this.onPriorityChange(e)}
                    />
                <input
                    className="moves"
                    style={this.props.moves === undefined ? errorStyle : undefined}
                    type="text"
                    value={!!this.props.moves ? this.props.moves.join(",") : ""}
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
        this.props.onPriorityChanged(priority);
    }

    protected onMovesChange(event: React.ChangeEvent<HTMLInputElement>) {
        const moves = event.target!.value.split(",");
        if (moves.length === 0) {
            this.setState({ moves: undefined });
            return;
        }
        this.props.onMovesChanged(moves);
    }
}