// The kit's localized screens (og.i18n, assets/strings/*.ogt) in five
// locales, each compared with crates/og-ui/references/<tier>/kit-locale-*:
// the main menu, the pause menu over the platformer HUD and the confirm
// dialog in English, simplified Chinese, Japanese, Korean and Arabic. Arabic
// is shaped (joined forms from the font's own tables, through rustybuzz) and
// right to left: the layout mirrors (the Gallery button moves to the bottom
// right) and text aligns to the right. The probe's `locale:<tag>` asks for
// each locale; `og.i18n.locale` (the facts) says which table is active.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

const LOCALES = ["en", "zh-CN", "ja", "ko", "ar"] as const;
const SCREENS = ["mainMenu", "pause", "confirm"] as const;

const steps: string[] = [];
const files: string[] = [];
let t = 0.3;
for (const l of LOCALES) {
  steps.push(`${t.toFixed(2)}:locale:${l}`);
  for (const s of SCREENS) {
    steps.push(`${(t + 0.05).toFixed(2)}:show:${s}`);
    steps.push(`${(t + 0.7).toFixed(2)}:screenshot:${s}-${l}.png`);
    files.push(`${s}-${l}.png`);
    t += 0.8;
  }
  steps.push(`${t.toFixed(2)}:dump_world:world-${l}.json`);
  t += 0.1;
}
steps.push(`${t.toFixed(2)}:quit`);

export default playtest({
  name: "the menu, pause and confirm screens in en, zh-CN, ja, ko and ar match their reference frames",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (every key is in every table)");
    t.ok(run.quit, "the script reached quit");
    for (const f of files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    checkFrames(run, t, files, "kit-locale-");
  },
});
