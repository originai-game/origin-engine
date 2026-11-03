// The introduction site's live hero: an open sea under og.PhysicalSky and
// og.Clouds, a wind-driven Gerstner sea (og.Water with a wind), a schooner
// floating on it by buoyancy (src/ship.ts), a headland and its lighthouse
// on the far-left horizon (src/land.ts), playing itself through a 90 s loop
// (src/loop.ts): golden hour, a storm with lightning (og.Weather,
// og.WeatherState), a moonlit night, dawn, noon, and golden hour again.
//
// Everything the loop sets is a function of SiteControl.time, which the
// tick advances: the world's state and every frame follow from the tick.
//
// Commands (dev builds): `loop:<seconds>` jumps the loop there (the ship
// with it); `rate:<r>` plays it at r times (0 holds the moment).
//
//   og run examples/site-ocean --size 1920x1080 --commands "4:screenshot:dusk.png;4.1:quit"

import { og, defineComponent, defineResource, defineSystem, f32, u32, WEATHER_NONE, WEATHER_RAIN } from "@origingame/engine";
import type { Entity, Material, StandardMaterialDesc, World } from "@origingame/engine";
import { buildLand, HEADLAND, lighthouseAt } from "./land.ts";
import { hourAt, LOOP, shipXAt, SHIP_SPEED, SHIP_Z, shotAt, stormAt } from "./loop.ts";
import { must } from "./mesh.ts";
import { buildShip, floatShip, Ship } from "./ship.ts";

/** The loop's clock, on the entity `site`. */
export const SiteControl = defineComponent("SiteControl", {
  time: f32(0, { min: 0, max: LOOP, unit: "s", doc: "Seconds into the loop." }),
  rate: f32(1, { min: 0, max: 8, doc: "How fast the loop plays (0 holds the moment)." }),
  hour: f32(17.62, { min: 0, max: 24, doc: "The hour drawn (written from time)." }),
  storm: f32(0, { min: 0, max: 1, doc: "The storm, 0 fair to 1 a gale with lightning (written from time)." }),
});

/** What the camera smooths between ticks: the sea under it, so it bobs and rolls a little. */
export const Rider = defineResource("Rider", {
  height: f32(0, { unit: "m" }),
  roll: f32(0, { unit: "rad" }),
  jumps: u32(0, { doc: "Loop jumps (the loop command), so the smoothing restarts." }),
});

// The sea: a breeze of 10 m/s blowing towards the camera (from -z), waves
// some 80 m long; its speed never changes (that would move every wave), the
// storm raises their height and leans their crests.
const WIND = { x: 2.6, y: 9.65 };
const SEA = 6000;

const scene = {
  site: 0 as unknown as Entity,
  sea: 0 as unknown as Entity,
  ship: 0 as unknown as Entity,
  sun: 0 as unknown as Entity,
  lamps: [] as Entity[],
  beam: 0 as unknown as Entity,
  lighthouseLight: 0 as unknown as Entity,
  // Materials whose glow follows the night, with what they were made with.
  glows: [] as Array<{ m: Material; desc: StandardMaterialDesc; key: "emissiveIntensity" | "opacity"; last: number }>,
};

function start(world: World): void {
  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.04, y: -0.09, z: 0.99 }) }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true, shadowDistance: 260, shadowSoftness: 0.01 }));
  scene.sun = sun;

  Object.assign(must(world.resource(og.DayNight)), { enabled: true, start: 17.62, speed: 0, tilt: 25, azimuth: -90, lightIntensity: 1.1, tintFog: true, step: 2, moonSize: 1.5, moonPhase: 0.42 });
  Object.assign(must(world.resource(og.PhysicalSky)), { enabled: true, haze: 1.6, groundAlbedo: { r: 0.05, g: 0.08, b: 0.1 } });
  Object.assign(must(world.resource(og.Clouds)), { enabled: true, steps: 64, scale: 11000, layerAltitude: 8500 });
  Object.assign(must(world.resource(og.Fog)), { enabled: true, start: 30, heightDensity: 0, maxOpacity: 1 });
  Object.assign(must(world.resource(og.WeatherState)), { lightningCenter: { x: 60, y: 0, z: -750 }, lightningRadius: 420, shelterSize: 0 });

  const sea = must(world.spawn({ name: "sea", position: { x: 0, y: 0, z: 0 } }));
  must(
    world.insert(sea, og.Water, {
      size: { x: SEA, y: SEA },
      segments: 256,
      wind: WIND,
      swell: 0.6,
      choppiness: 0.35,
      deep: { r: 0.008, g: 0.045, b: 0.06 },
      shallow: { r: 0.1, g: 0.3, b: 0.3 },
      depthFade: 30,
      foamWidth: 0,
      foam: { r: 0.6, g: 0.66, b: 0.68 },
      waveStrength: 0,
    }),
  );
  scene.sea = sea;

  const ship = buildShip(world, { x: shipXAt(0), y: 0, z: SHIP_Z });
  scene.ship = ship.ship;
  scene.lamps = ship.lights;
  const land = buildLand(world);
  scene.beam = land.beam;
  scene.lighthouseLight = land.light;
  beamState.material = land.beamMaterial;
  scene.glows = [
    { m: ship.lamps, desc: { color: { r: 1, g: 0.75, b: 0.45 }, emissive: { r: 1, g: 0.62, b: 0.3 }, emissiveIntensity: 0 }, key: "emissiveIntensity", last: -1 },
    { m: land.lamp, desc: { color: { r: 0.9, g: 0.85, b: 0.7 }, emissive: { r: 1, g: 0.8, b: 0.5 }, emissiveIntensity: 0.3, roughness: 0.2 }, key: "emissiveIntensity", last: -1 },
  ];

  const site = must(world.spawn({ name: "site" }));
  must(world.insert(site, SiteControl, {}));
  scene.site = site;

  const cam = must(world.resource(og.Camera));
  cam.near = 0.3;
  cam.far = 9000;
  must(world.resource(og.RenderSettings)).sky = true;
  pose(world, 0, 0, 0);
}

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

