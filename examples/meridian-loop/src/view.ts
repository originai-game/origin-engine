// The camera: a chase view behind the car (yaw only, so the banks and the
// jump tip the world, not the view), eased towards its place with a little
// lag and widening with speed; at the title a slow orbit of the start
// gantry; after the finish a slow orbit of the car. Writes og.Camera; the
// engine's shake, kick and punch ride on top.

import { og, defineSystem } from "@origingame/engine";
import { Phase, Prefs, Race, Tuning } from "./components.ts";
import { at } from "./track.ts";
import { entity, must } from "./world.ts";

type V = { x: number; y: number; z: number };

/** The rotation that looks from eye at target, +y up (the camera looks down its -z). */
export function lookAt(eye: V, target: V): { x: number; y: number; z: number; w: number } {
  let zx = eye.x - target.x;
  let zy = eye.y - target.y;
  let zz = eye.z - target.z;
  const zl = Math.hypot(zx, zy, zz) || 1;
  zx /= zl;
  zy /= zl;
  zz /= zl;
  // x = up × z
  let xx = zz;
  let xy = 0;
  let xz = -zx;
  const xl = Math.hypot(xx, xy, xz) || 1;
  xx /= xl;
  xy /= xl;
  xz /= xl;
  // y = z × x
  const yx = zy * xz - zz * xy;
  const yy = zz * xx - zx * xz;
  const yz = zx * xy - zy * xx;
  const tr = xx + yy + zz;
  if (tr > 0) {
    const s = Math.sqrt(tr + 1) * 2;
    return { w: 0.25 * s, x: (yz - zy) / s, y: (zx - xz) / s, z: (xy - yx) / s };
  }
  if (xx > yy && xx > zz) {
    const s = Math.sqrt(1 + xx - yy - zz) * 2;
    return { w: (yz - zy) / s, x: 0.25 * s, y: (yx + xy) / s, z: (zx + xz) / s };
  }
  if (yy > zz) {
    const s = Math.sqrt(1 + yy - xx - zz) * 2;
    return { w: (zx - xz) / s, x: (yx + xy) / s, y: 0.25 * s, z: (zy + yz) / s };
  }
  const s = Math.sqrt(1 + zz - xx - yy) * 2;
  return { w: (xy - yx) / s, x: (zx + xz) / s, y: (zy + yz) / s, z: 0.25 * s };
}

export const View = defineSystem("View", { reads: [Race, Prefs, Tuning, og.Transform], writes: [og.Camera], after: ["Fx"] }, (_q, time, world) => {
  const r = must(world.resource(Race));
  const t = must(world.resource(Tuning));
  const cam = must(world.resource(og.Camera));
  const car = must(world.get(entity(world, "car"), og.Transform));
  const p = car.position;
  const q = car.quaternion;
  const heading = Math.atan2(2 * (q.x * q.z + q.w * q.y), 1 - 2 * (q.x * q.x + q.y * q.y));
  let eye: V;
  let target: V;
  let ease = 1;
  if (r.phase === Phase.Title) {
    // A slow orbit round the gantry and the car on the grid.
    const g = at(0);
    const a = time.elapsed * 0.07 + 2.2;
    eye = { x: g.x + Math.sin(a) * 26, y: g.y + 7.5, z: g.z + Math.cos(a) * 26 };
    target = { x: g.x, y: g.y + 2.5, z: g.z - 4 };
  } else if (r.phase === Phase.Finished) {
    const a = time.elapsed * 0.25;
    eye = { x: p.x + Math.sin(a) * 9, y: p.y + 3.2, z: p.z + Math.cos(a) * 9 };
    target = { x: p.x, y: p.y + 0.6, z: p.z };
    ease = 1 - Math.exp(-time.dt / 0.4);
  } else {
    const back = t.cameraBack;
    eye = { x: p.x - Math.sin(heading) * back, y: p.y + t.cameraHeight, z: p.z - Math.cos(heading) * back };
    target = { x: p.x + Math.sin(heading) * 6, y: p.y + 0.9, z: p.z + Math.cos(heading) * 6 };
    ease = t.cameraLag <= 0 ? 1 : 1 - Math.exp(-time.dt / t.cameraLag);
    if (r.phase === Phase.Countdown && r.countdown > 170) ease = 1;
  }
  const now = cam.position;
  const pos = { x: now.x + (eye.x - now.x) * ease, y: now.y + (eye.y - now.y) * ease, z: now.z + (eye.z - now.z) * ease };
  // Never under the road's surface near the car (a bank's inside edge).
  if (r.phase !== Phase.Title) pos.y = Math.max(pos.y, p.y + 1.2);
  cam.position = pos;
  cam.quaternion = lookAt(pos, target);
  cam.fov = 58 + Math.min(14, r.kmh / 14);
  cam.far = 1600;
});
