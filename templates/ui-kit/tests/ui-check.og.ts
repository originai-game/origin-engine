// og ui check on the kit's phone sheets (docs/ui.md §16.5): the checker
// that `og ui check`, `og polish` and playtests share reads the platformer
// and puzzle HUDs laid out on a 390×844 phone (the kit's `viewport:phone`)
// and finds the two phone bugs the kit shipped with as undeclared overlap:
// the platformer's timer pill under its coin pill, and the Gallery button
// over the puzzle's score chip (the framed box the checker names since the
// HUD's words sit on the hud chip). The kit is rebuilt in U8; until then
// this playtest records the bugs rather than fixing them, and U8's kit must
// turn these expectations round (a clean check).
import { playtest } from "@origingame/engine/playtest";
import { checkUi } from "@origingame/engine/ui-check";

export default playtest({
  name: "og ui check finds the kit's two known phone overlaps",
  size: "390x844",
  commands: "0.0:viewport:phone;0.0:show:platformer;0.8:dump_ui:platformer.uicheck.json;0.9:show:puzzle;1.7:dump_ui:puzzle.uicheck.json;1.8:quit",
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const overlaps = (name: string) => checkUi(run, name, { skip: ["contrast"] }).filter((f) => f.rule === "overlap");
    const platformer = overlaps("platformer");
    t.ok(
      platformer.some((f) => f.paths.includes("Hud.top/Timer") && f.paths.includes("Hud.right/Coins")),
      `the platformer's timer pill under its coin pill is an undeclared overlap (found: ${platformer.map((f) => f.paths.join(" × ")).join("; ")})`,
    );
    const puzzle = overlaps("puzzle");
    t.ok(
      puzzle.some((f) => f.paths.includes("Back/Gallery") && f.paths.includes("Hud.score")),
      `the Gallery button over the score chip is an undeclared overlap (found: ${puzzle.map((f) => f.paths.join(" × ")).join("; ")})`,
    );
    for (const f of [...platformer, ...puzzle]) t.ok(f.prop !== "" && f.measured.includes("CSS px"), `${f.paths.join(" × ")} names its measure and the prop to change`);
  },
});
