import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

/** The Air resource (src/air.ts) as a `get:Air` step writes it. */
interface Air { readonly weather: number; readonly season: number; readonly time: number; readonly rain: number; readonly snow: number; readonly spring: number; readonly winter: number; readonly dawn: number }
// The runs step the ticks between their switches without drawing them
// (`advance`): a software renderer draws this street at a frame a second.
const SIZE = "320x180";

test("the game starts in spring rain at night", async () => {
  const run = await runPlaytest({ commands: "0.1:get:Air;0.1:quit", size: SIZE });
  expect(run.quit, "the game reached quit").toBe(true);
  const air = run.json<{ value: Air }>("get-000006.json").value;
  expect({ weather: air.weather, season: air.season, time: air.time, rain: air.rain, spring: air.spring }).toEqual({ weather: 0, season: 0, time: 0, rain: 1, spring: 1 });
});

test("a switch ramps the air to what it names, and snow lies in winter", async () => {
  // The switches come after the first tick, which takes the weather as the game started it (og.Weather.change).
  const run = await runPlaytest({ commands: "0:advance:1;0:weather:snow;0:season:winter;0:time:dawn;0:advance:89;1.5:get:Air;1.5:advance:480;9.5:get:Air;9.5:get:og.WeatherState.snowDepth;9.5:quit", size: SIZE });
  // Half way through the weather's 3 s (og.Weather.change) the two cross: neither full nor gone.
  const half = run.json<{ value: Air }>("get-000090.json").value;
  expect(half.snow).toBeGreaterThan(0.3);
  expect(half.snow).toBeLessThan(0.7);
  expect(half.rain).toBeCloseTo(1 - half.snow, 4);
  // At its end the targets are reached exactly, and what was in the air is gone.
  const air = run.json<{ value: Air }>("get-000570.json").value;
  expect({ snow: air.snow, rain: air.rain, winter: air.winter, spring: air.spring, dawn: air.dawn }).toEqual({ snow: 1, rain: 0, winter: 1, spring: 0, dawn: 1 });
  expect(run.json<{ value: number }>("get-000570-2.json").value).toBeGreaterThan(0.02);
});

test("a script of switches ends in the same world twice", async () => {
  const run = await runPlaytest({ commands: "0:weather:storm;0:advance:60;1:season:autumn;1:advance:60;2:time:dawn;2:advance:60;3:hud;3:advance:120;5:quit", size: SIZE, runs: 2 });
  expect(run.hashes[0]).toBe(run.hashes[1]);
});
