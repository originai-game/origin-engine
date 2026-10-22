// A small glTF 2.0 binary (.glb) writer for tools/make-assets.ts: meshes of
// triangles with flat-coloured PBR materials, one node per mesh. It writes the
// same bytes for the same calls on every machine (no clock, no randomness).

export type V3 = [number, number, number];

export interface Part {
  /** xyz per vertex. */
  positions: number[];
  /** xyz per vertex, unit length. */
  normals: number[];
  indices: number[];
  /** Index into the file's materials. */
  material: number;
}

export interface Mat {
  name: string;
  color: [number, number, number];
  roughness: number;
  metalness?: number;
}

/** A part being built: vertices with normals, and triangles. */
export class Builder {
  readonly positions: number[] = [];
  readonly normals: number[] = [];
  readonly indices: number[] = [];
  constructor(readonly material: number) {}

  vertex(p: V3, n: V3): number {
    const l = Math.hypot(n[0], n[1], n[2]) || 1;
    this.positions.push(p[0], p[1], p[2]);
    this.normals.push(n[0] / l, n[1] / l, n[2] / l);
    return this.positions.length / 3 - 1;
  }

  tri(a: number, b: number, c: number): void {
    this.indices.push(a, b, c);
  }

  quad(a: number, b: number, c: number, d: number): void {
    this.indices.push(a, b, c, a, c, d);
  }

  part(): Part {
    return { positions: this.positions, normals: this.normals, indices: this.indices, material: this.material };
  }
}

/**
 * A ring band of a surface of revolution about +y: from radius r0 at height y0
 * to r1 at y1, `segments` around, smooth normals from the profile's slope
 * (counter-clockwise seen from outside, so the faces point out).
 */
export function band(b: Builder, r0: number, y0: number, r1: number, y1: number, segments: number, n0?: V3, n1?: V3, wobble?: (angle: number) => number): void {
  const dy = y1 - y0;
  const dr = r1 - r0;
  // The outward normal of the profile line (dr, dy) is (dy, -dr).
  const base: [number, number] = [dy, -dr];
  const first = b.positions.length / 3;
  for (let i = 0; i <= segments; i++) {
    const a = (i / segments) * Math.PI * 2;
    const c = Math.cos(a), s = Math.sin(a);
    const w = wobble ? wobble(a) : 1;
    const na: V3 = n0 ? [n0[0] * c, n0[1], n0[0] * s] : [base[0] * c, base[1], base[0] * s];
    const nb: V3 = n1 ? [n1[0] * c, n1[1], n1[0] * s] : [base[0] * c, base[1], base[0] * s];
    b.vertex([r0 * w * c, y0, r0 * w * s], na);
    b.vertex([r1 * w * c, y1, r1 * w * s], nb);
  }
  for (let i = 0; i < segments; i++) {
    const v = first + i * 2;
    // Seen from outside: bottom i, bottom i+1, top i+1, top i.
    b.quad(v, v + 1, v + 3, v + 2);
  }
}

/** A flat disc facing +y (or -y) at height y. */
export function disc(b: Builder, r: number, y: number, segments: number, up = true, wobble?: (angle: number) => number): void {
  const centre = b.vertex([0, y, 0], [0, up ? 1 : -1, 0]);
  const first = b.positions.length / 3;
  for (let i = 0; i <= segments; i++) {
    const a = (i / segments) * Math.PI * 2;
    const w = wobble ? wobble(a) : 1;
    b.vertex([r * w * Math.cos(a), y, r * w * Math.sin(a)], [0, up ? 1 : -1, 0]);
  }
  for (let i = 0; i < segments; i++) {
    if (up) b.tri(centre, first + i + 1, first + i);
    else b.tri(centre, first + i, first + i + 1);
  }
}

