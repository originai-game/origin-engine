// The genre patterns' preview screens (docs/ui.md §11, U6d): each a game's
// screen built from one or more patterns over the kit's fixtures, with the
// game's side done as a game would do it (a move applied to the
// viewmodel, an item equipped, a pull resolved). `pat:<key>:<value>` sets
// a preview's state (the item carried, the hand fanned, the reveal step) so
// the playtests (tests/patterns*.og.ts) can draw each state.
//
// The previews' own state is the preview's, kept here between ticks: it is
// a gallery, never saved.

import { og } from "@origingame/engine";
import type { GridItem, InventoryCarry, InventoryMove, PaperdollSlot, Size, World } from "@origingame/engine";

/** A fixture the preview changes: its readonly fields writable. */
type Mut<T> = { -readonly [K in keyof T]: T[K] };
type Item = Mut<GridItem>;
type Slot = Mut<PaperdollSlot>;

const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

function phone(): boolean {
  const b = og.ui.frameViewport().breakpoint;
  return b === "phonePortrait" || b === "phoneLandscape";
}

/**
 * A pattern screen: a header (its title and a line under it), the body,
 * and the action bar at the foot; a scrolling column on a phone. Leaves
 * the kit's Gallery button its corner (bottom start on a desktop, the top
 * on a phone).
 */
