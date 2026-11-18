// Bundles og.load.bundle opens, streamed on both hosts: a game whose own
// scene places a cell `home` with a cell group, and two parcels cooked as
// projects of their own (`a-v1` and `a-v2`, two versions of one parcel
// open side by side), each placing a cell whose sub-scene declares a cell
// group with a textured model. An anchor walks from cell to cell
// (tests/support/runtime-bundles.ts, one host per process); at each stop
// the cell's group must be resident with its textures ready
// (StreamStatus.ready) and hold its own group alone in the budget, and
// nothing once every cell has unloaded.
//
// Ways this could fail: on the web, a runtime bundle's pack has its Basis
// textures looked up in the game's manifest, so none is transcoded and its
// group fails (og-host-web has no transcoder of its own); the game's pack
// is looked up in a parcel's manifest after a parcel's pack was handled
// (the packs of three bundles are fetched in turn); a group's budget holds
// another bundle's group, or stays up after its cell unloads; the two
// hosts disagree on any of it; on the web, a Basis texture of a runtime
// bundle in no group (a-v1's sign) is not transcoded at all, as neither
// the boot nor a pack holds it, so its model does not load; a runtime
// bundle's files are fetched without its manifest's integrity, so a
// changed byte loads as if intact (or against another bundle's, so intact
// bytes are refused).
import { beforeAll, expect, test } from "bun:test";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { encodePng } from "../src/node/png.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const staged = join(pkg, "..", "..", "target", "evidence", "runtime-bundles");
const game = join(staged, "game");

/** A 256 x 256 seeded noise, so the cook keeps it big enough to Basis-code. */
function noise(seed: number): Uint8Array {
  const size = 256;
  const rgba = new Uint8Array(size * size * 4);
  let s = (seed * 2654435761) >>> 0 || 1;
  for (let i = 0; i < size * size; i++) {
    s ^= s << 13;
    s >>>= 0;
    s ^= s >>> 17;
    s ^= s << 5;
    s >>>= 0;
    rgba.set([s & 255, (s >>> 8) & 255, (s >>> 16) & 255, 255], i * 4);
  }
  return encodePng({ width: size, height: size, rgba });
}

/** A glTF quad (10 m, 16 x 16 cells, so its proxy is coarser) whose material samples `png`. */
function texturedGlb(png: Uint8Array): Uint8Array {
  const N = 16;
  const pos: number[] = [];
  const uv: number[] = [];
  const idx: number[] = [];
  for (let z = 0; z <= N; z++) {
    for (let x = 0; x <= N; x++) {
      pos.push((x / N) * 10 - 5, 0.2 * Math.sin(x) * Math.cos(z), (z / N) * 10 - 5);
      uv.push(x / N, z / N);
    }
  }
  for (let z = 0; z < N; z++) {
    for (let x = 0; x < N; x++) {
      const i = z * (N + 1) + x;
      idx.push(i, i + N + 1, i + 1, i + 1, i + N + 1, i + N + 2);
    }
  }
  const parts = [new Uint8Array(new Float32Array(pos).buffer), new Uint8Array(new Float32Array(uv).buffer), new Uint8Array(new Uint32Array(idx).buffer), png];
  const views: Array<{ buffer: number; byteOffset: number; byteLength: number }> = [];
  let length = 0;
  for (const p of parts) {
    length = Math.ceil(length / 4) * 4;
    views.push({ buffer: 0, byteOffset: length, byteLength: p.length });
    length += p.length;
  }
  length = Math.ceil(length / 4) * 4;
  const bin = new Uint8Array(length);
  parts.forEach((p, i) => bin.set(p, views[i]!.byteOffset));
  const count = (N + 1) * (N + 1);
  const json = new TextEncoder().encode(
    JSON.stringify({
      asset: { version: "2.0" },
      scene: 0,
      scenes: [{ nodes: [0] }],
      nodes: [{ mesh: 0 }],
      meshes: [{ primitives: [{ attributes: { POSITION: 0, TEXCOORD_0: 1 }, indices: 2, material: 0 }] }],
      materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 } } }],
      textures: [{ source: 0 }],
      images: [{ bufferView: 3, mimeType: "image/png" }],
      accessors: [
        { bufferView: 0, componentType: 5126, count, type: "VEC3", min: [-5, -0.2, -5], max: [5, 0.2, 5] },
        { bufferView: 1, componentType: 5126, count, type: "VEC2" },
        { bufferView: 2, componentType: 5125, count: idx.length, type: "SCALAR" },
      ],
      bufferViews: views,
      buffers: [{ byteLength: bin.length }],
    }),
  );
  const text = new Uint8Array(Math.ceil(json.length / 4) * 4).fill(0x20);
  text.set(json);
  const glb = new Uint8Array(12 + 8 + text.length + 8 + bin.length);
  const v = new DataView(glb.buffer);
  v.setUint32(0, 0x46546c67, true);
  v.setUint32(4, 2, true);
  v.setUint32(8, glb.length, true);
  v.setUint32(12, text.length, true);
  v.setUint32(16, 0x4e4f534a, true);
  glb.set(text, 20);
  v.setUint32(20 + text.length, bin.length, true);
  v.setUint32(24 + text.length, 0x004e4942, true);
  glb.set(bin, 28 + text.length);
  return glb;
}

/** A sub-scene `<id>/content.ogt` whose cell group `<id>` holds `<id>/house.glb`, and the house in it. */
function content(id: string): string {
  return `type scene\nversion 1\n---\nname "${id} content"\nstream "${id}" {\n  cell\n  asset "${id}/house.glb"\n}\nentity "house-${id}" {\n  model "${id}/house.glb"\n}\n`;
}

