// Four frames at 1280×720 compared with references/<tier>/: the west room at
// rest (tiles, torches over the wall face, shadows, HUD), the crit's freeze
// frame (white squashed slime, the crit number, the shaken camera), a dash
// (the hero's afterimages) and the east room through the opened doorway
// (the re-tiled wall, the camera zone of room 2).
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { press } from "./lib.ts";

const setup = (t: number) => `${t}:place:12.3:6.5;${t}:slime:13.4:6.5;${t}:face:right`;
const FILES = ["start.png", "crit.png", "dash.png", "east.png"];

export default playtest({
  name: "the rooms, a crit's freeze frame and a dash match their reference frames",
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
    "2.8:screenshot:dash.png",
    "2.9:key_up:KeyD",
    "3.0:give:door",
    "3.0:place:23:6",
    "3.6:screenshot:east.png",
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
