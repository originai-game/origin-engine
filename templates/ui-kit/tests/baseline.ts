// The baseline sheets (the owner, 2026-11-04: the engine's UI is judged on
// its own, each component and pattern on the pack's own ground, never in a
// game's scene): every seed style's components, patterns and HUD widgets
// on the pack's backdrop (`demo:ground:pack`, src/ground.ts), composed into
// kit-baseline-<pack>.png on a desktop (1280×720, 100 % text) and
// kit-baseline-<pack>-phone.png on a phone in portrait (390×844@3), in
// BASELINE order, for the art director to read. Review sheets only: no
// reference is recorded or compared; the playtest checks that every page
// was built and settled before its frame.

import { playtest, type PlaytestDesc } from "@origingame/engine/playtest";
import { STYLES } from "../src/main.ts";
import { ui } from "./lib.ts";
import { STYLE_SETTLE } from "./script.ts";
import { composeSheet, SURFACES, unsettled, withSettleDumps } from "./styles.ts";

/**
 * The pages of a baseline sheet, in order: [file stem, the commands that set it up, the element path its dump must
 * hold]. The components (buttons and toggles, tabs, fields, ranges, lists and tables, containers, overlays, meters, rings
 * and touch controls), the patterns (inventory and paperdoll, shop, gacha, reward choice, dialogue, quests, lobby,
 * settings, pause, results) and the HUD widgets as the showcases compose them.
 */
export const BASELINE: ReadonlyArray<[string, readonly string[], string]> = [
  ["widgets", ["show:widgets"], "Back"],
  ["foundations", ["show:foundations"], "Back"],
  ["controls", ["show:controls"], "Back"],
  ["fields", ["show:fields"], "Back"],
  ["ranges", ["show:ranges"], "Back"],
  ["navigation", ["show:navigation"], "Back"],
  ["containers", ["show:containers"], "Back"],
  ["data", ["show:data"], "Back"],
  ["overlays-menu", ["show:overlays", "demo:overlay:menu"], "Back"],
  ["overlays-dialog", ["demo:overlay:dialog"], "Back"],
  ["gear", ["demo:overlay:none", "show:gear", "pat:reset"], "Back"],
  ["store", ["show:store"], "Back"],
  ["gacha", ["show:gacha"], "Back"],
  ["rewards", ["show:rewards"], "Back"],
  ["dialogue", ["show:dialogue"], "Back"],
  ["quests", ["show:quests"], "Back"],
  ["lobby", ["show:lobby"], "Back"],
  ["settings", ["show:settingsTree"], "Back"],
  ["pause", ["show:pauseMenu"], "pause"],
  ["results", ["show:matchResults"], "results"],
  ["actionHud", ["show:actionHud", "sc:reset"], "Back"],
  ["shooterHud", ["show:shooterHud"], "Back"],
  ["rpgHud", ["show:rpgHud"], "Back"],
  ["mmoHud", ["show:mmoHud"], "Back"],
  ["survivalHud", ["show:survivalHud"], "Back"],
  ["raceHud", ["show:raceHud"], "Back"],
];

/** The packs the sheets are drawn in: every seed style (the default style is origin-agent's, not a game's). */
export const BASELINE_PACKS: readonly string[] = STYLES.filter((s) => s !== "default");

/** One run per surface over every pack: the ground set once, then each pack's pages, each taken once settled. */
function script(packs: readonly string[]): { commands: string; sheets: Array<[string, string[]]> } {
  const settle = STYLE_SETTLE.default ?? 1;
  const steps: string[] = ["0.20:demo:ground:pack"];
  const sheets: Array<[string, string[]]> = [];
  let t = 0.3;
  for (const pack of packs) {
    steps.push(`${t.toFixed(2)}:style:${pack}`);
    t += 0.05;
    const files: string[] = [];
    for (const [stem, cmds] of BASELINE) {
      cmds.forEach((c, k) => steps.push(`${(t + 0.05 * k).toFixed(2)}:${c}`));
      t += 0.05 * (cmds.length - 1) + settle;
      const file = `bl-${pack}-${stem}`;
      steps.push(`${t.toFixed(2)}:dump_ui:${file}.json`, `${t.toFixed(2)}:screenshot:${file}.png`);
      files.push(`${file}.png`);
      t += 0.1;
    }
    sheets.push([pack, files]);
  }
  steps.push(`${t.toFixed(2)}:quit`);
  return { commands: withSettleDumps(steps.join(";")), sheets };
}

/** The baseline sheets of `packs` (every seed by default) on a surface. */
export function baselinePlaytest(on: "desktop" | "phone", packs: readonly string[] = BASELINE_PACKS): PlaytestDesc {
  const surface = SURFACES[on];
  const { commands, sheets } = script(packs);
  return playtest({
    name: `every seed style's components, patterns and HUD widgets on its own ground${on === "phone" ? " on a phone" : ""}, composed into kit-baseline-<pack>${surface.suffix}.png for review`,
    size: surface.size,
    commands: surface.viewport + commands,
    check(run, t) {
      t.equal(run.warnings.join("; "), "", "the script ran without warnings");
      t.ok(run.quit, "the script reached quit");
      for (const [pack, files] of sheets) {
        files.forEach((f, i) => {
          const stem = f.replace(/\.png$/, "");
          const want = BASELINE[i]![2];
          t.ok(ui(run, `${stem}.json`).some((e) => e.path === want || e.path.startsWith(`${want}/`) || e.path.endsWith(`/${want}`)), `${pack} ${BASELINE[i]![0]} built its page (${want})`);
          t.equal(unsettled(run, stem), "", `${pack} ${BASELINE[i]![0]} is captured with every entrance and exit settled`);
        });
        // Review sheets: composed for the art director, never compared (no reference is recorded).
        t.frame(files, () => {
          const name = `kit-baseline-${pack}${surface.suffix}.png`;
          composeSheet(files, name, surface);
          t.ok(true, `composed ${name} (${files.length} pages) for review`);
        });
      }
    },
  });
}
