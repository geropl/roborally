import * as Phaser from 'phaser';

type integer = number;

//The two base types of a board cell
enum TileType {
    Floor,
    Hole
}
//Directions for orientation on the board
enum Direction {
    Up = 0,
    Right = 1,
    Down=2,
    
    Left=3
}
enum Orientation {
    Clockwise,
    Counterclockwise
}
function charToDirection(dir: string): Direction
{
    if (dir == "N") return Direction.Up;
    if (dir == "S") return Direction.Down;
    if (dir == "W") return Direction.Left;
    if (dir == "E") return Direction.Right;

    throw Error("Unknown direction");
}
type Coneyortype = "normal" | "speed";
const Coneyortype_NORMAL: Coneyortype = "normal";
// const Coneyortype_SPEED: Coneyortype = "speed";
// enum Coneyortype {
//     normal,
//     speed
// }
enum WallType {
    none,
    wall,
    warn
}
enum static_type {
    rotater,
    repair,
    upgrade,
    playerstart,
    target
}
class Overlay {
    valid: boolean = true;
    /*
     * Baseclass for all overlays of board cells.
     */

};
class Conveyor extends Overlay {
    /*
     *A class representing all types of conveyors or overlays with similar rotation metrics
     *This means this overlay connects 2 or 3 directions and is not symmetric.
     *The first direction is always the output of the conveyor, the other two are the inputs. 
     */
    private input: Direction;
    private output: Direction;
    private merge: Direction | null = null;
    // private triple: boolean;
    private type: Coneyortype = Coneyortype_NORMAL;
    constructor(output: Direction = Direction.Left, input: Direction = Direction.Right, type?: Coneyortype, merge?: Direction) {
        super();
        this.input = input;
        this.output = output;
        if ( merge !=undefined) 
            this.merge = merge;
        this.sort_inputs();
        if (type)
            this.type = type;
    }
    static fromDirections(orientation: string, type: Coneyortype = Coneyortype_NORMAL): Conveyor {
        let res = new Conveyor(Direction.Up, Direction.Down, type);
        res.setOrientation(orientation);
        return res;
    }
    //Convenience function to set orientation from string (like "NS" for north to south etc)
    setOrientation(orientation: string) {
        if (orientation.length >= 2) {
            this.output = charToDirection(orientation[0]);
            this.input = charToDirection(orientation[1]);
        }
        if (orientation.length >= 3) {
            this.merge = charToDirection(orientation[2]);

        }
        else
            this.merge = null;
        this.sort_inputs();
    }
    //checks if the defined orientation is valid. In triple configuration input and output always form the curve!
    private sort_inputs(): void {
        if (this.input == this.merge)
            this.merge = null;
        this.valid = true;
        if (this.input == this.output) this.valid = false;
        if (this.merge != null) {
            // let test =( this.input + 2) % 4;
            if ((this.input+2) %4== this.output) {
                const t: Direction = this.merge;
                this.merge = this.input;
                this.input = t;
            }
        }
    }
    getType(): Coneyortype {
        return this.type;
    }
    isTriple(): boolean {
        return (this.merge != null)
    }
    isCurve(): boolean {
        if (this.isTriple()) return false;
        if ((this.output + 2) % 4 == this.input) return false;
        return true;
    }
    isStraight(): boolean {
        if (!this.isTriple() && !this.isCurve())
            return true;
        return false;
    }
    getInput(): Direction {
        return this.input;
    }
    isAVersion(): boolean {
        if (this.input == Direction.Left && this.output == Direction.Up) return true;
        if ((this.input > this.output) || (this.input == Direction.Up && this.output == Direction.Left))
            return false;
        return true;
    }

};


class Static extends Overlay {
    constructor(
            readonly type: static_type,
            readonly id: number = 0) {
        super();
    }
    getType(): static_type {
        return this.type;
    }
    // getID(): integer {
    //     return this.id;
    // };
}



