// The Phase 2 core round's cases on the host named by --host (node: the
// napi addon, a dev build; web: the wasm module outside a browser, a release
// build; publish: the publish wasm), one --case per process. Prints JSON.
//
//   bun tests/support/core-round.ts --host node|web|publish --case <name>
//
// Cases:
//   rng         tests/fixtures/rng-readonly: a fork from og.Rng declared in
//               reads only, and one declared in writes, on tick 1
//   procgen     og.procgen's generators and og.Rng's weighted and permutation
//               draws, each as an FNV-1a hash of its output (floats as f32)
//   audioEvents og.audio.createSound and og.AudioEvent's seeded picks (--seed)

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, type Sound } from "@origingame/engine";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { readConfig } from "../../shell/page.ts";

const argv = process.argv.slice(2);
const opt = (name: string) => argv[argv.indexOf(name) + 1] ?? "";
const host = opt("--host");
const which = opt("--case");
const pkg = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
if (host === "node") {
  (await import("../../src/node/index.ts")).loadNativeHost();
} else {
  const dir = host === "publish" ? "web-publish" : "web";
  const { loadWebHost } = await import("../../src/web/host.ts");
  const glue = await import(join(pkg, "dist", dir, "origin-engine.js"));
  const { fileModuleHost } = await import("../../src/node/modules.ts");
  await loadWebHost(glue, readFileSync(join(pkg, "dist", dir, "origin-engine_bg.wasm")), undefined, fileModuleHost(join(pkg, "dist", dir)));
}

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const out: Record<string, unknown> = { host, case: which };

/** FNV-1a over a byte sequence, as 16 hex digits (og-core's `hash_bytes`). */
const fnv = (bytes: Iterable<number>): string => {
  let h = 0xcbf29ce484222325n;
  for (const b of bytes) h = BigInt.asUintN(64, (h ^ BigInt(b)) * 0x100000001b3n);
  return h.toString(16).padStart(16, "0");
};

