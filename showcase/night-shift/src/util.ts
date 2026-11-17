// Small helpers every module of the scene shares: unwrapping results and
// spawning the boxes, quads and lights the set is laid out with.
import { math, og } from "@origingame/engine";
import type { Color, Entity, Geometry, Material, Quaternion, StandardMaterialDesc, Texture, Vector3, World } from "@origingame/engine";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export const v3 = (x: number, y: number, z: number): Vector3 => ({ x, y, z });
export const rgb = (r: number, g: number, b: number): Color => ({ r, g, b });
/** A linear colour from an sRGB hex such as "#ff3aa8". */
export function hex(h: string): Color {
  const n = parseInt(h.slice(1), 16);
  const lin = (c: number) => { const s = c / 255; return s <= 0.04045 ? s / 12.92 : math.pow((s + 0.055) / 1.055, 2.4); };
  return { r: lin((n >> 16) & 255), g: lin((n >> 8) & 255), b: lin(n & 255) };
}
export const scaled = (c: Color, k: number): Color => ({ r: c.r * k, g: c.g * k, b: c.b * k });
export const lerp = (a: number, b: number, t: number): number => a + (b - a) * t;
export const mix = (a: Color, b: Color, t: number): Color => ({ r: lerp(a.r, b.r, t), g: lerp(a.g, b.g, t), b: lerp(a.b, b.b, t) });
export const smooth = (t: number): number => { const x = Math.min(1, Math.max(0, t)); return x * x * (3 - 2 * x); };

/** Turn about +Y by `deg` degrees. */
export const yaw = (deg: number): Quaternion => og.transform.fromAxisAngle(v3(0, 1, 0), (deg * Math.PI) / 180);
/** Turn about +X by `deg` degrees. */
export const pitch = (deg: number): Quaternion => og.transform.fromAxisAngle(v3(1, 0, 0), (deg * Math.PI) / 180);

/** The unit shapes the set scales into place, made once a world. */
export interface Shapes { readonly box: Geometry; readonly quad: Geometry; readonly sphere: Geometry; readonly cylinder: Geometry; readonly capsule: Geometry; readonly cone: Geometry }
export function shapes(world: World): Shapes {
  return {
    box: must(og.render.boxGeometry(world, {})),
    quad: must(og.render.planeGeometry(world, {})),
    sphere: must(og.render.sphereGeometry(world, { radius: 0.5, widthSegments: 24, heightSegments: 16 })),
    cylinder: must(og.render.cylinderGeometry(world, { radiusTop: 0.5, radiusBottom: 0.5, height: 1, radialSegments: 20 })),
    capsule: must(og.render.capsuleGeometry(world, { radius: 0.5, height: 1 })),
    cone: must(og.render.coneGeometry(world, { radius: 0.5, height: 1, radialSegments: 16 })),
  };
}

/** A cooked texture by its asset id. */
export const tex = (world: World, path: string): Texture => must(og.load.texture(world, path));

export const mat = (world: World, desc: StandardMaterialDesc): Material => must(og.render.standardMaterial(world, desc));

export interface Placed { readonly name: string; readonly at: Vector3; readonly size?: Vector3; readonly turn?: Quaternion; readonly parent?: Entity; readonly shadow?: boolean }
/** One mesh entity: `geometry` scaled to `size` at `at`. */
export function place(world: World, geometry: Geometry, material: Material, p: Placed): Entity {
  const e = must(world.spawn({ name: p.name, position: p.at, scale: p.size ?? v3(1, 1, 1), ...(p.turn ? { quaternion: p.turn } : {}), ...(p.parent !== undefined ? { parent: p.parent } : {}) }));
  must(world.insert(e, og.Mesh, { geometry, material, castShadow: p.shadow ?? true }));
  return e;
}