class BoardTile {
    /*
     * Class that represents one cell of the board, including possible static overlays. All non-static stuff as players, restart markers etc.
     * are not stored here. However player-starts and target-markers are considered static (and therefore cannot be on top of conveyors or other overlays.
     * That would not work with the game-physics anyway)
     */
    type: TileType = TileType.Floor;//< floor type. A solid floor is considered as standard
    overlay: Overlay | null = null;//< an overlay, if the cell has one
    /*
     * The constructor needs a gameboard and a position. The standard position is -1 which renders the tile invalid. Invalid tiles makes it possible
     * to write some algorithms in a way, that they do not need to be aware that the board is bounded
     */
    constructor(
            protected readonly board: Board,
            readonly x: number = -1,
            readonly y: number = -1) {
    }
    getOverlay(): Overlay | null {
        return this.overlay;
    }
    getGround(): TileType {
        return this.type;
    }
    setOverlay(overlay: Overlay) {
        this.overlay = overlay;
    }
    isHole(): boolean {
        if (this.type == TileType.Hole)
            return true;
        return false;
    }
    setHole() {
        this.type = TileType.Hole;
    }
    isValid(): boolean {
        if (this.x >= 0 || this.y >= 0) {
            return true;
        }
        return false;
    }
}  
class Wall {
    // x: integer;
    // y: integer;
    // horizontal: boolean;
    // board: Board;
    type: WallType = WallType.none;
    constructor(
        protected readonly board: Board,
        protected readonly x: number,
        protected readonly y: number,
        protected readonly horizontal: boolean) {
    }
    setWall() {
        this.type = WallType.wall;
    }
    getType(): WallType {
        return this.type;
    }
};
class Board {
    /*
     * Stores the static data of the gameboard and provide functions to access the data in 2D indexing. It also chaches some
     * data that may be used by several renderers
     */
    boardCells: Array<BoardTile>;
    hWalls: Array<Wall>;
    vWalls: Array<Wall>;
    constructor(
            protected readonly width: number,
            protected readonly height: number) {
        this.boardCells = new Array<BoardTile>();
        this.hWalls = new Array<Wall>();
        this.vWalls = new Array<Wall>();
        for (let y: integer = 0; y <= height; y++)
            for (let x: integer = 0; x <= width; x++)
             {
                if (x < width && y < height) {
                    this.boardCells.push(new BoardTile(this, x, y));
                    this.hWalls.push(new Wall(this, x, y, true));
                    this.vWalls.push(new Wall(this, x, y, false));
                }
                else if (x < width) {
                    this.hWalls.push(new Wall(this, x, y, true));
                }
                else if (y < height) {
                    this.vWalls.push(new Wall(this, x, y, false));
                }
            }
    }
    numCells(): number {
        return this.boardCells.length;
    }
    forEachCell(f: (value: BoardTile, index: number, array: BoardTile[]) => void, o?: any) {
        this.boardCells.forEach(f, o);
    }
    forEachWall(f: (value: Wall, index: number, array: Wall[]) => void, o?: any) {
        this.hWalls.forEach(f, o);
        this.vWalls.forEach(f, o);
    }
    getVWall(x: integer, y: integer): Wall {
        if (x < 0 || x > this.width) return new Wall(this, -1, -1, false);
        if (y < 0 || y >= this.height) return new Wall(this, -1, -1, false);
        return this.vWalls[y * (this.width+1)+x]
    }
    getHWall(x: integer, y: integer): Wall {
        if (x < 0 || x >= this.width) return new Wall(this, -1, -1, true);
        if (y < 0 || y > this.height) return new Wall(this, -1, -1, true);
        return this.hWalls[y * (this.width ) + x]
    }
    getCell(x: integer, y: integer): BoardTile {
        if (x < 0 || x >= this.width) return new BoardTile(this);
        if (y < 0 || y >= this.height) return new BoardTile(this);
        return this.boardCells[y * this.width + x];
    }
}
class TileFactory {
    /*
     * Class that constructs the tiles and images ready to use for rendering. Objects can be configured to 
     * handle different rotation schemes for tilesets like symmetric/antimetric objects and map the transformations
     * to useable description systems
     */
    tilesize: number = 150;
    conveyorBasenames: Map<Coneyortype, string> = new Map<Coneyortype, string>();
    staticBasename: Map<static_type, string> = new Map<static_type, string>();
    getConveyorSprite(overlay: Conveyor): [string, number, boolean] | undefined {
        let spritename = this.conveyorBasenames.get(overlay.getType());
        if (!spritename) {
            return undefined;
        }

        let angle: number = Math.PI / 2. * overlay.getInput();
        let flip: boolean = false;
        //Next check which base type is needed
        if (overlay.isCurve()) {
            spritename += "_curve";
            //check which curve is needed
            if (!overlay.isAVersion())
                flip = true;
        }
        else if (overlay.isTriple()) {
            spritename += "_merge";
            if (!overlay.isAVersion())
                flip = true;
        }
        return [spritename, angle, flip];
    }

