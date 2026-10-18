// Every screen of the kit on a phone in portrait: 390×844 CSS px at 3×
// (1170×2532 device px) with a 47 px notch and a 34 px home indicator,
// compared with crates/og-ui/references/<tier>/kit-phone-<screen>.png.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { framesScript } from "./script.ts";

const script = framesScript(0.5);

export default playtest({
  name: "every screen on a phone in portrait matches its reference frame",
  size: "1170x2532",
  commands: `0.0:viewport:phone;${script.commands}`,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    checkFrames(run, t, script.files, "kit-phone-");
  },
});
