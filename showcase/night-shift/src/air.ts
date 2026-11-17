// The air over the alley: which weather, season and time of day it is, and
// the ramps between them. The game sets a target (a key, a command); the
// Air system moves the amounts in the air towards it a little each tick and
// writes what follows from them to the engine's sky, fog, weather, wind and
// lights. Everything is a function of the amounts, so a state looks the
// same however it was reached.
//
// The ramps between weather kinds are the game's own until the engine's
// `og.Weather.change` lands, and a season is a palette, an air and a rule
// for lying snow until `og.Season` does (the seasons and weather proposal).
import { defineResource, defineSystem, f32, og, u32, WEATHER_NONE, WEATHER_RAIN, WEATHER_SNOW } from "@origingame/engine";
import type { Color, World } from "@origingame/engine";
import { lerp, mix, must, rgb, smooth, v3 } from "./util.ts";

export const WEATHERS = ["rain", "storm", "fog", "clear", "snow"] as const;
export const SEASONS = ["spring", "summer", "autumn", "winter"] as const;
export const TIMES = ["night", "dawn"] as const;
export type WeatherName = (typeof WEATHERS)[number];
export type SeasonName = (typeof SEASONS)[number];
export type TimeName = (typeof TIMES)[number];

/** Seconds a weather takes to build or clear, a season to turn and the dawn to break. */
const WEATHER_RAMP = 6;
const SEASON_RAMP = 6;
const TIME_RAMP = 8;
/** How far standing water spreads over the street in steady rain, and in a storm. */
const PUDDLES = 0.45;
const PUDDLES_STORM = 0.75;

/** The targets (indices into WEATHERS, SEASONS and TIMES) and how much of each is in the air now, 0 to 1. */
export const Air = defineResource("Air", {
  weather: u32(), season: u32(), time: u32(),
  rain: f32(1), storm: f32(), fog: f32(), snow: f32(),
  spring: f32(1), summer: f32(), autumn: f32(), winter: f32(),
  dawn: f32(),
});

/** What a season does to the air: the haze's tint and weight, the wind, and how long snow lies. */
interface SeasonAir { readonly tint: Color; readonly haze: number; readonly wind: number; readonly melt: number }
const SEASON_AIR: Readonly<Record<SeasonName, SeasonAir>> = {
  // Mild and clear, blossom on the wind.
  spring: { tint: rgb(1.0, 0.86, 0.96), haze: 0.9, wind: 1.0, melt: 0.5 },
  // Humid: the air is heavy and holds the signs' light.
  summer: { tint: rgb(1.06, 0.92, 0.82), haze: 1.6, wind: 0.6, melt: 0.8 },
  // Dry gusts, amber leaves.
  autumn: { tint: rgb(1.08, 0.88, 0.7), haze: 1.1, wind: 1.5, melt: 0.3 },
  // Cold and thin; snow stays where it falls.
  winter: { tint: rgb(0.8, 0.9, 1.12), haze: 0.75, wind: 1.2, melt: 0 },
};

// The night sky is an overcast deck lit from below by the city: violet
// overhead, magenta towards the horizon, and the haze takes the same light.
const NIGHT = { zenith: rgb(0.06, 0.034, 0.12), horizon: rgb(0.3, 0.11, 0.36), fog: rgb(0.14, 0.055, 0.21) };
const DAWN = { zenith: rgb(0.1, 0.13, 0.3), horizon: rgb(0.95, 0.5, 0.42), fog: rgb(0.5, 0.32, 0.36) };
/** Towards the dawn sun: low over the towers at the alley's end. */
const TO_SUN = v3(0.25, 0.16, -1);
const tint = (c: Color, t: Color): Color => ({ r: c.r * t.r, g: c.g * t.g, b: c.b * t.b });

/** Sets the lights the air drives and the constants of its resources; the first tick's Air system does the rest. */
export function buildAir(world: World): void {
  const moon = must(world.spawn({ name: "moon", quaternion: og.transform.lookAt(v3(0, 0, 0), v3(-0.3, -0.8, -0.5)) }));
  must(world.insert(moon, og.DirectionalLight, { color: rgb(0.55, 0.6, 1), intensity: 0.08, castShadow: true }));
  const sun = must(world.spawn({ name: "dawn-sun", quaternion: og.transform.lookAt(v3(0, 0, 0), v3(-TO_SUN.x, -TO_SUN.y, -TO_SUN.z)) }));
  must(world.insert(sun, og.DirectionalLight, { color: rgb(1, 0.62, 0.45), intensity: 0, castShadow: true }));

  const sky = must(world.resource(og.Sky));
  sky.enabled = true;
  sky.ground = rgb(0.01, 0.008, 0.015);
  sky.sunDirection = TO_SUN;
  sky.sunColor = rgb(1, 0.6, 0.4);
  sky.glowPower = 6;
  sky.brightness = 1;

  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.height = 2;
  fog.falloff = 0.05;
  fog.volumetric = true;
  fog.volumetricDistance = 60;
  fog.scattering = 0.6;

  // The game's sky is the overcast the weather falls from, lit by the city
  // below it: the weather draws no deck of its own over it and dims no light.
  const weather = must(world.resource(og.Weather));
  weather.radius = 22;
  weather.dim = 0;
  // The street answers in seconds, so a switch shows while it is watched.
  const ground = must(world.resource(og.WeatherState));
  ground.soakTime = 4;
  ground.dryTime = 240;
  ground.drainTime = 400;
  ground.snowfall = 0.5;
  ground.lightningCenter = v3(0, 0, -60);
  ground.lightningRadius = 140;
  must(world.resource(og.Wind)).direction = v3(0.4, 0, -1);
}

