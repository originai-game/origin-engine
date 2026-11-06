// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { press } from "./lib.ts";

const setup = (t: number) => `${t}:place:12.3:6.5;${t}:slime:13.4:6.5;${t}:face:right`;
const FILES = ["start.png", "crit.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: [
    "0.5:screenshot:start.png",
    setup(0.6),
    press("Space", 0.65),
    setup(1.3),
    press("Space", 1.35),
    setup(2.0),
    press("Space", 2.05),
    // The third hit lands on tick 129 and holds seven ticks.
    "2.185:screenshot:crit.png",
    "2.6:place:5:3",
    "2.6:key_down:KeyD",
    press("ShiftLeft", 2.7),
    "2.9:key_up:KeyD",
    "3.0:give:door",
    "3.0:place:23:6",
    "3.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
