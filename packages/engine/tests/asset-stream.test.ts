// Streaming groups through the TypeScript API on the native host (IDL
// module `stream`, phase 3). A staged project's scene declares three groups
// along a path down -z: "meadow" (a hut and a sign texture, within 50 m of
// z = -60, kept to 110 m, priority 1), "cave" (a hut and a sound, while the
// camera is inside the "cave-mouth" entity's box, priority 2) and "tower"
// (a hut, within 50 m of z = -260). The camera flies the path with a
// budget that holds meadow or cave but not both.
//
// Ways this could fail: a group loads before its trigger wants it or stays
// after; the memory used passes the budget on some tick; the higher group
// does not evict the lower one, or the lower one comes back while the
// higher one is resident; a model draws its full texture before its group
// is resident (no stand-in) or keeps the stand-in after; og.StreamEvent is
// not sent, sent twice or out of order; og.stream.request does not load a
// group the triggers leave alone, or release does not let it go; a lowered
// budget keeps a group that no longer fits; a group bigger than the budget
// is queued for ever instead of failed with the reason; what is resident
// changes the simulation (the world hash); dump_stream misses the groups or
// the peak.
import { beforeAll, expect, test } from "bun:test";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost, native } from "../src/node/index.ts";
import { encodePng } from "../src/node/png.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { decodeCapture, pixel, type Image } from "./support/png.ts";
import { ogBin } from "./support/og-bin.ts";
import { DEFAULT_PUBLISH, settingsOf, webBuild } from "../src/node/web-build.ts";

declare const Bun: {
  spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array };
  CryptoHasher: new (a: string) => { update(d: Uint8Array): unknown; digest(e: "hex"): string };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const root = join(repo, "target", "evidence", "asset-stream", "project");
const evidence = join(repo, "target", "evidence", "asset-stream");

const W = 160;
const H = 90;

const SCENE = `type scene
version 1
---
name "Path"
stream "meadow" {
  asset "models/hut-a.glb"
  asset "textures/sign.png"
  distance 0 0 -60 50
  unload 110
  priority 1
}
stream "cave" {
  asset "models/hut-b.glb"
  asset "audio/blip.wav"
  volume "cave-mouth"
  priority 2
}
stream "tower" {
  asset "models/hut-c.glb"
  distance 0 0 -260 50
}
entity "cave-mouth" {
  Transform {
    position 0 0 -160
    quaternion 0 0 0 1
    scale 40 20 40
  }
}
`;

/**
 * The staged project's game, for its web build (tests/web/stream.ts plays
 * it in Chrome): the scene, the three huts and a sun, and a camera that
 * flies the path at 0.5 m a tick from z = 20 to z = -320.
 */
const MAIN = `import { og, defineSystem, type World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const Fly = defineSystem("Fly", { writes: [og.Camera] }, (_q, _t, world: World) => {
  const cam = world.resource(og.Camera);
  if (!cam.ok) return;
  const z = Math.max(-320, 20 - 0.5 * world.tick);
  cam.value.position = { x: 0, y: 0, z };
  cam.value.quaternion = og.transform.lookAt({ x: 0, y: 0, z }, { x: 0, y: 0, z: z - 1 });
  cam.value.fov = 50;
  cam.value.far = 500;
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Fly],
  start(world: World) {
    world.instantiate(must(og.load.scene(world, "scenes/path.ogt")));
    for (const [name, z] of [["hut-a", -60], ["hut-b", -160], ["hut-c", -260]] as const) {
      must(world.instantiate(must(og.load.gltf(world, \`models/\${name}.glb\`)), { name, position: { x: 0, y: 0, z } }));
    }
    must(og.load.texture(world, "textures/sign.png"));
    must(og.load.sound(world, "audio/blip.wav"));
    const sun = must(world.spawn({ name: "sun" }));
    world.insert(sun, og.DirectionalLight, { intensity: 2 });
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.3, y: -0.5, z: -1 }) });
  },
});
`;

