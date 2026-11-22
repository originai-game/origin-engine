// The course as the simulation runs it (docs/ci.md, the showcase step): the
// runner's og.MotionAgent and og.CharacterController take it up onto the
// terrace, which reaches the Follow system's site, and two runs end on the
// same world hash. The ticks are stepped without drawing: the path is the
// simulation's, never the drawn motion's.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("the runner reaches the terrace and the course replays to one hash", async () => {
  const run = await runPlaytest({
    runs: 2,
    size: "320x180",
    require: ["the runner is up on the terrace"],
    commands: "0:advance:780;13:get:runner.og.Transform:runner.json;13:quit",
  });
  const runner = run.json<{ value: { position: [number, number, number] } }>("runner.json").value;
  expect(runner.position[0]).toBeGreaterThan(10.5);
  expect(runner.position[1]).toBeGreaterThan(0.35);
  expect(run.hashes).toHaveLength(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
