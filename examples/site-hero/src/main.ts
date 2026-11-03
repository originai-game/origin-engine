// The introduction site's live hero, "Skyreach Meadow": a cliff-top meadow
// over a sea of cloud at a fixed late-afternoon sun (og.PhysicalSky with
// og.DayNight held still, og.Clouds below the eye).
//
// This first cut is the cloud-sea test the art director asked for before
// the build: a stand-in cliff for scale and three views, chosen with
// `shot:<n>` (dev builds).
//
//   og run examples/site-hero --size 1920x1080 --commands "6:screenshot:sea-0.png;6.1:shot:1;9:screenshot:sea-1.png;9.1:quit"

import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { must } from "./mesh.ts";

/** The hour whose sun stands 15° up in the afternoon (og.DayNight, tilt 25°). */
const HOUR = 16.89;
/** The eye stands on the cliff, 140 m over the cloud deck's top. */
const EYE_Y = 560;
const CLOUD_BASE = 120;
const CLOUD_THICKNESS = 300;

export const View = defineResource("View", {
  shot: u32(0, { max: 2, doc: "The test view: 0 the hero's framing, 1 looking down onto the deck, 2 wide into the sun." }),
});

const scene = { sun: 0 as unknown as Entity };

const DEG = Math.PI / 180;
type Quat = { x: number; y: number; z: number; w: number };
function mul(a: Quat, b: Quat): Quat {
  return {
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
  };
}

/** The views: yaw right of -z and pitch up, in degrees, and the fov. */
const SHOTS = [
  { yaw: -14, pitch: -3, fov: 42 },
  { yaw: -6, pitch: -14, fov: 42 },
  { yaw: 8, pitch: 0, fov: 70 },
];

function start(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true, shadowDistance: 200, shadowSoftness: 0.02 }));
  scene.sun = sun;
  Object.assign(must(world.resource(og.DayNight)), { enabled: true, start: HOUR, speed: 0, tilt: 25, azimuth: -90, lightIntensity: 1.1, tintFog: true, step: 6 });
  Object.assign(must(world.resource(og.PhysicalSky)), { enabled: true, haze: 1.2 });
  Object.assign(must(world.resource(og.Clouds)), {
    enabled: true,
    coverage: 0.72,
    density: 0.05,
    base: CLOUD_BASE,
    thickness: CLOUD_THICKNESS,
    scale: 2600,
    detail: 0.5,
    steps: 64,
    layer: 0.3,
    layerAltitude: 8000,
    wind: { x: 4, y: 0.3, z: 1.5 },
  });
  Object.assign(must(world.resource(og.Fog)), { enabled: true, start: 60, density: 0.00008, sky: 0.08, heightDensity: 0, maxOpacity: 0.85 });

  // A stand-in cliff for scale: a dark rock column under the eye, its top
  // a little below the eye, off to the right.
  const rock = must(og.render.standardMaterial(world, { color: { r: 0.16, g: 0.17, b: 0.15 }, roughness: 0.95 }));
  const grass = must(og.render.standardMaterial(world, { color: { r: 0.26, g: 0.4, b: 0.2 }, roughness: 0.9 }));
  const column = must(world.spawn({ name: "cliff", position: { x: 30, y: EYE_Y - 2 - 300, z: -20 } }));
  must(world.insert(column, og.Mesh, { geometry: must(og.render.boxGeometry(world, { width: 50, height: 600, depth: 70 })), material: rock, castShadow: true, receiveShadow: true }));
  const top = must(world.spawn({ name: "meadow", position: { x: 30, y: EYE_Y - 2 + 0.1, z: -20 } }));
  must(world.insert(top, og.Mesh, { geometry: must(og.render.boxGeometry(world, { width: 50, height: 0.2, depth: 70 })), material: grass, receiveShadow: true }));

  const cam = must(world.resource(og.Camera));
  cam.near = 0.3;
  cam.far = 60000;
  must(world.resource(og.RenderSettings)).sky = true;
}

const Frame = defineSystem("Frame", { reads: [View], writes: [og.Camera] }, (_q, _time, world) => {
  const s = SHOTS[must(world.resource(View)).shot] ?? SHOTS[0]!;
  const cam = must(world.resource(og.Camera));
  cam.position = { x: 0, y: EYE_Y, z: 0 };
  const yaw = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, -s.yaw * DEG);
  const pitch = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, s.pitch * DEG);
  cam.quaternion = mul(yaw, pitch);
  cam.fov = s.fov;
});

function shot(world: World, args: readonly string[]): void {
  const n = Number(args[0]);
  if (!Number.isInteger(n) || n < 0 || n >= SHOTS.length) throw new Error(`write shot:<0..${SHOTS.length - 1}>`);
  must(world.resource(View)).shot = n;
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/skyreach.ogt",
  resources: [View],
  systems: [Frame],
  start,
  commands: { shot },
  commandHelp: { shot: { usage: "<n>", doc: "Pick the test view (0 the hero's framing, 1 down onto the cloud deck, 2 wide into the sun)." } },
  facts: (world) => {
    const t = world.get(scene.sun, og.Transform);
    return t.ok ? { sun: t.value.quaternion } : null;
  },
});
