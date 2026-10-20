// One label in three scripts on the localized screens: `mixed:on` shows
// menu.mixed (assets/strings/*.ogt) under the main menu, the pause menu and
// the confirm dialog, CJK, Latin and Arabic in one label, in each of the
// kit's five locales. og-ui lays such a label out a run per face (the
// role's Latin, the locale fonts' CJK, the Arabic file shaped), each
// grapheme cluster in the face that has it whole, so no character is
// missing: the engine warns once when a rendered frame draws a character
// its fonts lack (FrameStats::missing_glyphs), and the run must have no
// warning at all. The frames are not reference frames (the locale
// references show the screens without the label).
// Ways it could fail: a run drawn from a face without its characters (a
// missing-glyph warning), the label falling back to one face, a key
// missing from a table (a warning), the label not shown.
import { playtest } from "@origingame/engine/playtest";
import { type UiElement, ui } from "./lib.ts";

const LOCALES = ["en", "zh-CN", "ja", "ko", "ar"] as const;
const SCREENS = ["mainMenu", "pause", "confirm"] as const;

const steps: string[] = ["0.20:mixed:on"];
const dumps: Array<{ file: string; locale: string; screen: string }> = [];
let t = 0.3;
for (const l of LOCALES) {
  steps.push(`${t.toFixed(2)}:locale:${l}`);
  for (const s of SCREENS) {
    steps.push(`${(t + 0.05).toFixed(2)}:show:${s}`);
    steps.push(`${(t + 0.6).toFixed(2)}:screenshot:mixed-${s}-${l}.png`);
    steps.push(`${(t + 0.65).toFixed(2)}:dump_ui:mixed-${s}-${l}.json`);
    dumps.push({ file: `mixed-${s}-${l}.json`, locale: l, screen: s });
    t += 0.7;
  }
}
steps.push(`${t.toFixed(2)}:quit`);

const CJK = /[぀-ヿ一-鿿가-힯]/;
const ARABIC = /[؀-ۿ]/;
const LATIN = /[A-Za-z]/;

export default playtest({
  name: "a label mixing CJK, Latin and Arabic on the menu, pause and confirm screens draws no missing glyph in any locale",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(
      run.warnings.filter((w) => w.includes("missing_glyphs")).join("; "),
      "",
      "missing_glyphs is 0 on every frame (no missing-glyph warning)",
    );
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (every key is in every table)");
    t.ok(run.quit, "the script reached quit");
    const shown = new Set<string>();
    for (const d of dumps) {
      const label = ui(run, d.file).find((e: UiElement) => e.id === "Text" && e.parent === "Mixed");
      const text = label?.text ?? "";
      t.ok(CJK.test(text) && ARABIC.test(text) && LATIN.test(text), `${d.screen} in ${d.locale}: the label holds CJK, Latin and Arabic (${text})`);
      t.ok((label?.width ?? 0) > 100, `${d.screen} in ${d.locale}: the label is laid out (${label?.width ?? 0} px wide)`);
      shown.add(`${d.locale}:${text}`);
    }
    t.equal(shown.size, LOCALES.length, "each locale shows its own mixed string");
    for (const d of dumps) t.equal(run.png(d.file.replace(".json", ".png")).width, 1280, `${d.file} rendered`);
  },
});
