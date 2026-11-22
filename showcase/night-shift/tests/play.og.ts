// The switches played with their keys (docs/ci.md, the showcase step): R,
// F and T each reach their Switches site, the air ramps all the way to what
// they chose, and two runs end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

/** The Air resource (src/air.ts) as a `get:Air` step writes it. */
interface Air { readonly weather: number; readonly season: number; readonly time: number; readonly storm: number; readonly rain: number; readonly summer: number; readonly spring: number; readonly dawn: number }
const SIZE = "320x180";

test("the keys switch the weather, the season and the time and replay to one hash", async () => {
  const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.05).toFixed(2)}:key_up:${key}`;
  const run = await runPlaytest({
    runs: 2,
    size: SIZE,
    require: ["a key switched the weather", "a key turned the season", "a key changed the time of day"],
    // Each ramp is at most 8 s; the ticks after the keys are stepped without drawing.
    commands: [tap(0.1, "KeyR"), tap(0.2, "KeyF"), tap(0.3, "KeyT"), "0.4:advance:540", "9.4:get:Air:air.json", "9.4:quit"].join(";"),
  });
  const air = run.json<{ value: Air }>("air.json").value;
  expect({ weather: air.weather, season: air.season, time: air.time }).toEqual({ weather: 1, season: 1, time: 1 });
  expect({ storm: air.storm, rain: air.rain, summer: air.summer, spring: air.spring, dawn: air.dawn }).toEqual({ storm: 1, rain: 0, summer: 1, spring: 0, dawn: 1 });
  expect(run.hashes).toHaveLength(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
