// The game of tests/uncooked-load.test.ts: it loads an image uncooked when
// it starts and branches on og.load.status, so the tick its load lands at
// shows in its world (UncookedProbe) and in every hash after it.
import { og, defineResource, defineSystem, f64, u32, type Asset, type World } from "@origingame/engine";
import { encodePng } from "../../../src/node/png.ts";

// readyAt and failedAt: world.tick + 1 of the first tick that saw it so (0: not yet).
export const Probe = defineResource("UncookedProbe", { asset: f64(), readyAt: u32(), failedAt: u32(), width: u32() });

// A 3x2 image, each texel its own colour.
const rgba = new Uint8Array(3 * 2 * 4).map((_, i) => (i * 37) & 0xff);
export const image = encodePng({ width: 3, height: 2, rgba });

const Watch = defineSystem("Watch", { writes: [Probe] }, (_q, _time, world) => {
  const p = world.resource(Probe);
  if (!p.ok) return;
  const status = og.load.status(world, p.value.asset as Asset);
  if (status === "ready" && p.value.readyAt === 0) {
    p.value.readyAt = world.tick + 1;
    const info = og.load.imageInfo(world, p.value.asset as Asset);
    if (info.ok) p.value.width = info.value.width;
  }
  if (status === "failed" && p.value.failedAt === 0) p.value.failedAt = world.tick + 1;
});

export default og.game({
  tick: 60,
  seed: 3,
  systems: [Watch],
  resources: [Probe],
  start(world: World) {
    const r = og.load.imageData(world, "drawings/cat.png", image, { kind: "pixel" });
    if (!r.ok) throw new Error(r.reason);
    const p = world.resource(Probe);
    if (p.ok) p.value.asset = r.value;
  },
});

