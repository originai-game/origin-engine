// Every screen of the kit at desktop size (1280×720 CSS px at 1×), compared
// with crates/og-ui/references/<tier>/kit-desktop-<screen>.png.
import { checkFrames } from "./frames.ts";
import { selectReferenceFrames } from "./reference-frames.ts";
import { framesScript } from "./script.ts";
import { playtest } from "./lib.ts";

const script = selectReferenceFrames(framesScript(0.5), "kit-desktop-");

export default playtest({
  name: "representative desktop screens match their reference frames",
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
