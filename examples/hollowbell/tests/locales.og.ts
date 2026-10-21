// Every screen in Chinese, then English again: the title, the HUD's area
// toast, the pause menu, the mole's talk and wares, the bell's refusal and
// the results (og.ui's own words "Best" and "New best" from the tables,
// ResultsDesc.bestLabel and newBestLabel). The shop's own "Price", "Buy"
// and "Not enough" stay English: ShopDesc has no field for them (FINDINGS.md #F-shopwords);
// the check records it.
import { playtest } from "@origingame/engine/playtest";
import { press, script, texts } from "./lib.ts";

export default playtest({
  name: "the title, HUD, pause, talk, shop and results in Chinese, then English",
  commands: script(
    "0.2:language:zh-CN",
    "0.2:name:小铃",
    "0.9:dump_ui:title.json",
    "1.0:play",
    "1.4:dump_ui:hud.json",
    press("Escape", 1.5),
    "1.9:dump_ui:pause.json",
    press("Escape", 2.0),
    "2.3:warp:mole",
    "2.4:give:glimmers:3",
    press("ArrowUp", 2.8),
    "3.0:dump_ui:talk.json",
    press("Escape", 3.1),
    "3.6:dump_ui:shop.json",
    press("Escape", 3.7),
    "4.0:give:chimes:3",
    "4.1:warp:bell:1.2",
    press("ArrowUp", 4.6),
    "9.2:dump_ui:results.json",
    "9.3:language:en",
    "9.6:dump_ui:results-en.json",
    "9.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = texts(run, "title.json");
    t.ok(title.includes("空心钟") && title.includes("新游戏") && title.includes("设置"), "the title is Chinese");
    const hud = texts(run, "hud.json");
    t.ok(hud.includes("空心钟村") && hud.includes("暂停"), "the area's toast and Pause are Chinese");
    const pause = texts(run, "pause.json");
    t.ok(pause.includes("已暂停") && pause.includes("继续") && pause.includes("保存并退出"), "the pause menu is Chinese");
    const talk = texts(run, "talk.json");
    t.ok(talk.includes("老洞鼠"), "Old Burrow is 老洞鼠");
    const shop = texts(run, "shop.json");
    t.ok(shop.includes("老洞鼠的货摊") && shop.includes("蛾翼斗篷") && shop.includes("6 微光"), "his wares are Chinese");
    t.ok(shop.includes("Price") && shop.includes("Not enough"), "the shop template's own Price and Not enough (Buy) stay English (FINDINGS.md #F-shopwords)");
    const results = texts(run, "results.json");
    t.ok(results.includes("大钟再次响起") && results.includes("新纪录") && results.some((s) => s.startsWith("最佳 ")) && results.includes("小铃"), "the results are Chinese, the kit's Best and New best included");
    const en = texts(run, "results-en.json");
    t.ok(en.includes("The bell rings again") && en.includes("New best"), "and English again");
  },
});
