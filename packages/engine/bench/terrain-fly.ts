// Frame times flying over a streamed terrain on the native host (phase 3,
// terrain streaming): the 2 km island of og-tests' terrain scenes (the same
// height function, crates/og-tests/src/terrain_scenes.rs `island_height`,
// on its 2 m grid) and an 8 km test terrain (that island four times wider
// with a fine ridged detail over it, 4097 x 4097 samples on a 2 m grid),
// cooked by `og cook` into tiles. The camera flies a line across each at
// 60 m over the ground, one display frame of 16.667 ms per sample, and each
// sample times engine.frame plus waiting for the GPU (og_node_wait_gpu), so
// a frame's tick, extraction, streaming, recording and GPU work all count.
//
//   bun bench/terrain-fly.ts                 # every terrain and mode, tables to --out
//   bun bench/terrain-fly.ts --terrain 8k --mode stream --budget 16 [--size 1280x720]
//
// Modes: `stream` (tiles decoded on the worker within --budget MiB a
// terrain) and `resident` (the probe's terrain_stream:resident: every wanted
// tile decoded before the frame that wants it, the same budget).
// Memory: the streamer's tiles (dump_stream's `terrain`) every 30 frames,
// and the process's resident set at the end. The project and its maps are
// generated under target/evidence/terrain-fly/ (not committed).

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { deflateSync } from "node:zlib";
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { encodePng } from "../src/node/png.ts";
import { createEngine, type GameToken } from "../src/runtime.ts";

declare const Bun: {
  spawnSync(cmd: string[], o?: { cwd?: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => (argv.includes(name) ? argv[argv.indexOf(name) + 1]! : fallback);
const work = join(repo, "target", "evidence", "terrain-fly");
const project = join(work, "project");
const out = opt("--out", join(work, "results"));
const [W, H] = opt("--size", "1280x720").split("x").map(Number) as [number, number];

// ── The height functions (og-tests' island, ported) ─────────────────────

function hash2(x: number, y: number, seed: number): number {
  let h = (Math.imul(x | 0, 0x8da6b343) ^ Math.imul(y | 0, 0xd8163841) ^ seed) >>> 0;
  h = Math.imul(h ^ (h >>> 13), 0x85ebca6b) >>> 0;
  h = (h ^ (h >>> 16)) >>> 0;
  return (h & 0xffff) / 65535;
}
function noise2(x: number, y: number, seed: number): number {
  const ix = Math.floor(x), iy = Math.floor(y);
  const fx = x - ix, fy = y - iy;
  const ux = fx * fx * (3 - 2 * fx), uy = fy * fy * (3 - 2 * fy);
  const a = hash2(ix, iy, seed), b = hash2(ix + 1, iy, seed), c = hash2(ix, iy + 1, seed), d = hash2(ix + 1, iy + 1, seed);
  const top = a + (b - a) * ux, bottom = c + (d - c) * ux;
  return top + (bottom - top) * uy;
}
const smooth = (e0: number, e1: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - e0) / (e1 - e0)));
  return t * t * (3 - 2 * t);
};
function fbm(x: number, y: number, octaves: number, seed: number): number {
  let v = 0, amp = 0.5, f = 1;
  for (let o = 0; o < octaves; o++) { v += (noise2(x * f, y * f, seed + o) - 0.5) * amp; f *= 2.03; amp *= 0.5; }
  return v;
}
function ridged(x: number, y: number, octaves: number, seed: number): number {
  let v = 0, amp = 0.5, f = 1;
  for (let o = 0; o < octaves; o++) { const n = 1 - Math.abs(noise2(x * f, y * f, seed + o) * 2 - 1); v += n * n * amp; f *= 2.1; amp *= 0.5; }
  return v;
}
const ISLAND = 2048;
function island(x: number, z: number): number {
  const r = Math.hypot(x, z * 1.15) / (ISLAND * 0.5);
  const land = 1 - smooth(0.62, 0.92, r);
  const hills = fbm(x / 420, z / 420, 6, 11) * 70 + 22;
  const north = smooth(-60, -520, z) * (1 - smooth(380, 820, Math.abs(x)));
  const peaks = ridged(x / 360, z / 360, 5, 23) * 330 * north;
  let h = (hills + peaks) * land - 30 * (1 - land);
  const d = Math.hypot(x - 180, z - 260) / 170;
  const basin = 1 - smooth(0.55, 1.15, d);
  h = h * (1 - basin) + (6 - 9 + d * d * 6) * basin;
  return h;
}
/** The 8 km terrain: the island four times wider and 1.5 times higher, with ridged detail every few hundred metres. */
const big = (x: number, z: number) => island(x / 4, z / 4) * 1.5 + (ridged(x / 240, z / 240, 4, 41) - 0.35) * 40;