if (which === "startup") {
  const game = og.game({}) as unknown as GameToken;
  // Exercise the exported page's JSON boundary, not only createEngine's
  // typed input. Dropping an object here silently boots with GameDesc defaults.
  const config = (project: unknown) => readConfig({
    getElementById: () => ({ textContent: JSON.stringify({ project }) }),
  } as unknown as Document).project!;
  const project = config({ tick: 30, seed: 83, saveVersion: 4, profile: "3d" });
  const created = createEngine(game, { project });
  if (!created.ok) throw new Error(created.reason);
  const engine = created.value;
  engine.advance(15);
  out.time = JSON.parse(JSON.stringify(engine.world.resource(og.Time)));
  out.rng = JSON.parse(JSON.stringify(engine.world.resource(og.Rng)));
  const saved = engine.world.snapshot();
  if (!saved.ok) throw new Error(saved.reason);
  const defaults = start(og.game({}));
  out.defaults = JSON.parse(JSON.stringify(defaults.world.resource(og.Time)));
  out.oldSave = defaults.world.restore(saved.value);
  const replay = createEngine(game, { project, seed: 197 });
  if (!replay.ok) throw new Error(replay.reason);
  out.override = JSON.parse(JSON.stringify(replay.value.world.resource(og.Rng)));
  out.conflicts = [
    og.game({ tick: 45 }), og.game({ seed: 84 }), og.game({ saveVersion: 5 }),
  ].map((game) => {
    const result = createEngine(game as unknown as GameToken, { project });
    if (result.ok) { result.value.destroy(); throw new Error("conflicting project startup accepted"); }
    return result.reason;
  });
  out.invalid = [
    "type project\nversion 1\n---\ntick 30", null,
    { tick: 0 }, { tick: 241 }, { tick: 30.5 }, { seed: -1 }, { seed: 4294967296 },
    { saveVersion: 0 }, { unknown: 1 }, { flow: { dialogue: "x", unknown: true } }, { tick: null },
  ].map((project) => {
    const result = createEngine(game, { project: config(project) });
    if (result.ok) { result.value.destroy(); throw new Error("invalid project startup accepted"); }
    return result.reason;
  });
  defaults.destroy();
  replay.value.destroy();
  engine.destroy();
} else if (which === "rng") {
  const { default: game, Repro } = await import("../fixtures/rng-readonly/src/main.ts");
  const engine = start(game);
  const advanced = engine.advance(3);
  const r = engine.world.resource(Repro);
  const rng = engine.world.resource(og.Rng);
  Object.assign(out, {
    advanced,
    repro: r.ok ? { ...r.value } : r.reason,
    rng: rng.ok ? { seed: rng.value.seed, state: rng.value.state } : rng.reason,
    hash: host === "publish" ? "" : engine.hash(),
  });
} else if (which === "audioEvents") {
  // og.audio.createSound and og.AudioEvent: four tones made from TypeScript,
  // an event triggered every fourth tick, its picks recorded per play; run
  // with --seed (the world's seed).
  const seed = Number(opt("--seed") || "1");
  const engine = start(og.game({ tick: 60, seed }));
  const w = engine.world;
  const sounds: Sound[] = [];
  for (let k = 0; k < 4; k++) {
    const samples = new Float32Array(2400);
    for (let i = 0; i < samples.length; i++) samples[i] = 0.2 * Math.sin((2 * Math.PI * (300 + 100 * k) * i) / 48000);
    const s = og.audio.createSound(w, { samples, rate: 48000, channels: 1 });
    if (!s.ok) throw new Error(s.reason);
    sounds.push(s.value);
  }
  out.info = og.load.soundInfo(w, sounds[0]!);
  out.refused = og.audio.createSound(w, { samples: new Float32Array(3), channels: 2 });
  const e = w.spawn({ name: "steps" });
  if (!e.ok) throw new Error(e.reason);
  const ins = w.insert(e.value, og.AudioEvent, { sounds, weights: [1, 2, 1, 0.5], pitchJitter: 2, volumeJitter: 3, cooldown: 0.05 });
  if (!ins.ok) throw new Error(ins.reason);
  const picks: number[] = [];
  for (let t = 1; t <= 120; t++) {
    if (t % 2 === 1) {
      const ev = w.get(e.value, og.AudioEvent);
      if (!ev.ok) throw new Error(ev.reason);
      w.set(e.value, og.AudioEvent, { ...(ev.value as object), trigger: true });
    }
    const before = (w.get(e.value, og.AudioEvent) as { value: { played: number } }).value.played;
    engine.advance(1);
    const ev = (w.get(e.value, og.AudioEvent) as { value: { played: number; index: number; trigger: boolean } }).value;
    if (ev.trigger) throw new Error("the trigger was not cleared");
    if (ev.played !== before) picks.push(ev.index);
  }
  out.picks = picks;
  out.hash = engine.hash();
} else if (which === "procgen") {
  // One scenario for repeatability and cross-host comparisons. Only the
  // independent og.Rng vectors remain pinned in core.test.ts; generated
  // worlds need not match older engine versions.
  const pin = JSON.parse(readFileSync(join(pkg, "..", "..", "tests", "fixtures", "procgen-pin.json"), "utf8")) as {
    noise: { seed: number; octaves: number; frequency: number; grid: { width: number; height: number; x: number; y: number; step: number; z: number } };
    poisson: { seed: number; width: number; height: number; radius: number; attempts: number; maxPoints: number };
    bsp: { seed: number };
    wfc: { seed: number; width: number; height: number; wrap: boolean; attempts: number; tiles: { weight: number; sockets: [number, number, number, number]; rotations: boolean }[] };
  };
  const f32s = (v: ArrayLike<number>) =>
    v instanceof Float32Array
      ? new Uint8Array(v.buffer, v.byteOffset, v.byteLength)
      : new Uint8Array(new Float32Array(v).buffer);
  const u32s = (v: ArrayLike<number>) =>
    v instanceof Uint32Array
      ? new Uint8Array(v.buffer, v.byteOffset, v.byteLength)
      : new Uint8Array(new Uint32Array(v).buffer);
  const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
    if (!r.ok) throw new Error(r.reason);
    return r.value;
  };
  const grid = (kind: "perlin" | "simplex", threeD: boolean) =>
    must(og.procgen.noiseGrid({
      noise: { seed: pin.noise.seed, kind, octaves: pin.noise.octaves, frequency: pin.noise.frequency, threeD },
      ...pin.noise.grid,
    }));
  const wfc = must(
    og.procgen.wfc({
      seed: pin.wfc.seed, width: pin.wfc.width, height: pin.wfc.height, wrap: pin.wfc.wrap, attempts: pin.wfc.attempts,
      tiles: pin.wfc.tiles.map((t) => ({ weight: t.weight, up: t.sockets[0], right: t.sockets[1], down: t.sockets[2], left: t.sockets[3], rotations: t.rotations })),
    }),
  );
  const poisson = must(og.procgen.poissonDisk({ ...pin.poisson }));
  const engine = start(og.game({ tick: 60, seed: 42 }));
  const rng = must(engine.world.resource(og.Rng));
  const picks = rng.fork("picks");
  const wfcBytes = new Uint8Array(wfc.tiles.length);
  for (let i = 0; i < wfc.tiles.length; i++) wfcBytes[i] = wfc.tiles[i]! * 4 + wfc.rotations[i]!;
  out.hashes = {
    perlin2: fnv(f32s(grid("perlin", false))),
    perlin3: fnv(f32s(grid("perlin", true))),
    simplex2: fnv(f32s(grid("simplex", false))),
    simplex3: fnv(f32s(grid("simplex", true))),
    poisson: fnv(f32s(poisson.flatMap((p) => [p.x, p.y]))),
    bsp: fnv(must(og.procgen.bsp({ seed: pin.bsp.seed })).cells),
    wfc: fnv(wfcBytes),
    weighted: fnv(Array.from({ length: 64 }, () => picks.weighted([1, 0, 3, 0.5]))),
    permutation: fnv(u32s(picks.permutation(32))),
    mainWeighted: rng.weighted([0, 0, 1]),
  };
  out.noise = og.procgen.noise({ seed: 7, kind: "simplex", octaves: 3 }, 0.37, -1.2, 0);
  out.refused = og.procgen.bsp({ width: 0 });
  out.hash = engine.hash();
} else {
  throw new Error(`unknown --case ${which}`);
}
console.log(JSON.stringify(out));