/** A solid-colour RGBA image. */
function solid(size: number, rgb: [number, number, number]): Image {
  const rgba = new Uint8Array(size * size * 4);
  for (let i = 0; i < size * size; i++) rgba.set([...rgb, 255], i * 4);
  return { width: size, height: size, rgba };
}

/**
 * A textured glTF: a 10 m wavy sheet of 48 x 48 quads standing in the XY
 * plane, facing +z, whose material samples `png` as its base colour.
 */
function texturedGlb(png: Uint8Array): Uint8Array {
  const N = 48;
  const pos: number[] = [];
  const nrm: number[] = [];
  const uv: number[] = [];
  const idx: number[] = [];
  for (let y = 0; y <= N; y++) {
    for (let x = 0; x <= N; x++) {
      const u = x / N;
      const v = y / N;
      pos.push(u * 10 - 5, v * 10 - 5, 0.3 * Math.sin(u * 9) * Math.cos(v * 7));
      nrm.push(0, 0, 1);
      uv.push(u, 1 - v);
    }
  }
  for (let y = 0; y < N; y++) {
    for (let x = 0; x < N; x++) {
      const i = y * (N + 1) + x;
      const r = N + 1;
      idx.push(i, i + 1, i + r, i + 1, i + r + 1, i + r);
    }
  }
  const parts: Uint8Array[] = [];
  const views: Array<{ buffer: number; byteOffset: number; byteLength: number }> = [];
  let length = 0;
  const push = (bytes: Uint8Array) => {
    const pad = (4 - (length % 4)) % 4;
    if (pad) parts.push(new Uint8Array(pad));
    length += pad;
    views.push({ buffer: 0, byteOffset: length, byteLength: bytes.length });
    parts.push(bytes);
    length += bytes.length;
  };
  push(new Uint8Array(new Float32Array(pos).buffer));
  push(new Uint8Array(new Float32Array(nrm).buffer));
  push(new Uint8Array(new Float32Array(uv).buffer));
  push(new Uint8Array(new Uint32Array(idx).buffer));
  push(png);
  const tail = (4 - (length % 4)) % 4;
  if (tail) parts.push(new Uint8Array(tail));
  length += tail;
  const bin = new Uint8Array(length);
  parts.reduce((at, p) => (bin.set(p, at), at + p.length), 0);
  const count = (N + 1) * (N + 1);
  const gltf = {
    asset: { version: "2.0" },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ mesh: 0 }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0, NORMAL: 1, TEXCOORD_0: 2 }, indices: 3, material: 0 }] }],
    materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 }, metallicFactor: 0, roughnessFactor: 1 } }],
    textures: [{ source: 0 }],
    images: [{ bufferView: 4, mimeType: "image/png" }],
    accessors: [
      { bufferView: 0, componentType: 5126, count, type: "VEC3", min: [-5, -5, -0.3], max: [5, 5, 0.3] },
      { bufferView: 1, componentType: 5126, count, type: "VEC3" },
      { bufferView: 2, componentType: 5126, count, type: "VEC2" },
      { bufferView: 3, componentType: 5125, count: idx.length, type: "SCALAR" },
    ],
    bufferViews: views,
    buffers: [{ byteLength: bin.length }],
  };
  let text = new TextEncoder().encode(JSON.stringify(gltf));
  const jpad = (4 - (text.length % 4)) % 4;
  if (jpad) text = new Uint8Array([...text, ...new Array(jpad).fill(0x20)]);
  const glb = new Uint8Array(12 + 8 + text.length + 8 + bin.length);
  const v = new DataView(glb.buffer);
  glb.set(new TextEncoder().encode("glTF"), 0);
  v.setUint32(4, 2, true);
  v.setUint32(8, glb.length, true);
  v.setUint32(12, text.length, true);
  glb.set(new TextEncoder().encode("JSON"), 16);
  glb.set(text, 20);
  v.setUint32(20 + text.length, bin.length, true);
  glb.set(new TextEncoder().encode("BIN\0"), 24 + text.length);
  glb.set(bin, 28 + text.length);
  return glb;
}

