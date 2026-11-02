// The systems games' previews on a phone in portrait (390×844 CSS px at 3×,
// a 47 px notch and a 34 px home indicator), driven by touch only, then
// right to left, in Japanese and at 200 % text: each frame compared with
// crates/og-ui/references/<tier>/kit-genres2-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genres2Script } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genres2Script(true);

export default playtest({
  name: "the systems games' previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["garage.json", "garage-layer.json"]) {
      for (const e of ui(run, f).filter((x) => x.role === "button" && !x.states.includes("disabled"))) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f}: ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    t.ok(ui(run, "garage-bought.json").some((e) => e.role === "button" && e.name === "Equip"), "the bought sport tier offers Equip");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres2-phone-");
  },
});
