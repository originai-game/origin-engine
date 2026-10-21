// The cars' models: models/car.glb (the player's wedge: magenta clear-coated
// paint, a dark canopy, a cyan side line, a red tail bar, white lamps),
// models/ghost.glb (the same shape in one translucent cyan glass, for the
// best lap's ghost) and models/wheel.glb (a tyre with a glowing rim ring).
// Forward is +z (og.Vehicle's), the origin the chassis's centre; the
// wheels' contact patch sits about 0.3 m below it (src/world.ts's tuning),
// the body drawn 0.17 m lower than the chassis's collider.

import { Glb, MeshBuilder } from "./gltf.ts";
import type { V3 } from "./gltf.ts";

/** Cross-sections along z: [z, half width, bottom, top]. */
const BODY: readonly [number, number, number, number][] = [
  [-2.12, 0.86, 0.02, 0.42],
  [-1.7, 0.92, -0.04, 0.5],
  [-0.7, 0.94, -0.05, 0.46],
  [0.5, 0.9, -0.05, 0.38],
  [1.5, 0.8, -0.03, 0.26],
  [2.18, 0.5, 0.03, 0.12],
];
const CANOPY: readonly [number, number, number, number][] = [
  [-1.25, 0.5, 0.48, 0.5],
  [-0.9, 0.6, 0.47, 0.74],
  [0.1, 0.58, 0.43, 0.8],
  [0.85, 0.42, 0.37, 0.42],
];

/** A loft through sections: sides, top, bottom and the end caps, by material. */
function loft(m: MeshBuilder, sections: readonly [number, number, number, number][], side: number, top: number, bottom: number, caps: number, inset = 0.88): void {
  for (let i = 0; i + 1 < sections.length; i++) {
    const [z0, w0, b0, t0] = sections[i]!;
    const [z1, w1, b1, t1] = sections[i + 1]!;
    // Each section is a hexagon: the top narrower than the waist.
    const tw0 = w0 * inset;
    const tw1 = w1 * inset;
    const mid0 = b0 + (t0 - b0) * 0.45;
    const mid1 = b1 + (t1 - b1) * 0.45;
    const L = (x: number, y: number, z: number): V3 => [x, y, z];
    // Right side (x > 0): lower and upper bands; left mirrored.
    for (const s of [1, -1]) {
      const lo = [L(s * w0 * 0.9, b0, z0), L(s * w1 * 0.9, b1, z1), L(s * w1, mid1, z1), L(s * w0, mid0, z0)] as const;
      const hi = [L(s * w0, mid0, z0), L(s * w1, mid1, z1), L(s * tw1, t1, z1), L(s * tw0, t0, z0)] as const;
      if (s < 0) {
        m.quad(side, lo[0], lo[1], lo[2], lo[3]);
        m.quad(side, hi[0], hi[1], hi[2], hi[3]);
      } else {
        m.quad(side, lo[3], lo[2], lo[1], lo[0]);
        m.quad(side, hi[3], hi[2], hi[1], hi[0]);
      }
    }
    m.quad(top, L(-tw0, t0, z0), L(-tw1, t1, z1), L(tw1, t1, z1), L(tw0, t0, z0));
    m.quad(bottom, L(w0 * 0.9, b0, z0), L(w1 * 0.9, b1, z1), L(-w1 * 0.9, b1, z1), L(-w0 * 0.9, b0, z0));
  }
  // End caps: the rear (first) facing -z, the nose (last) facing +z.
  for (const [k, dir] of [[0, -1], [sections.length - 1, 1]] as const) {
    const [z, w, b, t] = sections[k]!;
    const mid = b + (t - b) * 0.45;
    const tw = w * inset;
    const ring: V3[] = [[w * 0.9, b, z], [w, mid, z], [tw, t, z], [-tw, t, z], [-w, mid, z], [-w * 0.9, b, z]];
    const pts = dir > 0 ? ring : [...ring].reverse();
    for (let i = 1; i + 1 < pts.length; i++) m.tri(caps, pts[0]!, pts[i]!, pts[i + 1]!);
  }
}

function emissive(glb: Glb, name: string, c: [number, number, number], strength: number): number {
  return glb.material({ name, pbrMetallicRoughness: { baseColorFactor: [c[0], c[1], c[2], 1], metallicFactor: 0, roughnessFactor: 0.4 }, emissiveFactor: c, extensions: { KHR_materials_emissive_strength: { emissiveStrength: strength } } });
}

function prims(m: MeshBuilder) {
  return [...m.groups.entries()].map(([material, g]) => ({ pos: g.pos, nor: g.nor, uv: g.uv, idx: g.idx, material }));
}

