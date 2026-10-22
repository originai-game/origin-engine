// Swapping by a drag: press on (2,1), move half a cell down while held,
// release. The same line as the tap-tap swap clears.
import { playtest } from "@origingame/engine/playtest";
import { CASCADE_BOARD, PHASE, checkMapping, matchOf, px, tap } from "./lib.ts";

export default playtest({
  name: "a drag onto a neighbour swaps like two taps",
  commands: [
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    `1.0:touch_move:${px(2, 1)}`,
    "1.0:touch_start",
    `1.05:touch_move:${px(2, 1, 0, -0.3)}`,
    "1.1:dump_entity:board:short.json",
    `1.15:touch_move:${px(2, 1, 0, -0.6)}`,
    "1.25:dump_entity:board:dragged.json",
    "1.3:touch_end",
    "3.0:dump_entity:board:settled.json",
    "3.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const short = matchOf(run, "short.json");
    checkMapping(short, t);
    t.equal(short.phase, PHASE.idle, "a drag shorter than half a cell does not swap yet");
    t.equal(matchOf(run, "dragged.json").phase, PHASE.swap, "dragging past half a cell swaps");
    const settled = matchOf(run, "settled.json");
    t.equal(settled.movesLeft, 19, "the drag spent one move");
    t.ok(settled.cleared >= 3 && settled.score >= 30, `the line cleared (${settled.cleared} tiles, score ${settled.score})`);
  },
});
