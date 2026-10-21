// The kit's localized screens (og.i18n, assets/strings/*.ogt) in five
// locales, each compared with crates/og-ui/references/<tier>/kit-locale-*:
// the main menu, the pause menu over the platformer HUD and the confirm
// dialog in English, simplified Chinese, Japanese, Korean and Arabic. Arabic
// is shaped (joined forms from the font's own tables, through rustybuzz) and
// right to left: the layout mirrors (the Gallery button moves to the bottom
// right) and text aligns to the right. The probe's `locale:<tag>` asks for
// each locale; `og.i18n.locale` (the facts) says which table is active.
// Then the settings screen, the shop, the inventory, the leaderboard's error,
// the tutorial and the cutscene in zh-CN: every word the templates draw
// themselves (their descs' label fields, og-i18n §2) comes from the Chinese
// table, with none of the English defaults left.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { tap, ui } from "./lib.ts";

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
t += 1.45;
// The inventory with nothing selected (its hint), then its first slot (a stack of three: Count).
steps.push(`${t.toFixed(2)}:show:inventory`);
steps.push(`${(t + 0.65).toFixed(2)}:dump_ui:inventory-hint-zh-CN.json`);
steps.push(tap(t + 0.7, "Slots/Cell#0"));
steps.push(`${(t + 1.1).toFixed(2)}:dump_ui:inventory-zh-CN.json`);
t += 1.15;
// The leaderboard whose rows failed to load (Retry), the tutorial's first and last steps, the cutscene.
steps.push(`${t.toFixed(2)}:offline:on`);
steps.push(`${(t + 0.05).toFixed(2)}:show:leaderboard`);
steps.push(`${(t + 0.65).toFixed(2)}:dump_ui:leaderboard-zh-CN.json`);
steps.push(`${(t + 0.7).toFixed(2)}:offline:off`);
steps.push(`${(t + 0.75).toFixed(2)}:show:tutorial`);
steps.push(`${(t + 1.35).toFixed(2)}:dump_ui:tutorial-zh-CN.json`);
steps.push(tap(t + 1.4, "Tutorial.callout/Actions/Next"));
steps.push(tap(t + 1.7, "Tutorial.callout/Actions/Next"));
steps.push(`${(t + 2.1).toFixed(2)}:dump_ui:tutorial-done-zh-CN.json`);
steps.push(`${(t + 2.15).toFixed(2)}:show:cutscene`);
steps.push(`${(t + 2.8).toFixed(2)}:dump_ui:cutscene-zh-CN.json`);
t += 2.85;
steps.push(`${t.toFixed(2)}:locale:en`);
steps.push(`${(t + 0.05).toFixed(2)}:show:settings`);
steps.push(`${(t + 0.65).toFixed(2)}:dump_ui:settings-en.json`);
steps.push(`${(t + 0.7).toFixed(2)}:show:tutorial`);
steps.push(`${(t + 1.3).toFixed(2)}:dump_ui:tutorial-en.json`);
t += 1.4;
steps.push(`${t.toFixed(2)}:quit`);

export default playtest({
  name: "the menu, pause and confirm screens in en, zh-CN, ja, ko and ar match their reference frames; the settings, shop, inventory, leaderboard, tutorial and cutscene templates' own words are Chinese in zh-CN",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (every key is in every table)");
    t.ok(run.quit, "the script reached quit");
    t.frame(files, () => {
      for (const f of files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
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
    const hint = texts("inventory-hint-zh-CN.json");
    t.ok(hint.has("选择一件物品。把它拖到另一个格子即可移动。"), "the inventory's hint is Chinese in zh-CN");
    t.ok(![...hint].some((x) => x.startsWith("Select an item")), "and not the English hint");
    const inventory = texts("inventory-zh-CN.json");
    t.ok(inventory.has("数量") && !inventory.has("Count"), "a selected stack's count line reads 数量, not Count");
    const board = texts("leaderboard-zh-CN.json");
    t.ok(board.has("无法加载分数。") && board.has("重试") && !board.has("Retry"), "the leaderboard's error has 重试, not Retry");
    const tutorial = texts("tutorial-zh-CN.json");
    for (const word of ["第 1 步，共 3 步", "跳过", "下一步"]) t.ok(tutorial.has(word), `the tutorial shows ${word} in zh-CN`);
    for (const word of ["1 of 3", "Skip", "Next"]) t.ok(!tutorial.has(word), `and not the English ${word}`);
    const done = texts("tutorial-done-zh-CN.json");
    t.ok(done.has("第 3 步，共 3 步") && done.has("完成") && !done.has("Done"), "its last step counts 第 3 步，共 3 步 and finishes with 完成");
    const story = texts("cutscene-zh-CN.json");
    t.ok(story.has("跳过") && !story.has("Skip"), "the cutscene's Skip reads 跳过");
    // In English the fields' defaults are the words the template always drew.
    const en = texts("settings-en.json");
    for (const word of ["Settings", "Audio", "Master", "Mute everything", "Done"]) t.ok(en.has(word), `in en the settings screen shows ${word}`);
    const tutorialEn = texts("tutorial-en.json");
    for (const word of ["1 of 3", "Skip", "Next"]) t.ok(tutorialEn.has(word), `in en the tutorial shows ${word}`);
  },
});