/** The cell `cell-<id>` placing `<id>/content.ogt` at `x` (a 20 m box, loaded within 10 m of it). */
function cell(id: string, x: number): string {
  return `entity "cell-${id}" {\n  scene "${id}/content.ogt"\n  Transform {\n    position ${x} 0 0\n  }\n  Volume {\n    size 20 10 20\n  }\n  StreamCell {\n    load 10\n  }\n}\n`;
}

/** Writes and cooks the project at `dir` from `files` (paths under assets/). */
function cook(dir: string, id: string, files: Record<string, Uint8Array | string>): void {
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "project.ogt"), `type project\nversion 1\n---\nname "${id}"\nid "${id}"\nengine "0.8.0"\nentry "main.ts"\nprofile "3d"\ntick 60\nseed 1\n`);
  writeFileSync(join(dir, "main.ts"), "export {};\n");
  for (const [path, bytes] of Object.entries(files)) {
    mkdirSync(dirname(join(dir, "assets", path)), { recursive: true });
    writeFileSync(join(dir, "assets", path), bytes);
  }
  const r = Bun.spawnSync([ogBin(), "cook", dir]);
  if (r.exitCode !== 0) throw new Error(`cooking ${dir}: ${new TextDecoder().decode(r.stderr)}`);
}

/** The memory the cook measured for group `name` of the bundle at `dir` (its manifest's `memory`). */
function memoryOf(dir: string, name: string): number {
  const manifest = readFileSync(join(dir, "cooked", ".cook.ogt"), "utf8");
  const m = new RegExp(`\\nstream "${name}" \\{[^}]*?\\n  memory (\\d+)`).exec(manifest);
  if (!m) throw new Error(`no group ${name} with a memory in ${dir}`);
  return Number(m[1]);
}

beforeAll(() => {
  rmSync(staged, { recursive: true, force: true });
  const world = `type scene\nversion 1\n---\nname "World"\nentity "player" {\n  Transform {\n    position 0 0 0\n  }\n  StreamAnchor {\n    reach 0\n  }\n}\n${cell("home", -100)}`;
  cook(game, "runtime-bundles-game", { "scenes/world.ogt": world, "home/content.ogt": content("home"), "home/house.glb": texturedGlb(noise(1)) });
  for (const [id, x, seed] of [["a-v1", 100, 2], ["a-v2", 300, 3]] as const) {
    // a-v1 also holds a sign in no group: fetched and decoded when loaded, as a bundle's first group is.
    const sign = id === "a-v1" ? { "a-v1/sign.glb": texturedGlb(noise(4)) } : {};
    cook(join(game, "parcels", id), `runtime-bundles-${id}`, { [`${id}/content.ogt`]: content(id), [`${id}/house.glb`]: texturedGlb(noise(seed)), [`${id}/place.ogt`]: `type scene\nversion 1\n---\nname "${id} place"\n${cell(id, x)}`, ...sign });
  }
}, 120_000);

type Run = { host: string; stops: Array<{ at: string; group?: { state: string; reason: string; ready: boolean }; used: number }> };

function run<T = Run>(host: string, flags: string[] = [], dir = staged): T {
  const r = Bun.spawnSync(["bun", "tests/support/runtime-bundles.ts", "--host", host, "--dir", dir, ...flags], { cwd: pkg });
  const out = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(`${host} host: ${new TextDecoder().decode(r.stderr)}${out}`);
  return JSON.parse(out.trim().split("\n").pop()!) as T;
}

/** What each stop must find: its own group resident, alone in the budget; nothing away from every cell. */
function expected(host: string): Run {
  const own = { "a-v1": join(game, "parcels", "a-v1"), "a-v2": join(game, "parcels", "a-v2"), home: game } as const;
  return {
    host,
    stops: [
      ...(["a-v1", "a-v2", "home"] as const).map((at) => ({ at, group: { state: "resident", reason: "", ready: true }, used: memoryOf(own[at], at) })),
      { at: "away", used: 0 },
    ],
  };
}

test("a runtime bundle's cell group streams on the native host: resident with its cell, alone in the budget, let go after", () => {
  expect(run("node")).toEqual(expected("node"));
}, 120_000);

test("on the web host each bundle's pack has its Basis textures transcoded against its own manifest, so its group is resident as natively", () => {
  expect(run("web")).toEqual(expected("web"));
}, 120_000);

test("a runtime bundle's Basis texture in no group loads on both hosts: on the web the Worker transcodes it once asked, and its model waits for it", () => {
  for (const host of ["node", "web"]) expect(run<{ host: string; sign: unknown }>(host, ["--sign"])).toEqual({ host, sign: { loaded: true, prepared: true } });
}, 120_000);

test("on the web host a runtime bundle's file whose bytes differ from its manifest's integrity fails its load, naming the integrity", () => {
  const tampered = `${staged}-tampered`;
  rmSync(tampered, { recursive: true, force: true });
  cpSync(staged, tampered, { recursive: true });
  // One byte of a-v1's place, in its scene's name: unchecked, it loads as if intact.
  const file = join(tampered, "game", "parcels", "a-v1", "cooked", "a-v1", "place.ogt.ogb");
  const bytes = new Uint8Array(readFileSync(file));
  bytes[new TextDecoder("latin1").decode(bytes).indexOf("a-v1 place") + "a-v1 plac".length] ^= 1;
  writeFileSync(file, bytes);
  expect(() => run("web", [], tampered)).toThrow(/cannot fetch game\/parcels\/a-v1\/cooked\/a-v1\/place\.ogt\.ogb \(refused: the network, CORS or its integrity \(sha384-/);
}, 120_000);
