// The air: what the player switches (weather, season, time of day) and
// how the sky, the light, the fog and the wind ease from one choice to the
// next. Every choice is world state in the Scene resource, so a capture
// script, a replay and a save all land on the same frame.
import { og, defineEvent, defineResource, defineSystem, f64, math, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { must } from "./util.ts";

export const WEATHERS = ["clear", "rain", "storm", "fog", "snow"] as const;
export const SEASONS = ["spring", "summer", "autumn", "winter"] as const;
export const TIMES = ["dawn", "noon", "dusk", "night"] as const;

/** The heading the sun rises at: ahead and to the left, behind the oak. */
const SUNRISE = 196;
/** The hour each time of day holds. */
const HOURS = [6.9, 12.4, 18.55, 23.4];
/** Seconds a change of weather takes to settle, and game hours the clock runs per second while the time changes. */
const SETTLE = 6, CLOCK = 3.2;

/** What the player chose, the hour drawn now, and which kinds of switch they have made (bits: weather 1, season 2, night 4). */
export const Scene = defineResource("Scene", { weather: u32(0), season: u32(1), time: u32(0), hud: u32(1), hour: f64(HOURS[0]!), falling: u32(0), seen: u32(0) });

/** A hotbar slot pressed: 0 weather, 1 season, 2 time. */
export const Pick = defineEvent("Pick", { which: u32(0, { min: 0, max: 2 }) });

/** Each weather: the engine's kind and how heavy, the cumulus cover and how dark, the fog, the wind and the lightning. */
const AIR = [
  { kind: 0, intensity: 0, cover: 0.34, density: 0.04, grey: 1, fog: 0.00002, mist: 0.006, wind: 2.5, lightning: 0 },
  { kind: 1, intensity: 0.8, cover: 0.92, density: 0.055, grey: 0.7, fog: 0.00018, mist: 0.02, wind: 5, lightning: 0 },
  { kind: 1, intensity: 1.8, cover: 1, density: 0.08, grey: 0.42, fog: 0.0003, mist: 0.02, wind: 13, lightning: 9 },
  { kind: 4, intensity: 1, cover: 0.7, density: 0.04, grey: 0.9, fog: 0.004, mist: 0.07, wind: 0.8, lightning: 0 },
  { kind: 2, intensity: 1, cover: 0.95, density: 0.05, grey: 0.85, fog: 0.0004, mist: 0.02, wind: 3, lightning: 0 },
];

export function buildEnv(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true }));
  const day = must(world.resource(og.DayNight));
  day.enabled = true;
  day.start = HOURS[0]!;
  day.speed = 0;
  day.azimuth = SUNRISE;
  must(world.resource(og.PhysicalSky)).enabled = true;
  const clouds = must(world.resource(og.Clouds));
  clouds.enabled = true;
  clouds.coverage = AIR[0]!.cover;
  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.density = AIR[0]!.fog;
  fog.heightDensity = AIR[0]!.mist;
  fog.height = 1;
  fog.falloff = 0.14;
  fog.volumetric = true;
  fog.volumetricDistance = 160;
  const wind = must(world.resource(og.Wind));
  wind.direction = { x: 0.8, y: 0, z: -0.6 };
  wind.speed = AIR[0]!.wind;
  const post = must(world.resource(og.PostEffects));
  post.godRays = true;
  post.godRaysIntensity = 0.55;
}

/** The next choice of each kind, from the keys and the hotbar; `set_scene` writes them from a script. */
export const Choose = defineSystem("Choose", { reads: [og.Input, Pick], writes: [Scene] }, (_q, _t, world) => {
  const s = must(world.resource(Scene));
  const picked = [false, false, false];
  for (const pick of world.events(Pick)) picked[pick.which] = true;
  if (picked[0] || og.input.isActionJustPressed(world, "weather")) { s.weather = (s.weather + 1) % WEATHERS.length; s.seen |= 1; }
  if (picked[1] || og.input.isActionJustPressed(world, "season")) { s.season = (s.season + 1) % SEASONS.length; s.seen |= 2; }
  if (picked[2] || og.input.isActionJustPressed(world, "time")) s.time = (s.time + 1) % TIMES.length;
  if (s.time === 3) s.seen |= 4;
  // The HUD's tasks (hud.ts): a weather, a season and the night seen.
  if (s.seen === 7) og.assert.reachable("every task of the day done", {});
  if (og.input.isActionJustPressed(world, "hud")) s.hud = 1 - s.hud;
});

