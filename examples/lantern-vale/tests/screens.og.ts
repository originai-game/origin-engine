// The title, the Chinese settings and the dawn results laid out as the game
// shipped them, from the game's own screen documents (U8 moved them off the
// fixed templates): every element's parent, id, kind, role and text as
// shipped.ts has them, in that order, and og ui check finds no undeclared
// overlap, nothing past the screen and no text cut on each
// (tests/screens.ts). An engine update may lay a screen out better; it
// never adds, drops or reorders what is on it, nor breaks those rules. The
// steps are frames.og.ts's, a dump for each of those frames.
import { playtest } from "@origingame/engine/playtest";
import { tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

export default playtest({
  name: "the title, the Chinese settings and the results are laid out as the game shipped them",
  size: "1280x720",
  commands: [
    "1.0:dump_ui:title.uicheck.json",
    "1.1:run",
    "2.3:warp:0",
    "2.4:key_down:KeyE",
    "3.8:key_up:KeyE",
    "4.1:warp:-19:0",
    "4.7:warp:8:-20",
    "6.1:language:zh-CN",
    "6.2:settings",
    "7.0:dump_ui:settings-zh.uicheck.json",
    "7.1:language:en",
    tap("Done", 7.2),
    "7.5:light:1",
    "7.6:light:2",
    "7.7:light:3",
    "11.3:dump_ui:results.uicheck.json",
    "11.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "settings-zh", "results"]);
  },
});
