// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { hold, press, script, tap } from "./lib.ts";

const FILES = ["pub-title.png", "pub-straight.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  setup: "cover:12,12,40,40",
  commands: script(
    "1.2:screenshot:pub-title.png",
    tap("Items/start", 1.5),
    hold("KeyW", 4.6, 4.4),
    "8.5:screenshot:pub-straight.png",
    press("Escape", 9.2),
    "10.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    checkFrames(run, t, FILES, "");
  },
});
