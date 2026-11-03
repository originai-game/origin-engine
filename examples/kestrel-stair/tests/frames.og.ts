// Nine reference frames at 1280 x 720, compared with references/<tier>/
// (tests/frames.ts: og-tests' tolerance): the title over the climb, the
// intro cutscene, exploring the court, a strike's hit stop on the terrace
// warden, inside the crypt by the lamp, the bag, the four named chapter
// cards at the title, the records and the results under the summit look.
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { hold, press, script, tap } from "./lib.ts";

const FILES = ["title.png", "intro.png", "explore.png", "strike.png", "crypt.png", "bag.png", "chapters.png", "records.png", "results.png"];

export default playtest({
  name: "the title, the intro, the court, a strike, the crypt, the bag, the chapters, the records and the results match their reference frames",
  size: "1280x720",
  commands: script(
    "1.2:screenshot:title.png",
    "1.3:play",
    press("Enter", 2.3),
    "2.6:screenshot:intro.png",
    tap("Skip", 2.7),
    tap("Skip", 3.2),
    "3.6:set:Tuning.sight=0",
    "3.7:warp:-4:30",
    hold("KeyW", 3.8, 0.8),
    "5.4:screenshot:explore.png",
    "5.6:play:1:quick",
    "5.65:set:Tuning.wardenPatrol=0",
    "5.7:warp:warden-0:0:-1.2",
    press("KeyJ", 6.3),
    "6.47:screenshot:strike.png",
    "6.8:warp:crypt:-8:-3",
    "8.0:screenshot:crypt.png",
    "8.1:give:herbs:3",
    "8.15:give:sigils:2",
    press("Tab", 8.3),
    tap("Slots/Cell#4", 8.8),
    "9.4:screenshot:bag.png",
    press("Escape", 9.6),
    "9.7:set:Tuning.wardenPatrol=1.1",
    "9.8:play:3:quick",
    "9.9:ring",
    tap("Skip", 10.5),
    "13.2:screenshot:results.png",
    tap("records", 13.4),
    "15.2:screenshot:records.png",
    press("Escape", 15.4),
    // The results come back through the 700 ms wipe and stay inert while it
    // moves: Title is tapped after it (at 15.8 it was not, and the
    // chapters opened over the results). The four chapters reached, so
    // every card shows its name; the level select laid out by 18.3.
    tap("title", 16.6),
    "17.5:set:Game.chaptersReached=15",
    "17.6:open:chapters",
    "19.2:screenshot:chapters.png",
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
