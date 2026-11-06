// The kit's locale state and translated screen/HUD strings in five locales.
// These are behavioral checks from dumps, not a repeated image matrix. The
// probe's `locale:<tag>` asks for each locale; `og.i18n.locale` (the facts)
// says which table is active.
// Then the settings tree, the store, the gear screen, the leaderboard, the
// tutorial and the dialogue in zh-CN: every word they draw (the documents'
// keyed words, the patterns' descs' label fields, og-i18n §2) comes from the
// Chinese table, with none of the English left. Last the showcases' five
// HUDs in every locale, on a desktop and on a tablet (their touch controls
// shown): in zh-CN, ja, ko and ar no element shows a Latin word but the
// game's proper names (racers, players, teams, the weapon), key glyphs,
// compass points and units, and the touch controls are named in the
// locale's words; in en the words are the table's English. Not read: an
// icon's text (an identifier), og-ui's own d-pad (chevrons, arms named Up,
// Left, Right and Down), and the HUD components' own accessible names of
// their regions (Ammunition, Standings, Minimap…), which the engine's
// components name in English (reported, not this kit's words).
import { playtest, ui } from "./lib.ts";

const LOCALES = ["en", "zh-CN", "ja", "ko", "ar"] as const;
const SCREENS = ["actionTitle", "actionPause"] as const;

const steps: string[] = [];
let t = 0.3;
for (const l of LOCALES) {
  steps.push(`${t.toFixed(2)}:locale:${l}`);
  for (const s of SCREENS) {
    steps.push(`${(t + 0.05).toFixed(2)}:show:${s}`);
    t += 0.8;
  }
  steps.push(`${t.toFixed(2)}:dump_world:world-${l}.json`);
  t += 0.1;
}
// The pattern and showcase screens in zh-CN, then the settings tree and the tutorial in en.
const ZH = ["settingsTree", "store", "gear", "raceBoard", "actionTutorial", "dialogue"] as const;
steps.push(`${t.toFixed(2)}:locale:zh-CN`);
for (const s of ZH) {
  steps.push(`${(t + 0.05).toFixed(2)}:show:${s}`);
  steps.push(`${(t + 0.7).toFixed(2)}:dump_ui:${s}-zh-CN.json`);
  t += 0.75;
}
steps.push(`${t.toFixed(2)}:locale:en`);
for (const s of ["settingsTree", "actionTutorial"] as const) {
  steps.push(`${(t + 0.05).toFixed(2)}:show:${s}`);
  steps.push(`${(t + 0.7).toFixed(2)}:dump_ui:${s}-en.json`);
  t += 0.75;
}
// The HUDs: each locale, each surface, each HUD.
const HUDS = ["actionHud", "shooterHud", "manageHud", "raceHud", "sportsHud"] as const;
const SURFACES = ["desktop", "tablet"] as const;
const hudFiles: Array<{ locale: string; surface: string; hud: string; file: string }> = [];
for (const l of LOCALES) {
  steps.push(`${t.toFixed(2)}:locale:${l}`);
  for (const v of SURFACES) {
    steps.push(`${(t + 0.02).toFixed(2)}:viewport:${v}`);
    t += 0.05;
    for (const h of HUDS) {
      const file = `hud-${h}-${v}-${l}.json`;
      steps.push(`${t.toFixed(2)}:show:${h}`);
      steps.push(`${(t + 0.6).toFixed(2)}:dump_ui:${file}`);
      hudFiles.push({ locale: l, surface: v, hud: h, file });
      t += 0.65;
    }
  }
}
steps.push(`${t.toFixed(2)}:viewport:desktop`);
t += 0.05;
steps.push(`${t.toFixed(2)}:quit`);

/** The touch controls a HUD names with the game's words (src/huds.ts): their names are read too. */
const NAMED = new Set(["Pause", "Jump", "Attack", "Fire", "Left", "Right", "Brake", "Boost"]);
/** The game's proper names (src/huds.ts): racers, players, teams, the weapons; they stay as they are in every locale. */
const PROPER = new Set(["Vex", "Juno", "Orrin", "Sable", "Brask", "Raven", "Frag", "Nyx", "Ferro", "Lumen", "Quill", "Arlo", "Moreau", "Ilves", "RVR", "HLD"]);
/** Symbols, not words: compass points and units (m, km/h, per second, thousands). */
const SYMBOLS = new Set(["N", "NE", "E", "SE", "S", "SW", "W", "NW", "m", "km", "h", "s", "K", "M", "B"]);
/** A Latin token that is no word of a locale's: a proper name, a symbol, or one capital (a key's glyph). */
const allowed = (w: string) => PROPER.has(w) || SYMBOLS.has(w) || /^[A-Z]$/.test(w);

