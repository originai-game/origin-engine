// Packs the generated models into assets/models/: each raw GLB (a unit
// model centred on its origin, 4K PNG and JPEG maps, 14 000 to 83 000
// triangles) becomes a game-ready GLB at its true size standing on its
// origin, facing +z: its mesh simplified to the triangles its place in the
// frame needs and meshopt-compressed, its maps WebP at the size that place
// needs. The results are not in git: assets.json pins them
// (scripts/showcase-assets.ts) and README.md lists each one's source.
//
//   bun tools/models.ts <directory of raw .glb files> [name...]
//
// Needs the network once, for glTF Transform (the version pinned below).
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createHash } from "node:crypto";

const GLTF_TRANSFORM = "@gltf-transform/cli@4.5.1";

/** Each model: the metres its longest side spans, its turn about y in degrees (so its front faces +z), its maps' longest side in pixels and its triangles. The shepherd and the dog stand before the camera; everything else is 25 m away or more. */
const MODELS: Record<string, { size: number; turn: number; map: number; triangles: number; centre?: true }> = {
  shepherd: { size: 1.52, turn: 0, map: 2048, triangles: 40000 },
  dog: { size: 0.74, turn: 0, map: 2048, triangles: 25000 },
  "sheep-stand": { size: 1.25, turn: 0, map: 512, triangles: 5000 },
  "sheep-graze": { size: 1.3, turn: 0, map: 512, triangles: 5000 },
  "house-a": { size: 11.5, turn: 0, map: 1024, triangles: 20000 },
  "house-b": { size: 7.6, turn: 0, map: 1024, triangles: 15000 },
  "house-c": { size: 15, turn: 0, map: 1024, triangles: 20000 },
  "windmill-tower": { size: 14, turn: 0, map: 1024, triangles: 15000 },
  // The sails turn about their hub: their origin is their middle.
  "windmill-sails": { size: 13, turn: 0, map: 512, triangles: 6000, centre: true },
  jetty: { size: 14, turn: 0, map: 1024, triangles: 12000 },
  boat: { size: 3.8, turn: 0, map: 512, triangles: 6000 },
  gate: { size: 2.5, turn: 0, map: 512, triangles: 3000 },
  wall: { size: 4, turn: 0, map: 512, triangles: 4000 },
  boulder: { size: 2.2, turn: 0, map: 1024, triangles: 8000 },
  mushrooms: { size: 0.36, turn: 0, map: 512, triangles: 5000 },
};

const source = process.argv[2];
if (!source) throw new Error("usage: bun tools/models.ts <directory of raw .glb files>");
const out = join(import.meta.dir, "..", "assets", "models");
mkdirSync(out, { recursive: true });
const scratch = mkdtempSync(join(tmpdir(), "first-light-models-"));

type Gltf = {
  scenes: { nodes: number[] }[];
  scene?: number;
  nodes: Record<string, unknown>[];
  meshes: { primitives: { attributes: { POSITION: number }; indices?: number }[] }[];
  accessors: { min?: number[]; max?: number[]; count?: number }[];
  bufferViews: { buffer: number; byteOffset?: number; byteLength: number }[];
  buffers: { byteLength: number }[];
  images?: { bufferView?: number; mimeType?: string }[];
};

