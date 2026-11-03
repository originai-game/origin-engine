// The genre patterns (docs/ui.md §11, U6d) on the native host, driven as a
// player drives them: a drag, a tap, keys, a pad, Back. Each test names the
// wrong implementation it would catch.
import { expect, setDefaultTimeout, test } from "bun:test";
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

// A test here drives hundreds of ticks of a whole screen through the native host (a drag, a reveal, a scroll): 4 to 6 s
// each on the r41 host, past bun's 5 s default, so which ones timed out varied from run to run. Each is given 20 s, as
// the slowest already was; a test still fails on what it asserts, never on the machine's load.
setDefaultTimeout(20000);

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

test("long press: an item's menu and a card's inspect are og-ui's recogniser, timed in ms whatever the tick rate", () => {
  // Wrong: a hand-counted 30 ticks (a second at 30 Hz, not og-ui's 500 ms), so a 600 ms hold opens nothing.
  const menus: string[] = [];
  const inspects: string[] = [];
  const sys = defineSystem(`UpLong${systems++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, () => {
      const r = og.ui.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items: BAG, label: "Backpack" });
      if (r.menu) menus.push(r.menu);
    });
    og.ui.anchor("Table", { anchor: "bottom", width: { kind: "fixed", px: 1200 }, direction: "column" } as never, () => {
      const r = og.ui.cardHand("Hand", { cards: HAND, targets: TARGETS, cardWidth: 150 });
      if (r.inspect) inspects.push(r.inspect);
    });
  });
  const made = createEngine(og.game({ tick: 30, systems: [sys] }) as unknown as GameToken);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  engine.advance(4);
  const holdOn = (el: Element): void => {
    engine.probe.commands(`0:touch_move:${centre(el)};0.05:touch_start;0.7:touch_end`);
    engine.advance(18);
  };
  holdOn(byPath(engine, "Cells/helm")!);
  expect(menus).toEqual(["helm"]);
  engine.advance(8);
  holdOn(cardEl(engine, "defend"));
  expect(inspects).toEqual(["defend"]);
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

test("paperdoll: an empty slot shows its kind's silhouette, never a letter; a slot of no known kind is a plain socket", () => {
  // Wrong: an empty slot draws the first letter of its name (N, H, R read as debug output), or an off hand that also takes
  // weapons is drawn as a sword.
  const e = run(() => {
    og.ui.paperdoll("Doll", { slots: [{ key: "neck", label: "Neck", accept: ["amulet"] }, { key: "off", label: "Off hand", accept: ["weapon", "shield"] }, { key: "x", label: "Xyz" }] }, () => {});
  });
  expect(byPath(e, "neck/Glyph")).toBeDefined();
  expect(byPath(e, "off/Glyph")).toBeDefined();
  expect(byPath(e, "x/Glyph") === undefined).toBe(true);
  expect(elements(e).some((el) => /^(N|O|X)$/.test(el.text))).toBe(false);
});

test("heroButton: a press is a click; disabled it is inert; a desktop shows its keycap and says it", () => {
  // Wrong: a disabled call to action still answers clicked, or the keycap claims a key the name never says.
  let clicked = 0;
  let off = 0;
  const e = run(() => {
    if (og.ui.heroButton("Go", "End turn", { tone: "primary", key: "E" }).clicked) clicked += 1;
    if (og.ui.heroButton("Wait", "Waiting", { disabled: true }).clicked) off += 1;
  }, {});
  const go = byPath(e, "Root/Go")!;
  expect(go.name).toBe("End turn, E");
  expect(byPath(e, "Go/Key/Text")!.text).toBe("E");
  for (const b of [go, byPath(e, "Root/Wait")!]) {
    e.probe.commands(`0:touch_move:${centre(b)};0.05:touch_start;0.1:touch_end`);
    e.advance(10);
  }
  expect(clicked).toBe(1);
  expect(off).toBe(0);
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
  { key: "vigor", title: "Vigor", kind: "Blessing", text: "+20 % Max Health.", rarity: "epic", tag: "New" },
  { key: "crown", title: "Crown", kind: "Relic", text: "Gain a crown.", rarity: "legendary", locked: true, lockedHint: "Reach floor 3" },
];

/** A reward choice the game answers as it should: it keeps the selection, rerolls and banishes. */
function rewards(extra: Partial<Api.RewardChoiceDesc> = {}, after?: () => void) {
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
    after?.();
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
  // Its place as the pointer finds it (og.ui.bounds: where it was drawn, its transform in), read in the system.
  const xs: number[] = [];
  const r = rewards({}, () => xs.push(og.ui.bounds("Rew/Options/crown")?.x ?? -1));
  r.engine.advance(90);
  const rest = xs.at(-1)!;
  xs.length = 0;
  r.press("crown");
  r.engine.advance(30);
  // Pressed, it stood off its place for a few ticks (the shake), then came back to it.
  expect(xs.some((x) => Math.abs(x - rest) >= 1)).toBe(true);
  expect(Math.abs(xs.at(-1)! - rest)).toBeLessThan(0.5);
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

test("gachaBanner: the time left is in the game's words, days and hours on the timer, minutes on a tab", () => {
  // Wrong: the English units (3d 5h) under a locale's words, or a tab's minutes in the timer's form.
  const engine = run(() => {
    og.ui.gachaBanner("Banner", {
      banners: [{ key: "event", name: "Event", endsIn: 3 * 86400 + 5 * 3600 }, { key: "weapon", name: "Weapon", endsIn: 50 * 60 }],
      current: "event", title: "Moonlit Tide", endsIn: 3 * 86400 + 5 * 3600 + 30,
      daysLabel: "{d}日{h}時間", hoursLabel: "{h}時間{m}分", minutesLabel: "{m}分",
    }, () => {});
  }, { width: 1280, height: 720 });
  const texts = elements(engine).map((e) => e.text);
  expect(texts.filter((t) => t === "3日5時間").length).toBe(2);
  expect(texts).toContain("50分");
  expect(texts.some((t) => /\d[dhm]\b/.test(t))).toBe(false);
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
  { key: "p3", name: "Bow", stars: 4, rarity: "epic", extra: "+2 Star Shards" },
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

test("pullSequence: a unit seen for the first time wears the game's word for new", () => {
  let token = 0;
  const engine = run(() => {
    og.ui.pullSequence("Pull", { token, items: PULL, againPrice: 1600, newLabel: "新" });
  });
  token = 1;
  engine.advance(2);
  key(engine, "Escape");
  const texts = elements(engine).map((e) => e.text);
  expect(texts).toContain("新");
  expect(texts).not.toContain("NEW");
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

// ── saga map ────────────────────────────────────────────────────────────

const LEVELS: Api.SagaNode[] = Array.from({ length: 30 }, (_, i) => ({
  key: `l${i + 1}`, label: `${i + 1}`, stars: i < 20 ? 3 - (i % 3) : 0, locked: i > 20, boss: i === 24,
}));

function saga(viewport?: Partial<Api.Viewport>) {
  const results: Api.SagaMapResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: viewport ? 360 : 1100 }, height: { kind: "fixed", px: viewport ? 640 : 520 } } as never, () => {
      results.push(og.ui.sagaMap("Saga", { nodes: LEVELS, current: "l21" }));
    });
  }, viewport);
  engine.advance(6);
  return { engine, results };
}

test("sagaMap: it opens with the current level in view, far down the path", () => {
  // Wrong: the map opening at its first level, with the player's level off screen.
  for (const vp of [undefined, { width: 390, height: 844, breakpoint: "phonePortrait", coarsePointer: true }] as const) {
    const s = saga(vp as Partial<Api.Viewport> | undefined);
    const view = byPath(s.engine, "Saga/View")!;
    const cur = byPath(s.engine, "Path/l21")!;
    expect(cur.x).toBeGreaterThanOrEqual(view.x - 1);
    expect(cur.x + cur.width).toBeLessThanOrEqual(view.x + view.width + 1);
    expect(cur.y).toBeGreaterThanOrEqual(view.y - 1);
    expect(cur.y + cur.height).toBeLessThanOrEqual(view.y + view.height + 1);
    expect(cur.name).toContain("You are here");
    // The first level is far behind it, scrolled out of the view (the probe lists only what the view shows).
    expect(byPath(s.engine, "Path/l1") === undefined).toBeTrue();
  }
});

test("sagaMap: an open level is picked; a locked one is refused and said locked; stars are said", () => {
  const s = saga();
  const tapOn = (end: string) => {
    s.engine.probe.commands(`0:touch_move:${centre(byPath(s.engine, end)!)};0.05:touch_start;0.1:touch_end`);
    s.engine.advance(14);
  };
  tapOn("Path/l20");
  expect(s.results.some((r) => r.picked === "l20")).toBeTrue();
  tapOn("Path/l22");
  expect(s.results.some((r) => r.picked === "l22")).toBeFalse();
  expect(s.results.some((r) => r.refused === "l22")).toBeTrue();
  expect(byPath(s.engine, "Path/l22")!.name).toContain("Locked");
  expect(byPath(s.engine, "Path/l20")!.name).toContain("2 of 3 stars");
});

// ── puzzle goals ────────────────────────────────────────────────────────

test("starBar: a star is earned at its threshold and reported crossed once, not every tick after", () => {
  // Wrong: crossed reported on every tick while over the line (the game's fanfare looping), or a star lit below it.
  let score = 900;
  const results: Api.StarBarResult[] = [];
  const engine = run(() => {
    results.push(og.ui.starBar("Stars", { value: score, thresholds: [1000, 2000, 3000], label: "Score" }));
  });
  expect(results.at(-1)!.earned).toBe(0);
  score = 2100;
  engine.advance(10);
  expect(results.at(-1)!.earned).toBe(2);
  expect(results.reduce((n, r) => n + r.crossed, 0)).toBe(2);
  const meter = elements(engine).find((e) => e.role === "meter" && e.path.endsWith("Stars"))!;
  expect(meter.name).toBe("Score");
  expect(meter.value).toBe("2,100");
});

test("goalChips: a goal met says done; its count never shows past the need", () => {
  const engine = run(() => {
    og.ui.goalChips("Goals", { goals: [{ key: "blue", label: "Blue Gems", have: 14, need: 20 }, { key: "ice", label: "Ice", have: 7, need: 6 }] });
  });
  expect(byPath(engine, "Goals/blue")!.name).toBe("Blue Gems, 14 of 20");
  expect(byPath(engine, "Goals/ice")!.name).toBe("Ice, 6 of 6, Done");
  expect(elements(engine).some((e) => e.text === "7 / 6")).toBeFalse();
});

// ── idle ────────────────────────────────────────────────────────────────

function generator(affordable: () => boolean) {
  const results: Api.IdleGeneratorResult[] = [];
  const engine = run(() => {
    results.push(og.ui.idleGenerator("Solar", { name: "Solar Array", level: 125, ownedText: "348", rateText: "+1.24 M / sec", cycle: 0.4, cycleSeconds: 2, costText: "12.4 M", affordable: affordable() }));
  });
  return { engine, results, bought: () => results.filter((r) => r.bought).length };
}

test("idleGenerator: a press buys when the purse pays; short, it is refused and said so", () => {
  let can = true;
  const g = generator(() => can);
  const buy = centre(byPath(g.engine, "Solar/Buy")!);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start;0.1:touch_end`);
  g.engine.advance(14);
  expect(g.bought()).toBe(1);
  can = false;
  g.engine.advance(2);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start;0.1:touch_end`);
  g.engine.advance(14);
  expect(g.bought()).toBe(1);
  expect(g.results.some((r) => r.short)).toBeTrue();
  expect(byPath(g.engine, "Solar/Buy")!.name).toContain("short");
});

test("idleGenerator: Buy held repeats after a beat while the purse pays", () => {
  // Wrong: a hold that buys once, or one that buys every tick from the first.
  const g = generator(() => true);
  const buy = centre(byPath(g.engine, "Solar/Buy")!);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start`);
  g.engine.advance(12);
  expect(g.bought()).toBe(0);
  g.engine.advance(60);
  const held = g.bought();
  expect(held).toBeGreaterThanOrEqual(4);
  expect(held).toBeLessThanOrEqual(12);
});

