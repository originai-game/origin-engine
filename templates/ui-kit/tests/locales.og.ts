// The kit's localized screens (og.i18n, assets/strings/*.ogt) in five
// locales, each compared with crates/og-ui/references/<tier>/kit-locale-*:
// the main menu, the pause menu over the platformer HUD and the confirm
// dialog in English, simplified Chinese, Japanese, Korean and Arabic. Arabic
// is shaped (joined forms from the font's own tables, through rustybuzz) and
// right to left: the layout mirrors (the Gallery button moves to the bottom
// right) and text aligns to the right. The probe's `locale:<tag>` asks for
// each locale; `og.i18n.locale` (the facts) says which table is active.
// Then the settings screen and the shop in zh-CN: every word the templates
// draw themselves (SettingsDesc's and ShopDesc's label fields, og-i18n §2)
// comes from the Chinese table, with none of the English defaults left.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { ui } from "./lib.ts";

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
// The settings (its Audio tab) and the shop (its first ware selected: Buy).
steps.push(`${t.toFixed(2)}:locale:zh-CN`);
steps.push(`${(t + 0.05).toFixed(2)}:show:settings`);
steps.push(`${(t + 0.7).toFixed(2)}:dump_ui:settings-zh-CN.json`);
steps.push(`${(t + 0.75).toFixed(2)}:show:shop`);
steps.push(`${(t + 1.4).toFixed(2)}:dump_ui:shop-zh-CN.json`);
steps.push(`${(t + 1.45).toFixed(2)}:locale:en`);
steps.push(`${(t + 1.5).toFixed(2)}:show:settings`);
steps.push(`${(t + 2.1).toFixed(2)}:dump_ui:settings-en.json`);
t += 2.2;
steps.push(`${t.toFixed(2)}:quit`);

export default playtest({
  name: "the menu, pause and confirm screens in en, zh-CN, ja, ko and ar match their reference frames; the settings and shop templates' own words are Chinese in zh-CN",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (every key is in every table)");
    t.ok(run.quit, "the script reached quit");
    for (const f of files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    checkFrames(run, t, files, "kit-locale-");

    const texts = (file: string) => new Set(ui(run, file).map((e) => e.text).filter((x) => x !== ""));
    const settings = texts("settings-zh-CN.json");
    for (const word of ["设置", "音频", "画面", "操作", "语言", "主音量", "音乐", "音效", "全部静音", "完成"]) {
      t.ok(settings.has(word), `the settings screen shows ${word} in zh-CN`);
    }
    for (const word of ["Settings", "Audio", "Graphics", "Controls", "Language", "Master", "Music", "Effects", "Mute everything", "Done"]) {
      t.ok(!settings.has(word), `and not the English ${word}`);
    }
    const shop = texts("shop-zh-CN.json");
    for (const word of ["商店", "价格"]) t.ok(shop.has(word), `the shop shows ${word} in zh-CN`);
    t.ok(shop.has("购买") || shop.has("余额不足"), "its Buy button reads 购买 or 余额不足");
    for (const word of ["Shop", "Price", "Buy", "Not enough", "Owned"]) t.ok(!shop.has(word), `and not the English ${word}`);
    // In English the fields' defaults are the words the template always drew.
    const en = texts("settings-en.json");
    for (const word of ["Settings", "Audio", "Master", "Mute everything", "Done"]) t.ok(en.has(word), `in en the settings screen shows ${word}`);
  },
});