beforeAll(() => {
  loadNativeHost();
  rmSync(evidence, { recursive: true, force: true });
  for (const d of ["models", "textures", "audio", "scenes"]) mkdirSync(join(root, "assets", d), { recursive: true });
  writeFileSync(
    join(root, "project.ogt"),
    'type project\nversion 1\n---\nname "Asset stream fixture"\nid "asset-stream-fixture"\nengine "0.5.0"\nentry "main.ts"\nprofile "3d"\ntick 60\nseed 1\n',
  );
  writeFileSync(join(root, "main.ts"), MAIN);
  const colours: Array<[string, [number, number, number]]> = [
    ["hut-a", [230, 20, 20]],
    ["hut-b", [20, 230, 20]],
    ["hut-c", [20, 20, 230]],
  ];
  for (const [name, rgb] of colours) writeFileSync(join(root, "assets", "models", `${name}.glb`), texturedGlb(encodePng(solid(64, rgb))));
  writeFileSync(join(root, "assets", "textures", "sign.png"), encodePng(solid(256, [200, 160, 40])));
  copyFileSync(join(repo, "templates", "game", "assets", "audio", "blip.wav"), join(root, "assets", "audio", "blip.wav"));
  writeFileSync(join(root, "assets", "scenes", "path.ogt"), SCENE);
  const cooked = Bun.spawnSync([ogBin(), "cook", root]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Status = { name: string; state: string; wanted: boolean; requested: boolean; priority: number; trigger: string; memory: number; bytes: number; assets: string[]; reason: string; since: number; loads: number };
type Budget = { budget: number; used: number; peak: number; groups: Status[] };

const events: Array<{ tick: number; group: string; state: string }> = [];
const STATE = ["unloaded", "queued", "loading", "resident", "failed"];
const Watch = defineSystem("AsWatch", { reads: [og.StreamEvent] }, (_q, _t, world: World) => {
  for (const e of world.events(og.StreamEvent)) events.push({ tick: world.tick, group: String(e.group), state: STATE[Number(e.state)]! });
});

/** An engine on the staged project: the scene's entities and the three huts, one per group. */
function start(render: boolean): Engine {
  events.length = 0;
  const game = og.game({
    systems: [Watch],
    start(world: World) {
      world.instantiate(must(og.load.scene(world, "scenes/path.ogt")));
      for (const [name, z] of [["hut-a", -60], ["hut-b", -160], ["hut-c", -260]] as const) {
        const hut = must(og.load.gltf(world, `models/${name}.glb`));
        must(world.instantiate(hut, { name, position: { x: 0, y: 0, z } }));
      }
      must(og.load.texture(world, "textures/sign.png"));
      must(og.load.sound(world, "audio/blip.wav"));
      const sun = must(world.spawn({ name: "sun" }));
      world.insert(sun, og.DirectionalLight, { intensity: 2 });
      world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.3, y: -0.5, z: -1 }) });
    },
  });
  const r = createEngine(game as GameToken, { root });
  if (!r.ok) throw new Error(r.reason);
  if (render) {
    const attached = JSON.parse(native().og_node_attach_offscreen(r.value.id, W, H)) as { ok: boolean; reason?: string };
    if (!attached.ok) throw new Error(`no renderer: ${attached.reason}`);
  }
  place(r.value, 20);
  expect(r.value.advance(1).ok).toBeTrue();
  return r.value;
}

/** Puts the camera at z on the path, looking down -z. */
function place(engine: Engine, z: number): void {
  const cam = must(engine.world.resource(og.Camera));
  cam.position = { x: 0, y: 0, z };
  cam.quaternion = og.transform.lookAt({ x: 0, y: 0, z }, { x: 0, y: 0, z: z - 1 });
  cam.fov = 50;
  cam.far = 500;
}

const budget = (e: Engine) => must(og.stream.budget(e.world)) as unknown as Budget;
const states = (e: Engine) => Object.fromEntries(budget(e).groups.map((g) => [g.name, g.state]));
const memory = (e: Engine, name: string) => budget(e).groups.find((g) => g.name === name)!.memory;