test("prestigePanel: a tap does not reset the run; a full hold does; unavailable it cannot be held", () => {
  // Wrong: a reset on a press (a slip throws a run away), or one taken while nothing would be gained.
  let available = false;
  const results: Api.PrestigeResult[] = [];
  const engine = run(() => {
    results.push(og.ui.prestigePanel("Prestige", { lead: "Ascend for", rewardText: "+128 Shards", points: ["Resets generators"], available, holdMs: 600 }));
  }, { width: 1280, height: 720 });
  const at = () => centre(byPath(engine, "Action/Button")!);
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;1.2:touch_end`);
  engine.advance(80);
  expect(results.some((r) => r.confirmed)).toBeFalse();
  available = true;
  engine.advance(2);
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;0.15:touch_end`);
  engine.advance(14);
  expect(results.some((r) => r.confirmed)).toBeFalse();
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;1.0:touch_end`);
  engine.advance(70);
  expect(results.filter((r) => r.confirmed).length).toBe(1);
});

// ── store ───────────────────────────────────────────────────────────────

function store(gems: number) {
  const results: Api.StoreFrontResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1180 } } as never, () => {
      results.push(og.ui.storeFront("Store", {
        wallet: [{ key: "gems", name: "Gems", amount: gems, topUp: true }],
        sections: [{
          key: "daily",
          title: "Daily",
          endsIn: 3600 * 7,
          offers: [
            { key: "sword", name: "Sword", kind: "Weapon", price: 1200, original: 2000, currency: "gems", rarity: "epic", affordable: gems >= 1200 },
            { key: "potion", name: "Potion", price: 100, currency: "gems", limit: 3, bought: 1, affordable: gems >= 100 },
            { key: "key", name: "Key", price: 100, currency: "gems", limit: 1, bought: 1 },
            { key: "hat", name: "Hat", price: 900, currency: "gems", owned: true },
          ],
        }],
      }));
    });
  });
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  return { engine, results, press };
}

test("storeFront: an offer the purse pays is chosen; short or sold out it is refused and says why; owned it is viewed", () => {
  // Wrong: a short offer chosen (the game asked to charge what the purse cannot pay), a sold-out one sold again, or
  // an owned one bought twice.
  const s = store(500);
  s.press("Offers/potion");
  expect(s.results.some((r) => r.chosen === "potion")).toBeTrue();
  s.press("Offers/sword");
  expect(s.results.some((r) => r.chosen === "sword")).toBeFalse();
  expect(s.results.some((r) => r.refused === "sword")).toBeTrue();
  expect(byPath(s.engine, "Offers/sword")!.name).toContain("not enough");
  s.press("Offers/key");
  expect(s.results.some((r) => r.chosen === "key")).toBeFalse();
  expect(s.results.some((r) => r.refused === "key")).toBeTrue();
  expect(byPath(s.engine, "Offers/key")!.name).toContain("Sold out");
  s.press("Offers/hat");
  expect(s.results.some((r) => r.viewed === "hat")).toBeTrue();
  expect(s.results.some((r) => r.chosen === "hat")).toBeFalse();
});

test("storeFront: a discount shows its saving in whole percent; a limit says what is left; a plus asks to top up", () => {
  // Wrong: the saving worked from the wrong price (−67 % for 1,200 against 2,000), or the limit showing the bought count.
  const s = store(5000);
  expect(elements(s.engine).some((e) => e.text === "−40 %")).toBeTrue();
  expect(byPath(s.engine, "Offers/potion")!.name).toContain("2 left");
  s.press("Wallet/gems/TopUp");
  expect(s.results.some((r) => r.topUp === "gems")).toBeTrue();
  expect(byPath(s.engine, "Wallet/gems/TopUp")!.name).toBe("Get more Gems");
});

// ── battle pass ─────────────────────────────────────────────────────────

function pass(premium: boolean, viewport?: Partial<Api.Viewport>) {
  const claimed = new Set<string>(["1f"]);
  const results: Api.BattlePassResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: viewport ? 360 : 1180 }, height: { kind: "fixed", px: viewport ? 700 : 600 } } as never, () => {
      const r = og.ui.battlePass("Pass", {
        title: "Season 4",
        level: 3,
        xp: 400,
        xpNext: 1000,
        premium,
        tiers: Array.from({ length: 8 }, (_, i) => ({
          level: i + 1,
          free: { key: `f${i + 1}`, name: `Coins ${i + 1}` },
          premium: { key: `p${i + 1}`, name: `Outfit ${i + 1}` },
          freeClaimed: claimed.has(`${i + 1}f`),
          premiumClaimed: claimed.has(`${i + 1}p`),
        })),
      });
      for (const c of r.claimed) claimed.add(`${c.level}${c.premium ? "p" : "f"}`);
      results.push(r);
    });
  }, viewport);
  engine.advance(6);
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  const claims = () => results.flatMap((r) => r.claimed.map((c) => `${c.level}${c.premium ? "p" : "f"}`));
  return { engine, results, press, claims };
}

test("battlePass: a reward up to the level is claimed once; past the level, or premium while locked, it is refused", () => {
  // Wrong: a reward past the level granted, the premium track granted without the pass, or a claimed one granted again.
  const p = pass(false);
  p.press("Tiers/L2/Free");
  expect(p.claims()).toEqual(["2f"]);
  p.press("Tiers/L2/Free");
  expect(p.claims()).toEqual(["2f"]);
  expect(byPath(p.engine, "Tiers/L2/Free")!.name).toContain("Claimed");
  p.press("Tiers/L2/Premium");
  expect(p.claims()).toEqual(["2f"]);
  expect(p.results.some((r) => r.refused?.level === 2 && r.refused.premium)).toBeTrue();
  p.press("Tiers/L4/Free");
  expect(p.claims()).toEqual(["2f"]);
  expect(byPath(p.engine, "Tiers/L4/Free")!.name).toContain("Reach level 4");
}, 20000);

test("battlePass: Claim all takes every ready reward once, free and premium, and goes when none is left", () => {
  // Wrong: Claim all granting rewards past the level or already claimed, or staying after it emptied the track.
  const p = pass(true);
  expect(byPath(p.engine, "Actions/ClaimAll")!.name).toBe("Claim all (5)");
  p.press("Actions/ClaimAll");
  expect(p.claims().sort()).toEqual(["1p", "2f", "2p", "3f", "3p"]);
  p.engine.advance(4);
  expect(byPath(p.engine, "Actions/ClaimAll") === undefined).toBeTrue();
  expect(byPath(p.engine, "Actions/Unlock") === undefined).toBeTrue();
});

test("battlePass: the track opens on the player's tier, far along it, in a row and stacked", () => {
  // Wrong: the track opening at tier 1 with the player's tier off screen.
  for (const vp of [undefined, { width: 390, height: 844, breakpoint: "phonePortrait", coarsePointer: true }] as const) {
    const p = (() => {
      const claimed: string[] = [];
      const engine = run(() => {
        og.ui.column("Box", { width: { kind: "fixed", px: vp ? 360 : 1180 }, height: { kind: "fixed", px: vp ? 700 : 600 } } as never, () => {
          og.ui.battlePass("Pass", { level: 24, xpNext: 1000, tiers: Array.from({ length: 30 }, (_, i) => ({ level: i + 1, free: { key: `f${i}`, name: `R${i}` } })) });
        });
      }, vp as Partial<Api.Viewport> | undefined);
      engine.advance(6);
      return { engine, claimed };
    })();
    const view = byPath(p.engine, "Pass/Track/View")!;
    const cur = byPath(p.engine, "Tiers/L24")!;
    expect(cur.x).toBeGreaterThanOrEqual(view.x - 1);
    expect(cur.x + cur.width).toBeLessThanOrEqual(view.x + view.width + 1);
    expect(cur.y).toBeGreaterThanOrEqual(view.y - 1);
    expect(cur.y + cur.height).toBeLessThanOrEqual(view.y + view.height + 1);
    expect(byPath(p.engine, "Tiers/L1") === undefined).toBeTrue();
  }
});

// ── story: the conversation, the backlog, the quest log and tracker ─────

const storyFixture = join(dirname(fileURLToPath(import.meta.url)), "fixtures", "story");
{
  const cooked = Bun.spawnSync([ogBin(), "cook", storyFixture]);
  if (cooked.exitCode !== 0) throw new Error(new TextDecoder().decode(cooked.stderr));
}
const ok = <T>(r: unknown): T => {
  const x = r as { ok: boolean; value: T; reason?: string };
  if (!x.ok) throw new Error(x.reason);
  return x.value;
};

/** An engine over tests/fixtures/story (the smith, the rescue), its one system building `build(world)` on a 1280×720 desktop. */
function storyRun(build: (w: Api.World) => void): Engine {
  let world: Api.World | null = null;
  const sys = defineSystem(`StoryBuild${systems++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", margin: 0, width: { kind: "fixed", px: 1280 }, height: { kind: "fixed", px: 720 }, direction: "column" } as never, () => build(world!));
  });
  const r = createEngine(og.game({ seed: 42, systems: [sys] }) as unknown as GameToken, { root: storyFixture, locale: "en" });
  if (!r.ok) throw new Error(r.reason);
  world = r.value.world;
  ok(og.i18n.load(world, "strings/en.ogt"));
  ok(og.story.load(world, "story/smith.ogt"));
  ok(og.story.load(world, "story/rescue.ogt"));
  og.ui.previewViewport(world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  r.value.advance(2);
  return r.value;
}

