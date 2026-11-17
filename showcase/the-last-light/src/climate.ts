// The air and the year: the weather, the season and the time of day, each
// chosen by the player (or a command) and blended in over a few seconds.
//
// Each axis keeps a weight per state in the Climate resource; a switch moves
// the weights towards the chosen state, and what is drawn is the weighted
// mean of the states' tables below. A switch made halfway through another
// carries on from where the air is.
import { WEATHER_NONE, WEATHER_RAIN, WEATHER_SNOW, defineResource, defineSystem, f32, og, u32 } from "@origingame/engine";
import type { Color, World } from "@origingame/engine";
import { must } from "./must.ts";

export const WEATHERS = ["clear", "rain", "storm", "fog", "snow"] as const;
export const SEASONS = ["spring", "summer", "autumn", "winter"] as const;
export const TIMES = ["sunset", "night", "dawn"] as const;

/** Seconds a weather, a season and a day's hour take to come in. */
const WEATHER_SECONDS = 6, SEASON_SECONDS = 8;
/** Game hours the clock runs through per second on its way to the chosen time (always forwards). */
const HOURS_PER_SECOND = 1.6;
const HOURS = [17.9, 23.2, 6.25] as const;

export const Climate = defineResource("Climate", {
  weather: u32(0), season: u32(1), time: u32(0),
  clear: f32(1), rain: f32(), storm: f32(), fog: f32(), snow: f32(),
  spring: f32(), summer: f32(1), autumn: f32(), winter: f32(),
  hour: f32(HOURS[0]),
  /** 1 until the season as it stands has been put on the land. */
  stale: u32(1),
});

type Table = { readonly [key: string]: number | Color };
const rgb = (r: number, g: number, b: number): Color => ({ r, g, b });

// What each weather does to the sky, the air and the sea.
const AIR = [
  /* clear */ { cover: 0.46, cloud: 0.03, grey: 1, haze: 0.00005, mist: 0.0022, veil: 0.08, rain: 0, snow: 0, wind: 8, sea: 12.3, swell: 1.8, chop: 0.9, lightning: 0, warmth: 0, colour: 0 },
  /* rain  */ { cover: 0.86, cloud: 0.05, grey: 0.62, haze: 0.0016, mist: 0.012, veil: 0.2, rain: 1, snow: 0, wind: 12, sea: 15, swell: 2, chop: 0.95, lightning: 0, warmth: -0.22, colour: -0.12 },
  /* storm */ { cover: 1, cloud: 0.08, grey: 0.36, haze: 0.003, mist: 0.02, veil: 0.25, rain: 1.6, snow: 0, wind: 22, sea: 20, swell: 2.4, chop: 1, lightning: 30, warmth: -0.34, colour: -0.2 },
  /* fog   */ { cover: 0.7, cloud: 0.03, grey: 0.86, haze: 0.011, mist: 0.05, veil: 1, rain: 0, snow: 0, wind: 2.5, sea: 6, swell: 0.8, chop: 0.6, lightning: 0, warmth: -0.12, colour: -0.16 },
  /* snow  */ { cover: 0.9, cloud: 0.045, grey: 0.82, haze: 0.003, mist: 0.01, veil: 0.3, rain: 0, snow: 1.2, wind: 5, sea: 9, swell: 1, chop: 0.75, lightning: 0, warmth: -0.14, colour: -0.14 },
] as const satisfies readonly Table[];

// What each season does to the land. `melt` is metres of lying snow lost a
// minute: none in winter, so what falls there stays.
const YEAR = [
  /* spring */ { tip: rgb(0.2, 0.44, 0.09), base: rgb(0.035, 0.1, 0.022), dry: rgb(0.45, 0.4, 0.17), dryness: 0.03, height: 0.42, plumes: 0.55, flowers: 1.15, thrift: 1.15, turf: rgb(0.56, 0.8, 0.46), leaves: rgb(0.24, 0.5, 0.1), warmth: -0.04, melt: 0.7 },
  /* summer */ { tip: rgb(0.235, 0.39, 0.11), base: rgb(0.04, 0.085, 0.024), dry: rgb(0.45, 0.38, 0.17), dryness: 0.15, height: 0.55, plumes: 1, flowers: 1, thrift: 1, turf: rgb(0.62, 0.74, 0.5), leaves: rgb(0.2, 0.36, 0.1), warmth: 0, melt: 0.7 },
  /* autumn */ { tip: rgb(0.4, 0.33, 0.1), base: rgb(0.07, 0.075, 0.025), dry: rgb(0.52, 0.34, 0.12), dryness: 0.8, height: 0.5, plumes: 1.1, flowers: 0, thrift: 0.5, turf: rgb(0.78, 0.66, 0.4), leaves: rgb(0.95, 0.42, 0.08), warmth: 0.1, melt: 0.5 },
  /* winter */ { tip: rgb(0.34, 0.34, 0.24), base: rgb(0.07, 0.07, 0.04), dry: rgb(0.44, 0.42, 0.33), dryness: 0.92, height: 0.3, plumes: 0.75, flowers: 0, thrift: 0, turf: rgb(0.6, 0.62, 0.54), leaves: rgb(0.2, 0.16, 0.12), warmth: -0.18, melt: 0 },
] as const satisfies readonly Table[];

