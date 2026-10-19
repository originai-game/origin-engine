// One native engine per process on the reference scene (crates/og-tests/
// reference-scene: 500 turning objects, 20 skinned walkers, two shadowed
// lights, post) plus 2000 small boxes on a wide grid, most of them outside
// the view (so the frustum cull has more than two grains to split), and 48
// boxes dropped on a floor collider. The lanes are whatever OG_LANES gave the
// process. Prints JSON for lanes.test.ts: at ticks 60, 180 and 300 the world
// hash and the SHA-256 of a captured frame (anim palettes, extraction and
// cull reach the pixels, not the world hash); then a replay: restore the
// snapshot of tick 60, switch lanes with the probe's lanes:<on|off>, advance
// to 180 again; and the dump_lanes report.
//
//   OG_LANES=off|<n> bun tests/support/lanes-hash.ts --replay-lanes on|off

import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { og, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";
import { loadNativeHost, native } from "../../src/node/index.ts";
import { bundleGame } from "../../src/node/build.ts";
import { decodeCapture } from "./png.ts";

declare const Bun: { CryptoHasher: new (alg: string) => { update(d: Uint8Array): void; digest(enc: "hex"): string } };

const argv = process.argv.slice(2);
const replayLanes = argv.includes("--replay-lanes") ? argv[argv.indexOf("--replay-lanes") + 1]! : "off";
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const repo = join(pkg, "..", "..");
const scene = join(repo, "crates", "og-tests", "reference-scene");
const W = 320;
const H = 180;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

loadNativeHost();
const bundled = join(repo, "target", "tmp", "lanes-test", `scene-${process.pid}.js`);
const b = await bundleGame(join(scene, "src", "main.ts"), bundled, "native");
if (!b.ok) throw new Error(b.reason);
const ref = (await import(bundled)) as { build(world: World): void; Turn: unknown };

function extras(world: World): void {
  const cube = must(og.render.boxGeometry(world, { width: 0.3, height: 0.3, depth: 0.3 }));
  const paint = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.5, b: 0.9 }, roughness: 0.6 }));
  for (let i = 0; i < 2000; i++) {
    const position = { x: ((i % 50) - 24.5) * 2.4, y: 0.15, z: -Math.floor(i / 50) * 2.4 - 4 };
    const e = must(world.spawn({ name: `far-${i}`, position }));
    world.insert(e, og.Mesh, { geometry: cube, material: paint, castShadow: false, receiveShadow: true });
  }
  const ground = must(world.spawn({ name: "ground", position: { x: 0, y: -0.5, z: 0 } }));
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 80, y: 1, z: 80 } });
  const crate = must(og.render.boxGeometry(world, { width: 0.6, height: 0.6, depth: 0.6 }));
  for (let i = 0; i < 48; i++) {
    const position = { x: ((i % 6) - 2.5) * 0.9 + (i % 3) * 0.05, y: 3 + Math.floor(i / 6) * 0.8, z: 14 + ((i * 7) % 5) * 0.2 };
    const e = must(world.spawn({ name: `crate-${i}`, position }));
    world.insert(e, og.RigidBody, {});
    world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 0.6, y: 0.6, z: 0.6 } });
    world.insert(e, og.Mesh, { geometry: crate, material: paint, castShadow: true, receiveShadow: true });
  }
}

const game = og.game({
  seed: 1,
  systems: [ref.Turn as never],
  render: { antialias: "off" },
  start(world: World) {
    ref.build(world);
    extras(world);
  },
}) as unknown as GameToken;
const engine = must(createEngine(game, { root: scene }));
const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);

const frame = () => {
  const img = decodeCapture(engine.probe.capture(W, H));
  const h = new Bun.CryptoHasher("sha256");
  h.update(img.rgba);
  return h.digest("hex");
};
const at = (tick: number) => {
  const now = engine.world.tick;
  if (tick > now) must(engine.advance(tick - now));
  return { tick: engine.world.tick, hash: engine.hash(), frame: frame() };
};

const out: Record<string, unknown> = { tier: attached.value };
const first = at(60);
const snap = engine.probe.snapshot() as { id: string; hash: string };
out.checkpoints = [first, at(180), at(300)];
if (!engine.probe.restore(snap.id)) throw new Error("restore refused");
out.restored = engine.hash();
engine.probe.commands(`0:lanes:${replayLanes}`);
out.replay = at(180);
engine.probe.commands("0:dump_lanes:lanes.json");
must(engine.advance(1));
const dump = engine.takeOutputs().find((o) => o.file === "lanes.json");
out.lanes = dump ? JSON.parse(dump.data) : null;
engine.destroy();
console.log(JSON.stringify(out));
