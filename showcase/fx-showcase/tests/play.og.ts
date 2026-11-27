// The showcase played with its keys (docs/ci.md, the showcase step): an
// arrow steps to the next effect, another jumps a group, and two runs end on
// the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

/** A run may draw its frames on the CPU. */
const SLOW = 900_000;

test("the arrow keys step and jump through the effects and replay to one hash", async () => {
  const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.05).toFixed(2)}:key_up:${key}`;
  const run = await runPlaytest({
    runs: 2,
    require: ["an arrow key stepped to another effect", "an arrow key jumped to another group"],
    commands: [tap(0.2, "ArrowRight"), tap(0.6, "ArrowDown"), "1.2:quit"].join(";"),
  });
  expect(run.hashes).toHaveLength(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
}, SLOW);
