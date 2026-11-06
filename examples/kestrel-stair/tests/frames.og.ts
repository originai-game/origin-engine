// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { hold, press, script, tap } from "./lib.ts";

const FILES = ["title.png", "strike.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: script(
    "1.2:screenshot:title.png",
    "1.3:play",
    press("Enter", 2.3),
    // Preserve render history before strike; these warm-ups have no references.
    "2.6:screenshot:intro-warmup.png",
    tap("Skip", 2.7),
    tap("Skip", 3.2),
    "3.6:set:Tuning.sight=0",
    "3.7:warp:-4:30",
    hold("KeyW", 3.8, 0.8),
    "5.4:screenshot:explore-warmup.png",
    "5.6:play:1:quick",
    "5.65:set:Tuning.wardenPatrol=0",
    "5.7:warp:warden-0:0:-1.2",
    press("KeyJ", 6.3),
    "6.47:screenshot:strike.png",
    "6.8:warp:crypt:-8:-3",
    "8.1:give:herbs:3",
    "8.15:give:sigils:2",
    press("Tab", 8.3),
    tap("Slots/Cell#4", 8.8),
    press("Escape", 9.6),
    "9.7:set:Tuning.wardenPatrol=1.1",
    "9.8:play:3:quick",
    "9.9:ring",
    tap("Skip", 10.5),
    tap("records", 13.4),
    press("Escape", 15.4),
    // The results come back through the 700 ms wipe and stay inert while it
    // moves: Title is tapped after it (at 15.8 it was not, and the
    // chapters opened over the results). The four chapters reached, so
    // every card shows its name; the level select laid out by 18.3.
    tap("title", 16.6),
    "17.5:set:Game.chaptersReached=15",
    "17.6:open:chapters",
    "19.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
