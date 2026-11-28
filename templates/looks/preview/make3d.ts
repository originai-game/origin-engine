// Writes the preview's textured crate and rolling meadow from sources/.
//   bun templates/looks/preview/make3d.ts [preview directory]
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Gltf, bounds } from "../../../scripts/gltf.ts";

const HERE = process.argv[2] ?? import.meta.dir;
function glb(path: string, pos: number[][], nrm: number[][], uv: number[][], idx: number[], image: Uint8Array): void {
  const g = new Gltf(),
    nv = pos.length;
  g.add(pos.flat(), "f", nv, 5126, "VEC3", bounds(pos.flat()));
  g.add(nrm.flat(), "f", nv, 5126, "VEC3");
  g.add(uv.flat(), "f", nv, 5126, "VEC2");
  g.add(idx, "I", idx.length, 5125, "SCALAR");
  const im = g.bytes(image);
  g.write(path, {
    asset: { version: "2.0", generator: "templates/looks/preview/make3d.ts" },
    images: [{ bufferView: im, mimeType: "image/jpeg" }],
    samplers: [{ wrapS: 10497, wrapT: 10497 }],
    textures: [{ source: 0, sampler: 0 }],
    materials: [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 }, metallicFactor: 0, roughnessFactor: 0.9 } }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0, NORMAL: 1, TEXCOORD_0: 2 }, indices: 3, material: 0 }] }],
    nodes: [{ mesh: 0 }],
    scenes: [{ nodes: [0] }],
    scene: 0,
  });
}
function crate(): void {
  const faces = [
    [
      [1, 0, 0],
      [0, 0, -1],
      [0, 1, 0],
    ],
    [
      [-1, 0, 0],
      [0, 0, 1],
      [0, 1, 0],
    ],
    [
      [0, 1, 0],
      [1, 0, 0],
      [0, 0, -1],
    ],
    [
      [0, -1, 0],
      [1, 0, 0],
      [0, 0, 1],
    ],
    [
      [0, 0, 1],
      [1, 0, 0],
      [0, 1, 0],
    ],
    [
      [0, 0, -1],
      [-1, 0, 0],
      [0, 1, 0],
    ],
  ];
  const pos: number[][] = [],
    nrm: number[][] = [],
    uv: number[][] = [],
    idx: number[] = [];
  for (const [n, u, v] of faces) {
    const base = pos.length;
    for (const [a, b, s, t] of [
      [-1, -1, 0, 1],
      [1, -1, 1, 1],
      [1, 1, 1, 0],
      [-1, 1, 0, 0],
    ]) {
      pos.push([0, 1, 2].map((k) => 0.6 * (n![k]! + a! * u![k]! + b! * v![k]!) + (k === 1 ? 0.6 : 0)));
      nrm.push(n!);
      uv.push([s!, t!]);
    }
    idx.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }
  glb(join(HERE, "3d/assets/models/crate.glb"), pos, nrm, uv, idx, readFileSync(join(HERE, "sources/crate.jpg")));
}
function height(x: number, z: number): number {
  const r = Math.hypot(x, z + 1);
  let rise = Math.min(1, Math.max(0, (r - 5) / 7));
  rise = rise * rise * (3 - 2 * rise);
  const swell =
    0.55 * Math.sin(0.31 * x + 0.7) * Math.cos(0.27 * z - 0.4) +
    0.3 * Math.sin(0.63 * x - 0.52 * z + 1.9) +
    0.15 * Math.cos(1.1 * x + 0.9 * z);
  return rise * Math.min(1, Math.max(0, (z + 22) / 8)) * (0.8 + 0.8 * swell);
}
function ground(): void {
  const cells = 10,
    n = cells + 1,
    pos: number[][] = [],
    uv: number[][] = [],
    idx: number[] = [];
  for (let j = 0; j < n; j++)
    for (let i = 0; i < n; i++) {
      const x = -30 + (60 * i) / cells,
        z = -30 + (60 * j) / cells;
      pos.push([x, height(x, z), z]);
      uv.push([x / 4, z / 4]);
    }
  for (let j = 0; j < cells; j++)
    for (let i = 0; i < cells; i++) {
      const a = j * n + i;
      idx.push(a, a + n, a + 1, a + 1, a + n, a + n + 1);
    }
  const e = 0.05,
    nrm = pos.map(([x, , z]) => {
      const dx = (height(x! + e, z!) - height(x! - e, z!)) / (2 * e),
        dz = (height(x!, z! + e) - height(x!, z! - e)) / (2 * e),
        l = Math.sqrt(dx * dx + 1 + dz * dz);
      return [-dx / l, 1 / l, -dz / l];
    });
  glb(join(HERE, "3d/assets/models/ground.glb"), pos, nrm, uv, idx, readFileSync(join(HERE, "sources/grass.jpg")));
}
crate();
ground();