/** The smith's conversation on screen, the game passing the answer on to og.story as it should. */
function smith(desc: Partial<Api.ConversationDesc> = {}, fake?: (v: Api.DialogueView) => Api.DialogueView) {
  let conv = -1;
  const results: Api.ConversationResult[] = [];
  const engine = storyRun((w) => {
    if (conv < 0) conv = ok<number>(og.story.start(w, { dialogue: "smith" }));
    const v0 = og.story.view(w, conv);
    const v = v0 && fake ? fake(v0) : v0;
    const r = og.ui.conversation("Talk", { view: v, ...desc });
    results.push(r);
    if (r.advanced) og.story.advance(w, conv);
    if (r.chosen !== null) og.story.choose(w, conv, r.chosen);
  });
  const tapEl = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(10);
  };
  return { engine, results, view: () => og.story.view(engine.world, conv)!, tapEl };
}

test("conversation: the line types out by the tick; a press shows it whole before the next goes on", () => {
  // Wrong: a press while typing advancing (the rest of the line never read), or the whole line at once.
  const s = smith({ charsPerSecond: 20 }, (v) => ({ ...v, choices: [], ended: false }));
  const said = () => elements(s.engine).find((e) => e.role === "status" && e.path.endsWith("Line"))!;
  // Said whole at once to a screen reader, never letter by letter.
  expect(said().name).toBe("Brannoc: Steel's not cheap. You've 10 gold.");
  expect(s.results.at(-1)!.revealed).toBeFalse();
  s.tapEl("Talk/Hit");
  expect(s.results.at(-1)!.revealed).toBeTrue();
  expect(s.results.some((r) => r.advanced)).toBeFalse();
  s.tapEl("Talk/Hit");
  expect(s.results.filter((r) => r.advanced).length).toBe(1);
});

test("conversation: a choice answers its ChoiceView.index (a hidden one counted), a disabled one says why and takes no press", () => {
  // Wrong: answering the row's position (Leave is the third row but index 3), or a disabled choice chosen.
  const s = smith({ charsPerSecond: 0, statNames: { charm: "Charm" } }, (v) => ({
    ...v,
    ended: false,
    timer: null,
    choices: v.choices.map((c) => (c.index === 0 ? { ...c, enabled: false, reason: "You need 5 more gold." } : c)),
  }));
  s.engine.advance(4);
  const buy = elements(s.engine).find((e) => e.role === "button" && e.name.includes("Buy the sword"))!;
  expect(buy.name).toBe("1, Buy the sword (10 gold), You need 5 more gold.");
  expect(buy.states).toContain("disabled");
  const haggle = elements(s.engine).find((e) => e.role === "button" && e.name.includes("Haggle"))!;
  // d20 + 3 against 12: 60 %.
  expect(haggle.name).toBe("2, Charm · 60 %, Haggle");
  s.tapEl(buy.path.split("/").slice(-2).join("/"));
  expect(s.results.some((r) => r.chosen !== null)).toBeFalse();
  s.tapEl("Choices/List/smith.leave");
  expect(s.results.find((r) => r.chosen !== null)!.chosen).toBe(3);
});

test("conversation: auto holds a whole line until its voice has ended, then goes on once", () => {
  // Wrong: auto cutting off a voice still playing (advancing on the pause alone), or advancing every tick after.
  let left = 120;
  const s = smith({ charsPerSecond: 0, auto: true, autoDelayMs: 100 }, (v) => ({ ...v, choices: [], ended: false, voiceTicks: 240, voiceLeft: left }));
  s.engine.advance(60);
  expect(s.results.some((r) => r.advanced)).toBeFalse();
  expect(elements(s.engine).some((e) => e.path.endsWith("Head/Voice"))).toBeTrue();
  left = 0;
  s.engine.advance(30);
  expect(s.results.filter((r) => r.advanced).length).toBe(1);
});

test("backlog: each line under its speaker with the reply chosen on it; back closes it", () => {
  // Wrong: the reply lost (the player's choices missing from the log), or back ignored.
  const closed: boolean[] = [];
  const engine = storyRun((w) => {
    const r = og.ui.backlog("Log", {
      lines: [
        { seq: 1, node: "greet", speaker: { id: "smith", name: "Brannoc", entity: 0 as never, portrait: "", expression: "", side: "right" }, key: "smith.hello", args: {}, text: "Steel's not cheap.", voice: "vo/a.wav", seen: false, chosen: { index: 1, text: "Haggle" } },
        { seq: 2, node: "angry", speaker: { id: "narrator", name: "", entity: 0 as never, portrait: "", expression: "", side: "center" }, key: "n", args: {}, text: "He slams the anvil.", voice: "", seen: false, chosen: null },
      ],
    });
    closed.push(r.closed);
    void w;
  });
  engine.advance(4);
  const items = elements(engine).filter((e) => e.role === "listitem");
  expect(items.map((e) => e.name)).toEqual(["Brannoc: Steel's not cheap. → Haggle", "He slams the anvil."]);
  expect(elements(engine).some((e) => e.role === "button" && e.name === "Play voice: Steel's not cheap.")).toBeTrue();
  key(engine, "Escape");
  expect(closed.some((c) => c)).toBeTrue();
});

