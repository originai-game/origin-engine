// The title, the countdown, the racing HUD at speed, pause, the results,
// the leaderboard and the German settings laid out as the game's own
// screens lay them out (U8 moved them off the fixed templates, shaped as
// the game shipped them: the documents in assets/ui, the HUD src/hud.ts):
// every element's parent, id, kind, role and text as shipped.ts has them,
// in that order, and og ui check finds no undeclared overlap, nothing past
// the screen and no text cut on each (tests/screens.ts). An engine update
// may lay a screen out better; it never adds, drops or reorders what is on
// it, nor breaks those rules. Three autopilot laps from GO reach the
// results about 107 s in (pause holds the clock).
import { playtest } from "@origingame/engine/playtest";
import { press, script, tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

export default playtest({
  name: "the title, the countdown, the HUD, pause, the results, the leaderboard and the German settings are laid out as the game shipped them",
  size: "1280x720",
  commands: script(
    "1.2:dump_ui:title.uicheck.json",
    "1.3:race",
    "2.6:dump_ui:countdown.uicheck.json",
    "4.4:drive:auto",
    "14:dump_ui:race.uicheck.json",
    "20:open:pause",
    "20.6:dump_ui:pause.uicheck.json",
    press("Escape", 20.8),
    "112:dump_ui:results.uicheck.json",
    tap("Actions/board", 112.4),
    "114.4:dump_ui:board.uicheck.json",
    press("Escape", 114.6),
    tap("Actions/title", 115.4),
    "116:language:de",
    "116.2:open:settings",
    "117.2:dump_ui:settings-de.uicheck.json",
    "117.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "countdown", "race", "pause", "results", "board", "settings-de"]);
  },
});
