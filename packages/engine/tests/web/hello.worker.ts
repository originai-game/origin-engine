// The engine Worker of the browser run: the design's hello world, unchanged.
import { startWorker } from "../../src/web/worker.ts";
import game from "../../examples/hello.ts";

startWorker(game);
