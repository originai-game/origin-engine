// The text screen (og.ui text): the display face kerned and the rounded
// face for scores, labels cut with an ellipsis, Arabic shaped and in bidi
// order (a run of it inside a Latin rich-text span), Hebrew, Devanagari's
// conjuncts and marks, then a text field and a text area edited through
// the probe: typed text, Shift+arrows selecting, an input method's
// composition shown underlined and committed. The values are world state
// (the Typed resource's str fields, read with get:). Each frame is compared with
// crates/og-ui/references/<tier>/kit-desktop-text*.png.
// Ways it could fail: a script drawn with missing glyphs (a warning), a
// field that drops typed text or types a key twice, a selection or a
// composition not drawn, a commit that does not land in the value, a value
// that never reaches the world.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { type UiElement, key, tap, ui } from "./lib.ts";

const shiftLeft = (t: number, n: number): string =>
  [
    `${t.toFixed(2)}:key_down:ShiftLeft`,
    ...Array.from({ length: n }, (_, k) => key(t + 0.05 + k * 0.1, "ArrowLeft")),
    `${(t + 0.1 + n * 0.1).toFixed(2)}:key_up:ShiftLeft`,
  ].join(";");

export default playtest({
  name: "the text screen draws every script and edits a field and an area",
  size: "1280x720",
  commands: [
    "0.3:show:text",
    "1.0:screenshot:text.png",
    // The field: a tap focuses it, typing fills it, Shift+← ×4 selects "lace".
    tap(1.1, "Name"),
    "1.3:type:Ada Lovelace",
    shiftLeft(2.0, 4),
    "2.6:screenshot:text-select.png",
    "2.7:dump_ui:select.json",
    // The area: two lines, then an input method composing on the second.
    tap(2.8, "Notes"),
    "3.0:type:First line",
    key(3.5, "Enter"),
    "3.6:type:then",
    "3.9:compose:にほん",
    "4.0:screenshot:text-compose.png",
    "4.1:commit:日本",
    "4.2:dump_ui:committed.json",
    "4.2:get:Typed:typed.json",
    "4.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (no glyph missing from any script)");
    t.ok(run.quit, "the script reached quit");
    const text = (file: string, id: string) => ui(run, file).find((e: UiElement) => e.id === id)?.text;
    t.equal(text("select.json", "Name"), "Ada Lovelace", "the field holds what was typed, once");
    t.equal(text("committed.json", "Notes"), "First line\nthen日本", "Enter broke the line and the commit landed");
    const typed = run.json<{ value: { name: string; notes: string } }>("typed.json").value;
    t.equal(typed.name, "Ada Lovelace", "the field's value is world state (Typed.name)");
    t.equal(typed.notes, "First line\nthen日本", "so is the area's (Typed.notes)");
    checkFrames(run, t, ["text.png", "text-select.png", "text-compose.png"], "kit-desktop-");
  },
});
