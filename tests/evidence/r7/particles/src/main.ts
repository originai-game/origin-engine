// The R7 particles sample (batch 2): lit smoke, lit mesh debris that
// bounces and kicks dust at each bounce (a collision sub-emitter), little
// lit cubes from the built-in box mesh, a heat haze bending the checker
// wall behind it, and gameplay reading og.ParticleEvent: a tally column
// grows with every bounce of the debris. tests/evidence/r7/NOTES.md has the
// commands and what each frame shows.

import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, Geometry, Material, Vector3, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What the debris did so far, by og.ParticleEvent. */
export const Tally = defineResource("Tally", { births: u32(), deaths: u32(), bounces: u32() });

function mesh(world: World, name: string, geometry: Geometry, material: Material, position: Vector3, scale: Vector3): Entity {
  const e = must(world.spawn({ name, position }));
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

function particles(world: World, name: string, path: string, position: Vector3, seed: number): Entity {
  const e = must(world.spawn({ name, position }));
  world.insert(e, og.ParticleSystem, { emitter: must(og.load.emitter(world, path)), seed });
  return e;
}

/** Counts the debris' events; the tally column is 4 mm taller per bounce. */
export const Count = defineSystem("Count", { reads: [og.ParticleEvent], writes: [Tally, og.Transform] }, (_q, _time, world) => {
  const t = must(world.resource(Tally));
  for (const e of world.events(og.ParticleEvent)) {
    if (e.kind === "birth") t.births += 1;
    else if (e.kind === "death") t.deaths += 1;
    else t.bounces += 1;
  }
  const column = world.find("tally");
  if (column === null) return;
  const h = Math.min(0.05 + t.bounces * 0.004, 3);
  world.set(column, og.Transform, { position: { x: -1.2, y: h / 2, z: -1.6 }, scale: { x: 0.25, y: h, z: 0.25 } });
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Tally],
  systems: [Count],
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0, y: 2.4, z: 6.4 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 1, z: -0.8 });
    cam.fov = 50;

    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.85, y: -0.4, z: -0.3 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3, color: { r: 1, g: 0.95, b: 0.88 }, castShadow: true });

    const box = must(og.render.boxGeometry(world, {}));
    const checker = must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.8, b: 0.8 }, roughness: 0.8 }));
    const ground = must(og.render.standardMaterial(world, { color: { r: 0.42, g: 0.38, b: 0.33 }, roughness: 0.95 }));
    const coals = must(og.render.standardMaterial(world, { color: { r: 0.1, g: 0.08, b: 0.07 }, emissive: { r: 3, g: 0.9, b: 0.2 }, roughness: 0.9 }));
    const brass = must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.6, b: 0.25 }, metalness: 0.8, roughness: 0.35 }));
    // The ground is a collider, so the debris bounces on it in the tick.
    const floor = mesh(world, "ground", box, ground, { x: 0, y: -0.5, z: 0 }, { x: 20, y: 1, z: 14 });
    world.insert(floor, og.Collider, { shape: 0, size: { x: 20, y: 1, z: 14 } });
    // The wall: dark and light stripes, so the haze's bend reads.
    for (let i = 0; i < 18; i++) {
      const shade = i % 2 === 0 ? checker : ground;
      mesh(world, `wall-${i}`, box, shade, { x: -4.25 + i * 0.5, y: 1.75, z: -2.6 }, { x: 0.5, y: 3.5, z: 0.3 });
    }
    mesh(world, "brazier", box, coals, { x: 2.8, y: 0.25, z: -1.2 }, { x: 0.7, y: 0.5, z: 0.7 });
    mesh(world, "tally", box, brass, { x: -1.2, y: 0.025, z: -1.6 }, { x: 0.25, y: 0.05, z: 0.25 });

    particles(world, "smoke", "fx/smoke.ogt", { x: -2.8, y: 0, z: -0.6 }, 11);
    particles(world, "debris", "fx/debris.ogt", { x: 0, y: 0.1, z: 0.4 }, 23);
    particles(world, "chips", "fx/chips.ogt", { x: 1.2, y: 0.1, z: 1.2 }, 29);
    particles(world, "haze", "fx/haze.ogt", { x: 2.8, y: 0.55, z: -1.2 }, 31);
  },
});
