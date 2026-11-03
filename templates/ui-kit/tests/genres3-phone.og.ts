// The last previews on a phone in portrait (390×844 CSS px at 3×, a 47 px
// notch and a 34 px home indicator), driven by touch only: photo mode (the
// camera's sliders, a filter, a frame with the grid, the shutter's flash, the
// controls hidden and brought back); then each right to left, in Japanese and
// at 200 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-genres3-phone-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genres3Script } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genres3Script(true);

export default playtest({
  name: "the last previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["photo.json", "photo-filter.json", "photo-frame.json"]) {
      for (const e of ui(run, f).filter((x) => (x.role === "button" || x.role === "switch" || x.role === "radio" || x.role === "tab") && !x.states.includes("disabled"))) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f}: ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    t.ok(ui(run, "photo.json").some((e) => e.role === "slider" && e.name === "Field of view"), "the camera's tab shows the field of view's slider");
    t.ok(ui(run, "photo-filter.json").some((e) => e.role === "radio" && e.name === "Noir" && e.states.includes("checked")), "a press on Noir chose it");
    t.ok(ui(run, "photo-frame.json").some((e) => e.role === "switch" && e.name === "Grid" && e.states.includes("checked")), "Grid is on");
    t.ok(ui(run, "photo-hidden.json").every((e) => e.role !== "slider" && e.name !== "Gallery"), "hidden, no control stands over the shot, the kit's Gallery button neither");
    t.ok(ui(run, "photo-shown.json").some((e) => e.role === "tab" && e.name === "Frames"), "a press while hidden brought the controls back on their tab");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres3-phone-");
  },
});