type Map = { name: string; size: number; samples: number; height: (x: number, z: number) => number; low: number; high: number };
const MAPS: Record<string, Map> = {
  "2k": { name: "island-2k", size: 2048, samples: 1025, height: island, low: -40, high: 420 },
  "8k": { name: "island-8k", size: 8192, samples: 4097, height: big, low: -80, high: 680 },
};

/** A 16-bit grey PNG (black `low`, white `high`). */
function grey16(m: Map): Uint8Array {
  const n = m.samples, cell = m.size / (n - 1);
  const raw = new Uint8Array((n * 2 + 1) * n);
  for (let r = 0; r < n; r++) {
    raw[r * (n * 2 + 1)] = 0;
    for (let c = 0; c < n; c++) {
      const h = m.height(-m.size / 2 + c * cell, -m.size / 2 + r * cell);
      const v = Math.round(Math.min(1, Math.max(0, (h - m.low) / (m.high - m.low))) * 65535);
      const at = r * (n * 2 + 1) + 1 + c * 2;
      raw[at] = v >> 8;
      raw[at + 1] = v & 255;
    }
  }
  const ihdr = new Uint8Array(13);
  new DataView(ihdr.buffer).setUint32(0, n);
  new DataView(ihdr.buffer).setUint32(4, n);
  ihdr.set([16, 0, 0, 0, 0], 8);
  return png([["IHDR", ihdr], ["IDAT", deflateSync(raw, { level: 6 })], ["IEND", new Uint8Array(0)]]);
}
const CRC = Array.from({ length: 256 }, (_, n) => { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1; return c >>> 0; });
function png(chunks: [string, Uint8Array][]): Uint8Array {
  const parts = [new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10])];
  for (const [type, data] of chunks) {
    const c = new Uint8Array(12 + data.length);
    const v = new DataView(c.buffer);
    v.setUint32(0, data.length);
    for (let i = 0; i < 4; i++) c[4 + i] = type.charCodeAt(i);
    c.set(data, 8);
    let crc = 0xffffffff;
    for (let i = 4; i < 8 + data.length; i++) crc = CRC[(crc ^ c[i]!) & 255]! ^ (crc >>> 8);
    v.setUint32(8 + data.length, (crc ^ 0xffffffff) >>> 0);
    parts.push(c);
  }
  const all = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  parts.reduce((at, p) => (all.set(p, at), at + p.length), 0);
  return all;
}
function texture(base: [number, number, number], seed: number): Uint8Array {
  const n = 128, rgba = new Uint8Array(n * n * 4);
  for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
    const g = 0.7 + 0.5 * (noise2((x / n) * 16, (y / n) * 16, seed) - 0.5) + 0.3 * (noise2((x / n) * 64, (y / n) * 64, seed + 1) - 0.5);
    for (let k = 0; k < 3; k++) rgba[(y * n + x) * 4 + k] = Math.min(255, Math.round(base[k]! * g));
    rgba[(y * n + x) * 4 + 3] = 255;
  }
  return encodePng({ width: n, height: n, rgba });
}

