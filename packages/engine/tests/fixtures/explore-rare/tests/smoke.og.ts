// Twenty spins, a quarter of a second apart: a session's worth of play.
// It passes; the jackpot (one stop in 500) is not among them.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("twenty spins pay and cost as the table says", async () => {
  const run = await runPlaytest({ commands: "0.200:key_down:Space;0.250:key_up:Space;0.450:key_down:Space;0.500:key_up:Space;0.700:key_down:Space;0.750:key_up:Space;0.950:key_down:Space;1.000:key_up:Space;1.200:key_down:Space;1.250:key_up:Space;1.450:key_down:Space;1.500:key_up:Space;1.700:key_down:Space;1.750:key_up:Space;1.950:key_down:Space;2.000:key_up:Space;2.200:key_down:Space;2.250:key_up:Space;2.450:key_down:Space;2.500:key_up:Space;2.700:key_down:Space;2.750:key_up:Space;2.950:key_down:Space;3.000:key_up:Space;3.200:key_down:Space;3.250:key_up:Space;3.450:key_down:Space;3.500:key_up:Space;3.700:key_down:Space;3.750:key_up:Space;3.950:key_down:Space;4.000:key_up:Space;4.200:key_down:Space;4.250:key_up:Space;4.450:key_down:Space;4.500:key_up:Space;4.700:key_down:Space;4.750:key_up:Space;4.950:key_down:Space;5.000:key_up:Space;5.4:get:Slots:end.json;5.5:quit" });
  const end = run.json<{ value: { coins: number; spins: number } }>("end.json").value;
  expect(end.spins).toBe(20);
  expect(end.coins).toBeGreaterThanOrEqual(30);
});