/** Eases the air towards the chosen weather and runs the clock on to the chosen time. */
export const Air = defineSystem("Air", { reads: [og.Time], writes: [Scene, og.DayNight, og.Clouds, og.Fog, og.Wind, og.Weather, og.WeatherState], after: ["Choose"] }, (_q, time, world) => {
  const s = must(world.resource(Scene));
  const air = AIR[s.weather]!;
  const k = 1 - math.exp(-(3 / SETTLE) * time.dt);
  const ease = (from: number, to: number) => from + (to - from) * k;

  const clouds = must(world.resource(og.Clouds));
  clouds.coverage = ease(clouds.coverage, air.cover);
  clouds.density = ease(clouds.density, air.density);
  const grey = ease(clouds.color.r, air.grey);
  clouds.color = { r: grey, g: grey, b: grey };
  const fog = must(world.resource(og.Fog));
  fog.density = ease(fog.density, air.fog);
  fog.heightDensity = ease(fog.heightDensity, air.mist);
  const wind = must(world.resource(og.Wind));
  wind.speed = ease(wind.speed, air.wind);
  must(world.resource(og.WeatherState)).lightning = air.lightning;

  // What falls: the old kind thins out before the new one builds.
  const weather = must(world.resource(og.Weather));
  if (s.falling !== air.kind) {
    weather.intensity = Math.max(0, weather.intensity - time.dt / (SETTLE * 0.4));
    if (weather.intensity <= 0) {
      s.falling = air.kind;
      weather.kind = air.kind;
    }
  } else {
    weather.intensity = ease(weather.intensity, air.intensity);
  }

  // The clock only runs forward: dusk to dawn passes through the night.
  const ahead = (HOURS[s.time]! - s.hour + 24) % 24;
  const step = Math.min(ahead, Math.max(0.004, ahead * 1.6) * CLOCK * time.dt);
  s.hour = (s.hour + (ahead < 0.002 ? ahead : step)) % 24;
  must(world.resource(og.DayNight)).start = s.hour;
});

/** `set_scene:<weather|->:<season|->:<time|->[:now]`: a script's choice; `now` lands on it at once. */
export function setScene(world: World, args: readonly string[]): void {
  const s = must(world.resource(Scene));
  const pick = (names: readonly string[], arg: string | undefined, current: number) => {
    if (arg === undefined || arg === "-" || arg === "") return current;
    const i = names.indexOf(arg);
    if (i < 0) throw new Error(`set_scene: ${arg} is not one of ${names.join(", ")}`);
    return i;
  };
  s.weather = pick(WEATHERS, args[0], s.weather);
  s.season = pick(SEASONS, args[1], s.season);
  s.time = pick(TIMES, args[2], s.time);
  if (args[3] !== "now") return;
  const air = AIR[s.weather]!;
  const clouds = must(world.resource(og.Clouds));
  clouds.coverage = air.cover;
  clouds.density = air.density;
  clouds.color = { r: air.grey, g: air.grey, b: air.grey };
  const fog = must(world.resource(og.Fog));
  fog.density = air.fog;
  fog.heightDensity = air.mist;
  must(world.resource(og.Wind)).speed = air.wind;
  const weather = must(world.resource(og.Weather));
  weather.kind = air.kind;
  weather.intensity = air.intensity;
  s.falling = air.kind;
  s.hour = HOURS[s.time]!;
  must(world.resource(og.DayNight)).start = s.hour;
}
