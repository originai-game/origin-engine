// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { press, presses } from "./lib.ts";

const FILES = ["title.png", "dive.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: [
    "0.9:screenshot:title.png",
    "0.95:dive:7",
    presses(Array(7).fill("ArrowUp"), 1.15),
    "3.65:screenshot:dive.png",
    "3.75:hp:1",
    "3.75:warp:6:7",
    press("Space", 3.85),
    "6.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
