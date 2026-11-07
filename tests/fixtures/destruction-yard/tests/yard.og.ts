// The yard's beats as `og test` plays them (src/main.ts's BEATS): the
// charge at the wall's foot, the blow at the pillar, the ball at the pane.
// The game's og.assert sites hold on every tick (no chunk falls through the
// slab), each prop is seen to break, and one frame of the settled yard is
// compared with its Metal reference.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

// The script's expect: steps, require and invariants are its checks: runPlaytest
// rejects naming the first failing tick and value of each.
test("a wall, a pillar and a pane break on their beats and nothing falls through the slab", async () => {
  const run = await runPlaytest({
    size: "960x540",
    invariants: ["state:wall>=0", "state:pillar>=0"],
    require: ["the wall breaks", "the pillar breaks", "the pane breaks"],
    commands: [
      "1:boom",
      "1.6:strike",
      "2:throw",
      // The wall is down to its piers, the pillar to its stump, the pane to shards.
      "2.9:expect:state:wall<0.5",
      "2.9:expect:state:pillar<0.75",
      "2.9:expect:state:pillar>0.25",
      "2.9:expect:state:paneIntact==0",
      "3:screenshot:yard.png",
      "3:expect:frame:references/metal/yard.png:rmse:<=3",
      "3.05:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
});
