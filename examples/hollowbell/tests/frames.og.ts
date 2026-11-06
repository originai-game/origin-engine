// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { press, script, tap } from "./lib.ts";

const FILES = ["title.png", "combat.png"];
const ENTERS = [4.7, 4.9, 5.1, 5.3, 5.5, 5.7, 5.9, 6.1, 6.3];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: script(
    "0.2:name:Wren",
    "1.0:screenshot:title.png",
    "1.1:play",
    "2.1:warp:beetle-0:-2.6",
    press("KeyX", 2.9),
    "3.0:screenshot:combat.png",
    "3.3:warp:mole",
    press("ArrowUp", 4.0),
    ...ENTERS.map((t) => press("Enter", t)),
    "6.4:give:glimmers:12",
    press("Escape", 6.9),
    press("Escape", 7.2),
    "7.8:language:zh-CN",
    tap("settings", 7.9),
    "8.7:language:en",
    tap("Done", 8.8),
    tap("resume", 9.2),
    "9.6:warp:44.5:6.3",
    "12.2:give:chimes:3",
    "12.3:warp:bell:1.2",
    press("ArrowUp", 12.8),
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