    defGetStaticSprite(overlay: Static): string | undefined {
        let basename = this.staticBasename.get(overlay.getType())
        if (!basename) {
            return undefined;
        }

        basename += "_";
        basename += String(overlay.id);
        return basename;
    }

    getFloor(data: BoardTile, scene: Phaser.Scene, group: Phaser.GameObjects.Group): RenderedBoardCell {
        //render the floor
        let res = new RenderedBoardCell();
        const posX = data.x * this.tilesize;
        const posY = data.y * this.tilesize;
        let key: string = "Floor";
        if (data.isHole()) {
            key = "Hole";
        }
        let tile = new Phaser.GameObjects.Sprite(scene, posX, posY, key);
        scene.add.existing(tile);
        res.ground = tile;
        group.add(tile);
        //next render the overlay
        const overlay = data.overlay;
        if (overlay) {
            if (overlay instanceof Conveyor) {
                let desc = this.getConveyorSprite(overlay);
                if (desc) {
                    res.overlay = new Phaser.GameObjects.Image(scene, posX, posY, desc[0]);
                    res.overlay.setFlipX(desc[2]);
                    res.overlay.setRotation(desc[1]);
                    group.add(res.overlay);
                    scene.add.existing(res.overlay);
                }
            }
            if (overlay instanceof Static) {
                let img = this.defGetStaticSprite(overlay);
                if (img) {
                    res.overlay = new Phaser.GameObjects.Image(scene, posX, posY, img);
                    group.add(res.overlay);
                    scene.add.existing(res.overlay);
                }
            }
        }

        return res;
    }
    addConveyorBase(type: Coneyortype, resource: string) {
        this.conveyorBasenames.set(type, resource);
    }
    addStaticBase(type: static_type, resource: string) {
        this.staticBasename.set(type, resource);
    }
}
class RenderedBoardCell {
    ground: Phaser.GameObjects.Image | undefined;
    overlay: Phaser.GameObjects.Image | Phaser.GameObjects.Sprite | null=null;
};
export class BoardRenderer {
    /*
     * Class that renders the Gameboard based on a given tileset
     */
    cellTiles: Array<RenderedBoardCell>
    tileLibrary: TileFactory;
    board: Board;
    scene: Phaser.Scene;
    tiles: Phaser.GameObjects.Group;
    constructor(board: Board, tileCreator: TileFactory, scene: Phaser.Scene) {
        this.cellTiles = new Array<RenderedBoardCell>(board.numCells());
        this.board = board;
        this.scene = scene;
        this.tiles = scene.add.group();
        this.tileLibrary = tileCreator;
        this.board.forEachCell((value: BoardTile, index: number) => this.renderCell(value, index), this);
        // this.board.forEachWall(this.renderWall, this);
    }
    // renderWall(wall: Wall, k: number) {

    // }
    renderCell(cell: BoardTile, k: number) {
      
        this.cellTiles[k] = this.tileLibrary.getFloor(cell, this.scene,this.tiles);
    }
}

const MysceneConfig: Phaser.Types.Scenes.SettingsConfig = {
    active: false,
    visible: false,
    key: 'Board',

};

