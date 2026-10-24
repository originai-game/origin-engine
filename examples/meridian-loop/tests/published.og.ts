// The script the published page can play too (tests/web/published.ts
// --playtest): input and screenshots only, no game command, since a publish
// build has no command channel. Time trial pressed on the title, the
// countdown, W held straight up the start straight, Escape to pause. The
// page shows the Player's Back over the top-left corner, which the HUD
// keeps out of (Viewport.covered): `setup` covers the same rectangle here,
// so the native frames lay out as the page's. Natively it checks its
// frames against references/<tier>/ `pub-*.png`; on the page the harness
// compares the same captures with them.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { hold, press, script, tap } from "./lib.ts";

const FILES = ["pub-title.png", "pub-countdown.png", "pub-straight.png", "pub-pause.png"];

export default playtest({
  name: "the published page's script: the title, Time trial, the countdown, the start straight at speed and pause match their frames",
  size: "1280x720",
  setup: "cover:12,12,40,40",
  commands: script(
    "1.2:screenshot:pub-title.png",
    tap("Items/start", 1.5),
    "3:screenshot:pub-countdown.png",
    hold("KeyW", 4.6, 4.4),
    "8.5:screenshot:pub-straight.png",
    press("Escape", 9.2),
    "10:screenshot:pub-pause.png",
    "10.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    checkFrames(run, t, FILES, "");
  },
});
