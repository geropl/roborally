import React from "react";
import { Board } from "ts-client/lib/gamestate_pb";
import { PhaserBoardRenderer } from "./phaser-board-renderer";

export interface BoardViewProps {
    board: Board.AsObject;
}
export class BoardView extends React.Component<BoardViewProps, {}> {
    protected canvasRef: React.RefObject<HTMLDivElement>;
    protected renderer: PhaserBoardRenderer;

    constructor() {
        super({ board: {} as Board.AsObject });
        this.canvasRef = React.createRef();
        this.renderer = new PhaserBoardRenderer(this.canvasRef);
    }

    componentDidMount() {
        this.renderer.init(this.props.board);
    }

    render() {
        return (
            <div>
                <div id="canvas" ref={this.canvasRef}></div>
            </div>
        );
    }
}