/** The centre pixel's colour: "red", "green", "blue" or "grey" (no single channel leads). */
function hue(img: Image): string {
  const [r, g, b] = pixel(img, W / 2, H / 2);
  if (r > g + 60 && r > b + 60) return "red";
  if (g > r + 60 && g > b + 60) return "green";
  if (b > r + 60 && b > g + 60) return "blue";
  return "grey";
}

test("three groups load and unload along the path by trigger and priority, within the budget", () => {
  const engine = start(true);
  const w = engine.world;
  const shot = (name: string): string => {
    const frame = engine.probe.capture(W, H) as { png: string };
    writeFileSync(join(evidence, `${name}.png`), Uint8Array.from(atob(frame.png), (c) => c.charCodeAt(0)));
    return hue(decodeCapture(frame));
  };
  expect(states(engine)).toEqual({ cave: "unloaded", meadow: "unloaded", tower: "unloaded" });
  const groups = budget(engine).groups;
  expect(groups.map((g) => [g.name, g.trigger, g.priority])).toEqual([
    ["cave", "volume", 2],
    ["meadow", "distance", 1],
    ["tower", "distance", 0],
  ]);
  for (const g of groups) expect(g.memory).toBeGreaterThan(0);
  // meadow holds a 256 x 256 sign: it takes more than tower's lone hut.
  expect(memory(engine, "meadow")).toBeGreaterThan(memory(engine, "tower") + 256 * 256 * 4);
  // Room for meadow or cave, never both.
  const limit = memory(engine, "meadow") + memory(engine, "cave") - 1;
  must(og.stream.setBudget(w, limit));

  // Before its group is resident the meadow hut draws its untextured proxy.
  expect(shot("00-start-proxy")).toBe("grey");

  type Row = { tick: number; z: number; ms: number; used: number; meadow: string; cave: string; tower: string };
  const rows: Row[] = [];
  let z = 20;
  const frames: Record<number, string> = {};
  while (z > -320) {
    z -= 0.5;
    place(engine, z);
    const t0 = performance.now();
    expect(engine.advance(1).ok).toBeTrue();
    const ms = performance.now() - t0;
    const b = budget(engine);
    expect(b.used).toBeLessThan(limit + 1);
    const s = Object.fromEntries(b.groups.map((g) => [g.name, g.state]));
    rows.push({ tick: w.tick, z, ms, used: b.used, meadow: s.meadow!, cave: s.cave!, tower: s.tower! });
    // Each trigger, with hysteresis: meadow from 50 m to 110 m, cave in the box (z -180..-140).
    if (z > -10) expect(s.meadow).toBe("unloaded");
    if (z < -12 && z > -138) expect(s.meadow).toBe("resident");
    if (z < -142 && z > -168) {
      expect(s.cave).toBe("resident");
      expect(s.meadow).toBe("queued");
    }
    if (z < -172) expect(s.meadow).toBe("unloaded");
    if (z < -182) expect(s.cave).toBe("unloaded");
    if (z > -208) expect(s.tower).toBe("unloaded");
    if (z < -212) expect(s.tower).toBe("resident");
    if (z === -40) frames[z] = shot("01-meadow-resident");
    if (z === -110) frames[z] = shot("02-cave-proxy");
    if (z === -150) frames[z] = shot("03-cave-resident");
    if (z === -230) frames[z] = shot("04-tower-resident");
  }
  // The meadow hut is textured once resident; the cave hut, seen past the
  // meadow's (which the camera has passed) before its box, is its proxy.
  expect(frames).toEqual({ [-40]: "red", [-110]: "grey", [-150]: "green", [-230]: "blue" });

  const b = budget(engine);
  const sorted = rows.map((r) => r.ms).sort((x, y) => x - y);
  const at = (q: number) => sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))]!;
  writeFileSync(
    join(evidence, "run.json"),
    JSON.stringify({ budget: limit, peak: b.peak, memory: Object.fromEntries(b.groups.map((g) => [g.name, g.memory])), ticks: rows.length, advanceMs: { p50: at(0.5), p95: at(0.95), p99: at(0.99), max: sorted.at(-1) }, events, rows }, null, 1),
  );
  expect(b.peak).toBeLessThan(limit + 1);
  expect(b.peak).toBe(memory(engine, "meadow"));
  // Each state change once, in order, at the start of the tick after it.
  const seen = events.map((e) => `${e.group}:${e.state}`);
  expect(seen).toEqual([
    "meadow:loading",
    "meadow:resident",
    "meadow:queued",
    "cave:loading",
    "cave:resident",
    "meadow:unloaded",
    "cave:unloaded",
    "tower:loading",
    "tower:resident",
  ]);

  // og.stream.request loads a group no trigger wants; release lets it go.
  must(og.stream.request(w, "cave"));
  expect(engine.advance(1).ok).toBeTrue();
  expect(states(engine)).toEqual({ cave: "resident", meadow: "unloaded", tower: "resident" });
  expect((must(og.stream.status(w, "cave")) as unknown as Status).requested).toBeTrue();
  must(og.stream.release(w, "cave"));
  expect(engine.advance(1).ok).toBeTrue();
  expect(states(engine).cave).toBe("unloaded");

  // A lowered budget evicts what no longer fits; a group bigger than the
  // budget fails with the reason instead of waiting.
  must(og.stream.setBudget(w, memory(engine, "tower") - 1));
  expect(engine.advance(1).ok).toBeTrue();
  const tower = must(og.stream.status(w, "tower")) as unknown as Status;
  expect([tower.state, budget(engine).used]).toEqual(["failed", 0]);
  expect(tower.reason).toContain("more than the budget");
  must(og.stream.setBudget(w, limit));
  expect(engine.advance(1).ok).toBeTrue();
  expect(states(engine).tower).toBe("resident");
}, 120_000);

