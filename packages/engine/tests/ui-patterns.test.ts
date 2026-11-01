// The genre patterns (docs/ui.md §11, U6d) on the native host, driven as a
// player drives them: a drag, a tap, keys, a pad, Back. Each test names the
// wrong implementation it would catch.
import { expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import type * as Api from "@origingame/engine";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stderr: Uint8Array } };

const kit = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "templates", "ui-kit");

// At module level, not in a hook: the kit's string tables (Arabic for right to left), cooked incrementally.
{
  loadNativeHost();
  const cooked = Bun.spawnSync([ogBin(), "cook", kit]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
  if (!existsSync(join(kit, "cooked"))) throw new Error(`og cook ${kit} wrote no cooked/`);
}

type Element = { id: string; parent: string; path: string; kind: string; text: string; role: string; name: string; value: string; states: string[]; disabled: boolean; x: number; y: number; width: number; height: number };

let systems = 0;
/** An engine whose one system builds `build` at the top start of a 1280×720 desktop surface. */
export function run(build: () => void, viewport?: Partial<Api.Viewport>, rtl = false): Engine {
  const sys = defineSystem(`UpBuild${systems++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, build);
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, rtl ? { root: kit } : {});
  if (!r.ok) throw new Error(r.reason);
  if (rtl) {
    const loaded = og.i18n.load(r.value.world, "strings/ar.ogt");
    if (!loaded.ok) throw new Error(loaded.reason);
    og.i18n.setLocale(r.value.world, "ar-EG");
  }
  if (viewport) og.ui.previewViewport(r.value.world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, ...viewport } as never);
  r.value.advance(6);
  return r.value;
}

export function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}
export const byPath = (engine: Engine, end: string) => elements(engine).find((e) => e.path.endsWith(end));
const centre = (e: Element) => `${Math.round(e.x + e.width / 2)},${Math.round(e.y + e.height / 2)}`;

/** A drag from an element's centre to a point, in steps, then a release. */
function drag(engine: Engine, from: Element, to: string, mods = ""): void {
  const [tx, ty] = to.split(",").map(Number) as [number, number];
  const [fx, fy] = centre(from).split(",").map(Number) as [number, number];
  const steps = [`0:touch_move:${fx},${fy}`];
  if (mods) steps.push(`0.02:key_down:${mods}`);
  steps.push("0.05:touch_start");
  for (let k = 1; k <= 4; k++) steps.push(`${(0.05 + k * 0.05).toFixed(2)}:touch_move:${Math.round(fx + ((tx - fx) * k) / 4)},${Math.round(fy + ((ty - fy) * k) / 4)}`);
  steps.push("0.35:touch_end");
  if (mods) steps.push(`0.4:key_up:${mods}`);
  const r = engine.probe.commands(steps.join(";")) as unknown as { warnings: string[] };
  expect(r.warnings).toEqual([]);
  engine.advance(36);
}
function key(engine: Engine, code: string): void {
  engine.probe.commands(`0:key_down:${code};0.05:key_up:${code}`);
  engine.advance(8);
}

// ── inventory grid ──────────────────────────────────────────────────────

const BAG: Api.GridItem[] = [
  { key: "sword", name: "Ashbringer", x: 0, y: 0, w: 1, h: 3, rarity: "legendary", tag: "weapon" },
  { key: "potions", name: "Potion", x: 2, y: 0, count: 6, maxStack: 20, tag: "potion" },
  { key: "potions2", name: "Potion", x: 3, y: 0, count: 4, maxStack: 20, tag: "potion" },
  { key: "ring", name: "Ring", x: 2, y: 1, tag: "ring", locked: true },
  { key: "helm", name: "Helm", x: 4, y: 0, w: 2, h: 2, tag: "helmet", rarity: "rare" },
];

/** A grid of 8×4 cells of 50 px over BAG, the game applying each move as it should. */
function bag(rtl = false) {
  const items = BAG.map((i) => ({ ...i }));
  const moves: Api.InventoryMove[] = [];
  const results: Api.InventoryGridResult[] = [];
  let pick: Api.InventoryPick | null = null;
  const engine = run(() => {
    const r = og.ui.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items, label: "Backpack", ...(pick ? { pick } : {}) });
    pick = null;
    og.ui.button("Below", "Sort", {});
    results.push(r);
    if (r.move) {
      moves.push(r.move);
      const it = items.find((i) => i.key === r.move!.key)!;
      if (r.move.mergeInto) {
        const into = items.find((i) => i.key === r.move!.mergeInto)!;
        into.count = (into.count ?? 1) + r.move.count;
        if (r.move.split) it.count = (it.count ?? 1) - r.move.count;
        else items.splice(items.indexOf(it), 1);
      } else if (r.move.split) {
        it.count = (it.count ?? 1) - r.move.count;
        items.push({ ...it, key: `${it.key}-${moves.length}`, count: r.move.count, x: r.move.x, y: r.move.y });
      } else {
        Object.assign(it, { x: r.move.x, y: r.move.y, rotated: r.move.rotated });
      }
    }
  }, undefined, rtl);
  return { engine, items, moves, results, setPick: (p: Api.InventoryPick) => (pick = p) };
}

/** The point at the centre of cell (cx, cy) of the grid. */
function cell(engine: Engine, cx: number, cy: number): string {
  const g = byPath(engine, "Bag/Cells")!;
  return `${Math.round(g.x + cx * 50 + 25)},${Math.round(g.y + cy * 50 + 25)}`;
}

test("inventoryGrid: a drag lands a tall item by the cell it was grabbed at, and the grid says what it holds", () => {
  // Wrong: the footprint's top-left put under the pointer (the sword would land two rows up), or a move answered for a press that never moved.
  const b = bag();
  const grid = elements(b.engine).find((e) => e.role === "grid")!;
  expect(grid.name).toBe("Backpack, 10 / 32");
  expect(byPath(b.engine, "Cells/sword")?.name).toBe("Ashbringer, legendary");
  const sword = byPath(b.engine, "Cells/sword")!;
  // Grabbed by its bottom cell (row 2), dropped on cell (6, 3): it lands at (6, 1).
  drag(b.engine, { ...sword, y: sword.y + 100, height: 50 }, cell(b.engine, 6, 3));
  expect(b.moves.length).toBe(1);
  expect(b.moves[0]).toMatchObject({ key: "sword", from: "Bag", x: 6, y: 1, rotated: false, count: 1, split: false, mergeInto: null, swapWith: null });
  expect(byPath(b.engine, "Cells/sword")!.x).toBeCloseTo(byPath(b.engine, "Bag/Cells")!.x + 300, 0);
});

test("inventoryGrid: onto a stack of its kind it merges, onto another item it swaps, past the edge or onto a locked item it is refused", () => {
  // Wrong: every overlap refused (no stacking), or a locked item swapped away.
  const b = bag();
  drag(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(b.moves.at(-1)).toMatchObject({ key: "potions2", mergeInto: "potions", count: 4 });
  expect(b.items.find((i) => i.key === "potions")!.count).toBe(10);
  // The helm (2×2), grabbed by its top-left cell, dropped on the last cell: three of its cells past the edges, refused.
  const n = b.moves.length;
  const helm = byPath(b.engine, "Cells/helm")!;
  drag(b.engine, { ...helm, width: 50, height: 50 }, cell(b.engine, 7, 3));
  expect(b.moves.length).toBe(n);
  expect(b.results.some((r) => r.refused)).toBe(true);
  // The potions onto the locked ring: refused; onto the helm: a swap.
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 2, 1));
  expect(b.moves.length).toBe(n);
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 4, 0));
  expect(b.moves.at(-1)).toMatchObject({ key: "potions", swapWith: "helm", x: 4, y: 0 });
});

test("inventoryGrid: Shift and a drag carries half a stack; a locked item will not be picked up", () => {
  // Wrong: Shift ignored (the whole stack moves), or a locked item dragged off.
  const b = bag();
  drag(b.engine, byPath(b.engine, "Cells/potions")!, cell(b.engine, 6, 3), "ShiftLeft");
  expect(b.moves.at(-1)).toMatchObject({ key: "potions", split: true, count: 3, x: 6, y: 3 });
  expect(b.items.find((i) => i.key === "potions")!.count).toBe(3);
  const n = b.moves.length;
  drag(b.engine, byPath(b.engine, "Cells/ring")!, cell(b.engine, 7, 3));
  expect(b.moves.length).toBe(n);
  expect(b.results.some((r) => r.refused)).toBe(true);
});

test("inventoryGrid: the grid is one stop for keys and pads; its cursor walks the items, Enter picks up, R turns, Enter places turned", () => {
  // Wrong: a keyboard player unable to move items (only a pointer drags), a cursor that steps cell by cell through a
  // 2×2 item, the turn lost on the way, or the arrows kept at the grid's edge (the focus could never leave it).
  const b = bag();
  b.engine.advance(2);
  key(b.engine, "Tab");
  const grid = () => elements(b.engine).find((e) => e.role === "grid")!;
  expect(grid().states).toContain("focused");
  expect(grid().value).toBe("Ashbringer, legendary, 1, 1");
  // Right from the sword (column 0) over the empty column 1 to the potions, the second potions, then onto the 2×2 helm.
  for (const want of ["Empty, 2, 1", "Potion, ×6, 3, 1", "Potion, ×4, 4, 1", "Helm, rare, 5, 1"]) {
    key(b.engine, "ArrowRight");
    expect(grid().value).toBe(want);
  }
  expect(b.results.at(-1)!.hovered).toBe("helm");
  key(b.engine, "ArrowRight");
  expect(grid().value).toBe("Empty, 7, 1");
  key(b.engine, "ArrowLeft");
  expect(grid().value).toBe("Helm, rare, 5, 1");
  key(b.engine, "Enter");
  expect(b.results.at(-1)!.carried).toMatchObject({ key: "helm", source: "Bag", byFocus: true, w: 2, h: 2 });
  key(b.engine, "KeyR");
  expect(b.results.at(-1)!.carried).toMatchObject({ rotated: true });
  key(b.engine, "ArrowDown");
  key(b.engine, "ArrowDown");
  key(b.engine, "Enter");
  expect(b.moves.at(-1)).toMatchObject({ key: "helm", rotated: true, x: 4, y: 2 });
  // At the bottom edge Down is not the grid's: the focus leaves it.
  key(b.engine, "ArrowDown");
  expect(grid().states).not.toContain("focused");
  expect(byPath(b.engine, "Root/Below")!.states).toContain("focused");
});

test("inventoryGrid: Back puts a carried item back where it was", () => {
  const b = bag();
  b.setPick({ key: "helm", count: 0 });
  b.engine.advance(2);
  expect(b.results.at(-1)!.carried?.key).toBe("helm");
  key(b.engine, "Escape");
  expect(b.results.at(-1)!.carried).toBeNull();
  expect(b.moves).toEqual([]);
});

test("inventoryGrid: let go outside, the hand is reported released and a paperdoll beside it equips the item", () => {
  // Wrong: a carry that only lands inside its own grid (no equipping by drag), or a slot that takes any tag.
  const items = BAG.map((i) => ({ ...i }));
  let carry: Api.InventoryCarry | null = null;
  const equips: Api.PaperdollEquip[] = [];
  let refused = 0;
  const engine = run(() => {
    og.ui.row("Screen", { gap: 32 } as never, () => {
      const g = og.ui.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items });
      const d = og.ui.paperdoll("Doll", {
        slots: [{ key: "head", label: "Head", accept: ["helmet"], x: 0.5, y: 0.12 }, { key: "hand", label: "Main hand", accept: ["weapon"], x: 0.15, y: 0.5 }],
        title: "Aria", ...(carry ? { carry } : {}),
      }, () => {});
      carry = g.carried ?? (carry?.released ? null : carry);
      if (d.equip) equips.push(d.equip);
      if (d.refused) refused++;
    });
  });
  expect(byPath(engine, "Slots/head")!.name).toBe("Head, Empty");
  // The sword onto the head slot: refused; onto the main hand: equipped.
  drag(engine, byPath(engine, "Cells/sword")!, centre(byPath(engine, "Slots/head")!));
  expect(equips).toEqual([]);
  expect(refused).toBe(1);
  drag(engine, byPath(engine, "Cells/sword")!, centre(byPath(engine, "Slots/hand")!));
  expect(equips.length).toBe(1);
  expect(equips[0]).toMatchObject({ slot: "hand", key: "sword", from: "Bag", swapped: null });
});

test("inventoryGrid: right to left, columns count from the right edge", () => {
  // Wrong: a pointer's cell read from the left of a mirrored grid (the item lands in the mirror cell).
  const b = bag(true);
  const g = byPath(b.engine, "Bag/Cells")!;
  const sword = byPath(b.engine, "Cells/sword")!;
  // Column 0 is now at the right.
  expect(sword.x + sword.width).toBeCloseTo(g.x + g.width, 0);
  // Grabbed by its middle cell, dropped on column 5's second row: it lands at (5, 0).
  drag(b.engine, sword, `${Math.round(g.x + g.width - 5 * 50 - 25)},${Math.round(g.y + 75)}`);
  expect(b.moves.at(-1)).toMatchObject({ key: "sword", x: 5, y: 0 });
});
