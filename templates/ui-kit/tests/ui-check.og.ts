// og ui check on the kit's phone HUDs (docs/ui.md §16.5): the checker that
// `og ui check`, `og polish` and playtests share reads every showcase's
// play screen laid out on a 390×844 phone (the kit's
// `viewport:phone`). The fixed templates this test once read shipped two
// undeclared overlaps (the platformer's timer under its coins, the Gallery
// button over the puzzle's score); the HUDs built from components replace
// them, and their check comes back clean.
import { checkUi } from "@origingame/engine/ui-check";
import { playtest } from "./lib.ts";

/** Every showcase's play screen (src/showcases.ts). */
const HUDS = ["actionHud", "shooterHud", "manageHud", "raceHud", "sportsHud", "mobaHud", "rpgHud", "mmoHud", "rogueHud", "tdHud", "rhythmHud", "fightHud", "survivalHud", "horrorHud", "openHud"] as const;

export default playtest({
  name: "og ui check finds no undeclared overlap on the kit's phone HUDs",
  size: "390x844",
  commands: ["0.0:viewport:phone", ...HUDS.map((h, i) => `${(0.1 + i).toFixed(1)}:show:${h};${(0.9 + i).toFixed(1)}:dump_ui:${h}.uicheck.json`), `${HUDS.length + 0.2}:quit`].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    for (const h of HUDS) {
      const all = checkUi(run, h, { skip: ["contrast"] });
      t.ok(run.json<{ elements: unknown[] }>(`${h}.uicheck.json`).elements.length > 20, `${h} was laid out (${run.json<{ elements: unknown[] }>(`${h}.uicheck.json`).elements.length} elements)`);
      const overlaps = all.filter((f) => f.rule === "overlap");
      t.equal(overlaps.map((f) => f.paths.join(" × ")).join("; "), "", `${h} has no undeclared overlap`);
    }
  },
});
