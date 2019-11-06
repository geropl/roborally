import  * as BABYLON from "babylonjs";

export class BabylonBoardRenderer {
    protected engine: BABYLON.Engine | undefined;
    protected scene: BABYLON.Scene | undefined;

    constructor(protected canvasRef: React.RefObject<HTMLCanvasElement>) {}

    init() {
        const canvas = this.canvasRef.current;
        if (!canvas) {
            throw Error("Cannot access canvas!");
        }
        this.engine = new BABYLON.Engine(canvas, true); // Generate the BABYLON 3D engine
        /******* End of the create scene function ******/

        this.scene = this.createScene(this.engine, canvas); //Call the createScene function
    }

    run() {
        const engine = this.engine;
        const scene = this.scene;
        if (!engine || !scene) {
            throw new Error("initialize before run");
        }

        // Register a render loop to repeatedly render the scene
        engine.runRenderLoop(() => {
            scene.render();
        });

        // Watch for browser/canvas resize events
        window.addEventListener("resize", () => {
            engine.resize();
        });
    }

    protected createScene(engine: BABYLON.Engine, canvas: HTMLCanvasElement) {
        const scene = new BABYLON.Scene(engine);

        // Add a camera to the scene and attach it to the canvas
        const camera = new BABYLON.ArcRotateCamera("Camera", Math.PI / 2, Math.PI / 2, 2, new BABYLON.Vector3(0,0,5), scene);
        camera.attachControl(canvas, true);

        // Add lights to the scene
        new BABYLON.HemisphericLight("light1", new BABYLON.Vector3(1, 1, 0), scene);
        new BABYLON.PointLight("light2", new BABYLON.Vector3(0, 1, -1), scene);

        // Add and manipulate meshes in the scene
        BABYLON.MeshBuilder.CreateSphere("sphere", {diameter:2}, scene);

        return scene;
    }
}