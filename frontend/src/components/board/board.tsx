import React from "react";
import { MelonJSBoardRenderer } from "./melonjs-board-renderer";

export class Board extends React.Component<{}, {}> {
    protected canvasRef: React.RefObject<HTMLCanvasElement>;
    protected renderer: MelonJSBoardRenderer;

    constructor() {
        super({});
        this.canvasRef = React.createRef();
        this.renderer = new MelonJSBoardRenderer(this.canvasRef);
    }

    componentDidMount() {
        this.renderer.init();
    }

    render() {
        return (
            <div>
                <canvas id="canvas" ref={this.canvasRef} width={800} height={600} style={{
                    backgroundColor: '#121212',
                }}/>
            </div>
        );
    }
}