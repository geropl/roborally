import { Board } from 'ts-client/lib/gamestate_pb';
import { Game } from './phaser-game-board';


export class PhaserBoardRenderer {
    protected game: Game | undefined;
    
    constructor(protected canvasRef: React.RefObject<HTMLElement>) {}

    init(board: Board.AsObject) {
        console.log("renderer: init");

        const canvas = this.canvasRef.current;
        if (!canvas) {
            console.log("Cannot init game, no canvas!");
            return;
        }

        this.game = new Game(canvas, board);
        console.log("game started");
    }
}