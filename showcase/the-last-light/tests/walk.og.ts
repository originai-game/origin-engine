// The wanderer walks the curved path on the stick (walk:path), at a walk,
// and two runs end on the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

interface Player { value: { walked: number; speed: number } }

/** A run may draw its frames on the CPU. */
const SLOW = 900_000;

test("the wanderer walks along the path", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["the wanderer walked along the path"],
    commands: "1:walk:path;12:get:Player:end.json;12.1:quit",
  });
  const end = run.json<Player>("end.json").value;
  expect(end.walked).toBeGreaterThan(5);
  expect(end.speed).toBeLessThanOrEqual(1.5);
  expect(run.hashes[1]).toBe(run.hashes[0]);
}, SLOW);
