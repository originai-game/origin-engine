// The switches: a weather and a season blend in over their seconds and
// arrive whole, the clock runs on to the chosen time, and `now` skips the
// blend. Read from the Climate resource (src/climate.ts), which the drawn
// sky, air, sea and land follow.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

interface Climate {
  value: {
    weather: number; season: number; time: number; hour: number;
    clear: number; rain: number; storm: number; fog: number; snow: number;
    spring: number; summer: number; autumn: number; winter: number;
  };
}

/** A run may draw its frames on the CPU. */
const SLOW = 900_000;

test("a weather and a season blend in and arrive", async () => {
  const run = await runPlaytest({
    commands: "0.5:weather:storm;0.5:season:winter;2.5:get:Climate:mid.json;9.5:get:Climate:end.json;9.6:quit",
  });
  const mid = run.json<Climate>("mid.json").value, end = run.json<Climate>("end.json").value;
  // Two seconds into a weather's six and a season's eight.
  expect(Math.abs(mid.storm - 2 / 6)).toBeLessThanOrEqual(0.02);
  expect(Math.abs(mid.clear - 4 / 6)).toBeLessThanOrEqual(0.02);
  expect(Math.abs(mid.winter - 2 / 8)).toBeLessThanOrEqual(0.02);
  expect(Math.abs(mid.summer - 6 / 8)).toBeLessThanOrEqual(0.02);
  expect([end.clear, end.rain, end.storm, end.fog, end.snow]).toEqual([0, 0, 1, 0, 0]);
  expect([end.spring, end.summer, end.autumn, end.winter]).toEqual([0, 0, 0, 1]);
}, SLOW);

test("the clock runs forwards to the chosen time", async () => {
  const run = await runPlaytest({
    commands: "0.5:time:night;1.5:get:Climate:on-the-way.json;5:get:Climate:night.json;5.1:time:dawn;6.1:get:Climate:past-midnight.json;6.2:quit",
  });
  const hour = (file: string) => run.json<Climate>(file).value.hour;
  // Sunset is 17.9 and the clock runs 1.6 hours a second.
  expect(Math.abs(hour("on-the-way.json") - 19.5)).toBeLessThanOrEqual(0.05);
  expect(Math.abs(hour("night.json") - 23.2)).toBeLessThanOrEqual(0.001);
  // On to dawn through midnight, never back through the evening.
  expect(Math.abs(hour("past-midnight.json") - 0.8)).toBeLessThanOrEqual(0.05);
}, SLOW);

test("now skips the blend", async () => {
  const run = await runPlaytest({
    commands: "0.2:weather:fog:now;0.2:season:autumn:now;0.2:time:dawn:now;0.3:get:Climate:now.json;0.4:quit",
  });
  const now = run.json<Climate>("now.json").value;
  expect([now.fog, now.clear, now.autumn, now.summer]).toEqual([1, 0, 1, 0]);
  expect(Math.abs(now.hour - 6.25)).toBeLessThanOrEqual(0.001);
}, SLOW);
