// One host per process: a game-feel world on the host named by --host
// (node: the napi addon; web: the wasm module outside a browser). Tweens of
// every kind (sequence with elastic and back curves, a yoyo loop forever, a
// parallel group, squash and flash presets), a falling rigid body, a system
// that moves by the scaled step, a hit stop, slow motion and camera trauma,
// punch and kick. It prints the world hash at checkpoints, then restores the
// snapshot taken mid-hit-stop and replays to the end. Prints JSON.
//
//   bun tests/support/feel-hash.ts --host node|web [--no-hit-stop]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { og, defineComponent, defineResource, defineSystem, f32, u32, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const which = argv.includes("--host") ? argv[argv.indexOf("--host") + 1]! : "node";
const hitStop = !argv.includes("--no-hit-stop");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", "web", "origin-engine.js"));
  await loadWebHost(glue, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const Runner = defineComponent("FhRunner", { speed: f32(3) });
const Tally = defineResource("FhTally", { done: u32(), tags: u32(), frozen: u32() });
const Run = defineSystem("FhRun", { reads: [Runner, og.TweenCompleted], writes: [og.Transform, Tally] }, (q, time, world) => {
  const tally = must(world.resource(Tally));
  for (const c of world.events(og.TweenCompleted)) {
    tally.done += 1;
    tally.tags += c.tag;
  }
  if (time.scaledDt === 0) tally.frozen += 1;
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const v = a.column(Runner).speed;
    for (let i = 0; i < a.count; i++) p[i * 3] += v[i] * time.scaledDt;
  }
});

const game = og.game({
  resources: [Tally],
  systems: [Run],
  start: (w: World) => {
    const box = must(w.spawn({ name: "box", position: { x: 0, y: 6, z: 0 } }));
    w.insert(box, og.RigidBody);
    w.insert(box, og.Collider, { shape: SHAPE_BOX });
    const runner = must(w.spawn({ name: "runner", position: { x: -4, y: 0, z: 0 } }));
    w.insert(runner, Runner);
    const a = must(w.spawn({ name: "a", position: { x: 0, y: 0, z: 0 } }));
    w.insert(a, og.Sprite, { width: 1, height: 1 });
    must(og.tween.sequence(w, [
      { entity: a, field: "position.y", to: [3], duration: 0.4, ease: "elasticOut", tag: 1 },
      { entity: a, field: "position.x", to: [2], relative: true, duration: 0.3, ease: "backInOut", delay: 0.1, tag: 2 },
    ], { loops: 1, tag: 3 }));
    must(og.tween.to(w, { entity: a, component: og.Sprite, field: "opacity", to: [0.2], duration: 0.25, ease: "bounceOut", loops: -1, yoyo: true }));
    const b = must(w.spawn({ name: "b", position: { x: 1, y: 1, z: 0 } }));
    must(og.tween.parallel(w, [
      { entity: b, field: "scale", to: [2, 0.5, 1], duration: 0.5, ease: "quartInOut" },
      { entity: b, field: "position", to: [-1, 2, 0], duration: 0.7, ease: "expoOut" },
    ], { tag: 4 }));
  },
});

const engine = must(createEngine(game as unknown as GameToken));
const w = () => engine.world;
const checkpoints: { tick: number; hash: string }[] = [];
const step = (n: number) => {
  must(engine.advance(n));
  checkpoints.push({ tick: must(w().resource(og.Time)).tick, hash: engine.hash() });
};
step(20);
const a = w().find("a")!;
must(og.tween.squash(w(), { entity: a, impact: { x: 1.3, y: 0.7 }, duration: 0.2 }));
must(og.tween.flash(w(), { entity: a, duration: 0.15 }));
must(og.camera.shake(w(), 0.6));
must(og.camera.punch(w(), 0.1));
must(og.camera.kick(w(), { x: 1, y: -0.5, z: 0 }, 0.3));
if (hitStop) must(og.time.hitStop(w(), 8));
step(3);
const snap = engine.probe.snapshot() as { id: string };
step(17);
must(og.time.setScale(w(), 0.5));
step(40);
must(og.time.setScale(w(), 1));
step(60);
const end = checkpoints[checkpoints.length - 1]!;
const tally = must(w().resource(Tally));
const summary = { done: tally.done, tags: tally.tags, frozen: tally.frozen };
// The snapshot from inside the hit stop replays into the same world.
if (!engine.probe.restore(snap.id)) throw new Error("restore refused");
must(engine.advance(17));
must(og.time.setScale(w(), 0.5));
must(engine.advance(40));
must(og.time.setScale(w(), 1));
must(engine.advance(60));
console.log(JSON.stringify({ host: which, hitStop, checkpoints, tally: summary, end: end.hash, replayed: engine.hash() }));
