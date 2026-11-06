// The foundations and controls previews on a phone in portrait (390×844
// CSS px at 3×, a 47 px notch and a 34 px home indicator), driven by touch
// only: every part a finger presses is a whole 44 px target (where the scroll
// shows it whole). Each frame is
// compared with crates/og-ui/references/<tier>/kit-controls-phone-<file>.
import { checkFrames } from "./frames.ts";
import { selectReferenceFrames } from "./reference-frames.ts";
import { controlsScript } from "./controls.ts";
import { playtest, ui } from "./lib.ts";

const script = selectReferenceFrames(controlsScript(true), "kit-controls-phone-");
const PRESSED = new Set(["button", "switch", "checkbox", "radio", "slider", "combobox", "textbox", "spinbutton"]);

export default playtest({
  name: "the foundations and controls previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["controls.json", "ranges.json", "fields.json", "bindings.json"]) {
      for (const e of ui(run, f).filter((x) => PRESSED.has(x.role) && !x.clipped && x.id !== "Knob" && x.id !== "Low" && x.id !== "High")) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f} ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    const bound = ui(run, "bindings.json");
    t.ok(bound.some((e) => e.name === "Interact, 1: C"), "a finger's Swap gave Interact the C key");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-controls-phone-");
  },
});
