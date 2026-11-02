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

/** Pressed on `from` and held over `to` (not let go): the drag's state that tick. */
function hold(engine: Engine, from: Element, to: string): void {
  const [tx, ty] = to.split(",").map(Number) as [number, number];
  const [fx, fy] = centre(from).split(",").map(Number) as [number, number];
  const steps = [`0:touch_move:${fx},${fy}`, "0.05:touch_start"];
  for (let k = 1; k <= 4; k++) steps.push(`${(0.05 + k * 0.05).toFixed(2)}:touch_move:${Math.round(fx + ((tx - fx) * k) / 4)},${Math.round(fy + ((ty - fy) * k) / 4)}`);
  engine.probe.commands(steps.join(";"));
  engine.advance(24);
}
const release = (engine: Engine) => {
  engine.probe.commands("0:touch_end");
  engine.advance(24);
};

test("inventoryGrid: what is carried says what a drop would do, each verdict its own mark: merge +n, swap, invalid", () => {
  // Wrong: the verdict drawn only on the footprint under the carried item (hidden), the same mark for merge and swap, or a
  // merge's count that is the whole carried stack rather than what the stack takes.
  const b = bag();
  const verdict = () => elements(b.engine).find((e) => e.path.endsWith("Ghost/Badge"))?.name;
  // Let go off the grid each time: the item goes home, nothing moves.
  const away = () => {
    b.engine.probe.commands("0:touch_move:1270,700");
    b.engine.advance(6);
    release(b.engine);
  };
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(verdict()).toBe("merge +4");
  away();
  b.items.find((i) => i.key === "potions")!.count = 18;
  b.engine.advance(2);
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 2, 0));
  expect(verdict()).toBe("merge +2");
  away();
  hold(b.engine, byPath(b.engine, "Cells/potions2")!, cell(b.engine, 4, 0));
  expect(verdict()).toBe("swap");
  b.engine.probe.commands(`0:touch_move:${cell(b.engine, 2, 1)}`);
  b.engine.advance(6);
  expect(verdict()).toBe("invalid");
  away();
  expect(b.moves).toEqual([]);
});

test("paperdoll: while an item is carried, the slot under it says no in danger, and slots elsewhere that take it only hint", () => {
  // Wrong: every slot that takes the item glowing the same whether or not it is under it, or a slot that refuses it
  // looking like one that does.
  const items = BAG.map((i) => ({ ...i }));
  let carry: Api.InventoryCarry | null = null;
  const engine = run(() => {
    og.ui.row("Screen", { gap: 32 } as never, () => {
      const g = og.ui.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items });
      og.ui.paperdoll("Doll", {
        slots: [{ key: "head", label: "Head", accept: ["helmet"], x: 0.5, y: 0.12 }, { key: "feet", label: "Feet", accept: ["feet"], x: 0.5, y: 0.85 }],
        ...(carry ? { carry } : {}),
      }, () => {});
      carry = g.carried ?? (carry?.released ? null : carry);
    });
  });
  const target = (k: string) => (byPath(engine, `Slots/${k}`) as unknown as { visualStates: Record<string, string> }).visualStates.target;
  hold(engine, byPath(engine, "Cells/helm")!, centre(byPath(engine, "Slots/feet")!));
  expect(target("feet")).toBe("rejects");
  expect(byPath(engine, "Slots/feet/Rejects")).toBeDefined();
  expect(target("head")).toBe("hint");
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "Slots/head")!)}`);
  engine.advance(6);
  expect(target("head")).toBe("accepts");
  expect(target("feet")).toBe("dim");
  release(engine);
});

test("paperdoll: a compared stat's change is one signed run, sign first, in both directions, with the real minus", () => {
  // Wrong: right to left the sign lands after the number ("48 +"), or a minus set as a hyphen.
  for (const rtl of [false, true]) {
    const e = run(() => {
      og.ui.paperdoll("Doll", { slots: [], compare: true, stats: [{ label: "Armor", value: "120", delta: 48 }, { label: "Speed", value: "12%", delta: -6, deltaText: "−6%" }] }, () => {});
    }, undefined, rtl);
    for (const k of [0, 1]) {
      const mark = byPath(e, `Stat#${k}/Delta/Sign`)!;
      const digits = byPath(e, `Stat#${k}/Delta/Text`)!;
      expect(mark.x < digits.x).toBe(true);
      expect(mark.text).toBe(k === 0 ? "+" : "\u2212");
      expect(digits.text.includes("\u2212") || digits.text.startsWith("+") || digits.text.startsWith("-")).toBe(false);
    }
    expect(byPath(e, "Stat#1/Delta/Text")!.text).toBe("6%");
  }
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

