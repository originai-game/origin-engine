// Drag and drop in the inventory grid: a press on slot 0, a move to slot 7
// while held, a release there moves the potion; Use on the moved item runs.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, tap } from "./lib.ts";

export default playtest({
  name: "a drag from one inventory slot to another moves the item",
  commands: [
    "0.2:show:inventory",
    "0.8:touch_move_to:Slots/Cell#0",
    "0.85:touch_start",
    "0.95:touch_move_to:Slots/Cell#7",
    "1.05:touch_end",
    "1.3:dump_entity:demo:moved.json",
    tap(1.4, "Detail/Use"),
    "1.7:dump_entity:demo:used.json",
    "1.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const moved = demo(run, "moved.json");
    t.equal(moved.screen, SCREEN.inventory, "still in the inventory");
    t.equal([moved.inv0, moved.moves], [7, 1], "the potion moved from slot 0 to slot 7");
    t.equal(demo(run, "used.json").used, 1, "Use ran for the selected (moved) item");
  },
});
