// The demolition yard as `og run` plays it (examples/demolition-yard's own
// game and commands, on the native host): its beats at their times, then
// each end's pier height in standing brickwork (chunks within 10 cm of
// where they stood before the charge) and the pillar's stump.
//
//   bun tests/support/yard-piers.ts --project <examples/demolition-yard>
import { copyFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const project = argv[argv.indexOf("--project") + 1]!;
const { loadNativeHost } = await import("../../src/node/index.ts");
loadNativeHost();
// The game's own module, imported from beside this script so that its
// "@origingame/engine" is this checkout's package, as og run's is.
const here = dirname(fileURLToPath(import.meta.url));
const copy = join(here, ".yard-main.ts");
copyFileSync(join(project, "src", "main.ts"), copy);
const game = await import(copy).then((m) => m.default as GameToken).finally(() => rmSync(copy, { force: true }));
const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const engine = must(createEngine(game, { root: project }));

type V = { x: number; y: number; z: number };
type Q = { x: number; y: number; z: number; w: number };
const rotate = (q: Q, v: V): V => {
  const u = { x: q.x, y: q.y, z: q.z };
  const cross = (a: V, b: V): V => ({ x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x });
  const t = cross(u, v);
  const t2 = { x: 2 * t.x, y: 2 * t.y, z: 2 * t.z };
  const c = cross(u, t2);
  return { x: v.x + q.w * t2.x + c.x, y: v.y + q.w * t2.y + c.y, z: v.z + q.w * t2.z + c.z };
};
/** Every chunk of `prop` by its index, at its place in the world. */
function chunks(world: World, prop: number): Map<number, V> {
  const out = new Map<number, V>();
  const near: readonly Entity[] = must(og.physics.overlap(world, { shape: { shape: 0, size: { x: 30, y: 10, z: 20 } }, position: { x: 1, y: 4, z: 0 } } as never));
  for (const e of near) {
    const c = world.get(e as never, og.Chunk as never) as { ok: boolean; value?: { destructible: number; chunk: number } };
    if (!c.ok || c.value!.destructible !== prop) continue;
    const t = must(world.get(e as never, og.Transform as never)) as { position: V };
    let p = t.position;
    const parent = world.get(e as never, og.Parent as never) as { ok: boolean; value?: { entity: number } };
    if (parent.ok) {
      const b = must(world.get(parent.value!.entity as never, og.Transform as never)) as { position: V; quaternion: Q };
      const r = rotate(b.quaternion, p);
      p = { x: b.position.x + r.x, y: b.position.y + r.y, z: b.position.z + r.z };
    }
    out.set(c.value!.chunk, p);
  }
  return out;
}

const w = engine.world;
const wall = w.find("wall")!;
const pillar = w.find("pillar")!;
must(engine.advance(55));
const wallBefore = chunks(w, wall);
const pillarBefore = chunks(w, pillar);
// og run's beats (the example's header): boom at 1 s, strike 2.6, drop 3, throw 3.1, kick 3.2.
const beats: Array<[number, string]> = [[60, "boom"], [156, "strike"], [180, "drop"], [186, "throw"], [192, "kick"]];
let at = 55;
for (const [tick, name] of beats) {
  must(engine.advance(tick - at));
  must(engine.command(name, []) as never);
  at = tick;
}
must(engine.advance(390 - at));
const standing = (before: Map<number, V>, now: Map<number, V>) =>
  [...now].filter(([c, p]) => { const b = before.get(c); return b && Math.hypot(p.x - b.x, p.y - b.y, p.z - b.z) < 0.1; }).map(([, p]) => p);
const wallNow = standing(wallBefore, chunks(w, wall));
const wx = -3;
const pier = (side: number) => wallNow.filter((p) => (p.x - wx) * side > 0.9 && (p.x - wx) * side < 2.6).reduce((m, p) => Math.max(m, p.y), 0);
// What spans the hole: a block high over its middle with air under it
// (one on the rubble heap rests on the pieces under it).
const all = [...chunks(w, wall).values()];
const spans = all.filter((p) => Math.abs(p.x - wx) < 0.25 && p.y > 1.5 && all.filter((q) => Math.abs(q.x - p.x) < 0.3 && q.y < p.y - 0.05).length < 3).length;
const stump = standing(pillarBefore, chunks(w, pillar)).reduce((m, p) => Math.max(m, p.y), 0);
const pillarNow = [...chunks(w, pillar).values()];
// The pillar's tallest standing piece and, of what lies, the farthest from its axis.
const PX = 1.2, PZ = -0.4;
const lying = pillarNow.reduce((m, p) => Math.max(m, Math.hypot(p.x - PX, p.z - PZ)), 0);
console.log(JSON.stringify({ left: pier(-1), right: pier(1), spans, stump, pillarTop: pillarNow.reduce((m, p) => Math.max(m, p.y), 0), pillarChunks: pillarNow.length, lying, hash: engine.hash() }));
