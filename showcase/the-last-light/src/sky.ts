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
  // Fair-weather cumulus at sunset: a few large heads with clear sky
  // between, smaller and flatter towards the horizon, under a thin high veil.
  clouds.coverage = 0.28;
  clouds.detail = 0.35;
  clouds.scale = 11000;
  clouds.steps = 128;
  clouds.density = 0.035;
  clouds.base = 1500;
  clouds.thickness = 1500;
  clouds.layer = 0.08;
  clouds.seed = 7;

  const wind = must(world.resource(og.Wind));
  wind.direction = { x: -0.9, y: 0, z: 0.45 };
  wind.speed = 4;
  // A steady sea breeze in broad gusts: the grass and the cloak lean and
  // breathe with it rather than thrash.
  wind.gust = 0.4;
  wind.turbulence = 0.15;
  wind.gustSize = 36;

  // Sea mist lying under the cliffs; the air above stays clear.
  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.density = 0.00012;
  fog.heightDensity = 0.006;
  fog.height = 3;
  fog.falloff = 0.11;
  fog.sky = 0.25;
}
