// The day's tasks played with the keys (docs/ci.md, the showcase step):
// 1 steps the weather, 2 the season and 3 the time three times on to the
// night, which reaches the "every task of the day done" site, and two runs
// end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

/** A run may draw its frames on the CPU. */
const SLOW = 900_000;

test("the keys switch the day through its tasks and replay to one hash", async () => {
  const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.05).toFixed(2)}:key_up:${key}`;
  const run = await runPlaytest({
    runs: 2,
    require: ["every task of the day done"],
    commands: [
      tap(0.2, "Digit1"),
      tap(0.3, "Digit2"),
      tap(0.4, "Digit3"),
      tap(0.5, "Digit3"),
      tap(0.6, "Digit3"),
      "0.8:expect:state:weather==rain",
      "0.8:expect:state:season==autumn",
      "0.8:expect:state:time==night",
      "0.9:quit",
    ].join(";"),
  });
  expect(run.hashes).toHaveLength(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
}, SLOW);
