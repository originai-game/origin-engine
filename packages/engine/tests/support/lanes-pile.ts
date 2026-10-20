// One headless native engine per process: the pile of
// tests/support/lanes-pile-scene.ts (`--bodies N` boxes stacked ten layers
// high over a floor collider), stepped 240 ticks. The lanes are whatever
// OG_LANES gave the process. Prints JSON for lanes.test.ts: the world hash
// and the dump_lanes report (the physics site goes to the pool from
// og-physics's `pool_bodies` for the pool's thread count: 500 bodies on
// four lanes, 1,500 on eight).
//
//   OG_LANES=off|<n> bun tests/support/lanes-pile.ts --bodies N

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";
import { loadNativeHost } from "../../src/node/index.ts";
import { pile } from "./lanes-pile-scene.ts";

const argv = process.argv.slice(2);
const bodies = Number(argv[argv.indexOf("--bodies") + 1]);
if (!Number.isInteger(bodies) || bodies < 1) throw new Error("--bodies N");

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

loadNativeHost();
const game = og.game({
  seed: 1,
  start(world: World) {
    pile(world, bodies);
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
