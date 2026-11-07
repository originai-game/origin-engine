// The heal alone, as `og test` plays it: it resolves once, and the frame
// of its resolve flash over the post matches its Metal reference.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

// The script's expect: steps, require and invariants are its checks: runPlaytest
// rejects naming the first failing tick and value of each.
test("a heal resolves once and its resolve draws as recorded", async () => {
  const run = await runPlaytest({
    size: "960x540",
    require: ["the heal resolves"],
    commands: [
      "0.5:heal",
      "1.72:screenshot:heal.png",
      "1.72:expect:frame:references/metal/heal.png:rmse:<=3",
      "2.6:expect:state:healed==1",
      "2.6:expect:state:impacts==0",
      "2.65:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
});
