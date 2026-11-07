// One host per process: the destruction-yard fixture's cooked props (a brick
// wall, a concrete pillar, a glass pane, each tuned by og.destruction.
// material) on a slab, blown, struck and hit by a ball on fixed ticks
// between advances. Advances --ticks and prints JSON for destruction.test.ts: the
// world hash, the og.Fractured events read, and each prop's info.
//
//   bun tests/support/destruction-hash.ts --host node|web --project <tests/fixtures/destruction-yard> [--ticks 240]

import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { og, defineResource, defineSystem, u32, MATERIAL_BRICK, MATERIAL_CONCRETE, MATERIAL_GLASS, SHAPE_BOX, SHAPE_SPHERE } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const which = opt("--host", "node");
const ticks = Number(opt("--ticks", "240"));
const project = opt("--project", "");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");

let root = project;
if (which === "node") {
  const { loadNativeHost } = await import("../../src/node/index.ts");
  loadNativeHost();
} else {
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = (await import(join(pkg, "dist", "web", "origin-engine.js"))) as Record<string, (...a: unknown[]) => unknown>;
  await loadWebHost(glue as never, readFileSync(join(pkg, "dist", "web", "origin-engine_bg.wasm")));
  // The Worker's preload, by hand: the manifest and every blob under <root>/cooked/.
  root = "game";
  const manifest = readFileSync(join(project, "cooked", ".cook.ogt"), "utf8");
  glue["og_web_put_file"]!(`${root}/cooked/.cook.ogt`, new TextEncoder().encode(manifest));
  const listed = JSON.parse(glue["og_web_bundle_files"]!(manifest) as string) as Array<{ file: string; id: string; type: string }>;
  const textures: Array<{ id: string; bytes: Uint8Array }> = [];
  for (const o of listed) {
    const bytes = new Uint8Array(readFileSync(join(project, "cooked", o.file)));
    glue["og_web_put_file"]!(`${root}/cooked/${o.file}`, bytes);
    if (o.type === "og.Texture") textures.push({ id: o.id, bytes });
  }
  // And its transcode (worker.ts transcodeBundle): the wasm reads no Basis
  // texture itself. With no renderer every texture is RGBA8, level 0.
  const { loadBasisTranscoder, transcodeKtx2 } = await import("../../src/web/basis.ts");
  const basis = await loadBasisTranscoder();
  let parsed = manifest;
  for (const t of textures) {
    const ktx2 = glue["og_web_texture_ktx2"]!(parsed, t.id, t.bytes) as Uint8Array;
    parsed = "";
    if (ktx2.length === 0) continue;
    const out = transcodeKtx2(basis, ktx2, "rgba8");
    glue["og_web_put_texture"]!(t.id, out.width, out.height, out.levels[0], out.srgb);
  }
}

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};
const Tally = defineResource("DeTally", { fractured: u32(), chunks: u32() });

/** Counts og.Fractured and the chunks it reports, each tick. */
const Count = defineSystem("DeCount", { reads: [og.Fractured], writes: [Tally] }, (_q, _t, world) => {
  const t = world.resource(Tally);
  const events = world.events(og.Fractured);
  if (t.ok && events.length) {
    t.value.fractured += events.length;
    t.value.chunks += events.reduce((n, f) => n + f.chunks, 0);
  }
});

/** The yard's beats, between advances: the charge at the wall's foot, the blow at the pillar, a ball at the pane. */
const beats: Array<[number, (world: World) => void]> = [
  [20, (world) => must(og.destruction.explode(world, { point: { x: -3, y: 0.3, z: 0.15 }, radius: 1.8, innerRadius: 1.0, damage: 6000, impulse: 2500 }))],
  [40, (world) => must(og.destruction.damage(world, world.find("pillar")!, { point: { x: 1.2, y: 1.4, z: -0.15 }, radius: 0.5, innerRadius: 0.25, amount: 8000, impulse: { x: 0, y: 0, z: -600 } }))],
  [60, (world) => {
    const ball = must(world.spawn({ name: "ball", position: { x: 5.4, y: 1.2, z: 3 } }));
    must(world.insert(ball, og.RigidBody, { kind: "dynamic", mass: 6, ccd: true }));
    must(world.insert(ball, og.Collider, { shape: SHAPE_SPHERE, radius: 0.15 }));
    must(world.insert(ball, og.Velocity, { linear: { x: 0, y: 1, z: -14 } }));
  }],
];

function start(world: World): void {
  const ground = must(world.spawn({ name: "ground", position: { x: 1, y: -0.25, z: 0 } }));
  world.insert(ground, og.RigidBody, { kind: "fixed" });
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 24, y: 0.5, z: 14 }, friction: 0.8 });
  const prop = (model: string, name: string, x: number, y: number, z: number, kind: number) => {
    const e = must(world.instantiate(must(og.load.gltf(world, model)), { name, position: { x, y, z } }));
    must(world.insert(e, og.Destructible, must(og.destruction.material(world, kind))));
  };
  prop("props/wall.ogt", "wall", -3, 0, 0, MATERIAL_BRICK);
  prop("props/pillar.ogt", "pillar", 1.2, 0, -0.4, MATERIAL_CONCRETE);
  prop("props/pane.ogt", "pane", 5.4, 0.4, 0, MATERIAL_GLASS);
}

const game = og.game({ tick: 60, seed: 1, resources: [Tally], systems: [Count], start }) as unknown as GameToken;
const engine = must(createEngine(game, { root }));
const out: Record<string, unknown> = { host: which, hashAtLoad: engine.hash() };
let at = 0;
for (const [tick, beat] of beats) {
  must(engine.advance(tick - at));
  beat(engine.world);
  at = tick;
}
out.tick = must(engine.advance(ticks - at));
out.hash = engine.hash();
const w = engine.world;
const tally = w.resource(Tally);
out.fractured = tally.ok ? tally.value.fractured : null;
out.chunks = tally.ok ? tally.value.chunks : null;
out.info = Object.fromEntries(["wall", "pillar", "pane"].map((n) => [n, must(og.destruction.info(w, w.find(n)!))]));
console.log(JSON.stringify(out));
