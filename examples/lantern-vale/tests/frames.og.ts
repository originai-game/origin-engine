// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { tap } from "./lib.ts";

const FILES = ["title.png", "lantern.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: [
    "1.0:screenshot:title.png",
    "1.1:run",
    "2.3:warp:0",
    "2.4:key_down:KeyE",
    "3.8:key_up:KeyE",
    "4.0:screenshot:lantern.png",
    "4.1:warp:-19:0",
    "4.7:warp:8:-20",
    "6.1:language:zh-CN",
    "6.2:settings",
    "7.1:language:en",
    tap("Done", 7.2),
    "7.5:light:1",
    "7.6:light:2",
    "7.7:light:3",
    "11.4:quit",
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
