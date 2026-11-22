// The switches played with their keys (docs/ci.md, the showcase step): Z,
// X and C each reach their Seasons site, the climate blends all the way to
// what they chose, and two runs end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

interface Climate { value: { weather: number; season: number; time: number; hour: number; clear: number; rain: number; summer: number; autumn: number } }

/** A run may draw its frames on the CPU. */
const SLOW = 900_000;

test("the keys switch the weather, the season and the time and replay to one hash", async () => {
  const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.05).toFixed(2)}:key_up:${key}`;
  const run = await runPlaytest({
    runs: 2,
    require: ["a key switched the weather", "a key turned the season", "a key changed the time of day"],
    // The weather blends over 6 s, the season over 8; sunset to night is 5.3 hours at 1.6 a second.
    commands: [tap(0.1, "KeyZ"), tap(0.2, "KeyX"), tap(0.3, "KeyC"), "9.4:get:Climate:end.json", "9.5:quit"].join(";"),
  });
  const end = run.json<Climate>("end.json").value;
  expect({ weather: end.weather, season: end.season, time: end.time }).toEqual({ weather: 1, season: 2, time: 1 });
  expect([end.clear, end.rain, end.summer, end.autumn]).toEqual([0, 1, 0, 1]);
  expect(Math.abs(end.hour - 23.2)).toBeLessThanOrEqual(0.001);
  expect(run.hashes).toHaveLength(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
}, SLOW);