export default playtest({
  name: "locale state switches; the settings tree, store, gear, leaderboard, tutorial and dialogue are Chinese in zh-CN; the five showcase HUDs have no English left in zh-CN, ja, ko and ar",
  size: "1280x720",
  commands: steps.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings (every key is in every table)");
    t.ok(run.quit, "the script reached quit");

    /** What a screen shows and names (a tab's label is drawn in capitals; its name is the word). */
    const words = (file: string) => new Set(ui(run, file).flatMap((e) => [e.text, e.name]).filter((x) => x !== ""));
    const zh = (screen: string, chinese: string[], english: string[]) => {
      const w = words(`${screen}-zh-CN.json`);
      for (const word of chinese) t.ok(w.has(word), `the ${screen} screen shows ${word} in zh-CN`);
      for (const word of english) t.ok(!w.has(word), `and not the English ${word}`);
    };
    zh("settingsTree", ["设置", "图形", "音频", "操作", "语言", "画质预设", "应用", "还原", "默认"], ["Settings", "Graphics", "Audio", "Controls", "Language", "Quality preset", "Apply", "Revert", "Defaults"]);
    zh("store", ["商店", "精选", "每日特惠", "已拥有", "购买"], ["Store", "Featured", "Owned", "Buy"]);
    zh("gear", ["背包", "仓库", "属性", "护甲"], ["Backpack", "Stash", "Attributes", "Armor"]);
    zh("raceBoard", ["排行榜", "全球", "好友", "联赛"], ["Leaderboard", "Global", "Friends", "League"]);
    zh("actionTutorial", ["跳过", "下一步", "你的生命"], ["Skip", "Next", "Your health"]);
    zh("dialogue", ["跳过", "自动", "记录"], ["Skip", "Auto", "Log"]);
    // In English the table's English words.
    const en = (screen: string, list: string[]) => {
      const w = words(`${screen}-en.json`);
      for (const word of list) t.ok(w.has(word), `in en the ${screen} screen shows ${word}`);
    };
    en("settingsTree", ["Settings", "Graphics", "Audio", "Apply", "Revert", "Defaults"]);
    en("actionTutorial", ["Skip", "Next", "Your health"]);

    // The HUDs: no Latin word in another locale, shown (or naming a touch control).
    for (const { locale, surface, hud, file } of hudFiles) {
      if (locale === "en") continue;
      const els = ui(run, file).filter((e) => e.kind !== "icon" && e.parent !== "Dpad");
      const shown = els.flatMap((e) => (NAMED.has(e.id) ? [e.text, e.name] : [e.text]));
      const latin = [...new Set(shown.flatMap((x) => x.match(/[A-Za-z]+/g) ?? []).filter((w) => !allowed(w)))];
      t.equal(latin.join(", "), "", `the ${hud} HUD on a ${surface} in ${locale} has no English left`);
    }
    const hudWords = (hud: string, surface: string, locale: string) => new Set(ui(run, `hud-${hud}-${surface}-${locale}.json`).flatMap((e) => [e.text, e.name]).filter((x) => x !== ""));
    // In en the table's English, in zh-CN its Chinese: one word of each HUD's on each side.
    const pairs: Array<[hud: string, surface: string, en: string, zh: string]> = [
      ["actionHud", "desktop", "Combo", "连击"],
      ["actionHud", "desktop", "Time", "时间"],
      ["actionHud", "tablet", "Jump", "跳跃"],
      ["actionHud", "tablet", "Attack", "攻击"],
      ["shooterHud", "desktop", "Ring closes", "毒圈收缩"],
      ["shooterHud", "tablet", "Fire", "开火"],
      ["manageHud", "desktop", "Spring, day 12", "春季第12天"],
      ["raceHud", "desktop", "Best", "最佳"],
      ["raceHud", "desktop", "Lap", "圈"],
      ["raceHud", "tablet", "Brake", "刹车"],
      ["raceHud", "tablet", "Left", "左"],
      ["sportsHud", "desktop", "2nd half", "下半场"],
    ];
    for (const [hud, surface, word, cn] of pairs) {
      t.ok(hudWords(hud, surface, "en").has(word), `in en the ${hud} on a ${surface} shows ${word}`);
      t.ok(hudWords(hud, surface, "zh-CN").has(cn), `in zh-CN it shows ${cn}`);
    }
  },
});
