// The two effects as `og test` plays them: each sends its event once, on
// its tick, and one frame with both playing matches its Metal reference.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

// The script's expect: steps, require and invariants are its checks: runPlaytest
// rejects naming the first failing tick and value of each.
test("a hit and a heal play, send their events once and draw as recorded", async () => {
  const run = await runPlaytest({
    size: "960x540",
    require: ["the hit lands", "the heal resolves"],
    commands: [
      "0.5:heal",
      "1.5:hit",
      "1.6:screenshot:hit.png",
      "1.6:expect:frame:references/metal/hit.png:rmse:<=3",
      "2.6:expect:state:impacts==1",
      "2.6:expect:state:healed==1",
      "2.65:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
});
