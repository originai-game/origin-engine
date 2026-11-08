// The guard, then taunts a second later: each input on its own as a
// playtest presses it. It passes; the counter the very next tick is missed.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("a taunt long after the guard resets the power", async () => {
  const run = await runPlaytest({ commands: "0.200:key_down:KeyQ;0.250:key_up:KeyQ;1.2:intent:Taunt:{};1.5:intent:Taunt:{\"level\":2};1.9:get:Fighter:end.json;2:quit" });
  expect(run.json<{ value: { power: number; counters: number } }>("end.json").value).toMatchObject({ power: 1, counters: 0 });
});
