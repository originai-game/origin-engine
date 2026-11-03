// The last previews at desktop size (1280×720 CSS px at 1×): photo mode (the
// camera's sliders, a filter, a frame with the grid, the shutter's flash, the
// controls hidden and brought back) and the loadout (its attachments called
// out, one removed, another loadout and slot, a locked slot refused); then each right to left, in Japanese and
// at 200 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-genres3-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genres3Script } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genres3Script(false);

export default playtest({
  name: "the last previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(ui(run, "photo.json").some((e) => e.role === "slider" && e.name === "Field of view"), "the camera's tab shows the field of view's slider");
    t.ok(ui(run, "photo-filter.json").some((e) => e.role === "radio" && e.name === "Noir" && e.states.includes("checked")), "a press on Noir chose it");
    t.ok(ui(run, "photo-frame.json").some((e) => e.role === "switch" && e.name === "Grid" && e.states.includes("checked")), "Grid is on");
    t.ok(ui(run, "photo-hidden.json").every((e) => e.role !== "slider" && e.name !== "Gallery"), "hidden, no control stands over the shot, the kit's Gallery button neither");
    t.ok(ui(run, "photo-shown.json").some((e) => e.role === "tab" && e.name === "Frames"), "a press while hidden brought the controls back on their tab");
    t.ok(ui(run, "loadout.json").some((e) => e.role === "button" && e.name === "Optic: MK.2 holo sight"), "the optic is called out on the rifle");
    t.ok(ui(run, "loadout-removed.json").some((e) => e.id === "Count" && e.text === "5/6"), "Remove took the stock off: five of six points fitted");
    t.ok(ui(run, "loadout-refused.json").some((e) => e.role === "tab" && e.name.startsWith("Field upgrade") && !e.states.includes("selected")), "the locked field upgrade was refused: pressed, it is not selected");
    t.ok(ui(run, "loadout-refused.json").some((e) => e.role === "tab" && e.name === "Loadout 2" && e.states.includes("selected")), "the second loadout's tab is shown");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres3-");
  },
});
