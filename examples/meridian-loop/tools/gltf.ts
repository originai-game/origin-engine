// A small glTF 2.0 binary (.glb) writer and the vector and quaternion
// arithmetic the generators share. Deterministic: the same calls write the
// same bytes on every machine (no clock, no randomness of its own).

export type V3 = [number, number, number];
export type Q = [number, number, number, number]; // x, y, z, w

export const v3 = {
  add: (a: V3, b: V3): V3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]],
  sub: (a: V3, b: V3): V3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
  scale: (a: V3, s: number): V3 => [a[0] * s, a[1] * s, a[2] * s],
  dot: (a: V3, b: V3): number => a[0] * b[0] + a[1] * b[1] + a[2] * b[2],
  cross: (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]],
  len: (a: V3): number => Math.hypot(a[0], a[1], a[2]),
  norm: (a: V3): V3 => {
    const l = Math.hypot(a[0], a[1], a[2]) || 1;
    return [a[0] / l, a[1] / l, a[2] / l];
  },
};

export const quat = {
  identity: (): Q => [0, 0, 0, 1],
  axisAngle: (axis: V3, angle: number): Q => {
    const n = v3.norm(axis);
    const s = Math.sin(angle / 2);
    return [n[0] * s, n[1] * s, n[2] * s, Math.cos(angle / 2)];
  },
  mul: (a: Q, b: Q): Q => [
    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
    a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
    a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
  ],
  inv: (q: Q): Q => [-q[0], -q[1], -q[2], q[3]],
  rotate: (q: Q, v: V3): V3 => {
    const u: V3 = [q[0], q[1], q[2]];
    const t = v3.scale(v3.cross(u, v), 2);
    return v3.add(v3.add(v, v3.scale(t, q[3])), v3.cross(u, t));
  },
  /** The shortest rotation taking direction a to direction b. */
  between: (a: V3, b: V3): Q => {
    const x = v3.norm(a);
    const y = v3.norm(b);
    const d = v3.dot(x, y);
    if (d > 0.999999) return [0, 0, 0, 1];
    if (d < -0.999999) {
      let axis = v3.cross([1, 0, 0], x);
      if (v3.len(axis) < 1e-6) axis = v3.cross([0, 1, 0], x);
      return quat.axisAngle(axis, Math.PI);
    }
    const c = v3.cross(x, y);
    const q: Q = [c[0], c[1], c[2], 1 + d];
    const l = Math.hypot(q[0], q[1], q[2], q[3]);
    return [q[0] / l, q[1] / l, q[2] / l, q[3] / l];
  },
  /** Keeps consecutive keys on one hemisphere, so linear interpolation takes the short way. */
  align: (prev: Q, q: Q): Q => (prev[0] * q[0] + prev[1] * q[1] + prev[2] * q[2] + prev[3] * q[3] < 0 ? [-q[0], -q[1], -q[2], -q[3]] : q),
};

interface Prim {
  attributes: Record<string, number>;
  indices: number;
  material: number;
}

/** Builds one .glb: buffers, accessors, meshes, materials, nodes, skins and clips. */
export class Glb {
  private parts: Uint8Array[] = [];
  private offset = 0;
  readonly json = {
    asset: { version: "2.0", generator: "meridian-loop tools/make-assets.ts" } as Record<string, unknown>,
    scene: 0,
    scenes: [{ nodes: [] as number[] }],
    nodes: [] as Record<string, unknown>[],
    meshes: [] as { name: string; primitives: Prim[] }[],
    materials: [] as Record<string, unknown>[],
    textures: [] as Record<string, unknown>[],
    images: [] as Record<string, unknown>[],
    samplers: [] as Record<string, unknown>[],
    skins: [] as Record<string, unknown>[],
    animations: [] as Record<string, unknown>[],
    accessors: [] as Record<string, unknown>[],
    bufferViews: [] as Record<string, unknown>[],
    buffers: [] as Record<string, unknown>[],
    extensionsUsed: [] as string[],
  };

  private view(bytes: Uint8Array, target?: number): number {
    const pad = (4 - (bytes.length % 4)) % 4;
    this.parts.push(bytes);
    if (pad) this.parts.push(new Uint8Array(pad));
    this.json.bufferViews.push({ buffer: 0, byteOffset: this.offset, byteLength: bytes.length, ...(target ? { target } : {}) });
    this.offset += bytes.length + pad;
    return this.json.bufferViews.length - 1;
  }

