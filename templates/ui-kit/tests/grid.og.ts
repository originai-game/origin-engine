// The grids lay out the columns they are asked for (kestrel-stair FINDINGS
// #15): the gear screen's backpack ten cells a row on a desktop and eight
// on a phone in portrait (where eight cells of at least 44 px fill the
// width), every item on a whole cell; and the store's offers as a grid,
// the featured row's three offers side by side on a desktop and, asked for
// two columns on a phone, the featured bundle across the width with the
// next two sharing a row, each offer's name inside its card.
import { playtest, type UiElement, ui } from "./lib.ts";

const CELLS = (stack: string) => `Gear/Body/View/Content/${stack}/Bags/Bag/Cells`;
const OFFERS = "Shop/Body/View/Content/Store/Sections/featured/Offers";

export default playtest({
  name: "the backpack lays out ten cells a row (eight on a phone) and the store its offers in the columns asked",
  commands: [
    "0.2:show:gear",
    "0.9:dump_ui:gear.json",
    "1.0:show:store",
    "1.7:dump_ui:store.json",
    "1.8:viewport:phone",
    "1.9:show:gear",
    // The phone stacks the backpack under the paperdoll, past the body's foot: scroll it into view.
    "2.3:touch_move_to:Doll",
    "2.35:scroll:0,600",
    "2.6:dump_ui:gear-phone.json",
    "2.7:show:store",
    "3.4:dump_ui:store-phone.json",
    "3.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const at = (els: UiElement[], path: string) => els.find((e) => e.path === path);
    for (const [file, stack, columns, rows, surface] of [["gear.json", "Split", 10, 6, "desktop"], ["gear-phone.json", "Stack", 8, 7, "phone"]] as const) {
      const els = ui(run, file);
      const grid = at(els, CELLS(stack));
      // A one-cell item (the potions) is a cell wide.
      const cell = at(els, `${CELLS(stack)}/potions`)?.width ?? 0;
      t.ok(grid !== undefined && cell > 0, `the backpack is laid out on the ${surface}`);
      if (!grid) continue;
      t.ok(grid.name.endsWith(`/ ${columns * rows}`), `the backpack holds ${columns} × ${rows} cells on the ${surface} (${grid.name})`);
      const items = els.filter((e) => e.role === "gridcell" && e.parent === "Cells" && e.path.startsWith(CELLS(stack)));
      const off = items.filter((e) => {
        const col = (e.x - grid.x) / cell;
        return Math.abs(col - Math.round(col)) > 0.01 || Math.round(col) + Math.round(e.width / cell) > columns;
      });
      t.equal(off.map((e) => `${e.name} at ${e.x}`), [], `every item sits on a whole cell inside the ${columns} columns on the ${surface} (${items.length} items, cell ${cell} px)`);
      if (surface === "phone") {
        t.ok(cell >= 44, `the phone's cells are at least 44 px (${cell})`);
        t.ok(Math.abs(grid.width - columns * cell) < 0.5, `eight cells fill the phone's grid, with no sideways scroll (${grid.width} px)`);
      }
    }
    for (const [file, surface] of [["store.json", "desktop"], ["store-phone.json", "phone"]] as const) {
      const els = ui(run, file);
      const card = (k: string) => at(els, `${OFFERS}/${k}`);
      const [tide, sword, crown] = [card("tide"), card("ashbringer"), card("crown")];
      t.ok(tide !== undefined && sword !== undefined && crown !== undefined, `the featured offers are laid out on the ${surface}`);
      if (!tide || !sword || !crown) continue;
      if (surface === "desktop") {
        t.ok(tide.y === sword.y && sword.y === crown.y && tide.x < sword.x && sword.x < crown.x, "on the desktop the three featured offers share one row");
      } else {
        t.ok(sword.y > tide.y && sword.y === crown.y && sword.x < crown.x, "on the phone the bundle takes a row and the next two share the one under it (two columns)");
        t.ok(Math.abs(tide.width - (crown.x + crown.width - sword.x)) < 1, `the bundle spans both columns (${tide.width} px)`);
      }
      const spill = ["tide", "ashbringer", "crown"].filter((k) => {
        const c = card(k)!;
        const n = at(els, `${OFFERS}/${k}/Body/Name`);
        return !n || n.x < c.x - 0.5 || n.x + n.width > c.x + c.width + 0.5;
      });
      t.equal(spill, [], `every featured offer's name is inside its card on the ${surface}`);
    }
  },
});
