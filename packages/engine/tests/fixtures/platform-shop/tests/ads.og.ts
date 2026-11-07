// A rewarded ad through the fake: the first is skipped (no reward), the
// second watched to the end (10 gems). Each pauses the game while it shows
// and is answered after the resume.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("a skipped rewarded ad pays nothing and a completed one pays 10 gems", async () => {
  const run = await runPlaytest({
    commands: "0.5:ad;1.5:ad;2.5:get:Wallet:wallet.json;2.6:quit",
    platform: { outcomes: { "ad.show": ["skipped"] } },
  });
  const w = run.json<{ value: { gems: number; skipped: number } }>("wallet.json").value;
  expect(w.skipped, "the skipped ad was counted, not paid").toBe(1);
  expect(w.gems, "the completed ad paid 10 gems").toBe(10);
});
