// Swapping by two taps on an authored board: the swap that makes a line of
// three clears it, scores, spends one move, and the tiles that fall make a
// second line (a cascade worth double). A swap that makes nothing goes back
// and costs nothing.
import { playtest } from "@origingame/engine/playtest";
import { SIZE } from "../src/layout.ts";
import { CASCADE_BOARD, PHASE, checkMapping, matchOf, tap, tapCell } from "./lib.ts";

export default playtest({
  name: "a swap that makes a line clears, scores and cascades; one that makes nothing is free",
  commands: [
    "0.3:screenshot:title.png",
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    "0.9:dump_entity:board:dealt.json",
    "0.9:screenshot:play.png",
    // The refused swap first: (5,0) then (6,0).
    tapCell(1.0, 5, 0),
    "1.1:dump_entity:board:selected.json",
    tapCell(1.2, 6, 0),
    "1.42:dump_entity:board:swapping-back.json",
    "1.8:dump_entity:board:refused.json",
    // The line: (2,0) then (2,1).
    tapCell(2.0, 2, 0),
    tapCell(2.2, 2, 1),
    "2.45:screenshot:clearing.png",
    "2.45:dump_entity:board:clearing.json",
    "4.0:dump_entity:board:settled.json",
    "4.0:screenshot:settled.png",
    "4.0:dump_world:world.json",
    "4.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const dealt = matchOf(run, "dealt.json");
    checkMapping(dealt, t);
    t.equal(dealt.phase, PHASE.idle, "a tap on the title deals a round");
    t.equal(dealt.movesLeft, 20, "the round starts with Tuning.moves");

    t.equal(matchOf(run, "selected.json").selected, 5, "the first tap selects cell (5,0)");
    t.equal(matchOf(run, "swapping-back.json").phase, PHASE.swapBack, "the swap that makes nothing goes back");
    const refused = matchOf(run, "refused.json");
    t.equal(refused.phase, PHASE.idle, "back to idle");
    t.equal(refused.movesLeft, 20, "the refused swap cost no move");
    t.equal(refused.score, 0, "and scored nothing");

    const clearing = matchOf(run, "clearing.json");
    t.equal(clearing.phase, PHASE.clear, "the line is clearing");
    t.equal(clearing.movesLeft, 19, "the swap that made a line spent a move");

    const settled = matchOf(run, "settled.json");
    t.equal(settled.phase, PHASE.idle, "the board settled");
    t.ok(settled.maxChain >= 2, `the fall made a second line: a cascade (chain ${settled.maxChain})`);
    t.ok(settled.cleared >= 6, `both lines cleared (${settled.cleared} tiles)`);
    t.ok(settled.score >= 3 * 10 + 3 * 10 * 2, `three tiles at 10, then three at 10 x 2 (score ${settled.score})`);

    const world = run.json<{ systems: Array<{ name: string; matched: number | null }> }>("world.json");
    // The backdrop, the board, the 49 gems and the selection frame; the checker tilemap is no row.
    const rows = SIZE * SIZE + 3;
    for (const s of ["Flow", "Pick", "Resolve", "Animate"]) {
      const sys = world.systems.find((x) => x.name === s);
      t.equal(sys?.matched, rows, `${s} sees every row`);
    }
    t.equal(world.systems.find((x) => x.name === "Hud")?.matched, 1, "Hud sees the board row only");
  },
});