  accessor(data: Float32Array | Uint16Array | Uint32Array | Uint8Array, type: "SCALAR" | "VEC2" | "VEC3" | "VEC4" | "MAT4", target?: number, minmax = false): number {
    const width = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4, MAT4: 16 }[type];
    const componentType = data instanceof Float32Array ? 5126 : data instanceof Uint16Array ? 5123 : data instanceof Uint32Array ? 5125 : 5121;
    const count = data.length / width;
    const acc: Record<string, unknown> = { bufferView: this.view(new Uint8Array(data.buffer, data.byteOffset, data.byteLength), target), componentType, count, type };
    if (minmax) {
      const min = new Array(width).fill(Infinity);
      const max = new Array(width).fill(-Infinity);
      for (let i = 0; i < count; i++) for (let k = 0; k < width; k++) {
        min[k] = Math.min(min[k], data[i * width + k]!);
        max[k] = Math.max(max[k], data[i * width + k]!);
      }
      acc.min = min;
      acc.max = max;
    }
    this.json.accessors.push(acc);
    return this.json.accessors.length - 1;
  }

  /** A PNG embedded in the binary chunk, as a texture with a repeating linear sampler. */
  texture(png: Uint8Array): number {
    if (this.json.samplers.length === 0) this.json.samplers.push({ magFilter: 9729, minFilter: 9987, wrapS: 10497, wrapT: 10497 });
    this.json.images.push({ bufferView: this.view(png), mimeType: "image/png" });
    this.json.textures.push({ sampler: 0, source: this.json.images.length - 1 });
    return this.json.textures.length - 1;
  }

  material(m: Record<string, unknown>): number {
    this.json.materials.push(m);
    return this.json.materials.length - 1;
  }

  /** A mesh of one or more primitives; each primitive's arrays are given whole. */
  mesh(name: string, prims: { pos: number[]; nor: number[]; uv?: number[]; col?: number[]; joints?: number[]; weights?: number[]; idx: number[]; material: number }[]): number {
    const out: Prim[] = [];
    for (const p of prims) {
      const attributes: Record<string, number> = {
        POSITION: this.accessor(new Float32Array(p.pos), "VEC3", 34962, true),
        NORMAL: this.accessor(new Float32Array(p.nor), "VEC3", 34962),
      };
      if (p.uv) attributes.TEXCOORD_0 = this.accessor(new Float32Array(p.uv), "VEC2", 34962);
      if (p.col) attributes.COLOR_0 = this.accessor(new Float32Array(p.col), "VEC4", 34962);
      if (p.joints) attributes.JOINTS_0 = this.accessor(new Uint8Array(p.joints), "VEC4", 34962);
      if (p.weights) attributes.WEIGHTS_0 = this.accessor(new Float32Array(p.weights), "VEC4", 34962);
      const big = p.pos.length / 3 > 65535;
      out.push({ attributes, indices: this.accessor(big ? new Uint32Array(p.idx) : new Uint16Array(p.idx), "SCALAR", 34963), material: p.material });
    }
    this.json.meshes.push({ name, primitives: out });
    return this.json.meshes.length - 1;
  }

  node(n: Record<string, unknown>, root = false): number {
    this.json.nodes.push(n);
    const i = this.json.nodes.length - 1;
    if (root) this.json.scenes[0]!.nodes.push(i);
    return i;
  }

  bytes(): Uint8Array {
    const j: Record<string, unknown> = { ...this.json, buffers: [{ byteLength: this.offset }] };
    for (const k of Object.keys(j)) if (Array.isArray(j[k]) && (j[k] as unknown[]).length === 0) delete j[k];
    const text = new TextEncoder().encode(JSON.stringify(j));
    const jpad = (4 - (text.length % 4)) % 4;
    const total = 12 + 8 + text.length + jpad + 8 + this.offset;
    const out = new Uint8Array(total);
    const dv = new DataView(out.buffer);
    dv.setUint32(0, 0x46546c67, true);
    dv.setUint32(4, 2, true);
    dv.setUint32(8, total, true);
    dv.setUint32(12, text.length + jpad, true);
    dv.setUint32(16, 0x4e4f534a, true);
    out.set(text, 20);
    out.fill(0x20, 20 + text.length, 20 + text.length + jpad);
    let at = 20 + text.length + jpad;
    dv.setUint32(at, this.offset, true);
    dv.setUint32(at + 4, 0x004e4942, true);
    at += 8;
    for (const p of this.parts) {
      out.set(p, at);
      at += p.length;
    }
    return out;
  }
}

/** Triangles grouped by material while a model is built face by face. */
export class MeshBuilder {
  readonly groups = new Map<number, { pos: number[]; nor: number[]; uv: number[]; col: number[]; joints: number[]; weights: number[]; idx: number[] }>();

