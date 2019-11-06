import React from "react";
import { BabylonBoardRenderer } from "./babylon-board-renderer";

export class Board extends React.Component<{}, {}> {
    protected canvasRef: React.RefObject<HTMLCanvasElement>;
    protected renderer: BabylonBoardRenderer;

    constructor() {
        super({});
        this.canvasRef = React.createRef();
        this.renderer = new BabylonBoardRenderer(this.canvasRef);
    }

    componentDidMount() {
        this.renderer.init();
        this.renderer.run();
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