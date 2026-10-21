// Japanese (og-i18n): every screen's words come from assets/strings/ja.ogt
// in Noto Sans JP, subset to the table's characters by tools/subset-font.py
// (CJK only, so the footer's key names keep the skin's Latin face, run by
// run). The title, the intro, the HUD, the bag, the pause menu, the
// settings (og.ui.settingsScreen, every word passed from the table) and the
// results read in Japanese; back in English, the title does too. The
// templates' own words that have no desc field yet stay English (row 76 of
// the engine's phase-1 list: the cutscene's and the tutorial's Skip and
// Next, the tutorial's step count, the bag's hint and Count). And the
// accessibility tree (og.ui.a11y, counted by the Ui system): no focusable
// control unnamed on any of these screens.
import { playtest } from "@origingame/engine/playtest";
import { game, press, script, tap, texts } from "./lib.ts";

export default playtest({
  name: "the title, intro, HUD, bag, pause, settings and results in Japanese, then English; no unnamed control in the accessibility tree",
  commands: script(
    "0.3:language:ja",
    "0.9:dump_ui:title.json",
    "0.9:get:Game:a11y-title.json",
    "1.0:play",
    "1.6:dump_ui:intro.json",
    tap("Skip", 1.7),
    "2.2:dump_ui:tutorial.json",
    tap("Skip", 2.3),
    "2.8:dump_ui:hud.json",
    "2.85:give:herbs:1",
    press("Tab", 3.0),
    tap("Slots/Cell#4", 3.4),
    "3.8:dump_ui:bag.json",
    "3.8:get:Game:a11y-bag.json",
    press("Escape", 4.0),
    press("Escape", 4.4),
    "4.8:dump_ui:pause.json",
    "4.8:get:Game:a11y-pause.json",
    tap("settings", 5.0),
    "5.6:dump_ui:settings.json",
    "5.6:get:Game:a11y-settings.json",
    tap("Done", 5.8),
    tap("resume", 6.3),
    "6.8:ring",
    tap("Skip", 7.4),
    "9.4:dump_ui:results.json",
    "9.4:get:Game:a11y-results.json",
    "9.5:language:en",
    tap("title", 9.6),
    "10.6:dump_ui:title-en.json",
    "10.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (no glyph missing)");
    const has = (file: string, words: string[], what: string) => {
      const shown = texts(run, file);
      const missing = words.filter((w) => !shown.includes(w));
      t.equal(missing, [], `${what} in Japanese (${words.join(" ")})`);
    };
    has("title.json", ["隼の階段", "新しい旅", "章をえらぶ", "記録", "設定"], "the title");
    has("intro.json", ["院長マレン"], "the intro's speaker");
    has("tutorial.json", ["体力", "小鬼は噛み、番兵は叩きつぶす。薬草と道標石で回復する。"], "the tutorial's first step");
    has("hud.json", ["鞄", "一時停止", "0/3"], "the HUD");
    has("bag.json", ["鞄", "薬草", "苦い葉。体力が40戻る。", "使う"], "the bag");
    has("pause.json", ["一時停止", "再開", "設定", "休んで終わる"], "the pause menu");
    has("settings.json", ["設定", "音", "画面", "操作", "言語", "全体", "音楽", "効果音", "完了"], "the settings");
    has("results.json", ["鐘が鳴る", "時間", "紋章", "倒した敵", "もう一度", "タイトル"], "the results");
    // The templates' words without a desc field (row 76, open): still English.
    t.ok(texts(run, "intro.json").includes("Skip") && texts(run, "tutorial.json").includes("1 of 4"), "the cutscene's Skip and the tutorial's count stay English (row 76)");
    has("title-en.json", ["The Kestrel Stair", "New journey"], "and back in English, the title");
    for (const f of ["a11y-title", "a11y-bag", "a11y-pause", "a11y-settings", "a11y-results"]) {
      const g = game(run, `${f}.json`);
      t.ok(g.a11yNodes > 5 && g.a11yUnnamed === 0, `${f.slice(5)}: ${g.a11yNodes} accessible nodes, none focusable without a name`);
    }
  },
});
