// The title, the intro, the bag, the results, the records and the chapters
// laid out as the game shipped them, from the game's own screen documents
// (U8 moved them off the fixed templates): every element's parent, id,
// kind, role and text as shipped.ts has them, each within a pixel of its
// place. An engine update may draw a screen better; it never adds, drops or
// moves what is on it. The steps are frames.og.ts's, a dump for each frame.
import { playtest } from "@origingame/engine/playtest";
import { press, script, tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

export default playtest({
  name: "the title, the intro, the bag, the results, the records and the chapters are laid out as the game shipped them",
  size: "1280x720",
  commands: script(
    "1.2:dump_ui:title.json",
    "1.3:play",
    press("Enter", 2.3),
    "2.6:dump_ui:intro.json",
    tap("Skip", 2.7),
    tap("Skip", 3.2),
    "3.6:set:Tuning.sight=0",
    "6.8:warp:crypt:-8:-3",
    "8.1:give:herbs:3",
    "8.15:give:sigils:2",
    press("Tab", 8.3),
    tap("Slots/Cell#4", 8.8),
    "9.4:dump_ui:bag.json",
    press("Escape", 9.6),
    "9.8:play:3:quick",
    "9.9:ring",
    tap("Skip", 10.5),
    "13.2:dump_ui:results.json",
    tap("records", 13.4),
    "15.2:dump_ui:records.json",
    press("Escape", 15.4),
    tap("title", 16.6),
    "17.5:set:Game.chaptersReached=15",
    "17.6:open:chapters",
    "19.2:dump_ui:chapters.json",
    "19.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "intro", "bag", "results", "records", "chapters"]);
  },
});
