// The G2-3 soft-volume sample (og.SoftBody):
// - left: a green jelly ball (stiffness 0.3) dropped from 4 m onto the
//   floor: it squashes on landing and springs back to a ball;
// - centre: a pink soft box dropped onto a ramp, sliding down it;
// - right: a blue water balloon hanging by its top (pinTop) from a post
//   that swings it at 2 s, and an inflated ball (pressure 1.6) beside it.
// The Bench system records the jelly's least height (squash) and volume.
// tests/evidence/g2-3/NOTES.md has the commands and what each frame shows.

import { og, defineComponent, defineSystem, u32, f32, SHAPE_BOX, SOFT_SPHERE, SOFT_BOX } from "@origingame/engine";
import type { Entity, Geometry, Material, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What must replay: on the entity "bench". */
const Bench = defineComponent("Bench", {
  /** Ticks since the start. */
  tick: u32(0),
  /** The jelly's least height, lowest point to highest (m; 1 at rest). */
  squash: f32(1),
  /** The jelly's least volume over its rest volume. */
  leastVolume: f32(1),
  /** The jelly's volume over its rest volume now. */
  volume: f32(1),
  /** The inflated ball's volume over its rest volume now. */
  inflated: f32(1),
});

export const Watch = defineSystem("Watch", { reads: [og.SoftBody], writes: [Bench, og.Transform] }, (_q, _time, world) => {
  const bench = world.find("bench")!;
  const b = must(world.get(bench, Bench));
  const jelly = world.find("jelly")!;
  const s = must(world.get(jelly, og.SoftBody));
  const surface = og.physics.softBodyPoints(world, jelly);
  let squash = b.squash;
  if (surface.ok && b.tick > 0) {
    const ys = surface.value.points.map((p) => p.y);
    squash = Math.min(squash, Math.max(...ys) - Math.min(...ys));
  }
  // The post swings the balloon sideways at 2 s.
  if (b.tick === 120) must(world.set(world.find("balloon")!, og.Transform, { position: { x: 6.6, y: 4, z: 0 } }));
  const inflated = must(world.get(world.find("puffed")!, og.SoftBody)).volume;
  must(world.set(bench, Bench, { tick: b.tick + 1, squash, leastVolume: Math.min(b.leastVolume, s.volume), volume: s.volume, inflated }));
});

function mesh(world: World, e: Entity, geometry: Geometry, material: Material, scale: { x: number; y: number; z: number }): void {
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Watch],
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    cam.fov = 50;
    cam.background = { r: 0.05, g: 0.06, b: 0.09 };
    const eye = { x: 0, y: 4.5, z: 13 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 1.2, z: 0 });
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.45 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });

    const box = must(og.render.boxGeometry(world, {}));
    const mat = (r: number, g: number, b: number, roughness = 0.7) => must(og.render.standardMaterial(world, { color: { r, g, b }, roughness }));

    // The floor.
    const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
    world.insert(floor, og.Collider, { shape: SHAPE_BOX });
    mesh(world, floor, box, mat(0.3, 0.29, 0.27, 0.95), { x: 30, y: 1, z: 12 });

    // Centre: a ramp falling to the right.
    const ramp = must(world.spawn({ name: "ramp", position: { x: 0.5, y: 1.2, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, -0.35) }));
    world.insert(ramp, og.Collider, { shape: SHAPE_BOX });
    mesh(world, ramp, box, mat(0.45, 0.33, 0.2), { x: 4, y: 0.2, z: 2.5 });

    // Left: the jelly ball.
    const jelly = must(world.spawn({ name: "jelly", position: { x: -4.5, y: 4, z: 0 } }));
    world.insert(jelly, og.SoftBody, { shape: SOFT_SPHERE, radius: 0.5, detail: 2, stiffness: 0.4, material: mat(0.1, 0.7, 0.2, 0.3) });

    // Centre: a soft box onto the ramp.
    const cube = must(world.spawn({ name: "cube", position: { x: -0.4, y: 3.6, z: 0 } }));
    world.insert(cube, og.SoftBody, { shape: SOFT_BOX, size: { x: 0.8, y: 0.8, z: 0.8 }, detail: 4, stiffness: 0.5, friction: 0.1, material: mat(0.9, 0.2, 0.45, 0.4) });

    // Right: a post with the water balloon hung by its top, and an inflated ball.
    const post = must(world.spawn({ name: "post", position: { x: 6, y: 2.25, z: -0.6 } }));
    world.insert(post, og.Collider, { shape: SHAPE_BOX });
    mesh(world, post, box, mat(0.6, 0.62, 0.66, 0.35), { x: 0.2, y: 4.5, z: 0.2 });
    const balloon = must(world.spawn({ name: "balloon", position: { x: 6, y: 4, z: 0 } }));
    world.insert(balloon, og.SoftBody, { radius: 0.45, pinTop: 0.06, stiffness: 0.15, material: mat(0.15, 0.35, 0.95, 0.2) });
    const puffed = must(world.spawn({ name: "puffed", position: { x: 3.4, y: 1, z: 0.6 } }));
    world.insert(puffed, og.SoftBody, { radius: 0.4, pressure: 1.6, stiffness: 0.05, material: mat(0.95, 0.75, 0.1, 0.4) });

    world.insert(must(world.spawn({ name: "bench" })), Bench);
  },
});
