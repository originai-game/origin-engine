// The R5 buoyancy sample: three boats (box hulls with a cabin) and four
// buoys floating by og.Buoyancy on the storm frame's gale (og.Water.wind 20
// m/s, choppiness 1) under a dark overcast. Nothing sets their heights: the
// og.physics step lifts each hull's cells under the sea the renderer draws
// (og_core::ocean, computed in the tick), so they ride the swell, pitch and
// roll. tests/evidence/r5-water/NOTES.md has the commands and what each
// frame shows.

import { og, SHAPE_BOX, SHAPE_SPHERE } from "@origingame/engine";
import type { World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** The boats: where each starts and its heading (rad about y). */
const BOATS = [
  { x: -12, z: -30, heading: 0.4 },
  { x: 6, z: -45, heading: -0.9 },
  { x: -2, z: -70, heading: 1.6 },
];

/** The buoys, a line across the view 20 to 55 m out. */
const BUOYS = [
  { x: 10, z: -20 },
  { x: -6, z: -38 },
  { x: 18, z: -55 },
  { x: -20, z: -52 },
];

export default og.game({
  tick: 60,
  seed: 1,
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0, y: 10, z: 0 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: -4, y: 0, z: -60 });
    cam.fov = 50;
    cam.far = 3000;

    const day = must(world.resource(og.DayNight));
    day.enabled = true;
    day.start = 13;
    day.speed = 0;
    must(world.resource(og.PhysicalSky)).enabled = true;
    const clouds = must(world.resource(og.Clouds));
    clouds.enabled = true;
    clouds.coverage = 0.97;
    clouds.density = 0.09;
    clouds.color = { r: 0.5, g: 0.5, b: 0.5 };
    clouds.base = 700;
    clouds.layer = 0;
    const fog = must(world.resource(og.Fog));
    fog.enabled = true;
    fog.density = 0.0006;
    fog.start = 20;

    const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 0, z: 0 } }));
    world.insert(sea, og.Water, {
      size: { x: 6000, y: 6000 },
      segments: 256,
      wind: { x: 7, y: -18.7 },
      choppiness: 1,
      foamWidth: 0,
      depthFade: 30,
      waveStrength: 0,
      deep: { r: 0.012, g: 0.035, b: 0.035 },
    });

    const hullBox = must(og.render.boxGeometry(world, { width: 2.4, height: 1, depth: 6 }));
    const cabinBox = must(og.render.boxGeometry(world, { width: 1.6, height: 1, depth: 1.8 }));
    const ball = must(og.render.sphereGeometry(world, { radius: 0.6 }));
    const white = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.85, b: 0.82 }, roughness: 0.6 }));
    const red = must(og.render.standardMaterial(world, { color: { r: 0.7, g: 0.1, b: 0.06 }, roughness: 0.5 }));
    const orange = must(og.render.standardMaterial(world, { color: { r: 0.95, g: 0.4, b: 0.05 }, roughness: 0.5 }));

    // Each floater set down on the water where it starts (the sea at tick
    // 0), afloat as it would be, so the swell takes it up from there.
    const level = (x: number, z: number) => must(og.physics.waterHeight(world, sea, x, z));
    BOATS.forEach((b, i) => {
      const boat = must(world.spawn({ name: `boat-${i}`, position: { x: b.x, y: level(b.x, b.z) + 0.2, z: b.z } }));
      world.set(boat, og.Transform, { quaternion: { x: 0, y: Math.sin(b.heading / 2), z: 0, w: Math.cos(b.heading / 2) } });
      world.insert(boat, og.Mesh, { geometry: hullBox, material: i === 1 ? red : white });
      world.insert(boat, og.RigidBody);
      // A hull of density 0.6 in water of 2: 0.3 of its metre under.
      world.insert(boat, og.Collider, { shape: SHAPE_BOX, size: { x: 2.4, y: 1, z: 6 }, density: 0.6 });
      world.insert(boat, og.Buoyancy, { samples: 6 });
      // The cabin: drawn only, on the boat's deck aft of its middle.
      const cabin = must(world.spawn({ name: `cabin-${i}`, position: { x: 0, y: 1, z: 1 } }));
      world.insert(cabin, og.Parent, { entity: boat });
      world.insert(cabin, og.Mesh, { geometry: cabinBox, material: white });
    });

    BUOYS.forEach((b, i) => {
      const buoy = must(world.spawn({ name: `buoy-${i}`, position: { x: b.x, y: level(b.x, b.z) + 0.1, z: b.z } }));
      world.insert(buoy, og.Mesh, { geometry: ball, material: orange });
      world.insert(buoy, og.RigidBody);
      world.insert(buoy, og.Collider, { shape: SHAPE_SPHERE, radius: 0.6, density: 0.8 });
      world.insert(buoy, og.Buoyancy, { samples: 5 });
    });
  },
});