/** The camera at loop time t, riding `height` of swell with `roll`. */
function pose(world: World, t: number, height: number, roll: number): void {
  const s = shotAt(t);
  const cam = must(world.resource(og.Camera));
  cam.position = { x: s.eye[0], y: s.eye[1] + height, z: s.eye[2] };
  const yaw = og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, -s.yaw * DEG);
  const pitch = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, s.pitch * DEG);
  const tilt = og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, roll);
  cam.quaternion = mul(mul(yaw, pitch), tilt);
  cam.fov = s.fov;
}

const smoothstep = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};
const mix = (a: number, b: number, t: number) => a + (b - a) * t;

/** The sun's elevation (its direction's y) at an hour, as og.DayNight puts it (tilt 25°). */
function sunHeight(hour: number): number {
  const a = ((hour - 6) / 24) * Math.PI * 2;
  return Math.sin(a) * Math.cos(25 * DEG);
}

const Loop = defineSystem(
  "Loop",
  {
    reads: [og.Velocity],
    writes: [SiteControl, Rider, Ship, og.Camera, og.Transform, og.Water, og.DayNight, og.Clouds, og.PhysicalSky, og.Fog, og.Weather, og.WeatherState, og.PointLight, og.DirectionalLight, og.RenderSettings],
  },
  (_q, time, world) => {
    const c = must(world.get(scene.site, SiteControl));
    const before = c.time;
    let t = c.time + time.dt * c.rate;
    if (t >= LOOP) t -= LOOP;
    const hour = hourAt(t);
    const storm = stormAt(t);
    must(world.set(scene.site, SiteControl, { time: t, hour, storm }));

    // The ship: turned back with the loop (while the camera looks up at the moon), then floated.
    const x = shipXAt(t);
    if (x < shipXAt(before) - 1) {
      const p = must(world.get(scene.ship, og.Transform)).position;
      must(world.set(scene.ship, og.Transform, { position: { x: p.x + (x - shipXAt(before)), y: p.y, z: p.z } }));
    }
    floatShip(world, scene.ship, scene.sea, x, SHIP_SPEED * c.rate, SHIP_Z);

    // The camera rides the swell: the sea's height under it, smoothed over a
    // second, and a little of the sea's slope across it as roll.
    const rider = must(world.resource(Rider));
    const s = shotAt(t);
    const across = { x: Math.cos(s.yaw * DEG), z: Math.sin(s.yaw * DEG) };
    const hl = must(og.physics.waterHeight(world, scene.sea, s.eye[0] - across.x * 2, s.eye[2] - across.z * 2));
    const hr = must(og.physics.waterHeight(world, scene.sea, s.eye[0] + across.x * 2, s.eye[2] + across.z * 2));
    const k = 1 - Math.exp(-time.dt / 1.1);
    rider.height += ((hl + hr) * 0.5 * 0.55 - rider.height) * k;
    rider.roll += (Math.atan((hl - hr) / 4) * 0.35 - rider.roll) * k;
    pose(world, t, rider.height, rider.roll);

    // The sea follows the camera (its waves stay where they are).
    const cam = must(world.resource(og.Camera));
    must(world.set(scene.sea, og.Transform, { position: { x: cam.position.x, y: 0, z: cam.position.z } }));
    must(
      world.set(scene.sea, og.Water, {
        swell: mix(0.6, 1.6, storm),
        choppiness: mix(0.35, 0.85, storm),
        foam: { r: mix(0.6, 0.9, storm), g: mix(0.66, 0.92, storm), b: mix(0.68, 0.93, storm) },
        deep: { r: mix(0.008, 0.012, storm), g: mix(0.045, 0.035, storm), b: mix(0.06, 0.035, storm) },
      }),
    );

    // The day: the hour drawn, the sky's haze and the clouds by the storm.
    const day = must(world.resource(og.DayNight));
    day.start = hour;
    day.lightIntensity = mix(1.1, 0.7, storm);
    const sky = must(world.resource(og.PhysicalSky));
    sky.haze = mix(1.6, 7, storm);
    const clouds = must(world.resource(og.Clouds));
    Object.assign(clouds, {
      coverage: mix(0.36, 0.97, storm),
      density: mix(0.032, 0.09, storm),
      color: { r: mix(1, 0.42, storm), g: mix(1, 0.43, storm), b: mix(1, 0.46, storm) },
      base: mix(1500, 750, storm),
      thickness: mix(1700, 2400, storm),
      detail: mix(0.45, 0.3, storm),
      layer: mix(0.42, 0, storm),
      wind: { x: 3, y: 0.3, z: 9 },
    });
    const fog = must(world.resource(og.Fog));
    fog.density = mix(3.5 / SEA, 0.0045, storm);
    fog.sky = mix(0.25, 0.8, storm);
    const weather = must(world.resource(og.Weather));
    weather.kind = storm > 0.04 ? WEATHER_RAIN : WEATHER_NONE;
    weather.intensity = smoothstep(0.04, 0.8, storm) * 1.5;
    weather.wind = { x: 1.2, y: 0, z: 5 };
    weather.radius = 22;
    must(world.resource(og.WeatherState)).lightning = storm > 0.7 ? 22 * smoothstep(0.7, 1, storm) : 0;

    // After sunset the lamps light: the ship's lanterns, the lighthouse and its turning beam.
    const night = 1 - smoothstep(-0.03, 0.05, sunHeight(hour));
    for (const e of scene.lamps) must(world.set(e, og.PointLight, { intensity: 6 * night }));
    must(world.set(scene.lighthouseLight, og.PointLight, { intensity: 400 * night }));
    const turn = (time.elapsed * 0.6) % (Math.PI * 2);
    must(world.set(scene.beam, og.Transform, { quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) }));
    // The lamp flashes as the beam sweeps past the camera.
    const lh = lighthouseAt();
    const toCam = Math.atan2(-(cam.position.z - lh.z), cam.position.x - lh.x);
    const facing = Math.pow(Math.max(Math.abs(Math.cos(turn - toCam)), 0), 40);
    glow(world, 0, 7 * night);
    glow(world, 1, 0.3 + night * (8 + 40 * facing));
    beamOpacity(world, night * (1 - 0.6 * storm));
  },
);

