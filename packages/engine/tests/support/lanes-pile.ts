// One headless native engine per process: `--bodies N` boxes stacked ten
// layers high over a floor collider, each layer shifted so the columns
// topple into each other, stepped 240 ticks. The lanes are whatever
// OG_LANES gave the process. Prints JSON for lanes.test.ts: the world hash
// and the dump_lanes report (the physics site goes to the pool from
// og-physics's BODY_GRAIN, 250 bodies, for every lane thread).
//
//   OG_LANES=off|<n> bun tests/support/lanes-pile.ts --bodies N

import { og, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";
import { loadNativeHost } from "../../src/node/index.ts";

const argv = process.argv.slice(2);
const bodies = Number(argv[argv.indexOf("--bodies") + 1]);
if (!Number.isInteger(bodies) || bodies < 1) throw new Error("--bodies N");

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

loadNativeHost();
const layers = 10;
const perLayer = Math.ceil(bodies / layers);
const side = Math.ceil(Math.sqrt(perLayer));
const game = og.game({
  seed: 1,
  start(world: World) {
    const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: 0 } }));
    world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 80, y: 1, z: 80 } });
    for (let i = 0; i < bodies; i++) {
      const layer = Math.floor(i / perLayer);
      const k = i % perLayer;
      const shift = (layer % 3) * 0.17;
      const position = { x: ((k % side) - side / 2) * 0.7 + shift, y: 0.3 + layer * 0.62, z: (Math.floor(k / side) - side / 2) * 0.7 - shift };
      const e = must(world.spawn({ name: `box-${i}`, position }));
      world.insert(e, og.RigidBody, {});
      world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.5, y: 0.5, z: 0.5 } });
    }
  },
}) as unknown as GameToken;
const engine = must(createEngine(game));
must(engine.advance(240));
const hash = engine.hash();
engine.probe.commands("0:dump_lanes:lanes.json");
must(engine.advance(1));
const dump = engine.takeOutputs().find((o) => o.file === "lanes.json");
engine.destroy();
console.log(JSON.stringify({ bodies, hash, lanes: dump ? JSON.parse(dump.data) : null }));
