// The last previews on a phone in portrait (390×844 CSS px at 3×, a 47 px
// notch and a 34 px home indicator), driven by touch only: photo mode (the
// camera's sliders, a filter, a frame with the grid, the shutter's flash, the
// controls hidden and brought back) and the loadout (an attachment picked,
// one removed, another loadout and slot, a locked slot refused) and the deck
// builder (a card added, an unowned one and one at its limit refused, a copy
// taken out, the deck full), the CG gallery (a locked picture refused, the
// viewer with a version and next, a chapter), the route chart and the
// settings tree (a display change applied and reverted, the bindings, the HUD
// editor); then each right to left, in Japanese and
// at 150 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-genres3-phone-<file>.
import { checkFrames } from "./frames.ts";
import { selectReferenceFrames } from "./reference-frames.ts";
import { genres3Script } from "./patterns.ts";
import { playtest, ui } from "./lib.ts";

const script = selectReferenceFrames(genres3Script(true), "kit-genres3-phone-");

export default playtest({
  name: "the last previews in their states on a phone",
  size: "1170x2532",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    for (const f of ["photo.json", "photo-filter.json", "photo-frame.json", "loadout.json", "loadout-removed.json", "deck.json", "cg.json", "routes.json", "settings.json", "settings-controls.json"]) {
      // A control a sideways scroll cuts (the probe's `clipped`) is a partial view, not a small target.
      for (const e of ui(run, f).filter((x) => (x.role === "button" || x.role === "switch" || x.role === "radio" || x.role === "tab") && !x.states.includes("disabled") && !x.clipped)) {
        t.ok(e.width >= 43.5 && e.height >= 43.5, `${f}: ${e.id} (${e.name}) is a whole touch target: ${Math.round(e.width)}×${Math.round(e.height)}`);
      }
    }
    t.ok(ui(run, "photo.json").some((e) => e.role === "slider" && e.name === "Field of view"), "the camera's tab shows the field of view's slider");
    t.ok(ui(run, "photo-filter.json").some((e) => e.role === "radio" && e.name === "Noir" && e.states.includes("checked")), "a press on Noir chose it");
    t.ok(ui(run, "photo-frame.json").some((e) => e.role === "switch" && e.name === "Grid" && e.states.includes("checked")), "Grid is on");
    t.ok(ui(run, "photo-hidden.json").every((e) => e.role !== "slider" && e.name !== "Gallery"), "hidden, no control stands over the shot, the kit's Gallery button neither");
    t.ok(ui(run, "photo-shown.json").some((e) => e.role === "tab" && e.name === "Frames"), "a press while hidden brought the controls back on their tab");
    t.ok(ui(run, "loadout-removed.json").some((e) => e.id === "Count" && e.text === "5/6"), "Remove took the stock off: five of six points fitted");
    t.ok(ui(run, "loadout-refused.json").some((e) => e.role === "tab" && e.name.startsWith("Field upgrade") && !e.states.includes("selected")), "the locked field upgrade was refused: pressed, it is not selected");
    t.ok(ui(run, "loadout-refused.json").some((e) => e.role === "tab" && e.name === "Loadout 2" && e.states.includes("selected")), "the second loadout's tab is shown");
    t.ok(ui(run, "deck.json").some((e) => e.role === "tab" && e.name === "My deck 22/24"), "the deck's tab counts 22 of 24");
    t.ok(ui(run, "deck-unowned.json").some((e) => e.role === "tab" && e.name === "My deck 23/24"), "Anger went into the deck");
    t.ok(ui(run, "deck-unowned.json").some((e) => e.role === "status" && e.name === "Not owned"), "the unowned Wound was refused, its reason said");
    t.ok(ui(run, "deck-limit.json").some((e) => e.role === "status" && e.name === "No more copies allowed"), "a third Strike was refused at its limit");
    t.ok(ui(run, "deck-removed.json").some((e) => e.role === "status" && e.name === "My deck 22/24"), "a press on Defend's row took a copy out");
    t.ok(ui(run, "cg.json").some((e) => e.role === "button" && e.name === "???, Board the ferry in Chapter 2"), "the storm ferry is veiled with how to unlock it");
    t.ok(ui(run, "cg-viewer.json").some((e) => e.role === "dialog" && e.name === "The riverboat"), "the riverboat opened in the viewer");
    t.ok(ui(run, "cg-next.json").some((e) => e.role === "dialog" && e.name === "Lantern night"), "next opened the lantern night");
    t.ok(ui(run, "cg-chapter.json").filter((e) => e.role === "button" && e.parent.includes("Line#")).length === 2, "chapter 2 shows its two pictures");
    t.ok(ui(run, "routes.json").some((e) => e.role === "button" && e.name === "The ferry, Now"), "the ferry is the scene now");
    t.ok(ui(run, "routes-tower.json").some((e) => e.role === "button" && e.name === "Jump here"), "the tower, seen, offers Jump here");
    t.ok(ui(run, "settings-pending.json").some((e) => e.role === "button" && e.name === "Apply" && !e.states.includes("disabled")), "a pending change makes Apply live");
    t.ok(ui(run, "settings-confirm.json").some((e) => e.role === "alertdialog"), "applying fullscreen asks to keep it");
    t.ok(ui(run, "settings-reverted.json").every((e) => e.role !== "alertdialog"), "Revert put the display back and closed the dialog");
    t.ok(ui(run, "settings-controls.json").some((e) => e.role === "group" && e.name === "Move forward"), "the controls page binds actions by key");
    t.ok(ui(run, "settings-hud.json").some((e) => e.parent === "Editor" && e.text === "Edit HUD"), "the HUD page mounts the HUD layout editor");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1170, `${f} is 1170 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres3-phone-");
  },
});