function pack(name: string, spec: (typeof MODELS)[string]): string {
  const raw = readFileSync(join(source!, `${name}.glb`));
  const view = new DataView(raw.buffer, raw.byteOffset, raw.byteLength);
  if (view.getUint32(0, true) !== 0x46546c67) throw new Error(`${name}: not a GLB`);
  const jsonLength = view.getUint32(12, true);
  const gltf = JSON.parse(new TextDecoder().decode(raw.subarray(20, 20 + jsonLength))) as Gltf;
  const binStart = 20 + jsonLength + 8;
  const bin = raw.subarray(binStart, binStart + view.getUint32(20 + jsonLength, true));

  // Lay the buffer out again.
  const parts: Uint8Array[] = [];
  let length = 0;
  for (const v of gltf.bufferViews) {
    const bytes = bin.subarray(v.byteOffset ?? 0, (v.byteOffset ?? 0) + v.byteLength);
    v.byteOffset = length;
    v.byteLength = bytes.length;
    parts.push(bytes);
    length += bytes.length;
    const pad = (4 - (length % 4)) % 4;
    if (pad) parts.push(new Uint8Array(pad));
    length += pad;
  }
  gltf.buffers = [{ byteLength: length }];

  // A root that sizes the model, stands it on its origin and turns it.
  const low = [Infinity, Infinity, Infinity], high = [-Infinity, -Infinity, -Infinity];
  for (const mesh of gltf.meshes) for (const p of mesh.primitives) {
    const a = gltf.accessors[p.attributes.POSITION]!;
    for (let k = 0; k < 3; k++) {
      low[k] = Math.min(low[k]!, a.min![k]!);
      high[k] = Math.max(high[k]!, a.max![k]!);
    }
  }
  const scale = spec.size / Math.max(high[0]! - low[0]!, high[1]! - low[1]!, high[2]! - low[2]!);
  const half = (spec.turn * Math.PI) / 360, sin = Math.sin(half), cos = Math.cos(half);
  const cx = (low[0]! + high[0]!) / 2, cz = (low[2]! + high[2]!) / 2;
  const turned = { x: cx * (cos * cos - sin * sin) + cz * 2 * sin * cos, z: -cx * 2 * sin * cos + cz * (cos * cos - sin * sin) };
  const scene = gltf.scenes[gltf.scene ?? 0]!;
  gltf.nodes.push({ name, children: scene.nodes, scale: [scale, scale, scale], rotation: [0, sin, 0, cos], translation: [-turned.x * scale, -(spec.centre ? (low[1]! + high[1]!) / 2 : low[1]!) * scale, -turned.z * scale] });
  scene.nodes = [gltf.nodes.length - 1];

  let json = new TextEncoder().encode(JSON.stringify(gltf));
  if (json.length % 4) {
    const padded = new Uint8Array(json.length + 4 - (json.length % 4)).fill(0x20);
    padded.set(json);
    json = padded;
  }
  const glb = new Uint8Array(12 + 8 + json.length + 8 + length), o = new DataView(glb.buffer);
  o.setUint32(0, 0x46546c67, true);
  o.setUint32(4, 2, true);
  o.setUint32(8, glb.length, true);
  o.setUint32(12, json.length, true);
  o.setUint32(16, 0x4e4f534a, true);
  glb.set(json, 20);
  o.setUint32(20 + json.length, length, true);
  o.setUint32(24 + json.length, 0x004e4942, true);
  let at = 28 + json.length;
  for (const p of parts) {
    glb.set(p, at);
    at += p.length;
  }
  const sized = join(scratch, `${name}.glb`), packed = join(out, `${name}.glb`);
  writeFileSync(sized, glb);
  // Simplify, compress and re-encode: the simplifier stops at the triangles asked for or at an error of 2 % of the model's size.
  const triangles = gltf.meshes.reduce((n, mesh) => n + mesh.primitives.reduce((m, p) => m + (p.indices === undefined ? 0 : gltf.accessors[p.indices]!.count! / 3), 0), 0);
  const run = Bun.spawnSync(["bunx", "--bun", GLTF_TRANSFORM, "optimize", sized, packed, "--compress", "meshopt", "--texture-compress", "webp", "--texture-size", String(spec.map), "--simplify-ratio", Math.min(1, spec.triangles / triangles).toFixed(4), "--simplify-error", "0.02"]);
  if (run.exitCode !== 0) throw new Error(`${name}: glTF Transform: ${run.stderr.toString()}`);
  const bytes = readFileSync(packed);
  const size = [0, 1, 2].map(k => ((high[k]! - low[k]!) * scale).toFixed(2)).join(" x ");
  return `${name}.glb  ${size} m  ${(bytes.length / 1e6).toFixed(2)} MB  sha256 ${createHash("sha256").update(bytes).digest("hex")}`;
}

const only = process.argv.slice(3);
for (const [name, spec] of Object.entries(MODELS)) if (!only.length || only.includes(name)) console.log(pack(name, spec));
rmSync(scratch, { recursive: true });