export class NewGameScene extends Phaser.Scene {
    board: any ;
    tiles: any;
    keyW: any;
    keyA: any;
    keyS: any;
    keyD: any;
    keyQ: any;
    keyY: any;
    zoomspeed = 0.1;
    Zoomfactor: integer = 1;
    cameraSpeed: integer = 10;
    artwork: TileFactory | undefined;
    followPoint: any;
    renderer: any;
    constructor() {

        super(MysceneConfig);
        this.board = defineTestBoard();
    }
    public preload() {
        this.load.image("Hole", "/images/Hole.png");
        this.load.image("Floor", "/images/Floor2.png");
        // this.load.image("Wall", "images/wall_base.png");
        // this.load.image("Warn", "images/base_2.png");

        // this.load.image("Warn_connection", "images/cap_conn.png");
        // this.load.image("Warn_down", "images/cap_down.png");
        // this.load.image("Warn_down_left", "images/cap_down_left.png");
        // this.load.image("Warn_down_right", "images/cap_down_right.png");
        // this.load.image("Warn_quad", "images/cap_quad.png");
        // this.load.image("Warn_up", "images/cap_up.png");
        // this.load.image("Warn_up_left", "images/cap_up_left.png");
        // this.load.image("Warn_up_right", "images/cap_up_right.png");


        // this.load.image("Wall_cap", "images/Wall_cap.png");
        // this.load.image("Warn_corner", "images/Wall_cap_down_right.png");
        // this.load.image("Wall_quad", "images/Wall_cap_quad.png");
        // this.load.image("Wall_con", "images/wall_con.png");
        // this.load.image("Wall_trip", "images/Wall_trip.png");

        this.load.image("ConveyorSDT", "/images/conveyorSTD.png");
        this.load.image("ConveyorSDT_curve", "/images/conveyorSTD_curve.png");
        this.load.image("ConveyorSDT_merge", "/images/conveyorSTD_merge.png");

        this.load.image("Rotater_1", "/images/gear_left.png");
        this.load.image("Rotater_0", "/images/gear_right.png");

        // this.load.image("Player1Start", "images/player1_start.png");

        this.followPoint = new Phaser.Math.Vector2(
            0, 0
        );
        this.keyW = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.W);
        this.keyS = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.S);
        this.keyA = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.A);
        this.keyD = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.D);
        this.keyQ = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Q);
        this.keyY = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.Y);
    }
    public create() {
        this.artwork = new TileFactory();
        this.artwork.addConveyorBase(Coneyortype_NORMAL, "ConveyorSDT");
        this.artwork.addStaticBase(static_type.rotater, "Rotater");
        this.renderer = new BoardRenderer(this.board, this.artwork, this);

        //this.cameras.main.setBounds(0, 0, this.tilesize * this.width, this.tilesize * this.height);
    }

    public update() {
        if (this.keyW.isDown) {
            this.followPoint.y -= this.cameraSpeed;
        }
        if (this.keyS.isDown) {
            this.followPoint.y += this.cameraSpeed;
        }
        if (this.keyA.isDown) {
            this.followPoint.x -= this.cameraSpeed;
        }
        if (this.keyD.isDown) {
            this.followPoint.x += this.cameraSpeed;
        }
        if (this.keyQ.isDown) {
            this.Zoomfactor -= this.Zoomfactor * this.zoomspeed;
        }
        if (this.keyY.isDown) {
            this.Zoomfactor += this.Zoomfactor * this.zoomspeed;
        }
        this.cameras.main.centerOn(this.followPoint.x, this.followPoint.y);
        this.cameras.main.setZoom(this.Zoomfactor);
    }
}
export class Game {

    constructor(parent: HTMLElement) {
        const MygameConfig: Phaser.Types.Core.GameConfig = {
            title: 'Sample',
            type: Phaser.AUTO,
            width: 800,
            height: 600,
            scene: NewGameScene,

            physics: {
                default: 'arcade',
                arcade: {
                    debug: true,
                },
            },
            parent,
            backgroundColor: '#000000'
        };
        this.game = new Phaser.Game(MygameConfig);
    }

    game: Phaser.Game;

}