function screen(id: string, title: string, sub: string, build: () => void, hints: Array<{ action: string; hint: string }> = []): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  // A 1080p desktop reads from further away: the header steps up (18 px is the floor there, docs/ui.md §15.2).
  const big = !small && vp.height >= 1000;
  // On a phone in portrait the kit's Gallery button sits at the top, under the notch; it grows with the text size.
  const top = small ? (vp.breakpoint === "phonePortrait" ? 60 + Math.round(52 * Math.max(1, t.textScale ?? 1)) : 68) : t.space.lg;
  // The action hints sit on the bottom edge (actionHints anchors itself), and grow with the text size.
  const bottom = small ? t.space.sm : Math.round(72 * Math.max(1, t.textScale ?? 1));
  og.ui.anchor(id, { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(vp.height - sa.top - sa.bottom), direction: "column", gap: t.space.md, padding: { left: small ? t.space.sm : t.space.xl, top, right: small ? t.space.sm : t.space.xl, bottom } }, () => {
    og.ui.column("Header", { width: grow, gap: 2 }, () => {
      og.ui.label("Title", title, { step: small ? "xl2" : big ? "xl6" : "xl4", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      if (sub) og.ui.label("Sub", sub, { step: small ? "sm" : big ? "xl" : "base", color: { token: "inkMuted" }, width: grow, maxLines: 2 });
    });
    og.ui.scroll("Body", { width: grow, height: grow, gap: t.space.md }, build);
    if (hints.length > 0 && !small) og.ui.actionHints("Hints", { hints });
  });
}

// ── state ───────────────────────────────────────────────────────────────

const BAG0: Item[] = [
  { key: "greatsword", name: "Ashbringer", x: 0, y: 0, w: 1, h: 3, rarity: "legendary", tag: "weapon", icon: "sword" },
  { key: "bow", name: "Windrunner Bow", x: 1, y: 0, w: 1, h: 3, rarity: "epic", tag: "weapon", charge: 0.18 },
  { key: "potions", name: "Healing Potion", x: 2, y: 0, count: 12, maxStack: 20, tag: "potion", rarity: "common" },
  { key: "mana", name: "Mana Potion", x: 3, y: 0, count: 4, maxStack: 20, tag: "potion", rarity: "uncommon", fresh: true },
  { key: "ring", name: "Signet of the Vale", x: 2, y: 1, tag: "ring", rarity: "rare", locked: true },
  { key: "helm", name: "Crown of Thorns", x: 4, y: 0, w: 2, h: 2, tag: "helmet", rarity: "rare" },
  { key: "plate", name: "Dragonscale Plate", x: 6, y: 0, w: 2, h: 3, tag: "chest", rarity: "epic", unusable: true },
  { key: "gem", name: "Flawless Ruby", x: 3, y: 1, tag: "gem", rarity: "mythic" },
  { key: "boots", name: "Boots of Haste", x: 4, y: 2, w: 2, h: 2, tag: "feet", rarity: "uncommon", rotated: false },
  { key: "scroll", name: "Town Portal", x: 2, y: 3, count: 3, maxStack: 10, tag: "scroll" },
];

const STASH0: Item[] = [{ key: "ore", name: "Iron Ore", x: 0, y: 0, count: 40, maxStack: 50, tag: "ore" }, { key: "amulet", name: "Amulet of Dawn", x: 1, y: 0, tag: "amulet", rarity: "legendary" }];

const SLOTS0: Slot[] = [
  { key: "head", label: "Head", accept: ["helmet"], x: 0.5, y: 0.1, icon: "crown" },
  { key: "neck", label: "Neck", accept: ["amulet"], x: 0.78, y: 0.2 },
  { key: "chest", label: "Chest", accept: ["chest"], x: 0.5, y: 0.36, item: { key: "robe", name: "Silk Robe", rarity: "uncommon", tag: "chest" } },
  { key: "hands", label: "Hands", accept: ["gloves"], x: 0.16, y: 0.42 },
  { key: "main", label: "Main hand", accept: ["weapon"], x: 0.14, y: 0.66, aspect: 0.6, size: 0, item: { key: "dagger", name: "Rusty Dagger", rarity: "common", tag: "weapon", charge: 0.5 } },
  { key: "off", label: "Off hand", accept: ["weapon", "shield"], x: 0.86, y: 0.66, aspect: 0.6 },
  { key: "ring1", label: "Ring", accept: ["ring"], x: 0.84, y: 0.44 },
  { key: "feet", label: "Feet", accept: ["feet"], x: 0.5, y: 0.9 },
  { key: "relic", label: "Relic", x: 0.2, y: 0.12, locked: true, lockedHint: "Unlocks at level 30" },
];

const p = {
  bag: BAG0.map((i) => ({ ...i })),
  stash: STASH0.map((i) => ({ ...i })),
  slots: SLOTS0.map((s) => ({ ...s })),
  carry: null as InventoryCarry | null,
  selected: "helm",
  highlight: "",
  pick: null as { key: string; count: number } | null,
  menu: null as { key: string; at: { x: number; y: number } } | null,
  log: "",
};

/** `pat:<key>:<value>`: a pattern preview's state (pick:<item>, highlight:<tag>, reset). */
export function patternsCommand(_world: World, args: readonly string[]): void {
  const [key, value = ""] = args;
  if (key === "pick") p.pick = { key: value, count: 0 };
  else if (key === "highlight") p.highlight = value;
  else if (key === "select") p.selected = value;
  else if (key === "reset") {
    p.bag = BAG0.map((i) => ({ ...i }));
    p.slots = SLOTS0.map((s) => ({ ...s }));
    p.stash = STASH0.map((i) => ({ ...i }));
    p.carry = null;
    p.pick = null;
    p.selected = "helm";
    p.highlight = "";
    p.menu = null;
  }
}

/** The game's side of a move: what an inventory does with InventoryGridResult.move. */
function applyMove(list: Item[], from: Item[] | null, m: InventoryMove, fromSlot: Slot | null): void {
  const src = (fromSlot ? fromSlot.item ?? null : (from ?? list).find((i) => i.key === m.key) ?? null) as Item | null;
  if (!src) return;
  const moved: Item = { ...src, x: m.x, y: m.y, rotated: m.rotated, count: m.count };
  if (m.mergeInto) {
    const into = list.find((i) => i.key === m.mergeInto)!;
    const room = (into.maxStack ?? 1) - (into.count ?? 1);
    const n = Math.min(room, m.count);
    into.count = (into.count ?? 1) + n;
    if (fromSlot) fromSlot.item = null;
    else if ((src.count ?? 1) - n <= 0) (from ?? list).splice((from ?? list).indexOf(src), 1);
    else src.count = (src.count ?? 1) - n;
    return;
  }
  if (m.swapWith) {
    const other = list.find((i) => i.key === m.swapWith)!;
    if (fromSlot) {
      fromSlot.item = { ...other };
      list.splice(list.indexOf(other), 1);
    } else {
      Object.assign(other, { x: src.x, y: src.y });
      if (from && from !== list) {
        list.splice(list.indexOf(other), 1);
        from.push(other);
      }
    }
  }
  if (fromSlot) {
    fromSlot.item = m.swapWith ? (fromSlot.item ?? null) : null;
    list.push(moved);
  } else if (m.split) {
    src.count = (src.count ?? 1) - m.count;
    list.push({ ...moved, key: `${src.key}-${og.ui.tick()}` });
  } else if (from && from !== list) {
    from.splice(from.indexOf(src), 1);
    list.push(moved);
  } else Object.assign(src, { x: m.x, y: m.y, rotated: m.rotated });
}

// ── gear: paperdoll, backpack and stash ─────────────────────────────────

export function gearScreen(): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const portrait = vp.breakpoint === "phonePortrait";
  screen("Gear", "Inventory", "Drag to move · Shift splits a stack · R turns", () => {
    const body = () => {
      const doll = og.ui.paperdoll("Doll", {
        slots: p.slots,
        title: "Aria Vell",
        subtitle: "Level 42 Paladin",
        rating: "812",
        ratingLabel: "Item power",
        statsTitle: "Attributes",
        figureAspect: 0.5,
        compare: !!p.carry || p.selected === "helm",
        stats: [
          { label: "Armor", value: "1,240", delta: p.carry?.tag === "helmet" || p.selected === "helm" ? 48 : 0 },
          { label: "Strength", value: "312", delta: p.carry?.tag === "helmet" || p.selected === "helm" ? 12 : 0 },
          { label: "Fire resistance", value: "28%", delta: p.carry?.tag === "helmet" || p.selected === "helm" ? -6 : 0, deltaText: "−6%" },
          { label: "Move speed", value: "100%" },
          { label: "Cooldowns", value: "8.2 s", delta: -0.4, deltaText: "−0.4 s", lowerIsBetter: true },
        ],
        ...(p.carry ? { carry: p.carry } : {}),
      }, () => {
        // The figure: the game's character art; the kit draws a standing silhouette that fills the box.
        og.ui.column("Body", { width: grow, height: grow, align: "end", cross: "center", gap: 4 }, () => {
          og.ui.column("Head", { width: { kind: "percent", fraction: 0.26 }, aspect: 1, radius: 999, background: { token: "control" } }, () => {});
          og.ui.column("Torso", { width: { kind: "percent", fraction: 0.5 }, height: { kind: "percent", fraction: 0.4 }, radius: 24, background: { token: "control" } }, () => {});
          og.ui.row("Legs", { width: { kind: "percent", fraction: 0.44 }, height: { kind: "percent", fraction: 0.36 }, gap: 6 }, () => {
            og.ui.column("L", { width: grow, height: grow, radius: 12, background: { token: "control" } }, () => {});
            og.ui.column("R", { width: grow, height: grow, radius: 12, background: { token: "control" } }, () => {});
          });
        });
      });
      if (doll.equip) {
        const slot = p.slots.find((s) => s.key === doll.equip!.slot)!;
        const fromBag = p.bag.find((i) => i.key === doll.equip!.key);
        const fromStash = p.stash.find((i) => i.key === doll.equip!.key);
        const fromSlot = p.slots.find((s) => s.key === doll.equip!.fromSlot);
        const was = slot.item ?? null;
        const item = fromBag ?? fromStash ?? fromSlot?.item ?? null;
        if (item) {
          if (fromBag) p.bag.splice(p.bag.indexOf(fromBag), 1);
          if (fromStash) p.stash.splice(p.stash.indexOf(fromStash), 1);
          if (fromSlot) fromSlot.item = was && was.tag && (fromSlot.accept ?? []).includes(was.tag) ? was : null;
          slot.item = { ...item, x: 0, y: 0, rotated: false };
          if (was && !fromSlot) (fromStash ? p.stash : p.bag).push({ ...was, x: item.x ?? 0, y: item.y ?? 0, w: 1, h: 1 });
        }
        p.carry = null;
      }
      og.ui.column("Bags", { gap: t.space.sm }, () => {
        og.ui.label("BagTitle", "Backpack", { step: "lg", font: "display", heading: true });
        // Eight columns fill a phone in portrait at the finger's 44 px floor, so the grids need no sideways scroll.
        const cell = portrait ? { cellSize: Math.max(44, Math.floor((vp.width - vp.safeArea.left - vp.safeArea.right - 32) / 8)) } : {};
        const pick = p.pick;
        p.pick = null;
        const bag = og.ui.inventoryGrid("Bag", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 7 : 6, items: p.bag, label: "Backpack", selected: p.selected, highlight: p.highlight, ...(p.carry && p.carry.source !== "Bag" ? { carry: p.carry } : {}), ...(pick ? { pick } : {}) });
        og.ui.label("StashTitle", "Stash", { step: "lg", font: "display", heading: true });
        const stash = og.ui.inventoryGrid("Stash", { ...cell, columns: portrait ? 8 : 10, rows: portrait ? 2 : 3, capacity: portrait ? 12 : 24, items: p.stash, label: "Stash", ...(p.carry && p.carry.source !== "Stash" ? { carry: p.carry } : {}) });
        for (const [r, list, other] of [[bag, p.bag, p.stash], [stash, p.stash, p.bag]] as const) {
          if (r.move) {
            const fromSlot = r.move.fromSlot ? p.slots.find((s) => s.key === r.move!.fromSlot) ?? null : null;
            applyMove(list, r.move.from === (list === p.bag ? "Bag" : "Stash") ? list : fromSlot ? null : other, r.move, fromSlot);
            p.carry = null;
          }
          if (r.selected) p.selected = r.selected;
          if (r.quick) {
            const it = list.find((i) => i.key === r.quick)!;
            list.splice(list.indexOf(it), 1);
            other.push({ ...it, x: other.length % 10, y: Math.floor(other.length / 10) });
          }
          if (r.menu) p.menu = { key: r.menu, at: r.menuAt };
        }
        // What is in the hand, handed to the other places next tick; one let go (released) stays one tick more,
        // so the grid or paperdoll under the pointer (built before this one, or after) takes it, then it is gone.
        p.carry = bag.carried ?? stash.carried ?? doll.carried ?? null;
      });
      // The item's menu: a long press or the details action.
      const m = og.ui.contextMenu("ItemMenu", !!p.menu, p.menu?.at ?? { x: 0, y: 0 }, {
        entries: [{ id: "use", label: "Use" }, { id: "split", label: "Split stack" }, { id: "drop", label: "Drop" }],
      });
      if (m.chosen === "split" && p.menu) p.pick = { key: p.menu.key, count: 1 };
      if (m.chosen || m.dismissed) p.menu = null;
    };
    // Doll and grids side by side where both fit (a desktop), one above the other otherwise.
    if (vp.width >= 1100) og.ui.row("Split", { width: grow, gap: t.space.xl, cross: "start" }, body);
    else og.ui.column("Stack", { width: grow, gap: t.space.lg, cross: "start" }, body);
  }, [{ action: "confirm", hint: "Pick up / place" }, { action: "rotate", hint: "Rotate" }, { action: "details", hint: "Item menu" }, { action: "back", hint: "Put back" }]);
}
