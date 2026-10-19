// A high-DPR screen: the play surface is 640×360 CSS px at 2× over the
// same 1280×720 render target. The pointer arrives in CSS pixels, so the
// mapping Pick works out is too (half the 1× numbers), and a tap on a
// cell's CSS pixel selects that cell.
import { playtest } from "@origingame/engine/playtest";
import { cellCentre } from "../src/layout.ts";
import { CASCADE_BOARD, MAPPING, PHASE, matchOf } from "./lib.ts";

/** The 2× surface's mapping: the same view over half the CSS pixels. */
const HALF = { pxPerM: MAPPING.pxPerM / 2, screenX0: MAPPING.screenX0 / 2, screenY0: MAPPING.screenY0 / 2 } as const;

function tapCell(t: number, col: number, row: number): string {
  const c = cellCentre(col, row);
  const at = `${Math.round(HALF.screenX0 + c.x * HALF.pxPerM)},${Math.round(HALF.screenY0 - c.y * HALF.pxPerM)}`;
  return `${t}:touch_move:${at};${t}:touch_start;${(t + 0.05).toFixed(2)}:touch_end`;
}

export default playtest({
  name: "on a 2× screen the pointer maps in CSS pixels and a tap selects the cell under it",
  commands: [
    "0.05:viewport:2x",
    "0.5:touch_move:320,340;0.5:touch_start;0.55:touch_end",
    `0.8:deal:${CASCADE_BOARD}`,
    "0.9:dump_entity:board:dealt.json",
    tapCell(1.0, 5, 0),
    "1.1:dump_entity:board:selected.json",
    tapCell(1.3, 1, 4),
    "1.4:dump_entity:board:moved.json",
    "1.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const dealt = matchOf(run, "dealt.json");
    t.equal(dealt.phase, PHASE.idle, "a tap on the title dealt a round");
    t.equal(dealt.pxPerM, HALF.pxPerM, "30 CSS px per metre on the 360 px tall surface");
    t.equal(dealt.screenX0, HALF.screenX0, "the world origin is at CSS x 320");
    t.equal(dealt.screenY0, HALF.screenY0, "and at CSS y 195");
    t.equal(matchOf(run, "selected.json").selected, 5, "a tap on (5,0) selected cell 5");
    t.equal(matchOf(run, "moved.json").selected, 4 * 7 + 1, "a tap on (1,4), not a neighbour, selected cell 29 instead");
  },
});
