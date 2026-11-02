// The R10 virtual cameras sample: a runner crossing a room, four shots.
// tests/evidence/r10/NOTES.md has the commands and what each frame shows.
//
// - "wide", priority 10: a high corner shot looking at the runner, live
//   from the start (the first live shot cuts);
// - "close", priority 20 from 2 s: follows the runner from the front-left;
//   the brain blends to it over 1.5 s, cubicInOut;
// - "side", priority 30 from 4.5 s: a low side shot, a 1.5 s linear blend
//   that is interrupted at 5 s by disabling it again, so the brain turns
//   back to "close" from where the camera is (no jump);
// - "chase", priority 40 from 7 s (a cut): behind the runner with
//   collision on; the runner passes behind a wall, and the shot pulls in
//   in front of it at once and eases back out after.

import { og, defineSystem, SHAPE_BOX } from "@origingame/engine";
import type { Entity, Geometry, Material, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: { x: number; y: number; z: number }, scale: { x: number; y: number; z: number }): Entity {
  const e = must(world.spawn({ name, position }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

/** At a tick, enable (or disable) a shot. */
const STEPS: Array<[number, string, boolean]> = [
  [120, "close", true],
  [270, "side", true],
  [300, "side", false],
  [420, "chase", true],
];

export const Director = defineSystem("Director", { writes: [og.VirtualCamera] }, (_q, _time, world) => {
  for (const [at, name, on] of STEPS) {
    if (world.tick !== at) continue;
    const e = world.find(name);
    if (e !== null) world.set(e, og.VirtualCamera, { enabled: on });
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Director],
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    cam.fov = 50;
    cam.background = { r: 0.05, g: 0.06, b: 0.09 };
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.45 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 2.2, color: { r: 1, g: 0.95, b: 0.88 }, castShadow: true });

    const box = must(og.render.boxGeometry(world, {}));
    const mat = (r: number, g: number, b: number, roughness = 0.8) => must(og.render.standardMaterial(world, { color: { r, g, b }, roughness }));
    mesh(world, "floor", box, mat(0.3, 0.32, 0.36, 0.95), { x: 0, y: -0.5, z: 0 }, { x: 16, y: 1, z: 12 });
    mesh(world, "back", box, mat(0.42, 0.4, 0.38, 0.95), { x: 0, y: 2, z: -5 }, { x: 16, y: 4, z: 0.3 });
    for (const [k, x] of [-6, -3, 3, 6].entries()) {
      mesh(world, `crate${k}`, box, mat(0.85, 0.45 + k * 0.08, 0.15, 0.7), { x, y: 0.4, z: -3.5 }, { x: 0.8, y: 0.8, z: 0.8 });
    }
    // The wall the runner passes behind: drawn and a static collider.
    const wall = mesh(world, "wall", box, mat(0.55, 0.22, 0.2, 0.85), { x: 1, y: 1.5, z: 4 }, { x: 2, y: 3, z: 0.3 });
    world.insert(wall, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 } });

    const runner = mesh(world, "runner", box, mat(0.1, 0.6, 0.85, 0.4), { x: -4, y: 0.9, z: 0 }, { x: 0.6, y: 1.8, z: 0.6 });
    must(og.tween.to(world, { entity: runner, field: "position.x", from: [-4], to: [4], duration: 3, ease: "quadInOut", loops: -1, yoyo: true }));

    const shot = (name: string, position: { x: number; y: number; z: number }, vcam: Record<string, unknown>) => {
      const e = must(world.spawn({ name, position }));
      world.insert(e, og.VirtualCamera, { lookAt: runner, ...vcam } as never);
      return e;
    };
    shot("wide", { x: -7, y: 6, z: 9 }, { priority: 10, lookOffset: { x: 0, y: 0.5, z: 0 }, fov: 45 });
    shot("close", { x: 0, y: 0, z: 0 }, { priority: 20, enabled: false, follow: runner, followOffset: { x: -2.2, y: 1.2, z: 3 }, smoothTime: 0.2, fov: 40, blendTime: 1.5, blendEase: "cubicInOut" });
    shot("side", { x: 7, y: 0.8, z: 1 }, { priority: 30, enabled: false, fov: 35, roll: -6, blendTime: 1.5, blendEase: "linear" });
    shot("chase", { x: 0, y: 0, z: 0 }, { priority: 40, enabled: false, follow: runner, followOffset: { x: 0, y: 0.9, z: 6.5 }, smoothTime: 0, lookOffset: { x: 0, y: 0.4, z: 0 }, fov: 55, blendTime: 0, collide: true, radius: 0.25, damping: 0.45 });
  },
});
