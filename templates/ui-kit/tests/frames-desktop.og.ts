// Every screen of the kit at desktop size (1280×720 CSS px at 1×), compared
// with crates/og-ui/references/<tier>/kit-desktop-<screen>.png.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { framesScript } from "./script.ts";

const script = framesScript(0.5);

export default playtest({
  name: "every screen at desktop size matches its reference frame",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-desktop-");
  },
});