/** A flat ring facing +y between radii r0 < r1 at height y. */
export function ring(b: Builder, r0: number, r1: number, y: number, segments: number, wobble?: (angle: number) => number): void {
  const first = b.positions.length / 3;
  for (let i = 0; i <= segments; i++) {
    const a = (i / segments) * Math.PI * 2;
    const w = wobble ? wobble(a) : 1;
    const c = Math.cos(a), s = Math.sin(a);
    b.vertex([r0 * w * c, y, r0 * w * s], [0, 1, 0]);
    b.vertex([r1 * w * c, y, r1 * w * s], [0, 1, 0]);
  }
  for (let i = 0; i < segments; i++) {
    const v = first + i * 2;
    b.quad(v, v + 2, v + 3, v + 1);
  }
}

/** A box with bevelled edges (a chamfer of `bevel`), centred on `at`, flat-shaded faces. */
export function bevelBox(b: Builder, size: V3, bevel: number, at: V3 = [0, 0, 0]): void {
  const [hx, hy, hz] = [size[0] / 2, size[1] / 2, size[2] / 2];
  const e = bevel;
  // The six faces, each inset by the bevel.
  const face = (n: V3, corners: V3[]) => {
    const vs = corners.map((c) => b.vertex([c[0] + at[0], c[1] + at[1], c[2] + at[2]], n));
    b.quad(vs[0]!, vs[1]!, vs[2]!, vs[3]!);
  };
  face([0, 1, 0], [[-hx + e, hy, -hz + e], [-hx + e, hy, hz - e], [hx - e, hy, hz - e], [hx - e, hy, -hz + e]]);
  face([0, -1, 0], [[-hx + e, -hy, -hz + e], [hx - e, -hy, -hz + e], [hx - e, -hy, hz - e], [-hx + e, -hy, hz - e]]);
  face([1, 0, 0], [[hx, -hy + e, -hz + e], [hx, hy - e, -hz + e], [hx, hy - e, hz - e], [hx, -hy + e, hz - e]]);
  face([-1, 0, 0], [[-hx, -hy + e, -hz + e], [-hx, -hy + e, hz - e], [-hx, hy - e, hz - e], [-hx, hy - e, -hz + e]]);
  face([0, 0, 1], [[-hx + e, -hy + e, hz], [hx - e, -hy + e, hz], [hx - e, hy - e, hz], [-hx + e, hy - e, hz]]);
  face([0, 0, -1], [[-hx + e, -hy + e, -hz], [-hx + e, hy - e, -hz], [hx - e, hy - e, -hz], [hx - e, -hy + e, -hz]]);
  // The twelve chamfers between them.
  const s = Math.SQRT1_2;
  for (const sy of [1, -1]) {
    face([0, sy * s, s], sy > 0 ? [[-hx + e, hy - e, hz], [hx - e, hy - e, hz], [hx - e, hy, hz - e], [-hx + e, hy, hz - e]] : [[-hx + e, -hy, hz - e], [hx - e, -hy, hz - e], [hx - e, -hy + e, hz], [-hx + e, -hy + e, hz]]);
    face([0, sy * s, -s], sy > 0 ? [[-hx + e, hy, -hz + e], [hx - e, hy, -hz + e], [hx - e, hy - e, -hz], [-hx + e, hy - e, -hz]] : [[-hx + e, -hy + e, -hz], [hx - e, -hy + e, -hz], [hx - e, -hy, -hz + e], [-hx + e, -hy, -hz + e]]);
    face([s, sy * s, 0], sy > 0 ? [[hx, hy - e, hz - e], [hx, hy - e, -hz + e], [hx - e, hy, -hz + e], [hx - e, hy, hz - e]] : [[hx - e, -hy, hz - e], [hx - e, -hy, -hz + e], [hx, -hy + e, -hz + e], [hx, -hy + e, hz - e]]);
    face([-s, sy * s, 0], sy > 0 ? [[-hx, hy - e, -hz + e], [-hx, hy - e, hz - e], [-hx + e, hy, hz - e], [-hx + e, hy, -hz + e]] : [[-hx + e, -hy, -hz + e], [-hx + e, -hy, hz - e], [-hx, -hy + e, hz - e], [-hx, -hy + e, -hz + e]]);
  }
  for (const [sx, sz] of [[1, 1], [1, -1], [-1, 1], [-1, -1]] as const) {
    const n: V3 = [sx * s, 0, sz * s];
    const a: V3 = [sx * hx, -hy + e, sz * (hz - e)];
    const c: V3 = [sx * (hx - e), -hy + e, sz * hz];
    const quad = [a, [a[0], hy - e, a[2]] as V3, [c[0], hy - e, c[2]] as V3, c];
    face(n, sx * sz > 0 ? quad : [quad[0]!, quad[3]!, quad[2]!, quad[1]!]);
  }
}

