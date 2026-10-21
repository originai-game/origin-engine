// Nine frames at 1280x720 compared with references/<tier>/: the title over
// the village, a run's start, a strike's freeze frame in the dark upper
// hollow (the beetle white, the sparks, the number, the lit map), the
// mole's talk, his wares, the pause menu, the settings in Chinese (the
// mixed-script label), the gloam winding up in its hall, and the results
// under the dawn look. `OG_FX_CAPTURE=1 og test .` records this tier's
// references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { press, script, tap } from "./lib.ts";

const FILES = ["title.png", "play.png", "combat.png", "talk.png", "shop.png", "pause.png", "settings-zh.png", "gloam.png", "results.png"];
const ENTERS = [4.7, 4.9, 5.1, 5.3, 5.5, 5.7, 5.9, 6.1, 6.3];

export default playtest({
  name: "the title, play, a strike, the talk, the shop, pause, the Chinese settings, the gloam and the dawn results match their reference frames",
  size: "1280x720",
  commands: script(
    "0.2:name:Wren",
    "1.0:screenshot:title.png",
    "1.1:play",
    "2.0:screenshot:play.png",
    "2.1:warp:beetle-0:-2.6",
    press("KeyX", 2.9),
    "3.0:screenshot:combat.png",
    "3.3:warp:mole",
    press("ArrowUp", 4.0),
    "4.6:screenshot:talk.png",
    ...ENTERS.map((t) => press("Enter", t)),
    "6.4:give:glimmers:12",
    "6.8:screenshot:shop.png",
    press("Escape", 6.9),
    press("Escape", 7.2),
    "7.7:screenshot:pause.png",
    "7.8:language:zh-CN",
    tap("settings", 7.9),
    "8.6:screenshot:settings-zh.png",
    "8.7:language:en",
    tap("Done", 8.8),
    tap("resume", 9.2),
    "9.6:warp:44.5:6.3",
    "12.0:screenshot:gloam.png",
    "12.2:give:chimes:3",
    "12.3:warp:bell:1.2",
    press("ArrowUp", 12.8),
    "17.5:screenshot:results.png",
    "17.6:quit",
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