/** Writes and cooks the project once (the maps take a few seconds). */
function stage(): void {
  if (existsSync(join(project, ".staged-1"))) return;
  mkdirSync(join(project, "assets", "terrain"), { recursive: true });
  mkdirSync(join(project, "assets", "textures"), { recursive: true });
  writeFileSync(join(project, "project.ogt"), 'type project\nversion 1\n---\nname "Terrain fly bench"\nid "terrain-fly"\nengine "0.7.0"\nentry "main.ts"\nprofile "3d"\ntick 60\nseed 1\n');
  writeFileSync(join(project, "main.ts"), '// Cooked for its assets only: bench/terrain-fly.ts drives the world.\nimport { og } from "@origingame/engine";\n\nexport default og.game();\n');
  writeFileSync(join(project, "assets", "textures", "grass.png"), texture([70, 104, 44], 3));
  writeFileSync(join(project, "assets", "textures", "rock.png"), texture([118, 110, 100], 5));
  writeFileSync(join(project, "assets", "textures", "sand.png"), texture([196, 178, 132], 7));
  for (const m of Object.values(MAPS)) {
    writeFileSync(join(project, "assets", "terrain", `${m.name}.png`), grey16(m));
    writeFileSync(
      join(project, "assets", "terrain", `${m.name}.ogt`),
      `type terrain\nversion 1\n---\n# bench/terrain-fly.ts: ${m.size} m, ${m.samples} x ${m.samples} samples.\nheightmap "${m.name}.png"\nheight ${m.low} ${m.high}\nsize ${m.size} ${m.size}\n` +
        'layer {\n  albedo "textures/grass.png"\n  tile 6\n  slope 0 30\n  slopeFade 6\n  altitude 4 1000\n  altitudeFade 2\n}\n' +
        'layer {\n  albedo "textures/rock.png"\n  tile 12\n  triplanar true\n  slope 28 90\n  slopeFade 6\n}\n' +
        'layer {\n  albedo "textures/sand.png"\n  tile 5\n  altitude -200 5\n  altitudeFade 2\n}\n',
    );
  }
  const ogBin = process.env.OG_BIN ?? join(pkg, "native", `og.${process.platform}-${process.arch}`);
  const cooked = Bun.spawnSync([ogBin, "cook", project], { env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  writeFileSync(join(project, ".staged-1"), "");
}

// ── One run ─────────────────────────────────────────────────────────────

type Tiles = { resident: number; waiting: number; inFlight: number; cached: number; cachedBytes: number; decoded: number; handed: number; worstFrameMs: number };
type Run = {
  terrain: string; mode: string; budgetMiB: number; size: string; tier: string; frames: number;
  ms: { p50: number; p95: number; p99: number; max: number; mean: number };
  cpuMs: { p50: number; p95: number; p99: number; max: number };
  over: { "16.7": number; "33.3": number };
  tiles: { residentMax: number; waitingMax: number; cachedMax: number; cachedBytesMax: number; decoded: number; framesWaiting: number; streamWorstMs: number };
  /** With --every 1: each frame's time beside the tiles the streamer handed the renderer in it. */
  handed?: number[];
  rssMiB: number;
  times: number[];
};

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const q = (s: number[], p: number) => s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
const r3 = (v: number) => Math.round(v * 1000) / 1000;

function one(key: string, mode: string, budget: number): Run {
  const m = MAPS[key]!;
  loadNativeHost();
  let data = 0;
  const game = og.game({
    start(world: World) {
      const info = must(og.terrain.load(world, `terrain/${m.name}.ogt`)) as unknown as { data: number };
      data = info.data;
      const ground = must(world.spawn({ name: "ground" }));
      world.insert(ground, og.Terrain, { data: info.data as never });
      const sun = must(world.spawn({ name: "sun" }));
      world.insert(sun, og.DirectionalLight, { intensity: 3 });
      world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.5, y: -0.6, z: -0.4 }) });
    },
  });
  const engine = must(createEngine(game as GameToken, { root: project }));
  const attached = JSON.parse(native().og_node_attach_offscreen(engine.id, W, H)) as { ok: boolean; value?: string; reason?: string };
  if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  must(engine.advance(1));
  engine.probe.commands(`0:terrain_stream:${mode}:${budget}`);
  const w = engine.world;
  const settings = must(w.resource(og.RenderSettings));
  settings.sky = true;
  // The flight: corner to corner over the middle, 60 m above the ground (smoothed), looking ahead and down.
  const half = m.size * 0.45;
  const from = { x: -half, z: half * 0.8 }, to = { x: half, z: -half * 0.8 };
  const frames = key === "8k" ? 2400 : 1200;
  const ground = (x: number, z: number) => must(og.terrain.sample(w, { data: data as never, origin: { x: 0, y: 0, z: 0 }, x, z })).height;
  let alt = ground(from.x, from.z) + 60;
  const at = (t: number) => ({ x: from.x + (to.x - from.x) * t, z: from.z + (to.z - from.z) * t });
  const times: number[] = [], cpu: number[] = [];
  const tiles = { residentMax: 0, waitingMax: 0, cachedMax: 0, cachedBytesMax: 0, decoded: 0, framesWaiting: 0, streamWorstMs: 0 };
  const every = Number(opt("--every", "30"));
  const handed: number[] = [];
  const wait = () => {
    const r = JSON.parse(native().og_node_wait_gpu(engine.id)) as { ok: boolean; reason?: string };
    if (!r.ok) throw new Error(r.reason);
  };
  const warm = 30;
  for (let i = -warm; i < frames; i++) {
    const t = Math.max(0, i) / frames;
    const p = at(t), ahead = at(Math.min(1, t + 0.02));
    // Rise at once over a ridge, sink slowly after it.
    const want = Math.max(ground(p.x, p.z), ground(ahead.x, ahead.z)) + 60;
    alt = want > alt ? want : alt + (want - alt) * 0.02;
    const cam = must(w.resource(og.Camera));
    const eye = { x: p.x, y: alt, z: p.z };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: ahead.x, y: alt - 0.35 * Math.hypot(ahead.x - p.x, ahead.z - p.z) - 30, z: ahead.z });
    cam.fov = 60;
    cam.far = key === "8k" ? 6000 : 2500;
    const sample = i >= 0 && (i % every === 0 || i === frames - 1);
    // dump_stream runs at the next frame's tick: the streamer as this frame left it.
    const t0 = performance.now();
    const f = engine.frame(1000 / 60);
    if (!f.ok) throw new Error(f.reason);
    const t1 = performance.now();
    wait();
    const t2 = performance.now();
    if (i >= 0) { times.push(t2 - t0); cpu.push(t1 - t0); }
    if (sample) {
      engine.probe.commands("0:dump_stream:s.json");
      must(engine.advance(1));
      const o = engine.takeOutputs().find((x) => x.action === "dump_stream");
      const s = (JSON.parse(o!.data) as { terrain: Tiles }).terrain;
      tiles.residentMax = Math.max(tiles.residentMax, s.resident);
      tiles.waitingMax = Math.max(tiles.waitingMax, s.waiting);
      tiles.cachedMax = Math.max(tiles.cachedMax, s.cached);
      tiles.cachedBytesMax = Math.max(tiles.cachedBytesMax, s.cachedBytes);
      tiles.decoded = s.decoded;
      tiles.streamWorstMs = r3(s.worstFrameMs);
      if (every === 1) handed.push(s.handed);
      if (s.waiting > 0) tiles.framesWaiting++;
    }
  }
  // The last view, captured (a capture settles the tiles first, so after the timing).
  engine.probe.commands(`0:screenshot:${key}-${mode}-${budget}.png`);
  must(engine.advance(1));
  for (const o of engine.takeOutputs()) if (o.kind === "png") writeFileSync(join(out, o.file), Uint8Array.from(atob(o.data), (c) => c.charCodeAt(0)));
  const sorted = [...times].sort((a, b) => a - b), cs = [...cpu].sort((a, b) => a - b);
  const run: Run = {
    terrain: key, mode, budgetMiB: budget, size: `${W}x${H}`, tier: attached.value!, frames,
    ms: { p50: r3(q(sorted, 0.5)), p95: r3(q(sorted, 0.95)), p99: r3(q(sorted, 0.99)), max: r3(sorted.at(-1)!), mean: r3(times.reduce((a, b) => a + b, 0) / times.length) },
    cpuMs: { p50: r3(q(cs, 0.5)), p95: r3(q(cs, 0.95)), p99: r3(q(cs, 0.99)), max: r3(cs.at(-1)!) },
    over: { "16.7": times.filter((t) => t > 1000 / 60).length, "33.3": times.filter((t) => t > 2000 / 60).length },
    tiles,
    rssMiB: Math.round((process as unknown as { memoryUsage(): { rss: number } }).memoryUsage().rss / 1048576),
    times: times.map(r3),
    ...(every === 1 ? { handed } : {}),
  };
  engine.destroy();
  return run;
}

