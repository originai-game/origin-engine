// The foundations and controls previews at desktop size (1280×720 CSS px
// at 1×), driven by the pointer, keys and a pad: each part idle, hovered,
// pressed, focused and disabled; a hold, a drag, a capture's conflict, the
// on-screen keyboard; right to left and at 150 % text. Each frame is
// compared with crates/og-ui/references/<tier>/kit-controls-<file>.
import { checkFrames } from "./frames.ts";
import { selectReferenceFrames } from "./reference-frames.ts";
import { controlsScript } from "./controls.ts";
import { playtest, ui } from "./lib.ts";

const script = selectReferenceFrames(controlsScript(false), "kit-controls-");

export default playtest({
  name: "the foundations and controls previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const found = ui(run, "foundations.json");
    t.ok(found.some((e) => e.role === "heading" && e.text === "Chapter II"), "a title is a heading");
    t.ok(found.some((e) => e.role === "image" && e.name === "Character preview" && e.value === "60°"), "two Right presses turned the viewport by 30°");
    const controls = ui(run, "controls.json");
    t.ok(controls.some((e) => e.role === "switch" && e.name.startsWith("Music") && !e.states.includes("checked")), "the tapped switch turned off");
    t.ok(controls.some((e) => e.role === "radio" && e.name.startsWith("Normal") && e.states.includes("checked")), "Right stepped the difficulty from Story to Normal");
    t.ok(controls.some((e) => e.role === "radio" && e.name === "Map" && e.states.includes("checked")), "the tapped segment is chosen");
    const ranges = ui(run, "ranges.json");
    t.ok(ranges.some((e) => e.role === "slider" && e.name === "Look sensitivity" && e.value === "6"), "two Right presses stepped the sensitivity from 4 to 6");
    t.ok(ranges.some((e) => e.role === "slider" && e.name === "Minimum" && e.value === "500 g"), "the range's low knob dragged to the middle");
    const fields = ui(run, "fields.json");
    t.ok(fields.some((e) => e.role === "combobox" && e.name === "Language" && e.value === "日本語"), "the tapped option is the language");
    const conflict = ui(run, "bindings-conflict.json");
    t.ok(conflict.some((e) => e.role === "alert"), "a used key asks before it is bound");
    const bound = ui(run, "bindings.json");
    t.ok(bound.some((e) => e.name === "Interact, 1: C"), "Swap gave Interact the C key");
    t.ok(bound.some((e) => e.name === "Crouch, 1: F"), "and Crouch the key Interact had");
    const kana = ui(run, "bindings-kana.json");
    t.ok(kana.some((e) => e.role === "button" && e.name === "勇者"), "the composition ゆうしゃ offers the game's candidate");
    t.ok(kana.some((e) => e.role === "button" && e.name === "ユウシャ"), "and its katakana");
    const big = ui(run, "controls-150.json");
    const small = controls.filter((e) => e.role === "button").map((e) => e.height);
    t.ok(Math.min(...big.filter((e) => e.role === "button" && !e.clipped).map((e) => e.height)) > Math.max(...small) * 0.9, "at 150 % text the buttons grow with their labels");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-controls-");
  },
});