/** Sets a glowing material's emissive intensity when it has moved enough to see. */
function glow(world: World, i: number, value: number): void {
  const g = scene.glows[i]!;
  if (Math.abs(value - g.last) < 0.02 * Math.max(1, value)) return;
  g.last = value;
  must(og.render.updateMaterial(world, g.m, { ...g.desc, [g.key]: value }));
}

const beamState = { material: null as Material | null, last: -1 };
function beamOpacity(world: World, value: number): void {
  if (beamState.material === null) return;
  if (Math.abs(value - beamState.last) < 0.01) return;
  beamState.last = value;
  must(og.render.updateMaterial(world, beamState.material, { unlit: true, transparent: true, opacity: value, side: "double", color: { r: 1, g: 0.9, b: 0.7 } }));
}

function loopTo(world: World, args: readonly string[]): void {
  const t = Number(args[0]);
  if (!Number.isFinite(t)) throw new Error("write loop:<seconds>");
  const at = ((t % LOOP) + LOOP) % LOOP;
  must(world.set(scene.site, SiteControl, { time: at }));
  const p = must(world.get(scene.ship, og.Transform)).position;
  must(world.set(scene.ship, og.Transform, { position: { x: shipXAt(at), y: p.y, z: p.z } }));
  must(world.set(scene.ship, og.Velocity, { linear: { x: SHIP_SPEED, y: 0, z: 0 } }));
  const r = must(world.resource(Rider));
  r.jumps += 1;
}

function rate(world: World, args: readonly string[]): void {
  const r = Number(args[0]);
  if (!Number.isFinite(r) || r < 0) throw new Error("write rate:<r>");
  must(world.set(scene.site, SiteControl, { rate: r }));
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/site-ocean.ogt",
  resources: [Rider],
  systems: [Loop],
  start,
  commands: { loop: loopTo, rate },
  commandHelp: {
    loop: { usage: "<seconds>", doc: "Jump the loop to this many seconds in (0 is the golden-hour opening; the ship moves with it)." },
    rate: { usage: "<r>", doc: "Play the loop at r times (0 holds the moment; the sea and clouds keep moving)." },
  },
  facts: (world) => {
    const c = world.get(scene.site, SiteControl);
    const s = world.get(scene.ship, Ship);
    if (!c.ok || !s.ok) return null;
    return {
      loop: Math.round(c.value.time * 10) / 10,
      hour: Math.round(c.value.hour * 100) / 100,
      storm: Math.round(c.value.storm * 100) / 100,
      ship: { submerged: s.value.submerged, heave: Math.round(s.value.heave * 100) / 100, roll: Math.round(s.value.roll * 10) / 10, pitch: Math.round(s.value.pitch * 10) / 10 },
      headland: HEADLAND,
    };
  },
});
