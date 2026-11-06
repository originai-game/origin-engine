// The title with the records under its items, the pause menu over the
// crumble round and the French settings laid out as the game's own screen
// documents lay them out (U8 moved them off the fixed templates, shaped as
// the game shipped them): every element's parent, id, kind, role and text
// as shipped.ts has them, in that order, and og ui check finds no
// undeclared overlap, nothing past the screen and no text cut on each
// (tests/screens.ts). An engine update may lay a screen out better; it
// never adds, drops or reorders what is on it, nor breaks those rules. The
// steps are frames.og.ts' up to the settings.
import { playtest } from "@origingame/engine/playtest";
import { hold, pad, PAD0, script, tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

export default playtest({
  name: "the title, the pause menu and the French settings are laid out as the game shipped them",
  size: "1280x720",
  commands: script(
    "0.2:records:12:34.5",
    "0.8:dump_ui:title.uicheck.json",
    "1.0:seat:wasd",
    "1.1:seat:arrows",
    pad("a", PAD0, 1.2),
    "1.9:match",
    "3.1:go",
    hold("KeyA", 3.3, 0.8),
    hold("ArrowRight", 3.3, 0.8),
    "4.3:knock:2",
    "4.7:knock:1",
    "8.1:next",
    "8.2:go",
    "10.3:finish",
    "12.1:next",
    "12.2:go",
    "13.7:open_pause",
    "14.3:dump_ui:pause.uicheck.json",
    tap("settings", 14.4),
    "14.5:language:fr",
    "15.1:dump_ui:settings-fr.uicheck.json",
    "15.2:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "pause", "settings-fr"]);
  },
});