const towards = (now: number, goal: number, step: number): number => (now < goal ? Math.min(goal, now + step) : Math.max(goal, now - step));

export const AirSystem = defineSystem("Air", { after: ["Switches"], writes: [Air, og.Weather, og.WeatherState, og.Fog, og.Sky, og.Wind, og.DirectionalLight] }, (_q, time, world) => {
  const air = must(world.resource(Air));
  const goal = WEATHERS[air.weather] ?? "clear";
  const w = time.dt / WEATHER_RAMP;
  air.rain = towards(air.rain, goal === "rain" ? 1 : 0, w);
  air.storm = towards(air.storm, goal === "storm" ? 1 : 0, w);
  air.fog = towards(air.fog, goal === "fog" ? 1 : 0, w);
  air.snow = towards(air.snow, goal === "snow" ? 1 : 0, w);
  const s = time.dt / SEASON_RAMP;
  air.spring = towards(air.spring, air.season === 0 ? 1 : 0, s);
  air.summer = towards(air.summer, air.season === 1 ? 1 : 0, s);
  air.autumn = towards(air.autumn, air.season === 2 ? 1 : 0, s);
  air.winter = towards(air.winter, air.season === 3 ? 1 : 0, s);
  air.dawn = towards(air.dawn, air.time === 1 ? 1 : 0, time.dt / TIME_RAMP);

  // The season's air: its four weighed by how much of each is here.
  const parts = [air.spring, air.summer, air.autumn, air.winter];
  const total = Math.max(parts[0]! + parts[1]! + parts[2]! + parts[3]!, 1e-6);
  let seasonTint = rgb(0, 0, 0), haze = 0, gust = 0, melt = 0;
  for (const [i, name] of SEASONS.entries()) {
    const k = parts[i]! / total, a = SEASON_AIR[name];
    seasonTint = rgb(seasonTint.r + a.tint.r * k, seasonTint.g + a.tint.g * k, seasonTint.b + a.tint.b * k);
    haze += a.haze * k; gust += a.wind * k; melt += a.melt * k;
  }

  // One kind falls at a time: the heavier of rain (a storm is rain twice
  // as heavy) and snow, thinning to nothing where they cross.
  const rain = smooth(air.rain) + 2 * smooth(air.storm), snow = smooth(air.snow);
  const weather = must(world.resource(og.Weather));
  if (rain > snow) weather.kind = WEATHER_RAIN;
  else if (snow > rain) weather.kind = WEATHER_SNOW;
  else if (rain === 0) weather.kind = WEATHER_NONE;
  weather.intensity = Math.abs(rain - snow);

  const storm = smooth(air.storm), mist = smooth(air.fog), dawn = smooth(air.dawn);
  const ground = must(world.resource(og.WeatherState));
  ground.lightning = 14 * storm;
  ground.melt = melt;
  // The street drains: its water stands in the joints and the hollows,
  // more of them in a storm, never as one sheet. (The engine fills puddles
  // to 1 under steady rain and has no limit to set, so the fill is stopped
  // at the level here until it has one.)
  ground.puddleTime = ground.puddles < lerp(PUDDLES, PUDDLES_STORM, storm) ? 8 : 1e6;
  must(world.resource(og.Wind)).speed = (3 + 9 * storm + 2 * snow) * gust;

  // The haze: thicker in fog, a storm and falling snow, and by the season.
  const thick = haze * (1 + 3.2 * mist + 0.9 * storm + 0.7 * snow);
  const fog = must(world.resource(og.Fog));
  fog.color = tint(mix(NIGHT.fog, DAWN.fog, dawn), seasonTint);
  fog.density = 0.0045 * thick;
  fog.heightDensity = 0.004 * thick;

  // The sky and its two lights: the moon gives way to a low sun, which the
  // weather's cover dims.
  const open = 1 - 0.7 * Math.max(storm, mist, snow, smooth(air.rain) * 0.6);
  const sky = must(world.resource(og.Sky));
  sky.zenith = tint(mix(NIGHT.zenith, DAWN.zenith, dawn), seasonTint);
  sky.horizon = tint(mix(NIGHT.horizon, DAWN.horizon, dawn), seasonTint);
  sky.sunSize = 2.2 * dawn * open;
  sky.glow = 0.9 * dawn * open;
  const moon = world.find("moon"), sun = world.find("dawn-sun");
  if (moon !== null) must(world.set(moon, og.DirectionalLight, { intensity: lerp(0.08, 0, dawn) }));
  if (sun !== null) must(world.set(sun, og.DirectionalLight, { intensity: 1.6 * dawn * open }));
});

function pick<T extends string>(names: readonly T[], arg: string | undefined, what: string): number {
  const i = names.indexOf(arg as T);
  if (i < 0) throw new Error(`${what}: one of ${names.join(", ")}`);
  return i;
}
/** The switches by name, for the keys, the commands and the captures. */
export const setWeather = (world: World, name: string | undefined): void => { must(world.resource(Air)).weather = pick(WEATHERS, name, "weather"); };
export const setSeason = (world: World, name: string | undefined): void => { must(world.resource(Air)).season = pick(SEASONS, name, "season"); };
export const setTime = (world: World, name: string | undefined): void => { must(world.resource(Air)).time = pick(TIMES, name, "time"); };
