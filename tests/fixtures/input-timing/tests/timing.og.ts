// The actions as `og test` plays them (src/main.ts): a press 0.2 of a tick
// into tick 30 and one 0.3 of a tick before tick 60 report those times in
// seconds within 1 ms; a jump pressed on tick 90 is taken on tick 94, not
// again on 95, and one pressed on 120 is gone by 127 (7 ticks, past its
// 6); two runs end on one hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("presses carry their time in the tick and a buffered press is taken once", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["a buffered jump is taken"],
    commands: [
      // 30.2 and 59.7 ticks: the steps round to ticks 30 and 60.
      `${30.2 / 60}:key_down:Space`,
      "0.6:key_up:Space",
      `${59.7 / 60}:key_down:Space`,
      "1.1:key_up:Space",
      "1.5:key_down:KeyJ",
      "1.55:key_up:KeyJ",
      "2:key_down:KeyJ",
      "2.05:key_up:KeyJ",
      "2.2:expect:state:taps==2",
      // 0.2 / 60 s = 3.33 ms and -0.3 / 60 s = -5 ms, each within 1 ms.
      "2.2:expect:state:first>0.00233",
      "2.2:expect:state:first<0.00433",
      "2.2:expect:state:second>-0.006",
      "2.2:expect:state:second<-0.004",
      "2.2:expect:state:taken==1",
      "2.2:expect:state:jumps==1",
      "2.25:quit",
    ].join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
