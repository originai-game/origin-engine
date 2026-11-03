// The title, the level select and the Spanish settings laid out as the game
// shipped them: every element's parent, id, kind, role and text as
// screens.json has them (recorded from the engine its reference frames were
// drawn with), each within a few px of its place. An engine update may draw
// a screen better; it never adds, drops or moves what is on it.
import { playtest } from "@origingame/engine/playtest";
import { press, script, tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

export default playtest({
  name: "the title, the level select and the Spanish settings are laid out as the game shipped them",
  size: "1280x720",
  commands: script(
    "0.2:unlock:2",
    "1.0:dump_ui:title.json",
    tap("levels", 1.1),
    "2.0:dump_ui:levels.json",
    press("Escape", 2.1),
    tap("new", 2.6),
    press("Escape", 13.8),
    "14.6:language:es",
    tap("settings", 14.7),
    "15.5:dump_ui:settings-es.json",
    "15.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "levels", "settings-es"]);
  },
});