// ── Driver ──────────────────────────────────────────────────────────────

mkdirSync(out, { recursive: true });
stage();
if (argv.includes("--terrain")) {
  const run = one(opt("--terrain", "2k"), opt("--mode", "stream"), Number(opt("--budget", "16")));
  process.stdout.write(JSON.stringify(run) + "\n");
} else {
  // Each run in its own process, so the resident set is that run's.
  const runs: Run[] = [];
  for (const terrain of ["2k", "8k"]) {
    for (const [mode, budget] of [["stream", 16], ["resident", 16], ["stream", 4]] as const) {
      const r = Bun.spawnSync(["bun", fileURLToPath(import.meta.url), "--terrain", terrain, "--mode", mode, "--budget", String(budget), "--size", `${W}x${H}`, "--out", out], { cwd: pkg });
      if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr));
      const run = JSON.parse(new TextDecoder().decode(r.stdout).trim().split("\n").pop()!) as Run;
      runs.push(run);
      process.stderr.write(`${terrain} ${mode} ${budget} MiB: p50 ${run.ms.p50} p95 ${run.ms.p95} p99 ${run.ms.p99} max ${run.ms.max} ms\n`);
    }
  }
  writeFileSync(join(out, "frame-times.json"), JSON.stringify(runs, null, 1) + "\n");
  const lines = [
    `Frame times flying the terrains (bun bench/terrain-fly.ts, ${W}x${H}, ${runs[0]!.tier}; engine.frame plus the GPU wait, after 30 warm-up frames).`,
    "",
    "| Terrain | Mode | Budget | Frames | p50 ms | p95 ms | p99 ms | max ms | mean ms | CPU p50 / p99 / max ms | Streamer worst ms (with warm-up) | > 16.7 ms | > 33.3 ms |",
    "|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ...runs.map((r) => `| ${r.terrain} | ${r.mode} | ${r.budgetMiB} MiB | ${r.frames} | ${r.ms.p50} | ${r.ms.p95} | ${r.ms.p99} | ${r.ms.max} | ${r.ms.mean} | ${r.cpuMs.p50} / ${r.cpuMs.p99} / ${r.cpuMs.max} | ${r.tiles.streamWorstMs} | ${r.over["16.7"]} | ${r.over["33.3"]} |`),
    "",
    "| Terrain | Mode | Budget | Tiles resident (max) | Waiting (max) | Samples waiting | Cached tiles (max) | CPU cache (max) | Tiles decoded | Process RSS |",
    "|---|---|---|---|---|---|---|---|---|---|",
    ...runs.map((r) => `| ${r.terrain} | ${r.mode} | ${r.budgetMiB} MiB | ${r.tiles.residentMax} | ${r.tiles.waitingMax} | ${r.tiles.framesWaiting} of ${Math.ceil(r.frames / 30) + 1} | ${r.tiles.cachedMax} | ${(r.tiles.cachedBytesMax / 1048576).toFixed(1)} MiB | ${r.tiles.decoded} | ${r.rssMiB} MiB |`),
    "",
  ];
  writeFileSync(join(out, "frame-times.md"), lines.join("\n"));
  process.stdout.write(lines.join("\n"));
}
