// Six frames at 1280x720 compared with references/<tier>/: the title over
// the valley in late afternoon (the lanterns glowing), a run's start on the
// path, the first lantern caught (its glow, light and burst, the "+1", the
// toast), a slime turned alert by the hero (its bubble from the faces'
// array, the jelly's transmission), the far side resident (pines, the arch,
// the banner), the settings in Chinese (the mixed-script label) and the
// results under the dawn look. `OG_FX_CAPTURE=1 og test .` records this
// tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { tap } from "./lib.ts";

const FILES = ["title.png", "start.png", "lantern.png", "slime.png", "far.png", "settings-zh.png", "results.png"];

export default playtest({
  name: "the title, a run, a lantern, a slime, the far side, the Chinese settings and the dawn results match their reference frames",
  size: "1280x720",
  commands: [
    "1.0:screenshot:title.png",
    "1.1:run",
    "2.2:screenshot:start.png",
    "2.3:warp:0",
    "2.4:key_down:KeyE",
    "3.8:key_up:KeyE",
    "4.0:screenshot:lantern.png",
    "4.1:warp:-19:0",
    "4.6:screenshot:slime.png",
    "4.7:warp:8:-20",
    "6.0:screenshot:far.png",
    "6.1:language:zh-CN",
    "6.2:settings",
    "7.0:screenshot:settings-zh.png",
    "7.1:language:en",
    tap("Done", 7.2),
    "7.5:light:1",
    "7.6:light:2",
    "7.7:light:3",
    "11.3:screenshot:results.png",
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
