// The R5 sea sample (batch 1): a fresh breeze's sea (og.Water.wind) under
// the physical sky in the afternoon, five red buoys that the `float`
// command sets on og.physics.waterHeight (the tick's own sea, a readout
// read between ticks as og.physics' readouts are), so they sit on the
// waves the renderer draws. tests/evidence/r5-water/NOTES.md has the commands and
// what each frame shows.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** Where the buoys ride: a line across the view, 15 to 55 m out. */
const BUOYS = [
  { x: -9, z: -15 },
  { x: -4, z: -24 },
  { x: 2, z: -32 },
  { x: 8, z: -42 },
  { x: 15, z: -55 },
];

/** Each buoy on the sea's surface at its point, this tick. */
function float(world: World): void {
  const sea = world.find("sea");
  if (sea === null) return;
  BUOYS.forEach((b, i) => {
    const buoy = world.find(`buoy-${i}`);
    if (buoy === null) return;
    const y = must(og.physics.waterHeight(world, sea, b.x, b.z));
    world.set(buoy, og.Transform, { position: { x: b.x, y, z: b.z }, scale: { x: 0.8, y: 0.8, z: 0.8 } });
  });
}

export default og.game({
  tick: 60,
  seed: 1,
  commands: {
    /** `float`: set every buoy on the surface (the sea at this tick). */
    float: (world: World) => float(world),
  },
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0, y: 4, z: 0 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 1.5, z: -60 });
    cam.fov = 50;
    cam.far = 3000;

    const day = must(world.resource(og.DayNight));
    day.enabled = true;
    day.start = 15;
    day.speed = 0;
    must(world.resource(og.PhysicalSky)).enabled = true;
    const fog = must(world.resource(og.Fog));
    fog.enabled = true;
    fog.density = 0.005;
    fog.start = 20;

    const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 0, z: 0 } }));
    world.insert(sea, og.Water, {
      size: { x: 1200, y: 1200 },
      segments: 256,
      wind: { x: -3, y: -8.5 },
      choppiness: 0.6,
      foamWidth: 0,
      depthFade: 30,
      waveStrength: 0.2,
    });

    const ball = must(og.render.sphereGeometry(world, {}));
    const red = must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.12, b: 0.08 }, roughness: 0.5 }));
    BUOYS.forEach((b, i) => {
      const buoy = must(world.spawn({ name: `buoy-${i}`, position: { x: b.x, y: 0, z: b.z } }));
      world.set(buoy, og.Transform, { scale: { x: 0.8, y: 0.8, z: 0.8 } });
      world.insert(buoy, og.Mesh, { geometry: ball, material: red });
    });
  },
});
