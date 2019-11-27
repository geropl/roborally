import { Game } from './phaser-game-board';


export class PhaserBoardRenderer {
    protected game: Game | undefined;
    
    constructor(protected canvasRef: React.RefObject<HTMLElement>) {}

    init() {
        console.log("renderer: init");

        const canvas = this.canvasRef.current;
        if (!canvas) {
            console.log("Cannot init game, no canvas!");
            return;
        }

        this.game = new Game(canvas);
        console.log("game started");
    }
}