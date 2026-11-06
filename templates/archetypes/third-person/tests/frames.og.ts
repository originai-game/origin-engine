// Smoke: the title and a lit beacon, with the existing reference tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

const FILES = ["title.png", "beacon.png"];

export default playtest({
  name: "smoke: the title and a lit beacon match their reference frames",
  size: "1280x720",
  commands: [
    "0.9:screenshot:title.png",
    "1.0:run",
    "1.1:warp:0",
    "1.3:key_down:KeyF",
    "3.4:screenshot:beacon.png",
    "3.5:key_up:KeyF",
    "3.6:light:1",
    "3.7:light:2",
    "3.8:light:3",
    "7.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