// ── card hand ───────────────────────────────────────────────────────────

const HAND: Api.HandCard[] = [
  { key: "strike", name: "Strike", cost: 1, typeLine: "Attack", text: "Deal [b]6[/b] damage.", needsTarget: true },
  { key: "defend", name: "Defend", cost: 1, typeLine: "Skill", text: "Gain [b]5[/b] Block." },
  { key: "bash", name: "Bash", cost: 2, typeLine: "Attack", text: "Deal 8 damage.", needsTarget: true, rarity: "rare" },
  { key: "anger", name: "Anger", cost: 0, typeLine: "Attack", text: "Deal 6 damage." },
  { key: "wound", name: "Wound", typeLine: "Status", text: "Unplayable.", playable: false },
];
// Two enemies on the table: the first can be targeted, the second (behind a taunt) cannot.
const TARGETS: Api.CardTarget[] = [
  { key: "cultist", name: "Cultist", rect: { x: 760, y: 80, width: 160, height: 180 } },
  { key: "louse", name: "Louse", rect: { x: 980, y: 80, width: 160, height: 180 }, valid: false },
];

/** A hand along the bottom of a 1280×720 desktop (or right to left), the game removing each card played. */
export function hand(rtl = false, width = 1200, phone = false) {
  const cards = (phone ? [...HAND, { key: "clash", name: "Clash", cost: 0, typeLine: "Attack", text: "Deal 14 damage." }] : HAND).map((c) => ({ ...c }));
  const played: Api.CardPlay[] = [];
  const results: Api.CardHandResult[] = [];
  const sys = defineSystem(`UpHand${systems++}`, {}, () => {
    og.ui.anchor("Table", { anchor: "bottom", width: { kind: "fixed", px: width }, direction: "column" } as never, () => {
      const r = og.ui.cardHand("Hand", { cards, targets: TARGETS, cardWidth: 150 });
      results.push(r);
      if (r.played) {
        played.push(r.played);
        cards.splice(cards.findIndex((c) => c.key === r.played!.key), 1);
      }
    });
  });
  const made = createEngine(og.game({ systems: [sys] }) as unknown as GameToken, rtl ? { root: kit } : {});
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  if (rtl) {
    og.i18n.load(engine.world, "strings/ar.ogt");
    og.i18n.setLocale(engine.world, "ar-EG");
  }
  og.ui.previewViewport(engine.world, phone ? { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never : { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  engine.advance(6);
  return { engine, played, results, cards };
}
const cardEl = (engine: Engine, k: string) => byPath(engine, `Fan/${k}`)!;

test("cardHand: a card dragged above the play line plays on release; let go below it, it goes back", () => {
  // Wrong: a release anywhere plays (a misdrag spends the card), or a release never does.
  const h = hand();
  drag(h.engine, cardEl(h.engine, "defend"), "640,640");
  expect(h.played).toEqual([]);
  expect(cardEl(h.engine, "defend")).toBeDefined();
  drag(h.engine, cardEl(h.engine, "defend"), "640,200");
  expect(h.played).toEqual([{ key: "defend", target: null }]);
  expect(byPath(h.engine, "Fan/defend") === undefined).toBe(true);
});

test("cardHand: a card that needs a target is aimed with an arc and plays only on a valid target", () => {
  // Wrong: a targeted card played on release above the line with no target, or on the taunted enemy.
  const h = hand();
  drag(h.engine, cardEl(h.engine, "bash"), "640,200");
  expect(h.played).toEqual([]);
  drag(h.engine, cardEl(h.engine, "bash"), "1060,170");
  expect(h.played).toEqual([]);
  drag(h.engine, cardEl(h.engine, "bash"), "840,170");
  expect(h.played).toEqual([{ key: "bash", target: "cultist" }]);
});

test("cardHand: while aiming, the arc and the result say which target is under the tip", () => {
  const h = hand();
  const s = cardEl(h.engine, "strike");
  const [fx, fy] = [Math.round(s.x + s.width / 2), Math.round(s.y + s.height / 2)];
  h.engine.probe.commands(`0:touch_move:${fx},${fy};0.05:touch_start;0.1:touch_move:${fx},400;0.2:touch_move:1060,170`);
  h.engine.advance(20);
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "strike", target: "louse" });
  expect(byPath(h.engine, "Hand/Fan/Arc")!.name).toBe("invalid: Louse");
  h.engine.probe.commands("0:touch_move:840,170");
  h.engine.advance(6);
  expect(byPath(h.engine, "Hand/Fan/Arc")!.name).toBe("valid: Cultist");
  // The aimed card stays over the hand, laid out larger than at rest (150 wide; its bounds at rest are turned).
  expect(cardEl(h.engine, "strike").width).toBeGreaterThan(150 * 1.25);
  h.engine.probe.commands("0:touch_end");
  h.engine.advance(6);
  expect(h.played).toEqual([{ key: "strike", target: "cultist" }]);
});

test("cardHand: an unplayable card shakes and is refused; it is said disabled", () => {
  const h = hand();
  expect(cardEl(h.engine, "wound").states).toContain("disabled");
  drag(h.engine, cardEl(h.engine, "wound"), "640,200");
  expect(h.played).toEqual([]);
  expect(h.results.some((r) => r.refused === "wound")).toBe(true);
});

test("cardHand: hovering lifts a card clear of the fan; its neighbours part", () => {
  // Wrong: the lifted card scaled in place (its text drawn small) or covered by its neighbour.
  const h = hand();
  const before = elements(h.engine);
  const at = (es: Element[], k: string) => es.find((e) => e.path.endsWith(`Fan/${k}`))!;
  const b = at(before, "bash");
  h.engine.probe.commands(`0:touch_move:${Math.round(b.x + b.width / 2)},${Math.round(b.y + b.height / 2)}`);
  h.engine.advance(40);
  const after = elements(h.engine);
  expect(at(after, "bash").width).toBeGreaterThan(b.width * 1.2);
  expect(at(after, "bash").y).toBeLessThan(b.y - 40);
  expect(at(after, "bash").states).toContain("selected");
  expect(at(after, "defend").x).toBeLessThan(at(before, "defend").x);
  expect(at(after, "anger").x).toBeGreaterThan(at(before, "anger").x);
  expect(h.results.at(-1)!.hovered).toBe("bash");
});

test("cardHand: a click picks a card and a second click plays it", () => {
  const h = hand();
  const tapAt = (e: Element) => h.engine.probe.commands(`0:touch_move:${Math.round(e.x + e.width / 2)},${Math.round(e.y + e.height / 2)};0.05:touch_start;0.1:touch_end`);
  tapAt(cardEl(h.engine, "anger"));
  h.engine.advance(30);
  expect(h.played).toEqual([]);
  tapAt(cardEl(h.engine, "anger"));
  h.engine.advance(12);
  expect(h.played).toEqual([{ key: "anger", target: null }]);
});

test("cardHand: keys walk the hand, Enter aims a targeted card, the arrows step the targets, Enter plays", () => {
  // Wrong: keys play a targeted card with no target, or step onto targets the card cannot take without saying so.
  const h = hand();
  key(h.engine, "Tab");
  expect(byPath(h.engine, "Hand/Fan")!.states).toContain("focused");
  expect(h.results.at(-1)!.hovered).toBe("strike");
  key(h.engine, "ArrowRight");
  key(h.engine, "ArrowRight");
  expect(h.results.at(-1)!.hovered).toBe("bash");
  key(h.engine, "Enter");
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "bash", target: "cultist" });
  key(h.engine, "ArrowRight");
  expect(h.results.at(-1)!.aiming).toMatchObject({ key: "bash", target: "louse" });
  key(h.engine, "Enter");
  expect(h.played).toEqual([]);
  key(h.engine, "ArrowRight");
  key(h.engine, "Enter");
  expect(h.played).toEqual([{ key: "bash", target: "cultist" }]);
  // The cursor stays at its place (now Anger); back to Strike, aimed, then Back stops aiming without playing.
  expect(h.results.at(-1)!.hovered).toBe("anger");
  key(h.engine, "ArrowLeft");
  key(h.engine, "ArrowLeft");
  key(h.engine, "Enter");
  expect(h.results.at(-1)!.aiming?.key).toBe("strike");
  key(h.engine, "Escape");
  expect(h.results.at(-1)!.aiming).toBeNull();
  expect(h.played.length).toBe(1);
});

