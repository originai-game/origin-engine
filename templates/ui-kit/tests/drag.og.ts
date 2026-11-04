// Drag and drop in the gear screen's backpack (rpg.inventoryGrid): a
// press on the healing potions, a move while held to the backpack's last
// column, sixth row (an empty cell), and a release there moves the stack
// to that cell; the cell it left is empty and the backpack holds as many
// items as before.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo, ui, type UiElement } from "./lib.ts";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

const CELLS = "Gear/Body/View/Content/Split/Bags/Bag/Cells";
/** The cell the stack is dropped on: column 9, row 5 (from 0), empty in the kit's backpack. */
const TO = { col: 9, row: 5 };

export default playtest({
  name: "a drag from one backpack cell to another moves the stack",
  commands: [
    "0.2:show:gear",
    "0.7:dump_ui:before.json",
    "0.8:touch_move_to:Bag/Cells/potions",
    "0.85:touch_start",
    "0.9:touch_move:1000,300",
    "0.95:touch_move:1233,410",
    "1.05:touch_end",
    "1.3:dump_ui:moved.json",
    "1.3:dump_entity:demo:moved-demo.json",
    "1.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "moved-demo.json").screen, SCREEN.gear, "still on the gear screen");
    const at = (els: UiElement[], path: string) => els.find((e) => e.path === path);
    const before = ui(run, "before.json");
    const moved = ui(run, "moved.json");
    const grid = at(before, CELLS);
    const from = at(before, `${CELLS}/potions`);
    const to = at(moved, `${CELLS}/potions`);
    t.ok(grid !== undefined && from !== undefined && to !== undefined, "the backpack and its potions are laid out before and after");
    if (!grid || !from || !to) return;
    const cell = from.height;
    t.equal([Math.round((to.x - grid.x) / cell), Math.round((to.y - grid.y) / cell)], [TO.col, TO.row], `the potions moved to column ${TO.col}, row ${TO.row}`);
    const left = moved.filter((e) => e.role === "gridcell" && e.path.startsWith(`${CELLS}/`) && e.x <= from.x + 1 && e.x + e.width > from.x + 1 && e.y <= from.y + 1 && e.y + e.height > from.y + 1);
    t.equal(left.map((e) => e.path), [], "the cell they left is empty");
    t.equal(at(moved, CELLS)?.name, at(before, CELLS)?.name, `the backpack holds as many items as before (${at(before, CELLS)?.name})`);
  },
});