  private group(material: number) {
    let g = this.groups.get(material);
    if (!g) {
      g = { pos: [], nor: [], uv: [], col: [], joints: [], weights: [], idx: [] };
      this.groups.set(material, g);
    }
    return g;
  }

  /**
   * A flat quad a, b, c, d (counter-clockwise seen from its front), its UVs
   * the corners' world positions projected on the face's plane over `tile`
   * metres, a vertex colour, and (for a skinned mesh) one joint.
   */
  quad(material: number, a: V3, b: V3, c: V3, d: V3, opts: { tile?: number; color?: [number, number, number]; joint?: number } = {}): void {
    const g = this.group(material);
    const n = v3.norm(v3.cross(v3.sub(b, a), v3.sub(c, a)));
    // Project on the plane's two axes: the dominant normal axis is dropped.
    const ax = Math.abs(n[0]) > Math.abs(n[1]) && Math.abs(n[0]) > Math.abs(n[2]) ? 0 : Math.abs(n[1]) > Math.abs(n[2]) ? 1 : 2;
    const [u, w] = ax === 0 ? [2, 1] : ax === 1 ? [0, 2] : [0, 1];
    const tile = opts.tile ?? 2;
    const first = g.pos.length / 3;
    const col = opts.color ?? [1, 1, 1];
    for (const p of [a, b, c, d]) {
      g.pos.push(p[0], p[1], p[2]);
      g.nor.push(n[0], n[1], n[2]);
      g.uv.push(p[u]! / tile, -p[w]! / tile);
      g.col.push(col[0], col[1], col[2], 1);
      if (opts.joint !== undefined) {
        g.joints.push(opts.joint, 0, 0, 0);
        g.weights.push(1, 0, 0, 0);
      }
    }
    g.idx.push(first, first + 1, first + 2, first, first + 2, first + 3);
  }

  /** A triangle (the sides of a ramp). */
  tri(material: number, a: V3, b: V3, c: V3, opts: { tile?: number; color?: [number, number, number] } = {}): void {
    const g = this.group(material);
    const n = v3.norm(v3.cross(v3.sub(b, a), v3.sub(c, a)));
    const ax = Math.abs(n[0]) > Math.abs(n[1]) && Math.abs(n[0]) > Math.abs(n[2]) ? 0 : Math.abs(n[1]) > Math.abs(n[2]) ? 1 : 2;
    const [u, w] = ax === 0 ? [2, 1] : ax === 1 ? [0, 2] : [0, 1];
    const tile = opts.tile ?? 2;
    const first = g.pos.length / 3;
    const col = opts.color ?? [1, 1, 1];
    for (const p of [a, b, c]) {
      g.pos.push(p[0], p[1], p[2]);
      g.nor.push(n[0], n[1], n[2]);
      g.uv.push(p[u]! / tile, -p[w]! / tile);
      g.col.push(col[0], col[1], col[2], 1);
    }
    g.idx.push(first, first + 1, first + 2);
  }

  /**
   * An axis-aligned box from min to max; `faces` leaves some out (a box
   * standing on the floor needs no bottom). Corners in a rotated frame when
   * `frame` is given (a box along a bone: origin, then its x, y, z axes).
   */
  box(material: number, min: V3, max: V3, opts: { tile?: number; color?: [number, number, number]; joint?: number; skip?: string; frame?: { o: V3; x: V3; y: V3; z: V3 } } = {}): void {
    const P = (x: number, y: number, z: number): V3 => {
      const l: V3 = [[min[0], max[0]][x]!, [min[1], max[1]][y]!, [min[2], max[2]][z]!];
      const f = opts.frame;
      if (!f) return l;
      return v3.add(f.o, v3.add(v3.add(v3.scale(f.x, l[0]), v3.scale(f.y, l[1])), v3.scale(f.z, l[2])));
    };
    const skip = opts.skip ?? "";
    const q = (k: string, a: V3, b: V3, c: V3, d: V3) => {
      if (!skip.includes(k)) this.quad(material, a, b, c, d, opts);
    };
    q("+x", P(1, 0, 1), P(1, 0, 0), P(1, 1, 0), P(1, 1, 1));
    q("-x", P(0, 0, 0), P(0, 0, 1), P(0, 1, 1), P(0, 1, 0));
    q("+y", P(0, 1, 1), P(1, 1, 1), P(1, 1, 0), P(0, 1, 0));
    q("-y", P(0, 0, 0), P(1, 0, 0), P(1, 0, 1), P(0, 0, 1));
    q("+z", P(0, 0, 1), P(1, 0, 1), P(1, 1, 1), P(0, 1, 1));
    q("-z", P(1, 0, 0), P(0, 0, 0), P(0, 1, 0), P(1, 1, 0));
  }
}
