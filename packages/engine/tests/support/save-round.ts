// One host's side of tests/save.test.ts's cross-host case (node: the napi
// addon; web: the wasm module outside a browser; publish: the publish wasm).
// Runs a small game 30 ticks and prints its save and hash; with --restore
// <file>, first runs it 7 ticks the other way (another seed), then restores
// the save in the file and prints the restored world's hash and tick.
//
//   bun tests/support/save-round.ts --host node|web|publish [--restore <file>]

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineComponent, defineResource, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => (argv.includes(name) ? (argv[argv.indexOf(name) + 1] ?? "") : "");
const host = opt("--host");
const restore = opt("--restore");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const dir = host === "publish" ? "web-publish" : "web";
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", dir, "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", dir, "origin-engine_bg.wasm")));
}

const RsPos = defineComponent("RsPos", { x: f32(), v: f32(1) });
const RsRun = defineResource("RsRun", { coins: u32() });
const RsMove = defineSystem("RsMove", { writes: [RsPos, RsRun, og.Rng] }, (q, time, world) => {
  for (const a of q.archetypes()) {
    const p = a.column(RsPos);
    for (let i = 0; i < a.count; i++) p.x[i] = p.x[i]! + p.v[i]! * time.scaledDt;
  }
  const run = world.resource(RsRun);
  const rng = world.resource(og.Rng);
  if (run.ok && rng.ok) run.value.coins += rng.value.integer(0, 5);
});
const spawn = (world: World) => {
  for (let i = 0; i < 4; i++) {
    const e = world.spawn({ name: `rs-${i}`, position: { x: i, y: 0, z: 0 } });
    if (e.ok) world.insert(e.value, RsPos, { x: i, v: 0.5 + i });
  }
};

const created = createEngine(
  og.game({ tick: 60, seed: restore ? 99 : 5, systems: [RsMove], resources: [RsRun], start: spawn }) as unknown as GameToken,
);
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const hash = () => (host === "publish" ? "" : engine.hash());

if (restore) {
  engine.advance(7);
  const r = engine.world.restore(readFileSync(restore, "utf8"));
  console.log(JSON.stringify({ host, save: "", hash: "", restored: { ok: r.ok, reason: r.ok ? "" : r.reason, hash: hash(), tick: engine.world.tick } }));
} else {
  engine.advance(30);
  const s = engine.world.snapshot();
  if (!s.ok) throw new Error(s.reason);
  console.log(JSON.stringify({ host, save: s.value, hash: hash() }));
}