function defineTestBoard():Board
{
    let board: Board = new Board(12, 12);
    board.getCell(0, 0).setHole();
    board.getCell(3, 0).setHole();
    board.getCell(0, 1).setHole();
    board.getCell(5, 0).setHole();
    board.getCell(2, 1).setHole();
    board.getCell(0, 10).setHole();
    board.getCell(11, 2).setHole();
    board.getCell(2, 11).setHole();

    board.getVWall(2, 4).setWall();
    board.getVWall(3, 4).setWall();
    board.getVWall(3, 5).setWall();

    board.getHWall(3,5).setWall();
    board.getVWall(4, 4).setWall();
    board.getVWall(4,5).setWall();
    board.getHWall(4, 4).setWall();
    board.getHWall(5, 4).setWall();

    board.getVWall(5, 4).setWall();
    board.getVWall(5, 5).setWall();
    board.getHWall(4, 6).setWall();
    board.getHWall(5, 6).setWall();

    board.getVWall(7,7).setWall();
    board.getHWall(7,7).setWall();
    board.getVWall(7,6).setWall();
    board.getHWall(6,7).setWall();


    board.getHWall(1, 7).setWall();
    board.getHWall(2, 7).setWall();
    board.getVWall(3, 7).setWall();
    board.getVWall(3, 8).setWall();
   

    board.getHWall(2, 9).setWall();
    board.getHWall(1, 9).setWall();
    board.getVWall(1, 8).setWall();
    board.getVWall(1, 7).setWall();
    board.getHWall(0, 2).setWall();
    board.getHWall(7, 0).setWall();
    board.getHWall(0, 0).setWall();


    board.getCell(8, 8).setOverlay(Conveyor.fromDirections("EW"));
    board.getCell(9, 8).setOverlay(Conveyor.fromDirections("WE"));

    board.getCell(10, 8).setOverlay(Conveyor.fromDirections("NS"));
    board.getCell(10, 9).setOverlay(Conveyor.fromDirections("SN"));

    board.getCell(6, 8).setOverlay(Conveyor.fromDirections("ES"));
    board.getCell(7, 8).setOverlay(Conveyor.fromDirections("SW"));

    board.getCell(7, 9).setOverlay(Conveyor.fromDirections("WN"));
    board.getCell(6, 9).setOverlay(Conveyor.fromDirections("NE"));


    board.getCell(4, 8).setOverlay(Conveyor.fromDirections("SE"));
    board.getCell(5, 8).setOverlay(Conveyor.fromDirections("WS"));
    board.getCell(5, 9).setOverlay(Conveyor.fromDirections("NW"));
    board.getCell(4, 9).setOverlay(Conveyor.fromDirections("EN"));

    board.getCell(2, 2).setOverlay(Conveyor.fromDirections("ES"));
    board.getCell(3, 2).setOverlay(Conveyor.fromDirections("EWN"));
    board.getCell(4, 2).setOverlay(Conveyor.fromDirections("SW"));
    board.getCell(4, 3).setOverlay(Conveyor.fromDirections("SNE"));
    board.getCell(4, 4).setOverlay(Conveyor.fromDirections("WN"));
    board.getCell(3, 4).setOverlay(Conveyor.fromDirections("WES"));
    board.getCell(2, 4).setOverlay(Conveyor.fromDirections("NE"));
    board.getCell(2, 3).setOverlay(Conveyor.fromDirections("NSW"));

    board.getCell(7, 2).setOverlay(Conveyor.fromDirections("SE"));
    board.getCell(8, 2).setOverlay(Conveyor.fromDirections("WEN"));
    board.getCell(9, 2).setOverlay(Conveyor.fromDirections("WS"));
    board.getCell(9, 3).setOverlay(Conveyor.fromDirections("NES"));
    board.getCell(9, 4).setOverlay(Conveyor.fromDirections("NW"));
    board.getCell(8, 4).setOverlay(Conveyor.fromDirections("EWS"));
    board.getCell(7, 4).setOverlay(Conveyor.fromDirections("EN"));
    board.getCell(7, 3).setOverlay(Conveyor.fromDirections("SNW"));
    board.getCell(1, 2).setOverlay(new Static(static_type.rotater, Orientation.Clockwise));
    board.getCell(1, 3).setOverlay(new Static(static_type.rotater, Orientation.Counterclockwise));
    return board;
}