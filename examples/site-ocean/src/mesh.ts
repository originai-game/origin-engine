// A small mesh builder for the scene's procedural models: triangles with
// vertex colours gathered per material and handed to og.render.bufferGeometry
// once (normals computed by the engine, smooth where vertices are shared).

import { og } from "@origingame/engine";
import type { Geometry, World } from "@origingame/engine";

export type V3 = { x: number; y: number; z: number };
export type Rgb = [number, number, number];

export const v3 = (x: number, y: number, z: number): V3 => ({ x, y, z });
export const add = (a: V3, b: V3): V3 => v3(a.x + b.x, a.y + b.y, a.z + b.z);
export const sub = (a: V3, b: V3): V3 => v3(a.x - b.x, a.y - b.y, a.z - b.z);
export const scale = (a: V3, s: number): V3 => v3(a.x * s, a.y * s, a.z * s);
export const dot = (a: V3, b: V3): number => a.x * b.x + a.y * b.y + a.z * b.z;
export const cross = (a: V3, b: V3): V3 => v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x);
export const len = (a: V3): number => Math.hypot(a.x, a.y, a.z);
export const norm = (a: V3): V3 => {
  const l = len(a) || 1;
  return scale(a, 1 / l);
};
export const lerp3 = (a: V3, b: V3, t: number): V3 => v3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t);

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export class MeshBuilder {
  positions: number[] = [];
  colors: number[] = [];
  indices: number[] = [];

  vertex(p: V3, c: Rgb): number {
    this.positions.push(p.x, p.y, p.z);
    this.colors.push(c[0], c[1], c[2]);
    return this.positions.length / 3 - 1;
  }

  tri(a: number, b: number, c: number): void {
    this.indices.push(a, b, c);
  }

  quad(a: number, b: number, c: number, d: number): void {
    this.indices.push(a, b, c, a, c, d);
  }

  /** A grid of rows × cols vertices (row-major) joined into quads; `flip` reverses the winding. */
  grid(first: number, rows: number, cols: number, flip = false): void {
    for (let r = 0; r + 1 < rows; r++) {
      for (let c = 0; c + 1 < cols; c++) {
        const a = first + r * cols + c;
        const b = a + 1;
        const d = a + cols;
        const e = d + 1;
        if (flip) this.quad(a, d, e, b);
        else this.quad(a, b, e, d);
      }
    }
  }

  /** A tapered round rod from `a` to `b` (radii ra at a, rb at b), capped at both ends. */
  rod(a: V3, b: V3, ra: number, rb: number, c: Rgb, sides = 8): void {
    const axis = norm(sub(b, a));
    const ref = Math.abs(axis.y) < 0.9 ? v3(0, 1, 0) : v3(1, 0, 0);
    const u = norm(cross(axis, ref));
    const w = cross(axis, u);
    const ring = (o: V3, r: number) => {
      const first = this.positions.length / 3;
      for (let i = 0; i < sides; i++) {
        const t = (i / sides) * Math.PI * 2;
        this.vertex(add(o, add(scale(u, Math.cos(t) * r), scale(w, Math.sin(t) * r))), c);
      }
      return first;
    };
    const r0 = ring(a, ra);
    const r1 = ring(b, rb);
    for (let i = 0; i < sides; i++) {
      const j = (i + 1) % sides;
      this.quad(r0 + i, r0 + j, r1 + j, r1 + i);
    }
    const ca = this.vertex(a, c);
    const cb = this.vertex(b, c);
    for (let i = 0; i < sides; i++) {
      const j = (i + 1) % sides;
      this.tri(ca, r0 + j, r0 + i);
      this.tri(cb, r1 + i, r1 + j);
    }
  }

  /** An axis-aligned box from its centre and full size (flat faces: vertices not shared). */
  box(centre: V3, size: V3, c: Rgb): void {
    const h = scale(size, 0.5);
    const p = (sx: number, sy: number, sz: number) => v3(centre.x + sx * h.x, centre.y + sy * h.y, centre.z + sz * h.z);
    const face = (a: V3, b: V3, cc: V3, d: V3) => {
      const i = this.vertex(a, c);
      this.vertex(b, c);
      this.vertex(cc, c);
      this.vertex(d, c);
      this.quad(i, i + 1, i + 2, i + 3);
    };
    face(p(-1, -1, 1), p(1, -1, 1), p(1, 1, 1), p(-1, 1, 1));
    face(p(1, -1, -1), p(-1, -1, -1), p(-1, 1, -1), p(1, 1, -1));
    face(p(1, -1, 1), p(1, -1, -1), p(1, 1, -1), p(1, 1, 1));
    face(p(-1, -1, -1), p(-1, -1, 1), p(-1, 1, 1), p(-1, 1, -1));
    face(p(-1, 1, 1), p(1, 1, 1), p(1, 1, -1), p(-1, 1, -1));
    face(p(-1, -1, -1), p(1, -1, -1), p(1, -1, 1), p(-1, -1, 1));
  }

  /** A solid of revolution about +y at `at`: profile [radius, height] pairs bottom to top. */
  lathe(at: V3, profile: Array<[number, number]>, c: Rgb, sides = 24): void {
    const first = this.positions.length / 3;
    for (const [r, y] of profile) {
      for (let i = 0; i <= sides; i++) {
        const t = (i / sides) * Math.PI * 2;
        this.vertex(v3(at.x + Math.cos(t) * r, at.y + y, at.z + Math.sin(t) * r), c);
      }
    }
    this.grid(first, profile.length, sides + 1, true);
  }

  geometry(world: World): Geometry {
    return must(
      og.render.bufferGeometry(world, {
        positions: new Float32Array(this.positions),
        colors: new Float32Array(this.colors),
        indices: new Uint32Array(this.indices),
      }),
    );
  }
}
