// The same walk as explore-deep's playtest, on the corridor without the bug.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("level 1 walks to its end and descends", async () => {
  const run = await runPlaytest({ commands: "0.100:key_down:KeyD;0.150:key_up:KeyD;0.200:key_down:KeyD;0.250:key_up:KeyD;0.300:key_down:KeyD;0.350:key_up:KeyD;0.400:key_down:KeyD;0.450:key_up:KeyD;0.500:key_down:KeyD;0.550:key_up:KeyD;0.600:key_down:KeyD;0.650:key_up:KeyD;0.700:key_down:KeyD;0.750:key_up:KeyD;0.800:key_down:KeyD;0.850:key_up:KeyD;0.900:key_down:KeyD;0.950:key_up:KeyD;1.000:key_down:KeyD;1.050:key_up:KeyD;1.200:key_down:KeyE;1.250:key_up:KeyE;1.400:key_down:KeyD;1.450:key_up:KeyD;1.600:key_down:KeyA;1.650:key_up:KeyA;1.9:get:Dungeon:end.json;2:quit" });
  expect(run.json<{ value: { level: number; x: number } }>("end.json").value).toEqual({ level: 2, x: 0 });
});
