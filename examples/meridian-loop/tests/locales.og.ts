// German, every screen: the title, the leaderboard, the race's HUD, pause,
// the settings and the results, read from dump_ui. No word of the English
// table is left on them (the ones both languages share aside), and no
// focusable control is unnamed. The racing HUD's own words stay English:
// it has no field for them (FINDINGS #8), asserted here so a fix shows.
import { playtest } from "@origingame/engine/playtest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { press, race, script, tap, texts } from "./lib.ts";
import { referencesDir } from "./frames.ts";

/** The English table's strings, `{arguments}` cut out. */
function english(): string[] {
  const text = readFileSync(join(referencesDir(), "..", "assets", "strings", "en.ogt"), "utf8");
  const out: string[] = [];
  for (const line of text.split("\n")) {
    const m = /^string "([^"]+)" "(.*)"$/.exec(line);
    if (m) out.push(...m[2]!.split(/\{[^}]*\}/).map((s) => s.trim()).filter((s) => s.length > 3));
  }
  return out;
}

/** Strings both tables share: names, units and the words German borrows. */
const SHARED = new Set(["English", "Deutsch", "Meridian Loop", "km/h", "Audio", "Vibration"]);

export default playtest({
  name: "every screen in German: no English left of the game's table, every control named; the racing HUD's own words still English",
  commands: script(
    "0.3:language:de",
    "1:dump_ui:title.json",
    "1:get:Race:title-race.json",
    tap("Items/board", 1.2),
    "2.2:dump_ui:board.json",
    press("Escape", 2.4),
    "3:race:go",
    "3.1:drive:auto",
    "6:dump_ui:hud.json",
    "6.1:open:pause",
    "6.6:dump_ui:pause.json",
    "6.6:get:Race:pause-race.json",
    "6.7:open:settings",
    "7.3:dump_ui:settings.json",
    "7.3:get:Race:settings-race.json",
    press("Escape", 7.5),
    press("Escape", 8),
    "106:dump_ui:results.json",
    "106:get:Race:results-race.json",
    "106.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const en = english().filter((s) => !SHARED.has(s));
    for (const [file, what] of [["title.json", "title"], ["board.json", "leaderboard"], ["hud.json", "HUD"], ["pause.json", "pause"], ["settings.json", "settings"], ["results.json", "results"]] as const) {
      const shown = texts(run, file);
      const left = shown.filter((s) => en.some((e) => s.includes(e)));
      t.equal(left.join(" | "), "", `the ${what} shows no English of the game's table`);
    }
    t.ok(texts(run, "title.json").includes("Zeitfahren"), "the title in German");
    t.ok(texts(run, "board.json").includes("Schnellste Runden"), "the leaderboard in German");
    t.ok(texts(run, "pause.json").includes("Pausiert"), "pause in German");
    t.ok(texts(run, "settings.json").includes("Einstellungen"), "the settings in German");
    t.ok(texts(run, "results.json").includes("Im Ziel") || texts(run, "results.json").includes("Neuer Rundenrekord"), "the results in German");
    // The racing HUD's own words (FINDINGS #8): no RacingHudDesc field to pass them.
    const hud = texts(run, "hud.json");
    t.ok(hud.includes("lap"), "the racing HUD still says `lap` (FINDINGS #8)");
    t.ok(hud.includes("Boost"), "and `Boost`, a meter a time trial has no use for (FINDINGS #8)");
    t.ok(hud.includes("Pause"), "and its pause button `Pause` (FINDINGS #8)");
    for (const f of ["title-race.json", "pause-race.json", "settings-race.json", "results-race.json"]) t.equal(race(run, f).a11yUnnamed, 0, `no focusable control unnamed (${f.replace("-race.json", "")})`);
  },
});
