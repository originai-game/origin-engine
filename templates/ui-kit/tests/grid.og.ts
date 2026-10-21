// The grids lay out the columns they are asked for (kestrel-stair FINDINGS
// #15): the inventory's six slots a row (not the five that fit at the slot
// size), and the level select's four named cards a row, sharing the
// screen's width, each name read whole or wrapped, never clipped.
import { playtest } from "@origingame/engine/playtest";
import { ui, type UiElement } from "./lib.ts";

/** The rows of a grid's cells, top to bottom: each the cells' indices left to right. */
function rows(elements: UiElement[], grid: string): number[][] {
  const cells = elements
    .filter((e) => e.parent.endsWith(grid) && e.id.startsWith("Cell#"))
    .map((e) => ({ i: Number(e.id.slice(5)), x: e.x, y: e.y }))
    .sort((a, b) => a.y - b.y || a.x - b.x);
  const out: Array<{ y: number; cells: number[] }> = [];
  for (const c of cells) {
    const row = out.find((r) => Math.abs(r.y - c.y) < 1);
    if (row) row.cells.push(c.i);
    else out.push({ y: c.y, cells: [c.i] });
  }
  return out.map((r) => r.cells);
}

export default playtest({
  name: "the inventory lays out six slots a row and the level select four named cards, as asked",
  commands: [
    "0.2:show:inventory",
    "1.0:dump_ui:inventory.json",
    "1.1:show:levels",
    "1.9:dump_ui:levels.json",
    "2.0:viewport:phone",
    "2.6:dump_ui:levels-phone.json",
    "2.7:show:inventory",
    "3.4:dump_ui:inventory-phone.json",
    "3.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    for (const [file, surface] of [["inventory.json", "desktop"], ["inventory-phone.json", "phone"]] as const) {
      const r = rows(ui(run, file), "Slots");
      t.equal(r.slice(0, 3), [[0, 1, 2, 3, 4, 5], [6, 7, 8, 9, 10, 11], [12, 13, 14, 15, 16, 17]], `the inventory's slots run six a row on the ${surface}`);
    }
    for (const [file, surface] of [["levels.json", "desktop"], ["levels-phone.json", "phone"]] as const) {
      const elements = ui(run, file);
      const r = rows(elements, "Levels");
      t.equal(r.slice(0, 2), [[0, 1, 2, 3], [4, 5, 6, 7]], `the level cards run four a row on the ${surface}`);
      const cards = new Map(elements.filter((e) => e.parent.endsWith("Levels") && e.id.startsWith("Cell#")).map((e) => [e.id, e]));
      const names = elements.filter((e) => e.id === "Name" && cards.has(e.parent));
      const spill = names.filter((e) => {
        const card = cards.get(e.parent)!;
        return e.x < card.x - 0.5 || e.x + e.width > card.x + card.width + 0.5;
      });
      t.ok(names.length >= 9 && spill.length === 0, `every level's name fits its card, whole or wrapped, on the ${surface} (${names.length} names; spilling: ${spill.map((e) => e.text).join(", ") || "none"})`);
      const first = names.find((e) => e.text === "The Lower Court of the Abbey");
      t.ok(first !== undefined && first.height > 30, `the long name wraps to a second line on the ${surface} (${first?.height.toFixed(1)} px tall)`);
    }
  },
});
