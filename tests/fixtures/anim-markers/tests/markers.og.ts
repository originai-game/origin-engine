// The markers as `og test` plays them (src/main.ts): every og.AnimationEvent
// of the cross-fade and of the three-way fade names its state and layer and
// is dominant exactly when its clip weighs most of its layer; each case is
// seen at least once, and two runs end on one world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("cross-fade and blend markers say where they came from and which clip dominates", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: [
      "the clip faded from dominates the fade's first half",
      "the clip faded to dominates its second half",
      "a marker of the lighter clip fires too",
      "a blend's tie goes to its later clip",
    ],
    commands: ["2.5:expect:state:markers>10", "2.5:quit"].join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
