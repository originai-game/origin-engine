// Small helpers every file of the scene shares.
import { og, math } from "@origingame/engine";
import type { Quaternion, TerrainData, World } from "@origingame/engine";
import { CENTER } from "./layout.ts";

export function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Degrees to radians. */
export const rad = (degrees: number) => (degrees * Math.PI) / 180;

/** The product a * b (b first, then a). */
export function mul(a: Quaternion, b: Quaternion): Quaternion {
  return {
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  };
}

/** Where the valley's terrain stands in the world. */
export const ORIGIN = { x: CENTER.x, y: 0, z: CENTER.z };

/** The ground's height under a scene point. */
export function groundAt(world: World, data: TerrainData, x: number, z: number): number {
  return must(og.terrain.sample(world, { data, origin: ORIGIN, x, z })).height;
}