test("questLog: shows the tracked quest, its hidden objective as ???, and answers Track for the game to pass on", () => {
  // Wrong: the first quest shown instead of the tracked one, a hidden objective's text leaked, Track flipping state itself.
  const tracks: string[] = [];
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) {
      ok(og.story.startQuest(w, "rescue"));
      og.story.progress(w, "rescue", "wolves", 2);
    }
    const qs = og.story.quests(w).map((q) => ({ ...q, objectives: [...q.objectives, { id: "secret", text: "Find the traitor", count: 0, target: 1, done: false, failed: false, optional: false, hidden: true }] }));
    const decoy = { ...qs[0]!, id: "decoy", title: "Decoy", tracked: false };
    const r = og.ui.questLog("Quests", { quests: [decoy, ...qs] });
    if (r.track) tracks.push(r.track);
  });
  engine.advance(4);
  expect(byPath(engine, "Detail/Body/Head/TitleRow/Title")!.text).toBe("The Lost Sword");
  const objectives = elements(engine).filter((e) => e.role === "checkbox").map((e) => e.name);
  expect(objectives).toContain("Drive off the wolves (2/3), 2 of 3");
  expect(objectives).toContain("???");
  expect(objectives.some((n) => n.includes("traitor"))).toBeFalse();
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "Actions/Track")!)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
  expect(tracks).toEqual(["rescue"]);
});

