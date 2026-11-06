// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

const FILES = ["title.png", "wave.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: [
    "0.5:screenshot:title.png",
    "0.6:begin",
    "0.7:plant:hedge:4:4",
    "0.7:plant:thorn:3:4",
    "0.7:plant:frost:5:4",
    "6.0:screenshot:wave.png",
    "6.1:win",
    "8.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