/** The states' tables' mean by the weights. */
function blend<T extends Table>(tables: readonly T[], weights: readonly number[]): T {
  const total = weights.reduce((a, b) => a + b, 0);
  const out: { [key: string]: number | Color } = {};
  for (const key of Object.keys(tables[0]!)) {
    const sum = { r: 0, g: 0, b: 0 };
    let scalar = 0;
    for (const [i, table] of tables.entries()) {
      const v = table[key]!, w = weights[i]! / total;
      if (typeof v === "number") scalar += v * w;
      else { sum.r += v.r * w; sum.g += v.g * w; sum.b += v.b * w; }
    }
    out[key] = typeof tables[0]![key] === "number" ? scalar : sum;
  }
  return out as T;
}

type Weights = { [name: string]: number };

/** Moves an axis's weights `rate` towards its chosen state; whether any moved. */
function step(climate: Weights, names: readonly string[], chosen: number, rate: number): boolean {
  let moved = false;
  for (const [i, name] of names.entries()) {
    const goal = i === chosen ? 1 : 0, was = climate[name]!;
    const next = was < goal ? Math.min(goal, was + rate) : Math.max(goal, was - rate);
    if (next !== was) { climate[name] = next; moved = true; }
  }
  return moved;
}

const smooth = (a: number, b: number, v: number) => { const t = Math.min(1, Math.max(0, (v - a) / (b - a))); return t * t * (3 - 2 * t); };
const mix = (a: number, b: number, t: number) => a + (b - a) * t;

/** Puts the season as it stands on the land: the grass, the turf, what flowers and the tree. */
function dress(world: World, weights: readonly number[]): void {
  const year = blend(YEAR, weights);
  const ground = world.find("coast");
  if (ground !== null) must(world.set(ground, og.Grass, { tipColor: year.tip, baseColor: year.base, dryColor: year.dry, dryness: year.dryness, height: year.height }));
  const terrain = must(og.terrain.load(world, "terrain/coast.ogt"));
  // The turf layer as assets/terrain/coast.ogt has it, in the season's colour.
  must(og.terrain.setLayer(world, terrain.data, 0, { color: year.turf, roughness: 0.95, tile: 7, antiTiling: 0.7, macroVariation: 0.6 }));
  const grow = (name: string, size: number) => {
    const e = world.find(name);
    if (e !== null) must(world.set(e, og.TerrainScatter, { minScale: 0.8 * size, maxScale: 1.25 * size }));
  };
  grow("plume-grass", year.plumes);
  for (const flower of ["oxeye", "buttercup", "foxglove"]) grow(flower, year.flowers);
  grow("thrift", year.thrift);
  const tree = world.find("hawthorn");
  if (tree !== null) must(world.set(tree, og.Model, { tint: year.leaves }));
  must(world.resource(og.WeatherState)).melt = year.melt;
}

/** The weather's own pace here: ground soaks, puddles fill and snow lies within a switch's few seconds. */
export function buildClimate(world: World): void {
  const state = must(world.resource(og.WeatherState));
  state.soakTime = 6;
  state.dryTime = 25;
  state.puddleTime = 10;
  state.drainTime = 40;
  state.snowfall = 0.45;
  state.lightningCenter = { x: 60, y: 0, z: -160 };
  state.lightningRadius = 220;
  must(world.resource(og.DayNight)).step = 1;
  must(og.input.defineAction(world, { name: "weather", kind: "button", bindings: [{ control: "KeyZ" }, { control: "pad.x" }] }));
  must(og.input.defineAction(world, { name: "season", kind: "button", bindings: [{ control: "KeyX" }, { control: "pad.y" }] }));
  must(og.input.defineAction(world, { name: "time", kind: "button", bindings: [{ control: "KeyC" }, { control: "pad.b" }] }));
}