test("dump_stream lists the budget, the peak, every group and the events; og.stream refuses an unknown group", () => {
  const engine = start(false);
  place(engine, -30);
  expect(engine.advance(2).ok).toBeTrue();
  engine.probe.commands("0:dump_stream:stream.json");
  expect(engine.advance(1).ok).toBeTrue();
  const out = engine.takeOutputs().find((o) => o.file === "stream.json")!;
  const dump = JSON.parse(out.data) as Omit<Budget, "groups"> & { groups: Array<Omit<Status, "trigger"> & { file: string; slots: number; trigger: { kind: string } }>; events: unknown[] };
  expect(dump.budget).toBe(256 * 1024 * 1024);
  expect(dump.used).toBe(dump.groups.find((g) => g.name === "meadow")!.memory);
  expect(dump.peak).toBe(dump.used);
  expect(dump.groups.map((g) => [g.name, g.file, g.slots, g.state])).toEqual([
    ["cave", "stream/cave.ogb", 2, "unloaded"],
    ["meadow", "stream/meadow.ogb", 2, "resident"],
    ["tower", "stream/tower.ogb", 1, "unloaded"],
  ]);
  expect(dump.groups.find((g) => g.name === "meadow")!.trigger).toEqual({ kind: "distance", anchor: [0, 0, -60], radius: 50, unload: 110 });
  expect(dump.events.length).toBe(2);
  const r = og.stream.status(engine.world, "nowhere") as unknown as { ok: boolean; reason: string };
  expect(r.ok).toBeFalse();
  expect(r.reason).toContain("the scenes declare cave, meadow, tower");
});

test("what is resident never changes the simulation: the same flight hashes the same with no budget at all", () => {
  const fly = (limit: number | undefined): string => {
    const engine = start(false);
    if (limit !== undefined) must(og.stream.setBudget(engine.world, limit));
    for (let z = 20; z > -300; z -= 1) {
      place(engine, z);
      expect(engine.advance(1).ok).toBeTrue();
    }
    const resident = budget(engine).groups.filter((g) => g.state === "resident").length;
    expect(resident).toBe(limit === 0 ? 0 : 1);
    return engine.hash();
  };
  expect(fly(0)).toBe(fly(undefined));
});