test("cardHand: right to left, the hand fans from the right", () => {
  const h = hand(true);
  expect(cardEl(h.engine, "strike").x).toBeGreaterThan(cardEl(h.engine, "wound").x);
});

// ── reward choice ───────────────────────────────────────────────────────

const REWARDS: Api.RewardOption[] = [
  { key: "strike", title: "Thunder Strike", kind: "Attack", text: "Deal [b]12[/b] damage.", rarity: "rare" },
  { key: "vigor", title: "Vigor", kind: "Boon", text: "+20 % Max Health.", rarity: "epic", tag: "New" },
  { key: "crown", title: "Crown", kind: "Relic", text: "Gain a crown.", rarity: "legendary", locked: true, lockedHint: "Reach floor 3" },
];

/** A reward choice the game answers as it should: it keeps the selection, rerolls and banishes. */
function rewards(extra: Partial<Api.RewardChoiceDesc> = {}) {
  let options = REWARDS.map((o) => ({ ...o }));
  let selected: string[] = [];
  let rerolls = extra.rerolls ?? -1;
  let banishes = extra.banishes ?? -1;
  const results: Api.RewardChoiceResult[] = [];
  const engine = run(() => {
    const r = og.ui.rewardChoice("Rew", { ...extra, options, selected, rerolls, banishes, cardWidth: 240 });
    results.push(r);
    selected = [...r.selected];
    if (r.rerolled) {
      rerolls -= 1;
      options = options.map((o) => ({ ...o, key: `${o.key}2` }));
    }
    if (r.banished) {
      banishes -= 1;
      options = options.filter((o) => o.key !== r.banished);
    }
  });
  const press = (k: string) => {
    const e = byPath(engine, `Rew/Options/${k}`) ?? byPath(engine, `Rew/Actions/${k}`)!;
    engine.probe.commands(`0:touch_move:${centre(e)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  const picked = () => results.flatMap((r) => r.picked);
  return { engine, results, press, picked, options: () => options };
}

test("rewardChoice: with one pick a press takes the reward at once; a locked one shakes and is refused", () => {
  // Wrong: a locked reward taken, or a single pick waiting for a confirm it never shows.
  const r = rewards();
  r.press("crown");
  expect(r.picked()).toEqual([]);
  expect(r.results.some((x) => x.refused === "crown")).toBe(true);
  expect(byPath(r.engine, "Rew/Options/crown")!.states).toContain("disabled");
  r.press("vigor");
  expect(r.picked()).toEqual(["vigor"]);
  expect(byPath(r.engine, "Rew/Actions/Confirm") === undefined).toBe(true);
});

test("rewardChoice: with two picks presses select, a third press does not overflow, Confirm takes them in offer order", () => {
  // Wrong: a press takes at once, the selection grows past picks, or the order follows the presses.
  const r = rewards({ picks: 2 });
  r.press("vigor");
  r.press("strike");
  expect(r.picked()).toEqual([]);
  expect(byPath(r.engine, "Rew/Options/vigor")!.states).toContain("checked");
  r.press("crown");
  expect(r.results.at(-1)!.selected).toEqual(["strike", "vigor"]);
  r.press("Confirm");
  expect(r.picked()).toEqual(["strike", "vigor"]);
});

test("rewardChoice: Banish arms, every card shows its cross, the next press banishes rather than takes; Back disarms", () => {
  // Wrong: an armed press takes the reward, the arming stays after a banish, or Back leaves it armed.
  const r = rewards({ banishes: 2 });
  r.press("Banish");
  expect(byPath(r.engine, "Rew/Options/strike/Banish")).toBeDefined();
  r.press("strike");
  expect(r.picked()).toEqual([]);
  expect(r.options().map((o) => o.key)).toEqual(["vigor", "crown"]);
  expect(byPath(r.engine, "Rew/Options/vigor/Banish") === undefined).toBe(true);
  r.press("Banish");
  key(r.engine, "Escape");
  expect(byPath(r.engine, "Rew/Options/vigor/Banish") === undefined).toBe(true);
  r.press("vigor");
  expect(r.picked()).toEqual(["vigor"]);
});

test("rewardChoice: Reroll offers new rewards until none are left, then is disabled", () => {
  const r = rewards({ rerolls: 1 });
  r.press("Reroll");
  expect(r.options()[0]!.key).toBe("strike2");
  expect(byPath(r.engine, "Rew/Actions/Reroll")!.disabled).toBe(true);
  r.press("Reroll");
  expect(r.options()[0]!.key).toBe("strike2");
});

test("rewardChoice: keys walk the rewards and Enter takes the focused one", () => {
  // Wrong: the locked reward taken by Enter, or the keys skipping the cards.
  const r = rewards();
  key(r.engine, "Tab");
  expect(r.results.at(-1)!.hovered).toBe("strike");
  key(r.engine, "Tab");
  key(r.engine, "Enter");
  expect(r.picked()).toEqual(["vigor"]);
});

// ── gacha banner and pull ───────────────────────────────────────────────

function banner(balance: number) {
  let current = "event";
  const results: Api.GachaBannerResult[] = [];
  const engine = run(() => {
    const r = og.ui.gachaBanner("Banner", {
      banners: [{ key: "event", name: "Event" }, { key: "weapon", name: "Weapon" }, { key: "std", name: "Standard" }],
      current, title: "Moonlit Tide", balance, single: 160, multi: 1600, pityMax: 90, pity: 72,
      featured: [{ name: "Aria", stars: 5, rarity: "legendary" }],
    }, () => {});
    results.push(r);
    if (r.banner) current = r.banner;
  }, { width: 1280, height: 720 });
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  return { engine, results, press, current: () => current };
}

test("gachaBanner: a pull the purse pays is asked for; one it cannot is reported short and said so", () => {
  // Wrong: a pull asked for that the balance cannot pay (the game would spend money it has not), or short shown only by colour.
  const b = banner(1000);
  b.press("Buttons/Single");
  expect(b.results.some((r) => r.pull === 1)).toBe(true);
  b.press("Buttons/Multi");
  expect(b.results.some((r) => r.pull === 10)).toBe(false);
  expect(b.results.some((r) => r.short === 10)).toBe(true);
  expect(byPath(b.engine, "Buttons/Multi")!.name).toContain("not enough");
  expect(byPath(b.engine, "Pulls/Status/Pity")!.name).toBe("Guaranteed in 18");
});

test("gachaBanner: tabs and the bumpers switch banners, wrapping at the ends", () => {
  const b = banner(5000);
  b.press("Tabs/std");
  expect(b.current()).toBe("std");
  key(b.engine, "KeyE");
  expect(b.current()).toBe("event");
  key(b.engine, "KeyQ");
  expect(b.current()).toBe("std");
  expect(byPath(b.engine, "Tabs/std")!.states).toContain("selected");
});

const PULL: Api.PullItem[] = [
  { key: "p1", name: "Iron Sword", stars: 3, rarity: "common" },
  { key: "p2", name: "Aria", stars: 5, rarity: "legendary", isNew: true },
  { key: "p3", name: "Bow", stars: 4, rarity: "epic", extra: "+2 Starglitter" },
];

function pull() {
  let token = 0;
  const results: Api.PullSequenceResult[] = [];
  const engine = run(() => {
    const r = og.ui.pullSequence("Pull", { token, items: PULL, againPrice: 1600 });
    results.push(r);
    if (r.done) token = 0;
  });
  const tap = () => {
    engine.probe.commands("0:touch_move:640,360;0.05:touch_start;0.1:touch_end");
    engine.advance(14);
  };
  return { engine, results, tap, start: (t: number) => { token = t; engine.advance(2); }, last: () => results.at(-1)! };
}

test("pullSequence: a new token charges, then reveals each result in order, then the summary sorts the highest tier first", () => {
  // Wrong: the summary in draw order, a reveal skipped, or a reveal reported twice.
  const p = pull();
  expect(p.last().phase).toBe("idle");
  p.start(1);
  expect(p.last().phase).toBe("charge");
  p.engine.advance(100);
  expect(p.last().phase).toBe("reveal");
  p.tap();
  p.tap();
  expect(p.last().index).toBe(2);
  expect(p.results.flatMap((r) => (r.revealed ? [r.revealed] : []))).toEqual(["p1", "p2", "p3"]);
  p.tap();
  expect(p.last().phase).toBe("summary");
  const tiles = elements(p.engine).filter((e) => e.role === "listitem").sort((a, b) => a.y - b.y || a.x - b.x).map((e) => e.path.split("/").at(-1));
  expect(tiles).toEqual(["p2", "p3", "p1"]);
});

test("pullSequence: Skip goes straight to the summary; Done closes it; a new token starts again", () => {
  const p = pull();
  p.start(1);
  p.engine.advance(4);
  p.engine.probe.commands(`0:touch_move:${centre(byPath(p.engine, "Bar/Skip")!)};0.05:touch_start;0.1:touch_end`);
  p.engine.advance(14);
  expect(p.last().phase).toBe("summary");
  key(p.engine, "Escape");
  expect(p.results.some((r) => r.done)).toBe(true);
  expect(p.last().phase).toBe("idle");
  p.start(2);
  expect(p.last().phase).toBe("charge");
});

test("cardHand: let go on a target it cannot take, the card is refused there (it shakes) and the result names that target", () => {
  // Wrong: a release on the taunted enemy silently cancels (no shake, nothing to say why), or plays.
  const h = hand();
  const louse = TARGETS[1]!.rect!;
  drag(h.engine, cardEl(h.engine, "bash"), `${louse.x + 80},${louse.y + 90}`);
  expect(h.played).toEqual([]);
  expect(h.results.some((r) => r.refused === "bash" && r.aiming?.target === "louse")).toBe(true);
});

test("cardHand: on a phone, a refused drag leaves no card standing; the cursor's card lifts only for keys", () => {
  // Wrong: the press gave the fan the focus, and the focused fan lifts its cursor's card (the refused Wound) over the screen.
  const h = hand(false, 382, true);
  h.engine.advance(60);
  drag(h.engine, cardEl(h.engine, "wound"), "195,300");
  expect(h.results.some((r) => r.refused === "wound")).toBe(true);
  const standing = elements(h.engine).filter((e) => /Fan\/[a-z]+$/.test(e.path) && e.states.includes("selected"));
  expect(standing.map((e) => e.id)).toEqual([]);
});
