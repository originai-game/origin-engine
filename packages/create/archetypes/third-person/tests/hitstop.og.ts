// A pause pressed during a hit-stop: a grab freezes scaled time
// (og.time.hitStop, here held 20 ticks), and Escape pressed inside that
// freeze still pauses the run.
import { playtest } from "@origingame/engine/playtest";
import { game, press } from "./lib.ts";

export default playtest({
  name: "Escape pressed during a grab's hit-stop pauses the run",
  commands: [
    "0.3:run",
    "0.4:set:Tuning.hitStop=20",
    "0.5:warp:-27:13.2",
    "0.6:grab:0",
    press("Escape", 0.65),
    "1.0:get:Game:after.json",
    "1.0:get:og.TimeScale.scale:scale.json",
    "1.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = game(run, "after.json");
    t.equal(g.hearts, 2, "the west sentinel's grab landed (the hit-stop ran)");
    t.equal(g.paused, true, "Escape inside the hit-stop paused the run");
    t.equal(run.json<{ value: number }>("scale.json").value, 0, "and the run holds: time scale 0");
  },
});