export function carModel(): Uint8Array {
  const glb = new Glb();
  glb.json.extensionsUsed.push("KHR_materials_emissive_strength", "KHR_materials_clearcoat");
  const paint = glb.material({ name: "paint", pbrMetallicRoughness: { baseColorFactor: [0.85, 0.04, 0.42, 1], metallicFactor: 0.55, roughnessFactor: 0.32 }, extensions: { KHR_materials_clearcoat: { clearcoatFactor: 1, clearcoatRoughnessFactor: 0.05 } } });
  const under = glb.material({ name: "under", pbrMetallicRoughness: { baseColorFactor: [0.03, 0.03, 0.04, 1], metallicFactor: 0.2, roughnessFactor: 0.8 } });
  const glass = glb.material({ name: "canopy", pbrMetallicRoughness: { baseColorFactor: [0.02, 0.03, 0.06, 1], metallicFactor: 0.1, roughnessFactor: 0.08 } });
  const line = emissive(glb, "line", [0.15, 0.9, 1], 3);
  const tail = emissive(glb, "tail", [1, 0.05, 0.12], 5);
  const lamp = emissive(glb, "lamp", [1, 0.95, 0.85], 6);
  const m = new MeshBuilder();
  loft(m, BODY, paint, paint, under, paint);
  loft(m, CANOPY, glass, glass, glass, glass, 0.7);
  // The side line, the tail bar and the lamps: thin boxes standing proud of the paint.
  for (const s of [1, -1]) m.box(line, [s > 0 ? 0.93 : -0.96, 0.14, -1.6], [s > 0 ? 0.96 : -0.93, 0.18, 1.3]);
  m.box(tail, [-0.78, 0.28, -2.16], [0.78, 0.36, -2.1]);
  for (const s of [1, -1]) m.box(lamp, [s * 0.18 - 0.14, 0.06, 2.16], [s * 0.18 + 0.14, 0.11, 2.22]);
  // A small wing on two struts.
  m.box(under, [-0.05, 0.42, -1.95], [0.05, 0.62, -1.85]);
  m.box(paint, [-0.9, 0.62, -2.08], [0.9, 0.67, -1.72]);
  // Drawn 0.17 m below the chassis's centre: the body sits over its wheels, the collider above the road.
  glb.node({ name: "body", mesh: glb.mesh("body", prims(m)), translation: [0, -0.17, 0] }, true);
  return glb.bytes();
}

export function ghostModel(): Uint8Array {
  const glb = new Glb();
  glb.json.extensionsUsed.push("KHR_materials_emissive_strength");
  const ghost = glb.material({ name: "ghost", alphaMode: "BLEND", doubleSided: true, pbrMetallicRoughness: { baseColorFactor: [0.2, 0.85, 1, 0.32], metallicFactor: 0, roughnessFactor: 0.2 }, emissiveFactor: [0.12, 0.55, 0.7], extensions: { KHR_materials_emissive_strength: { emissiveStrength: 1.5 } } });
  const m = new MeshBuilder();
  loft(m, BODY, ghost, ghost, ghost, ghost);
  loft(m, CANOPY, ghost, ghost, ghost, ghost, 0.7);
  m.box(ghost, [-0.9, 0.62, -2.08], [0.9, 0.67, -1.72]);
  glb.node({ name: "ghost", mesh: glb.mesh("ghost", prims(m)), translation: [0, -0.17, 0] }, true);
  return glb.bytes();
}

/** A tyre about the x axis (radius 0.35, 0.3 wide) with a glowing ring on its outer face. */
export function wheelModel(ghost: boolean): Uint8Array {
  const glb = new Glb();
  glb.json.extensionsUsed.push("KHR_materials_emissive_strength");
  const tyre = ghost
    ? glb.material({ name: "ghost-tyre", alphaMode: "BLEND", doubleSided: true, pbrMetallicRoughness: { baseColorFactor: [0.2, 0.85, 1, 0.25], metallicFactor: 0, roughnessFactor: 0.3 }, emissiveFactor: [0.1, 0.4, 0.5] })
    : glb.material({ name: "tyre", pbrMetallicRoughness: { baseColorFactor: [0.04, 0.04, 0.05, 1], metallicFactor: 0, roughnessFactor: 0.9 } });
  const ring = emissive(glb, "rim", ghost ? [0.3, 0.9, 1] : [0.15, 0.9, 1], 3);
  const m = new MeshBuilder();
  const R = 0.35;
  const W = 0.15;
  const n = 18;
  for (let i = 0; i < n; i++) {
    const a0 = (i / n) * Math.PI * 2;
    const a1 = ((i + 1) / n) * Math.PI * 2;
    const p = (a: number, r: number, x: number): V3 => [x, Math.sin(a) * r, Math.cos(a) * r];
    m.quad(tyre, p(a0, R, W), p(a1, R, W), p(a1, R, -W), p(a0, R, -W));
    m.tri(tyre, [W, 0, 0], p(a1, R, W), p(a0, R, W));
    m.tri(tyre, [-W, 0, 0], p(a0, R, -W), p(a1, R, -W));
    // The rim ring, a hair proud of both faces.
    for (const s of [1, -1]) {
      const x = s * (W + 0.004);
      const q = [p(a0, R * 0.62, x), p(a1, R * 0.62, x), p(a1, R * 0.72, x), p(a0, R * 0.72, x)] as const;
      if (s > 0) m.quad(ring, q[0], q[1], q[2], q[3]);
      else m.quad(ring, q[3], q[2], q[1], q[0]);
    }
  }
  glb.node({ name: "wheel", mesh: glb.mesh("wheel", prims(m)) }, true);
  return glb.bytes();
}
