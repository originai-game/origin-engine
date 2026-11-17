// The light: the sun going down over the sea, the sky and its clouds, the
// wind and the sea mist.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { must } from "./must.ts";

export function buildSky(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true, shadowDistance: 160 }));
  const day = must(world.resource(og.DayNight));
  day.enabled = true;
  day.start = 17.8;
  day.speed = 0;
  // The sun goes down over the sea, to the right of the headland.
  day.azimuth = 333;
  day.tilt = 35;
  const air = must(world.resource(og.PhysicalSky));
  air.enabled = true;
  // A sea haze: a wider glow round the low sun, a redder horizon.
  air.haze = 1;
  air.airDensity = 1.1;
  must(world.resource(og.RenderSettings)).sky = true;

  const clouds = must(world.resource(og.Clouds));
  clouds.enabled = true;
  clouds.coverage = 0.52;
  clouds.detail = 1;
  clouds.scale = 7000;
  clouds.steps = 96;
  clouds.density = 0.03;
  clouds.base = 1300;
  clouds.thickness = 1100;
  clouds.layer = 0.6;

  const wind = must(world.resource(og.Wind));
  wind.direction = { x: -0.9, y: 0, z: 0.45 };
  wind.speed = 8;

  // Sea mist lying under the cliffs; the air above stays clear.
  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.density = 0.00012;
  fog.heightDensity = 0.006;
  fog.height = 3;
  fog.falloff = 0.11;
  fog.sky = 0.25;
}
