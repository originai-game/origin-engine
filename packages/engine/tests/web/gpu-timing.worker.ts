// The engine Worker of tests/web/gpu-timing.ts: the hello world's system,
// and one command, `caps`, that leaves the renderer's capabilities
// (og.render.capabilities) where the harness reads them in this Worker over
// the DevTools protocol.
import { og } from "@origingame/engine";
import { startWorker } from "../../src/web/worker.ts";
import { Move } from "../../examples/hello.ts";

startWorker(
  og.game({
    tick: 60,
    scenes: { main: "scenes/main.ogt" },
    systems: [Move],
    commands: {
      caps: (world) => {
        (globalThis as Record<string, unknown>)["__og_caps__"] = og.render.capabilities(world);
      },
    },
  }),
);