function pad4(n: number): number {
  return (n + 3) & ~3;
}

/** Writes parts and materials as one .glb: one mesh per part list, one node each, under the default scene. */
export function glb(meshes: { name: string; parts: Part[]; translation?: V3 }[], materials: Mat[]): Uint8Array {
  const chunks: Uint8Array[] = [];
  let offset = 0;
  const bufferViews: Record<string, unknown>[] = [];
  const accessors: Record<string, unknown>[] = [];
  const add = (bytes: Uint8Array, target: number): number => {
    const padded = new Uint8Array(pad4(bytes.length));
    padded.set(bytes);
    chunks.push(padded);
    bufferViews.push({ buffer: 0, byteOffset: offset, byteLength: bytes.length, target });
    offset += padded.length;
    return bufferViews.length - 1;
  };
  const vec3 = (data: number[]): number => {
    const f = new Float32Array(data);
    const view = add(new Uint8Array(f.buffer), 34962);
    const min = [Infinity, Infinity, Infinity], max = [-Infinity, -Infinity, -Infinity];
    for (let i = 0; i < f.length; i += 3) for (let k = 0; k < 3; k++) {
      min[k] = Math.min(min[k]!, f[i + k]!);
      max[k] = Math.max(max[k]!, f[i + k]!);
    }
    accessors.push({ bufferView: view, componentType: 5126, count: f.length / 3, type: "VEC3", min, max });
    return accessors.length - 1;
  };
  const idx = (data: number[]): number => {
    const big = data.some((i) => i > 65535);
    const arr = big ? new Uint32Array(data) : new Uint16Array(data);
    const view = add(new Uint8Array(arr.buffer), 34963);
    accessors.push({ bufferView: view, componentType: big ? 5125 : 5123, count: data.length, type: "SCALAR" });
    return accessors.length - 1;
  };
  const gltfMeshes = meshes.map((m) => ({
    name: m.name,
    primitives: m.parts.map((p) => ({ attributes: { POSITION: vec3(p.positions), NORMAL: vec3(p.normals) }, indices: idx(p.indices), material: p.material })),
  }));
  const json = {
    asset: { version: "2.0", generator: "rumpus-isle tools/make-assets.ts" },
    scene: 0,
    scenes: [{ nodes: meshes.map((_, i) => i) }],
    nodes: meshes.map((m, i) => ({ name: m.name, mesh: i, ...(m.translation ? { translation: m.translation } : {}) })),
    meshes: gltfMeshes,
    materials: materials.map((m) => ({ name: m.name, pbrMetallicRoughness: { baseColorFactor: [...m.color, 1], metallicFactor: m.metalness ?? 0, roughnessFactor: m.roughness } })),
    accessors,
    bufferViews,
    buffers: [{ byteLength: offset }],
  };
  const bin = new Uint8Array(offset);
  let at = 0;
  for (const c of chunks) {
    bin.set(c, at);
    at += c.length;
  }
  const text = new TextEncoder().encode(JSON.stringify(json));
  const jsonLen = pad4(text.length);
  const total = 12 + 8 + jsonLen + 8 + bin.length;
  const out = new Uint8Array(total);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, 0x46546c67, true);
  dv.setUint32(4, 2, true);
  dv.setUint32(8, total, true);
  dv.setUint32(12, jsonLen, true);
  dv.setUint32(16, 0x4e4f534a, true);
  out.set(text, 20);
  for (let i = text.length; i < jsonLen; i++) out[20 + i] = 0x20;
  dv.setUint32(20 + jsonLen, bin.length, true);
  dv.setUint32(24 + jsonLen, 0x004e4942, true);
  out.set(bin, 28 + jsonLen);
  return out;
}