export const Seasons = defineSystem("Seasons", {
  writes: [Climate, og.Clouds, og.Fog, og.Wind, og.Weather, og.WeatherState, og.DayNight, og.RenderSettings, og.PostEffects, og.Water, og.Grass, og.TerrainScatter, og.Model],
}, (_q, time, world) => {
  const climate = must(world.resource(Climate));
  if (og.input.isActionJustPressed(world, "weather")) climate.weather = (climate.weather + 1) % WEATHERS.length;
  if (og.input.isActionJustPressed(world, "season")) climate.season = (climate.season + 1) % SEASONS.length;
  if (og.input.isActionJustPressed(world, "time")) climate.time = (climate.time + 1) % TIMES.length;

  const weights = climate as unknown as Weights;
  step(weights, WEATHERS, climate.weather, time.dt / WEATHER_SECONDS);
  const turned = step(weights, SEASONS, climate.season, time.dt / SEASON_SECONDS);
  // The hour is stored in 32 bits: against the goal as written, 23.2 held there reads as a day away.
  const goal = Math.fround(HOURS[climate.time]!), ahead = (goal - climate.hour + 24) % 24, pace = HOURS_PER_SECOND * time.dt;
  climate.hour = ahead <= pace ? goal : (climate.hour + pace) % 24;

  if (turned || climate.stale === 1) {
    dress(world, SEASONS.map(name => weights[name]!));
    climate.stale = 0;
  }
  const year = blend(YEAR, SEASONS.map(name => weights[name]!));
  const air = blend(AIR, WEATHERS.map(name => weights[name]!));

  const clouds = must(world.resource(og.Clouds));
  clouds.coverage = air.cover;
  clouds.density = air.cloud;
  clouds.color = rgb(air.grey, air.grey, air.grey);
  const fog = must(world.resource(og.Fog));
  fog.density = air.haze;
  fog.heightDensity = air.mist;
  fog.sky = air.veil;
  must(world.resource(og.Wind)).speed = air.wind;
  // Rain and snow are one fall: rain gives way to snow through a dry moment.
  const fall = air.rain - air.snow;
  const weather = must(world.resource(og.Weather));
  weather.kind = fall > 0.02 ? WEATHER_RAIN : fall < -0.02 ? WEATHER_SNOW : WEATHER_NONE;
  weather.intensity = Math.abs(fall);
  must(world.resource(og.WeatherState)).lightning = air.lightning;
  const sea = world.find("sea");
  if (sea !== null) must(world.set(sea, og.Water, { wind: { x: -0.894 * air.sea, y: 0.447 * air.sea }, swell: air.swell, choppiness: air.chop }));

  must(world.resource(og.DayNight)).start = climate.hour;
  // How far into the night the hour is: the exposure is held down, so a
  // night stays a night under the meter.
  const night = climate.hour >= 12 ? smooth(17.9, 19.4, climate.hour) : 1 - smooth(4.6, 6, climate.hour);
  must(world.resource(og.RenderSettings)).exposure = mix(1, 0.2, night);
  const post = must(world.resource(og.PostEffects));
  post.temperature = mix(0.1 + year.warmth + air.warmth, -0.2, night);
  post.saturation = 0.08 + air.colour;
});

/** Chooses a state by name on one axis; with `now`, without the blend. */
function choose(world: World, axis: "weather" | "season" | "time", names: readonly string[], args: readonly string[]): void {
  const chosen = names.indexOf(args[0] ?? "");
  if (chosen < 0) return;
  const climate = must(world.resource(Climate));
  climate[axis] = chosen;
  if (args[1] !== "now") return;
  const weights = climate as unknown as Weights;
  if (axis === "time") climate.hour = HOURS[chosen]!;
  else for (const [i, name] of names.entries()) weights[name] = i === chosen ? 1 : 0;
  climate.stale = 1;
}

export const climateCommands = {
  weather: (world: World, args: readonly string[]) => choose(world, "weather", WEATHERS, args),
  season: (world: World, args: readonly string[]) => choose(world, "season", SEASONS, args),
  time: (world: World, args: readonly string[]) => choose(world, "time", TIMES, args),
};

export const climateHelp = {
  weather: { usage: "<clear|rain|storm|fog|snow>[:now]", doc: "Turn the weather; with now, at once." },
  season: { usage: "<spring|summer|autumn|winter>[:now]", doc: "Turn the season; with now, at once." },
  time: { usage: "<sunset|night|dawn>[:now]", doc: "Run the clock on to that time of day; with now, at once." },
};
