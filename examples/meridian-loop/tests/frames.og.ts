// Ten reference frames at 1280 x 720, compared with references/<tier>/
// (tests/frames.ts: og-tests' tolerance): the title over the grid; the jump
// in the air; a slide with smoke and skid marks; the countdown; the
// sweeper at speed; the third lap with the record's ghost ahead; pause; the results; the leaderboard; the German settings.
// `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { hold, press, script, tap } from "./lib.ts";

const FILES = ["title.png", "jump.png", "slide.png", "countdown.png", "race.png", "ghost.png", "pause.png", "results.png", "board.png", "settings-de.png"];

export default playtest({
  name: "the title, the jump, a slide, the countdown, the sweeper, the ghost, pause, the results, the leaderboard and the German settings match their reference frames",
  size: "1280x720",
  commands: script(
    "1.2:screenshot:title.png",
    "1.3:race:go",
    "1.5:warp:205:36",
    hold("KeyW", 1.55, 1.7),
    "3.15:screenshot:jump.png",
    "4:warp:150:30",
    hold("KeyW", 4.05, 0.9),
    hold("KeyA", 4.05, 0.9),
    "4.75:screenshot:slide.png",
    "6:race",
    "7.3:screenshot:countdown.png",
    "9.1:drive:auto",
    "19:screenshot:race.png",
    // The third lap: the car brakes for a moment off the line, so the record's ghost (the second lap) draws ahead.
    "74.8:drive:off",
    hold("KeyS", 74.8, 0.3),
    "75.2:drive:auto",
    "77:screenshot:ghost.png",
    "82:open:pause",
    "82.6:screenshot:pause.png",
    press("Escape", 82.8),
    "113:screenshot:results.png",
    tap("Actions/board", 113.4),
    "115.4:screenshot:board.png",
    press("Escape", 115.6),
    tap("Actions/title", 116.6),
    "118:language:de",
    "118.2:open:settings",
    "119.2:screenshot:settings-de.png",
    "119.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    checkFrames(run, t, FILES, "");
  },
});
