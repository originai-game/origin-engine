// Jumps: at a steady 6 m/s an obstacle placed at x=1 reaches the runner
// (x=-5) one second later. A held press (a full jump, 1.9 m) clears a crate
// and a fence; a tap (the 6-tick hop, 1.1 m) clears a crate but not a fence.
import { playtest } from "@origingame/engine/playtest";
import { OVER, RUNNING, positionOf, runOf, tap } from "./lib.ts";

export default playtest({
  name: "a full jump clears a crate and a fence; a hop clears a crate, not a fence",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    "0.1:set:runner.Tuning.speedRamp=0",
    tap(0.5),
    // Crate 1 at t=1: arrives at t=2. Hold Space from 1.65 to 2.15: a full jump.
    "1.0:place:crate:1",
    "1.65:key_down:Space",
    "1.9:dump_entity:runner:airborne.json",
    "1.9:screenshot:jump.png",
    "2.15:key_up:Space",
    "2.6:dump_entity:runner:after-full.json",
    // Crate 2 at t=3 arrives at t=4: a tap 12 ticks before is a hop over it.
    "3.0:place:crate:1",
    tap(3.8),
    "4.6:dump_entity:runner:after-hop.json",
    // Fence 1 at t=5: a full jump 18 ticks before its arrival clears it.
    "5.0:place:fence:1",
    "5.7:key_down:Space",
    "6.2:key_up:Space",
    "6.6:dump_entity:runner:after-fence.json",
    // Fence 2 at t=7: a tap at the same moment as that full jump is a hop, too low for it.
    "7.0:place:fence:1",
    tap(7.7),
    "8.6:dump_entity:runner:after-hop-fence.json",
    "8.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const air = positionOf(run, "airborne.json");
    t.ok(air.y > 1, `the held jump is high over the crate (feet at ${air.y.toFixed(2)} m)`);
    t.equal(runOf(run, "after-full.json").phase, RUNNING, "the full jump cleared crate 1");
    t.equal(runOf(run, "after-hop.json").phase, RUNNING, "the hop cleared crate 2");
    t.equal(runOf(run, "after-fence.json").phase, RUNNING, "the full jump cleared fence 1");
    t.equal(runOf(run, "after-hop-fence.json").phase, OVER, "the hop did not clear fence 2");
  },
});
