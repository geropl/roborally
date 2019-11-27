import React from "react";
import { PhaserBoardRenderer } from "./phaser-board-renderer";

export class BoardView extends React.Component<{}, {}> {
    protected canvasRef: React.RefObject<HTMLDivElement>;
    protected renderer: PhaserBoardRenderer;

    constructor() {
        super({});
        this.canvasRef = React.createRef();
        this.renderer = new PhaserBoardRenderer(this.canvasRef);
    }

    componentDidMount() {
        this.renderer.init();
    }

    render() {
        return (
            <div>
                <div id="canvas" ref={this.canvasRef}></div>
            </div>
        );
    }
}