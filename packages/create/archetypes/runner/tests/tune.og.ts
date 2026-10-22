// Tune writes through the schema: `set:runner.Tuning.speed` changes the
// start speed of the next round, `jumpHeight` and `timeToApex` the arc of a
// full jump (og.PlatformerController's designer units).
import { playtest } from "@origingame/engine/playtest";
import { RUNNING, positionOf, runOf, tap } from "./lib.ts";

export default playtest({
  name: "set writes Tuning: start speed and jump height change the run",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    "0.1:set:runner.Tuning.speedRamp=0",
    "0.1:set:runner.Tuning.speed=10",
    "0.1:set:runner.Tuning.jumpHeight=4",
    "0.1:set:runner.Tuning.timeToApex=0.667",
    "0.1:dump_entity:runner:tuning.json",
    tap(0.5),
    "1.5:dump_entity:runner:fast.json",
    "1.6:key_down:Space",
    // A full jump of height 4 m peaks after about 0.67 s.
    "2.27:dump_entity:runner:apex.json",
    "2.4:key_up:Space",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const tuning = run.json<{ components: { Tuning: { speed: number; jumpHeight: number } } }>("tuning.json").components.Tuning;
    t.equal(tuning.speed, 10, "the start speed was set");
    t.equal(tuning.jumpHeight, 4, "the jump height was set");
    const fast = runOf(run, "fast.json");
    t.equal(fast.phase, RUNNING, "running");
    t.near(fast.speed, 10, 1e-4, "the round starts at the tuned speed");
    t.near(fast.distance, 10, 0.4, `about 10 m in the first second (${fast.distance.toFixed(2)})`);
    const apex = positionOf(run, "apex.json").y;
    t.near(apex, 4, 0.15, `the jump peaks near the tuned 4 m (${apex.toFixed(2)} m)`);
  },
});
