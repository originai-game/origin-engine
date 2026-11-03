// The title, the mole's talk, his wares and the Chinese settings laid out
// as the game shipped them: every element's parent, id, kind, role and text
// as shipped.ts has them (recorded from the engine its reference frames
// were drawn with), each within a few px of its place. An engine update may
// draw a screen better; it never adds, drops or moves what is on it.
import { playtest } from "@origingame/engine/playtest";
import { press, script, tap } from "./lib.ts";
import { checkScreens } from "./screens.ts";

const ENTERS = [4.7, 4.9, 5.1, 5.3, 5.5, 5.7, 5.9, 6.1, 6.3];

export default playtest({
  name: "the title, the talk, the shop and the Chinese settings are laid out as the game shipped them",
  size: "1280x720",
  commands: script(
    "0.2:name:Wren",
    "1.0:dump_ui:title.json",
    "1.1:play",
    "3.3:warp:mole",
    press("ArrowUp", 4.0),
    "4.6:dump_ui:talk.json",
    ...ENTERS.map((t) => press("Enter", t)),
    "6.4:give:glimmers:12",
    "6.8:dump_ui:shop.json",
    press("Escape", 6.9),
    press("Escape", 7.2),
    "7.8:language:zh-CN",
    tap("settings", 7.9),
    "8.6:dump_ui:settings-zh.json",
    "8.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    checkScreens(run, t, ["title", "talk", "shop", "settings-zh"]);
  },
});