test("og build --publish ships each group as its pack beside the first group, and precaches only the first group", async () => {
  // What each group takes once loaded, from the runtime, to hold the build's manifest to.
  const engine = start(false);
  const runtime = Object.fromEntries(budget(engine).groups.map((g) => [g.name, g.memory]));
  const dist = join(evidence, "publish");
  const settings = { ...settingsOf(root), publish: { ...DEFAULT_PUBLISH, gate: "none" as const } };
  const b = await webBuild(root, join(root, "main.ts"), dist, true, settings);
  if (!b.ok) throw new Error(b.reason);
  const sha = (f: string) => {
    const h = new Bun.CryptoHasher("sha256");
    h.update(new Uint8Array(readFileSync(join(dist, f))));
    return h.digest("hex");
  };
  // A grouped output is in its pack only; the proxies, the scene and the manifest stay in the first group.
  const grouped = ["models/hut-a.glb.ogb", "models/hut-a.glb.tex0.ogb", "textures/sign.png.ogb", "models/hut-b.glb.ogb", "models/hut-b.glb.tex0.ogb", "audio/blip.wav.ogb", "models/hut-c.glb.ogb", "models/hut-c.glb.tex0.ogb"];
  for (const f of grouped) expect([f, existsSync(join(dist, "cooked", f))]).toEqual([f, false]);
  const first = [".cook.ogt", "models/hut-a.glb.proxy.ogb", "models/hut-b.glb.proxy.ogb", "models/hut-c.glb.proxy.ogb", "scenes/path.ogt.ogb"];
  for (const f of first) expect([f, existsSync(join(dist, "cooked", f))]).toEqual([f, true]);
  const packs = ["cooked/stream/cave.ogb", "cooked/stream/meadow.ogb", "cooked/stream/tower.ogb"];
  // og-build.json lists the groups' packs with their bytes, hashes and what they take.
  const m = JSON.parse(readFileSync(join(dist, "og-build.json"), "utf8")) as { files: Array<{ path: string }>; stream: { first: { files: number; bytes: number }; groups: Array<{ name: string; file: string; bytes: number; sha256: string; priority: number; memory: number; assets: string[] }> } };
  expect(m.stream.groups.map((g) => [g.name, g.file, g.priority, g.memory, g.assets])).toEqual([
    ["cave", packs[0], 2, runtime.cave, ["models/hut-b.glb", "audio/blip.wav"]],
    ["meadow", packs[1], 1, runtime.meadow, ["models/hut-a.glb", "textures/sign.png"]],
    ["tower", packs[2], 0, runtime.tower, ["models/hut-c.glb"]],
  ]);
  for (const g of m.stream.groups) expect([g.file, g.bytes, g.sha256]).toEqual([g.file, statSync(join(dist, g.file)).size, sha(g.file)]);
  for (const p of packs) expect(m.files.some((f) => f.path === p)).toBeTrue();
  const firstBytes = first.reduce((n, f) => n + statSync(join(dist, "cooked", f)).size, 0);
  expect(m.stream.first).toEqual({ files: first.length, bytes: firstBytes });
  // The service worker precaches the first group and caches a pack when it is fetched.
  const sw = readFileSync(join(dist, "sw.js"), "utf8");
  const config = JSON.parse(/^const OG_SW=(.*);$/m.exec(sw)![1]!) as { files: string[]; lazy: string[] };
  const cooked = config.files.filter((f) => f.startsWith("cooked/")).sort();
  expect(cooked).toEqual(first.map((f) => `cooked/${f}`).sort());
  expect(config.lazy).toEqual(packs);
  // The shell's boot progress expects the first group's bytes only (the manifest is not counted).
  const html = readFileSync(join(dist, "index.html"), "utf8");
  expect(html).toContain(`"cooked":${firstBytes - statSync(join(dist, "cooked", ".cook.ogt")).size}`);
  writeFileSync(join(evidence, "publish.json"), JSON.stringify({ firstGroup: m.stream.first, groups: m.stream.groups, precachedCooked: cooked, lazy: config.lazy, total: b.total }, null, 1) + "\n");
}, 120_000);