test("questLog: a reward's caption stands in its own row under the tile; a phone's Back is the secondary button with a chevron", () => {
  // Wrong: the name inside the card a style dresses (its frame and notches cut it), or Back a bare glass pane.
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) ok(og.story.startQuest(w, "rescue"));
    og.ui.questLog("Quests", { quests: og.story.quests(w) });
  });
  engine.advance(4);
  const tile = byPath(engine, "Rewards/R#0/Tile")!;
  const caption = byPath(engine, "Rewards/R#0/Caption")!;
  expect(tile.width).toBe(tile.height);
  expect(caption.y).toBeGreaterThanOrEqual(tile.y + tile.height);
  expect(byPath(engine, "Rewards/R#0/Caption/Name")!.text).toBe("Gold");
  expect(elements(engine).some((e) => e.path.includes("Rewards/R#0/Tile/") && e.text === "Gold")).toBeFalse();
  og.ui.previewViewport(engine.world, { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as never);
  engine.advance(4);
  engine.probe.commands(`0:touch_move:${centre(byPath(engine, "List/Active/rescue")!)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
  expect(byPath(engine, "Detail/Back/Chevron")).toBeDefined();
  expect(byPath(engine, "Detail/Back/Label")!.text).toBe("Back");
});

test("questTracker: a tracked quest that completes flashes Complete instead of vanishing at once", () => {
  // Wrong: a completed quest dropped from the tracker the tick it completes (no Complete), or an untracked one shown.
  let done = false;
  const engine = storyRun((w) => {
    if (og.story.quests(w).length === 0) ok(og.story.startQuest(w, "rescue"));
    if (done && og.story.quest(w, "rescue")!.status === "active") ok(og.story.completeQuest(w, "rescue"));
    og.ui.questTracker("Track", { quests: og.story.quests(w), anchor: null });
  });
  engine.advance(4);
  expect(elements(engine).some((e) => e.path.endsWith("Track/rescue"))).toBeTrue();
  done = true;
  engine.advance(6);
  expect(elements(engine).some((e) => e.text === "Complete")).toBeTrue();
});

// ── lobby ───────────────────────────────────────────────────────────────

/** A party of three in four seats, the local player second in the game's order; the game applying what it is asked. */
function party(ready = false) {
  const results: Api.LobbyResult[] = [];
  const state = { check: false, accepted: false, ready };
  const engine = run(() => {
    const members: Api.LobbyMember[] = [
      { key: "kael", name: "Kael", level: 42, ready: true, accepted: state.check },
      { key: "me", name: "Aria", level: 37, you: true, ready: state.ready, accepted: state.accepted },
      { key: "mira", name: "Mira", level: 18 },
    ];
    let r = {} as Api.LobbyResult;
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      r = og.ui.lobby("Lobby", { title: "Play", members, slots: 4, mode: { name: "Ranked" }, ...(state.check ? { readyCheck: { secondsLeft: 8, seconds: 10 } } : {}) });
    });
    results.push(r);
    if (r.ready) state.ready = !state.ready;
    if (r.accept) state.accepted = true;
  });
  return { engine, results, state };
}
const tapOn = (engine: Engine, el: Element): void => {
  engine.probe.commands(`0:touch_move:${centre(el)};0.05:touch_start;0.1:touch_end`);
  engine.advance(10);
};

test("lobby: the local player's card stands first and larger; a seat answers its place; the toggle asks to flip ready", () => {
  // Wrong: the party drawn in the game's order (the local player second), a seat answering its index among all cards, or ready flipped by the component.
  const p = party();
  const cards = elements(p.engine).filter((e) => /Lineup\/(kael|me|mira)$/.test(e.path)).sort((a, b) => a.x - b.x);
  expect(cards.map((c) => c.path.split("/").at(-1))).toEqual(["me", "kael", "mira"]);
  expect(cards[0]!.height).toBeGreaterThan(cards[1]!.height);
  // The cards rise in one after another: pressed once they stand.
  p.engine.advance(30);
  tapOn(p.engine, byPath(p.engine, "Lineup/Seat#0")!);
  expect(p.results.some((r) => r.invite === 0)).toBeTrue();
  expect(byPath(p.engine, "Bar/Action/Label")!.text).toBe("Ready up");
  tapOn(p.engine, byPath(p.engine, "Bar/Action")!);
  expect(p.results.filter((r) => r.ready).length).toBe(1);
  expect(byPath(p.engine, "Bar/Action/Label")!.text).toBe("Ready");
});

test("lobby: a ready check is accepted by confirm whatever has the focus, once; Back declines only before accepting", () => {
  // Wrong: the check answering only its button (a pad's A does nothing), accept answered every tick, or Back declining after accepting.
  const p = party(true);
  p.state.check = true;
  p.engine.advance(4);
  expect(byPath(p.engine, "Check/Panel/Title")!.text).toBe("Match found");
  key(p.engine, "Enter");
  expect(p.results.filter((r) => r.accept).length).toBe(1);
  expect(byPath(p.engine, "Check/Panel/Actions/Accept/Label")!.text).toBe("Accepted");
  key(p.engine, "Escape");
  expect(p.results.some((r) => r.decline)).toBeFalse();
});

// ── character select and versus ─────────────────────────────────────────

const ROSTER: Api.RosterEntry[] = [
  { key: "aria", name: "Aria", role: "Support" },
  { key: "kael", name: "Kael", role: "Duelist" },
  { key: "mira", name: "Mira", role: "Ranger", taken: true },
  { key: "brann", name: "Brann", role: "Tank" },
  { key: "vell", name: "Vell", role: "Mage", locked: true, lockedHint: "Unlock at level 10" },
];

function selectScreen() {
  const results: Api.CharacterSelectResult[] = [];
  const me = { key: "p1", name: "P1", entry: "kael", you: true, confirmed: false, team: 0 };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.characterSelect("Sel", { title: "Choose", roster: ROSTER, randomLabel: "", cursors: [me, { key: "p2", name: "P2", entry: "brann", team: 1 }] });
      results.push(r);
      if (r.hovered) me.entry = r.hovered;
      if (r.picked) me.confirmed = true;
      if (r.cancelled) me.confirmed = false;
    });
  });
  engine.advance(20);
  return { engine, results, me };
}

test("characterSelect: a taken or locked tile is refused, an open one picked; the rival's tag stands on its tile; Back unlocks", () => {
  // Wrong: a taken character picked twice, a locked one picked, the rival's cursor not shown, or Back ignored once locked in.
  const s = selectScreen();
  expect(byPath(s.engine, "Roster/Row#0/brann/Tags/Tag#0/Text")!.text).toBe("P2");
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Kael");
  tapOn(s.engine, byPath(s.engine, "Roster/Row#0/mira")!);
  tapOn(s.engine, byPath(s.engine, "Roster/Row#1/vell") ?? byPath(s.engine, "Roster/Row#0/vell")!);
  expect(s.results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["mira", "vell"]);
  expect(s.results.some((r) => r.picked)).toBeFalse();
  tapOn(s.engine, byPath(s.engine, "Roster/Row#0/aria")!);
  expect(s.results.filter((r) => r.picked).map((r) => r.picked)).toEqual(["aria"]);
  expect(byPath(s.engine, "Foot/Lock/Label")!.text).toBe("Locked in");
  key(s.engine, "Escape");
  expect(s.results.some((r) => r.cancelled)).toBeTrue();
});

test("characterSelect: a cursor the game moves stays where it put it; the focus asks to move it only when it changes tile", () => {
  // Wrong: the focused tile asked for the cursor every tick, so a cursor the game moved by other means (a pad's stick,
  // a random pick) snapped back to the tile the focus had stayed on.
  const s = selectScreen();
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Kael");
  s.me.entry = "aria";
  s.engine.advance(10);
  expect(s.me.entry).toBe("aria");
  expect(byPath(s.engine, "Stage/Start/Words/Name")!.text).toBe("Aria");
  key(s.engine, "ArrowRight");
  expect(s.me.entry).not.toBe("aria");
});

test("a screen-filling pattern given less than the safe area (under a game's header) lays out within its box, its action inside", () => {
  // Wrong: the pattern sizes its parts from its first guess (the whole safe area), its box grows to fit them, and the
  // grown box is kept as its room from then on, so its foot stands past the parent's on every tick (the kit's phone
  // character select had Lock in 42 px under the safe area).
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const songs: Api.SongEntry[] = ["a", "b", "c", "d", "e", "f"].map((k) => ({ key: k, title: `Song ${k}`, artist: "Artist", difficulties: [{ key: "easy", name: "Easy", level: 3 }] }));
  const cases: Array<[string, () => void]> = [
    ["Foot/Lock", () => og.ui.characterSelect("Sel", { roster: [...ROSTER, ...ROSTER.map((e) => ({ ...e, key: `${e.key}2` }))], cursors: [{ key: "p1", name: "P1", entry: "kael", you: true, team: 0 }] })],
    ["Bar/Action", () => og.ui.lobby("Lobby", { members: [{ key: "a", name: "Aria", you: true }, { key: "b", name: "Kael" }, { key: "c", name: "Mira" }], slots: 5, mode: { name: "Ranked" } })],
    ["Play", () => og.ui.songSelect("Songs", { songs, selected: "c", difficulty: "easy", preview: {} })],
  ];
  for (const [action, build] of cases) {
    const engine = run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 600 } }, build), phone);
    engine.advance(20);
    const box = byPath(engine, "Root/Box")!;
    const a = elements(engine).find((e) => e.path.endsWith(action) && e.role === "button")!;
    expect(a).toBeDefined();
    expect(a.y + a.height).toBeLessThanOrEqual(box.y + box.height + 0.5);
  }
});

test("on a phone in portrait the draft leaves the roster two rows, the map's legend is one row, the song wheel only whole rows", () => {
  // Wrong: the draft's two teams of tall portraits leave the roster one row (117 px of 691), the legend's chips wrap
  // into rows over the map, or a song row past the wheel's edge stands over the detail above it.
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const box = (build: () => void) => run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 691 } }, build), phone);
  const roster = [...ROSTER, ...ROSTER.map((e) => ({ ...e, key: `${e.key}2` }))].map((e) => ({ ...e, locked: false, taken: false }));
  const team = (n: number): Api.DraftTeam => ({ name: `Team ${n}`, team: n, picks: [{ player: "A" }, { player: "B" }, { player: "C" }], bans: ["", ""] });
  const d = box(() => og.ui.draft("Draft", { teams: [team(0), team(1)], roster, phase: "pick", secondsLeft: 10, yourTurn: true }));
  d.advance(10);
  expect(byPath(d, "Middle/Room")!.height).toBeGreaterThanOrEqual(200);
  const kinds = ["quest", "objective", "fastTravel", "shop", "poi", "boss"];
  const m = box(() => og.ui.worldMap("Map", { markers: kinds.map((k, i) => ({ key: k, kind: k, name: k, x: 0.1 + i * 0.15, y: 0.5 })) }));
  m.advance(10);
  expect(byPath(m, "Side/Legend")!.height).toBeLessThanOrEqual(60);
  const songs: Api.SongEntry[] = ["a", "b", "c", "d", "e", "f", "g"].map((k) => ({ key: k, title: `Song ${k}`, artist: "Artist", difficulties: [{ key: "easy", name: "Easy", level: 3 }] }));
  const s = box(() => og.ui.songSelect("Songs", { songs, selected: "d", difficulty: "easy", preview: {} }));
  s.advance(10);
  const wheel = byPath(s, "Songs/Wheel")!;
  const rows = elements(s).filter((e) => e.role === "option" && e.path.includes("/Wheel/"));
  expect(rows.length).toBeGreaterThan(1);
  for (const r of rows) expect(r.y).toBeGreaterThanOrEqual(wheel.y - 0.5);
});

test("versus: both sides named in one region; confirm skips only once the sides have landed", () => {
  // Wrong: a skip answered the first tick (the press that opened it skips it at once), or a side left unnamed.
  const results: Api.VersusResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      results.push(og.ui.versus("Vs", { left: { name: "Aria", character: "Kael" }, right: { name: "Mira", character: "Brann", team: 1 }, stage: "Round 1" }));
    });
  });
  expect(elements(engine).some((e) => e.role === "region" && e.name === "Kael VS Brann")).toBeTrue();
  engine.probe.commands("0:key_down:Enter;0.02:key_up:Enter");
  engine.advance(2);
  expect(results.some((r) => r.skipped)).toBeFalse();
  engine.advance(10);
  key(engine, "Enter");
  expect(results.filter((r) => r.skipped).length).toBeGreaterThan(0);
});

// ── draft and ban ───────────────────────────────────────────────────────

test("draft: a banned or picked character is refused; in the local turn a press hovers and the action locks the hovered one in", () => {
  // Wrong: a banned character hoverable, the action live outside the local turn, or the action locking something other than the hover.
  const results: Api.DraftResult[] = [];
  const st = { hovered: "", yourTurn: false };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.draft("Draft", {
        teams: [
          { name: "Blue", team: 0, bans: ["vell"], picks: [{ player: "You", you: true, active: st.yourTurn, entry: st.hovered }, { player: "Kael" }] },
          { name: "Red", team: 1, bans: [""], picks: [{ player: "Nyx", entry: "mira", locked: true }, { player: "Zed" }] },
        ],
        roster: ROSTER.map((e) => ({ ...e, taken: false, locked: false })),
        phase: "pick",
        phaseLabel: "Pick",
        secondsLeft: 20,
        yourTurn: st.yourTurn,
        hovered: st.hovered,
      });
      results.push(r);
      if (r.hovered) st.hovered = r.hovered;
    });
  });
  const tile = (k: string) => elements(engine).find((e) => e.path.endsWith(`/${k}`) && e.path.includes("/Roster/"))!;
  tapOn(engine, tile("aria"));
  expect(results.some((r) => r.hovered)).toBeFalse();
  expect(byPath(engine, "Foot/Action/Label")!.text).toBe("Waiting");
  st.yourTurn = true;
  engine.advance(2);
  tapOn(engine, tile("vell"));
  tapOn(engine, tile("mira"));
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["vell", "mira"]);
  tapOn(engine, tile("aria"));
  expect(st.hovered).toBe("aria");
  expect(byPath(engine, "Start/Picks/Slot#0/Words/Character")!.text).toBe("Aria");
  tapOn(engine, byPath(engine, "Foot/Action")!);
  expect(results.filter((r) => r.locked).map((r) => r.locked)).toEqual(["aria"]);
});

// ── world map ───────────────────────────────────────────────────────────

test("worldMap: a pin opens its detail; a fast-travel point travels, another place takes a waypoint; a kind turned off draws no pins", () => {
  // Wrong: Fast travel offered on an undiscovered point or a shop, a hidden kind's pins still pressable, or the legend flipping state itself.
  const results: Api.WorldMapResult[] = [];
  const st = { selected: "", hidden: [] as string[] };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.worldMap("World", {
        title: "Realm",
        markers: [
          { key: "gate", kind: "fastTravel", name: "Gate", x: 0.3, y: 0.4 },
          { key: "far", kind: "fastTravel", name: "Far", x: 0.5, y: 0.6, discovered: false },
          { key: "mill", kind: "shop", name: "Mill", x: 0.45, y: 0.7 },
        ],
        selected: st.selected,
        hidden: st.hidden,
      });
      results.push(r);
      if (r.selected !== null) st.selected = r.selected;
    });
  });
  tapOn(engine, byPath(engine, "Pins/gate")!);
  expect(st.selected).toBe("gate");
  expect(byPath(engine, "Detail/Action/Label")!.text).toBe("Fast travel");
  tapOn(engine, byPath(engine, "Detail/Action")!);
  expect(results.filter((r) => r.travel).map((r) => r.travel)).toEqual(["gate"]);
  tapOn(engine, byPath(engine, "Pins/far")!);
  expect(byPath(engine, "Detail/Action/Label")!.text).toBe("Set waypoint");
  tapOn(engine, byPath(engine, "Detail/Action")!);
  expect(results.filter((r) => r.track).map((r) => r.track)).toEqual(["far"]);
  // On a desktop the open detail stands in the legend's place; Back closes it and the legend returns.
  expect(byPath(engine, "Legend/shop") === undefined).toBeTrue();
  key(engine, "Escape");
  expect(st.selected).toBe("");
  tapOn(engine, byPath(engine, "Legend/shop")!);
  expect(results.filter((r) => r.toggled).map((r) => r.toggled)).toEqual(["shop"]);
  expect(byPath(engine, "Pins/mill")).toBeDefined();
  st.hidden = ["shop"];
  engine.advance(2);
  expect(byPath(engine, "Pins/mill") === undefined).toBeTrue();
});

// ── song select ─────────────────────────────────────────────────────────

test("songSelect: a press on another row selects it, on the selected one plays; the next-tab action steps over a locked chart; a locked song is refused", () => {
  // Wrong: a first press playing a song never selected, the step landing on a locked chart, or a locked song played.
  const results: Api.SongSelectResult[] = [];
  const st = { selected: "a", difficulty: "easy" };
  const charts = [{ key: "easy", name: "Easy", level: 3 }, { key: "hard", name: "Hard", level: 9, locked: true }, { key: "expert", name: "Expert", level: 12 }];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.songSelect("Songs", {
        songs: [{ key: "a", title: "Alpha", difficulties: charts }, { key: "b", title: "Beta", difficulties: charts }, { key: "c", title: "Gamma", difficulties: charts, locked: true }],
        selected: st.selected,
        difficulty: st.difficulty,
      });
      results.push(r);
      if (r.selected) st.selected = r.selected;
      if (r.difficulty) st.difficulty = r.difficulty;
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/b")!);
  expect(st.selected).toBe("b");
  expect(results.some((r) => r.play)).toBeFalse();
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/b")!);
  expect(results.filter((r) => r.play).length).toBe(1);
  key(engine, "KeyE");
  expect(st.difficulty).toBe("expert");
  tapOn(engine, byPath(engine, "Wheel/c")!);
  engine.advance(10);
  tapOn(engine, byPath(engine, "Wheel/c")!);
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["c"]);
  expect(results.filter((r) => r.play).length).toBe(1);
});

// ── garage ──────────────────────────────────────────────────────────────

test("garage: a fitted part is not bought again; a short or locked part is refused, an affordable one bought, an owned one equipped", () => {
  // Wrong: a part dearer than the purse answered as bought, a locked one answered, an owned one sold again.
  const results: Api.GarageResult[] = [];
  const st = { selected: "short" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.garage("Garage", {
        title: "Garage",
        balance: 300,
        categories: [{ key: "engine", name: "Engine" }, { key: "paint", name: "Livery", livery: true }],
        category: "engine",
        parts: [
          { key: "stock", name: "Stock", owned: true, equipped: true, tier: 1 },
          { key: "owned", name: "Street", owned: true, tier: 2 },
          { key: "short", name: "Race", price: 900, tier: 3 },
          { key: "cheap", name: "Sport", price: 200, tier: 2 },
          { key: "locked", name: "Works", price: 50, locked: true, lockedHint: "Class A", tier: 3 },
        ],
        selected: st.selected,
        stats: [{ key: "power", label: "Power", value: 5, preview: 7, max: 10 }],
      });
      results.push(r);
      if (r.selected) st.selected = r.selected;
    });
  });
  engine.advance(10);
  const act = (): void => {
    tapOn(engine, byPath(engine, "Side/Action")!);
    engine.advance(4);
  };
  act();
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["short"]);
  for (const k of ["locked", "cheap", "owned", "stock"]) {
    tapOn(engine, byPath(engine, `List/${k}`)!);
    engine.advance(4);
    expect(st.selected).toBe(k);
    act();
  }
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["short", "locked"]);
  expect(results.filter((r) => r.buy).map((r) => r.buy)).toEqual(["cheap"]);
  expect(results.filter((r) => r.equip).map((r) => r.equip)).toEqual(["owned"]);
});

test("garage: the livery's layers answer their own eye and order; the first layer cannot go up nor the last down; the next-tab action wraps to the first category", () => {
  // Wrong: the eye or the arrows selecting the row instead, or a first layer answered as moved up.
  const results: Api.GarageResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      results.push(og.ui.garage("Garage", {
        balance: 0,
        categories: [{ key: "engine", name: "Engine" }, { key: "paint", name: "Livery", livery: true }],
        category: "paint",
        layers: [
          { key: "base", name: "Base", kind: "Paint", color: { r: 0.8, g: 0.1, b: 0.1, a: 1 } },
          { key: "stripe", name: "Stripe", kind: "Decal", color: { r: 1, g: 1, b: 1, a: 1 } },
        ],
        layer: "base",
      }));
    });
  });
  engine.advance(10);
  for (const p of ["stripe/Up", "base/Down", "stripe/Eye"]) {
    tapOn(engine, byPath(engine, `List/${p}`)!);
    engine.advance(4);
  }
  expect(results.filter((r) => r.layerMove).map((r) => [r.layerMove!.key, r.layerMove!.by])).toEqual([["stripe", -1], ["base", 1]]);
  expect(results.filter((r) => r.layerVisible).map((r) => r.layerVisible)).toEqual(["stripe"]);
  expect(results.some((r) => r.layer)).toBeFalse();
  for (const p of ["base/Up", "stripe/Down"]) {
    tapOn(engine, byPath(engine, `List/${p}`)!);
    engine.advance(4);
  }
  expect(results.filter((r) => r.layerMove).length).toBe(2);
  key(engine, "KeyE");
  expect(results.filter((r) => r.category).map((r) => r.category)).toEqual(["engine"]);
  tapOn(engine, byPath(engine, "List/stripe")!);
  engine.advance(4);
  expect(results.filter((r) => r.layer).map((r) => r.layer)).toEqual(["stripe"]);
});

// ── ledger and overlay bar ──────────────────────────────────────────────

const CITY: Api.LedgerLine[] = [
  { key: "res", section: "in", name: "Residential", amount: 5200, previous: 4800, funding: 9 },
  { key: "com", section: "in", name: "Commercial", amount: 3100, previous: 3400, funding: 12 },
  { key: "tour", section: "in", name: "Tourism", amount: 900, previous: 600 },
  { key: "police", section: "out", name: "Police", amount: 1800, previous: 1700, funding: 100 },
  { key: "fire", section: "out", name: "Fire", amount: 1200, previous: 1250, funding: 90, locked: true },
  { key: "parks", section: "out", name: "Parks", amount: 400, previous: 300, funding: 110 },
];

function cityLedger(st: { sort: string; descending: boolean }, results: Api.LedgerResult[]): Engine {
  return run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.ledger("Budget", {
        sections: [{ key: "in", name: "Income" }, { key: "out", name: "Expenses", expense: true }],
        lines: CITY.map((l) => (l.section === "in" ? { ...l, funding: -1 } : l)),
        sort: st.sort,
        descending: st.descending,
        fundingMin: 50,
        fundingMax: 150,
      });
      results.push(r);
      if (r.sort !== null) {
        st.sort = r.sort;
        st.descending = r.descending;
      }
    });
  });
}

test("ledger: a heading sorts each section by it, amounts high to low first; the same heading again turns the order over; the net is income less expenses", () => {
  // Wrong: lines sorted across sections, the order not turned over, or expenses added to the net.
  const results: Api.LedgerResult[] = [];
  const st = { sort: "", descending: false };
  const engine = cityLedger(st, results);
  engine.advance(10);
  const order = (): string[] => elements(engine).filter((e) => /\/List\/[a-z]+$/.test(e.path) && CITY.some((l) => e.path.endsWith(`/${l.key}`))).sort((a, b) => a.y - b.y).map((e) => e.path.split("/").pop()!);
  expect(order()).toEqual(["res", "com", "tour", "police", "fire", "parks"]);
  tapOn(engine, byPath(engine, "Header/amount")!);
  engine.advance(4);
  expect([st.sort, st.descending]).toEqual(["amount", true]);
  expect(order()).toEqual(["res", "com", "tour", "police", "fire", "parks"]);
  tapOn(engine, byPath(engine, "Header/amount")!);
  engine.advance(4);
  expect([st.sort, st.descending]).toEqual(["amount", false]);
  expect(order()).toEqual(["tour", "com", "res", "parks", "fire", "police"]);
  tapOn(engine, byPath(engine, "Header/change")!);
  engine.advance(4);
  expect(order()).toEqual(["tour", "res", "com", "parks", "police", "fire"]);
  expect(elements(engine).find((e) => e.path.endsWith("Net/Value"))?.text).toBe("+5,800");
});

test("ledger: a line's budget slider answers its key and value (not a press on the line); a locked one answers no budget; a press on a line selects it", () => {
  // Wrong: the slider's move lost or answered under another line, a locked policy moved, a line's press not answered.
  const results: Api.LedgerResult[] = [];
  const engine = cityLedger({ sort: "", descending: false }, results);
  engine.advance(10);
  const end = (key: string): string => {
    const el = elements(engine).find((e) => e.path.includes(`/List/${key}/`) && e.path.endsWith("/Funding"))!;
    return `${Math.round(el.x + el.width - 2)},${Math.round(el.y + el.height / 2)}`;
  };
  for (const k of ["police", "fire"]) {
    engine.probe.commands(`0:touch_move:${end(k)};0.05:touch_start;0.1:touch_end`);
    engine.advance(10);
  }
  const moves = results.filter((r) => r.funding).map((r) => r.funding!);
  expect(moves.length > 0 && moves.every((m) => m.key === "police")).toBeTrue();
  expect(moves[moves.length - 1]!.value).toBe(150);
  tapOn(engine, byPath(engine, "List/tour/Main/Name")!);
  const picked = results.filter((r) => r.selected).map((r) => r.selected);
  expect([picked.includes("police"), picked[picked.length - 1]]).toEqual([false, "tour"]);
});

test("overlayBar: a press shows an overlay and its legend; a press on the shown one turns it off", () => {
  // Wrong: a second press on the shown overlay showing it again, or a legend with no overlay shown.
  const results: Api.OverlayBarResult[] = [];
  const st = { selected: "" };
  const engine = run(() => {
    const r = og.ui.overlayBar("Lenses", {
      overlays: [
        { key: "traffic", name: "Traffic", ramp: ["#2ecc71", "#f1c40f", "#e74c3c"], low: "Free", high: "Jammed" },
        { key: "power", name: "Power", ramp: ["#222244", "#ffd23f"], low: "None", high: "Full" },
      ],
      selected: st.selected,
    });
    results.push(r);
    if (r.selected !== null) st.selected = r.selected;
  });
  engine.advance(10);
  expect(byPath(engine, "Lenses/Legend") === undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Row/power")!);
  engine.advance(4);
  expect(st.selected).toBe("power");
  expect(elements(engine).find((e) => e.path.endsWith("Legend/Name"))?.text).toBe("POWER");
  tapOn(engine, byPath(engine, "Row/power")!);
  engine.advance(4);
  expect(st.selected).toBe("");
  expect(byPath(engine, "Lenses/Legend") === undefined).toBeTrue();
});

// ── chat ────────────────────────────────────────────────────────────────

const CHANNELS: Api.ChatChannel[] = [
  { key: "all", name: "All", all: true },
  { key: "party", name: "Party", color: "info" },
  { key: "guild", name: "Guild", color: "success" },
  { key: "system", name: "System", readOnly: true },
];

function chatRun(st: { draft: string; sendTo: string; messages: Api.ChatMessage[] }, results: Api.ChatResult[], h = 420): Engine {
  return run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 520 }, height: { kind: "fixed", px: h } }, () => {
      const r = og.ui.chat("Chat", {
        channels: CHANNELS,
        channel: "all",
        messages: st.messages,
        me: "Lumen",
        members: [{ key: "lumen", name: "Lumen" }, { key: "luka", name: "Luka" }, { key: "riven", name: "Riven" }],
        draft: st.draft,
        sendTo: st.sendTo,
      });
      results.push(r);
      if (r.draft !== null) st.draft = r.draft;
      if (r.sendTo !== null) st.sendTo = r.sendTo;
    });
  });
}

test("chat: a player's words show as typed, markup and all; a link and an author answer their keys; @me lights the line", () => {
  // Wrong: a player's [b] read as markup, a link answered as the author (or not at all), a mention of another lit as one of mine.
  const results: Api.ChatResult[] = [];
  const st = { draft: "", sendTo: "", messages: [
    { key: "m1", channel: "party", author: "Riven", authorKey: "riven", text: "[b]loot[/b] {0} for @Lumen", links: [{ key: "blade", label: "Ashbringer", kind: "item", rarity: "legendary" as const }] },
    { key: "m2", channel: "guild", author: "Juno", authorKey: "juno", text: "gg @Riven" },
  ] };
  const engine = chatRun(st, results);
  engine.advance(10);
  const words = (k: string): string => elements(engine).filter((e) => e.path.includes(`/Log/${k}/Text`)).map((e) => e.text).join("");
  expect(words("m1")).toContain("[b]loot[/b]");
  expect(words("m1")).toContain("[Ashbringer]");
  expect(byPath(engine, "Log/m1/Edge") !== undefined && byPath(engine, "Log/m2/Edge") === undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Log/m1/Text/Lblade")!);
  tapOn(engine, byPath(engine, "Log/m2/Text/Ujuno")!);
  expect([results.filter((r) => r.link).map((r) => r.link), results.filter((r) => r.author).map((r) => r.author)]).toEqual([["blade"], ["juno"]]);
});

test("chat: an @word offers the members it starts, a press writes the name; Send posts to the chip's channel, which steps past a read-only one", () => {
  // Wrong: no suggestion, a suggestion that drops the rest of the draft, a send to System, or a send of nothing.
  const results: Api.ChatResult[] = [];
  const st = { draft: "nice one @lu", sendTo: "", messages: [] as Api.ChatMessage[] };
  const engine = chatRun(st, results);
  engine.advance(10);
  expect(byPath(engine, "Suggest/riven") === undefined && byPath(engine, "Suggest/luka") !== undefined).toBeTrue();
  tapOn(engine, byPath(engine, "Suggest/luka")!);
  engine.advance(4);
  expect(st.draft).toBe("nice one @Luka ");
  for (let i = 0; i < 2; i++) {
    tapOn(engine, byPath(engine, "Input/Channel")!);
    engine.advance(4);
  }
  expect(st.sendTo).toBe("party");
  tapOn(engine, byPath(engine, "Input/Send")!);
  expect(results.filter((r) => r.send).map((r) => [r.send!.channel, r.send!.text])).toEqual([["party", "nice one @Luka"]]);
});

test("chat: at its foot the log follows a new message; scrolled up, new ones gather in the pill, whose press brings the view down", () => {
  // Wrong: the view yanked down while the player reads back, or new messages arriving unseen with no pill.
  const results: Api.ChatResult[] = [];
  const say = (i: number): Api.ChatMessage => ({ key: `m${i}`, channel: "party", author: "Riven", text: `line ${i} of the raid plan` });
  const st = { draft: "", sendTo: "", messages: Array.from({ length: 30 }, (_, i) => say(i)) };
  const engine = chatRun(st, results, 300);
  engine.advance(30);
  const shows = (k: string): boolean => {
    const e = byPath(engine, `Log/${k}`);
    const log = byPath(engine, "Chat/Log")!;
    return !!e && e.y + e.height <= log.y + log.height + 2 && e.y >= log.y - 2;
  };
  expect(shows("m29")).toBeTrue();
  st.messages = [...st.messages, say(30)];
  engine.advance(30);
  expect(shows("m30")).toBeTrue();
  const log = byPath(engine, "Chat/Log")!;
  engine.probe.commands(`0:touch_move:${Math.round(log.x + log.width / 2)},${Math.round(log.y + log.height / 2)};0.05:scroll:0,-400`);
  engine.advance(30);
  expect(shows("m30")).toBeFalse();
  st.messages = [...st.messages, say(31), say(32)];
  engine.advance(30);
  expect(shows("m32")).toBeFalse();
  expect(elements(engine).find((e) => e.path.endsWith("Chat/New/Text"))?.text).toBe("2 new");
  tapOn(engine, byPath(engine, "Chat/New")!);
  engine.advance(40);
  expect(shows("m32")).toBeTrue();
  expect(byPath(engine, "Chat/New") === undefined).toBeTrue();
});

// ── RTS command layer ───────────────────────────────────────────────────

const ACT = (name: string, keys: string[]): Api.UiActionDesc => ({ name, keys, buttons: [], trigger: "press", holdMs: 0, toggle: false, label: name });

function rtsRun(st: { page: string; targeting: string }, results: Api.RtsCommandsResult[]): Engine {
  const sys = defineSystem(`RtsCmd${systems++}`, {}, (_q: unknown, _t: unknown, world: Api.World) => {
    for (const a of [ACT("cmdAttack", ["KeyA"]), ACT("cmdStorm", ["KeyS"]), ACT("group1", ["Digit1"]), ACT("group2", ["Digit2"])]) og.ui.defineAction(world, a);
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column" } as never, () => {
      const r = og.ui.rtsCommands("Rts", {
        console: {
          selection: { units: [{ key: "u1", name: "Marine", health: 40, healthMax: 45 }, { key: "u2", name: "Marine", health: 45, healthMax: 45 }], groups: [{ key: "g1", label: "1", count: 2 }, { key: "g2", label: "2", count: 0 }], group: "g1" },
          commands: { columns: 5 },
          anchor: "bottom",
        },
        pages: [
          { key: "root", commands: [{ key: "attack", name: "Attack", action: "cmdAttack" }, { key: "build", name: "Build" }, { key: "storm", name: "Storm", action: "cmdStorm", locked: true }] },
          { key: "build", commands: [{ key: "barracks", name: "Barracks" }] },
        ],
        page: st.page,
        groupActions: ["group1", "group2"],
        targeting: st.targeting,
      });
      results.push(r);
      if (r.page !== null) st.page = r.page;
    });
  });
  const r = createEngine(og.game({ systems: [sys] }) as unknown as GameToken);
  if (!r.ok) throw new Error(r.reason);
  r.value.advance(10);
  return r.value;
}

test("rtsCommands: a command naming a page opens it, whose Back returns; a locked command is refused by press or key; Shift queues", () => {
  // Wrong: Build answered as an order, a sub-page with no way back, a locked command fired by its hotkey, or Shift ignored.
  const results: Api.RtsCommandsResult[] = [];
  const st = { page: "", targeting: "" };
  const engine = rtsRun(st, results);
  tapOn(engine, byPath(engine, "Commands/Grid/Row#0/build")!);
  expect(st.page).toBe("build");
  expect(results.some((r) => r.command === "build")).toBeFalse();
  tapOn(engine, byPath(engine, "PageBack")!);
  expect(st.page).toBe("root");
  key(engine, "KeyS");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["storm"]);
  engine.probe.commands("0:key_down:ShiftLeft;0.05:key_down:KeyA;0.1:key_up:KeyA;0.15:key_up:ShiftLeft");
  engine.advance(12);
  expect(results.filter((r) => r.command).map((r) => [r.command, r.queued])).toEqual([["attack", true]]);
});

test("rtsCommands: a group's key recalls it and twice quickly focuses it; Ctrl sets and Shift adds; the prompt names a command from another page; Back cancels it", () => {
  // Wrong: Ctrl+1 recalling instead of setting, a slow second tap focusing, or Back leaving the targeting order in place.
  const results: Api.RtsCommandsResult[] = [];
  const st = { page: "", targeting: "" };
  const engine = rtsRun(st, results);
  key(engine, "Digit1");
  engine.advance(60);
  key(engine, "Digit1");
  key(engine, "Digit1");
  engine.probe.commands("0:key_down:ControlLeft;0.05:key_down:Digit1;0.1:key_up:Digit1;0.15:key_up:ControlLeft");
  engine.advance(12);
  engine.probe.commands("0:key_down:ShiftLeft;0.05:key_down:Digit2;0.1:key_up:Digit2;0.15:key_up:ShiftLeft");
  engine.advance(12);
  const got = (k: keyof Api.RtsCommandsResult) => results.filter((r) => r[k]).map((r) => r[k]);
  expect([got("recall"), got("focus"), got("assign"), got("add")]).toEqual([["g1", "g1"], ["g1"], ["g1"], ["g2"]]);
  st.targeting = "barracks";
  engine.advance(4);
  expect(elements(engine).find((e) => e.path.endsWith("RtsPrompt/Text"))?.text).toBe("Barracks: choose a target");
  key(engine, "Escape");
  expect(results.some((r) => r.cancelTarget)).toBeTrue();
});

// ── photo mode ──────────────────────────────────────────────────────────

test("photoMode: a tab, a filter tile and the shutter answer; Back while hidden only brings the controls back, Back while shown exits; the flash follows a capture", () => {
  // Wrong: Back while hidden leaving photo mode (the player loses the shot), the chosen tile answered again, or no flash after Capture.
  const results: Api.PhotoModeResult[] = [];
  const st = { tab: "camera", filter: "none", hidden: false };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.photoMode("Photo", {
        tabs: [{ key: "camera", label: "Camera" }, { key: "filters", label: "Filters" }],
        tab: st.tab,
        controls: [{ key: "fov", tab: "camera", label: "Field of view", value: 60, min: 20, max: 110, step: 1, unit: "°" }],
        filters: [{ key: "none", name: "None" }, { key: "noir", name: "Noir", tint: "#00000080" }],
        filter: st.filter,
        strength: 0.5,
        hidden: st.hidden,
      });
      results.push(r);
      if (r.tab) st.tab = r.tab;
      if (r.filter) st.filter = r.filter;
      if (r.hide) st.hidden = !st.hidden;
    });
  });
  engine.advance(10);
  expect(byPath(engine, "Rows/fov/Head/Value")?.text).toBe("60°");
  tapOn(engine, byPath(engine, "Tabs/filters")!);
  expect(st.tab).toBe("filters");
  tapOn(engine, byPath(engine, "Rows/Line#0/noir")!);
  tapOn(engine, byPath(engine, "Rows/Line#0/noir")!);
  expect(results.filter((r) => r.filter).map((r) => r.filter)).toEqual(["noir"]);
  tapOn(engine, byPath(engine, "Bar/Shutter")!);
  expect(results.filter((r) => r.capture).length).toBe(1);
  expect(byPath(engine, "Photo/Flash")).toBeDefined();
  engine.advance(60);
  expect(byPath(engine, "Photo/Flash") ?? null).toBeNull();
  key(engine, "KeyX");
  expect(st.hidden).toBeTrue();
  expect(byPath(engine, "Photo/Panel") ?? null).toBeNull();
  expect(byPath(engine, "Photo/Hint")).toBeDefined();
  key(engine, "Escape");
  expect(st.hidden).toBeFalse();
  expect(results.some((r) => r.exit)).toBeFalse();
  key(engine, "Escape");
  expect(results.filter((r) => r.exit).length).toBe(1);
});

// ── loadout ─────────────────────────────────────────────────────────────

test("loadout: a locked slot is refused, not selected; a tab answers its index; Remove answers apart from its row; a callout's dot lands on its point of the art as drawn", () => {
  // Wrong: a locked slot selected, a press on Remove also answered as the row's, or a leader ending off the art (the stage's padding or the contain fit not counted).
  const results: Api.LoadoutResult[] = [];
  const st = { slot: "primary" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = og.ui.loadout("Loadout", {
        title: "Loadout",
        loadouts: ["One", "Two", "Three"],
        loadout: 0,
        slots: [
          { key: "primary", label: "Primary", name: "Raven", detail: "Lvl 12" },
          { key: "secondary", label: "Secondary", name: "Sidearm" },
          { key: "perk3", label: "Perk 3", name: "Ghost", locked: true },
        ],
        slot: st.slot,
        name: "Raven",
        aspect: 2,
        attachments: [
          { key: "optic", label: "Optic", name: "Holo", x: 0.5, y: 0.2 },
          { key: "barrel", label: "Barrel", name: "Suppressor", x: 0.1, y: 0.5, rarityLabel: "Rare", rarity: "rare" },
        ],
        stats: [{ key: "damage", label: "Damage", value: 32, delta: 4 }],
      });
      results.push(r);
      if (r.slot) st.slot = r.slot;
    });
  });
  engine.advance(10);
  tapOn(engine, byPath(engine, "Slots/perk3")!);
  expect(st.slot).toBe("primary");
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["perk3"]);
  tapOn(engine, byPath(engine, "Slots/secondary")!);
  expect(st.slot).toBe("secondary");
  tapOn(engine, byPath(engine, "Tabs/Row/2")!);
  expect(results.filter((r) => r.loadout !== null).map((r) => r.loadout)).toEqual([2]);
  tapOn(engine, byPath(engine, "List/barrel/Remove")!);
  tapOn(engine, byPath(engine, "List/optic")!);
  expect(results.filter((r) => r.remove).map((r) => r.remove)).toEqual(["barrel"]);
  expect(results.filter((r) => r.attachment).map((r) => r.attachment)).toEqual(["optic"]);
  // The art is 2:1, contained in its slot: the optic's point is at its middle across, a fifth down.
  const art = byPath(engine, "Room/Art")!;
  const dw = Math.min(art.width, art.height * 2);
  const dh = dw / 2;
  const dot = byPath(engine, "Callouts/optic~Dot")!;
  expect(Math.abs(dot.x + dot.width / 2 - (art.x + art.width / 2))).toBeLessThan(2);
  expect(Math.abs(dot.y + dot.height / 2 - (art.y + (art.height - dh) / 2 + 0.2 * dh))).toBeLessThan(2);
  const callout = byPath(engine, "Callouts/optic")!;
  expect(callout.y + callout.height).toBeLessThanOrEqual(art.y + (art.height - dh) / 2 + 1);
});
