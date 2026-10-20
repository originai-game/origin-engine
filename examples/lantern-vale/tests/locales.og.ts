// Both languages on every screen that has words: the engine's `locale:`
// action switches og.i18n to Chinese, and the title, the HUD, the pause
// menu and the results (og.ui.resultsScreen's best line and new-best badge
// too, through ResultsDesc.bestLabel and newBestLabel) read in Chinese (Noto Sans SC from the zh-CN table's
// font chain, the skin's faces for the Latin and the digits); back in
// English the same screens read in English. og cook warned of no missing
// key (og polish scores it).
import { playtest } from "@origingame/engine/playtest";
import { press, texts } from "./lib.ts";

export default playtest({
  name: "the title, the HUD, the pause menu and the results in Chinese, then English",
  commands: [
    "0.2:locale:zh-CN",
    "0.9:dump_ui:title-zh.json",
    "0.9:screenshot:title-zh.png",
    "1.0:run",
    "1.8:dump_ui:hud-zh.json",
    press("Escape", 1.9),
    "2.6:dump_ui:pause-zh.json",
    press("Escape", 2.7),
    "3.0:light:0",
    "3.1:light:1",
    "3.2:light:2",
    "3.3:light:3",
    "6.0:dump_ui:results-zh.json",
    "6.1:locale:en",
    "6.6:dump_ui:results-en.json",
    "6.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = texts(run, "title-zh.json");
    t.ok(title.includes("灯笼谷") && title.includes("出发") && title.includes("在夜幕降临前点亮四盏灯笼"), "the title reads in Chinese");
    t.ok(texts(run, "hud-zh.json").includes("暂停"), "the HUD's Pause too");
    const pause = texts(run, "pause-zh.json");
    t.ok(pause.includes("已暂停") && pause.includes("继续") && pause.includes("返回标题"), "and the pause menu");
    const results = texts(run, "results-zh.json");
    t.ok(results.includes("山谷灯火通明") && results.includes("再来一次") && results.includes("无名氏"), "and the results");
    t.ok(results.some((x) => /^最佳 [\d,]+$/.test(x)) && results.includes("新纪录") && !results.some((x) => x.startsWith("Best") || x === "New best"), "with og.ui's own best line and badge in Chinese");
    const en = texts(run, "results-en.json");
    t.ok(en.includes("The vale is lit") && en.includes("Again"), "back in English the results read in English");
    t.ok(en.some((x) => /^Best [\d,]+$/.test(x)) && en.includes("New best"), "and the best line and badge in English");
  },
});
