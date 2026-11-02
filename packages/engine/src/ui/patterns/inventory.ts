// Inventory grids and paperdolls (docs/ui.md §11 genre patterns, Appendix
// D: grid inventory with rotate, split, stack, quick transfer; drag and
// drop of items). Items cover several cells, are drawn on wells in their
// tier's colour with pips, and are carried by the pointer, a finger, keys
// or a pad. What is carried is the component's own state (og.ui.memory,
// so a snapshot carries it); what the player did is answered as a move
// for the game to apply to its viewmodel, which stays the one truth.

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, clamp, flags, frame, grow, inside, keyHash, keyOf, kit, lastInteraction, line, mix, over, overProps, paint, pips, pointer, px,
  rarityColor, rarityPips, recallPath, rememberPath, role, rtl, shadow, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type SymbolKind,
} from "./core.ts";

// ── the hand: what is carried ───────────────────────────────────────────

/** A carry's state, kept in the UI's memory under the component's root. */
interface Hand {
  /** The carried key's hash (0: nothing). */
  key: number;
  count: number;
  rot: number;
  split: number;
  byFocus: number;
  /** The cell of the item the pointer grabbed, so the footprint keeps under the finger. */
  gx: number;
  gy: number;
  /** A press: the key it began on, where and when. */
  pk: number;
  px: number;
  py: number;
  pt: number;
  shift: number;
  /** A long press already asked for the menu. */
  menu: number;
  /** The last pointer click: key and tick (a double press). */
  ck: number;
  ct: number;
  /** A refusal's tick (the shake). */
  rt: number;
  /** The grid's cursor for keys and pads: a cell. */
  cx: number;
  cy: number;
  /** The tick it was last built, plus one (0 never). */
  seen: number;
}

const HAND_KEYS: Array<keyof Hand> = ["key", "count", "rot", "split", "byFocus", "gx", "gy", "pk", "px", "py", "pt", "shift", "menu", "ck", "ct", "rt", "cx", "cy", "seen"];

/** The hand as the last tick left it. A screen that was not built last tick (closed, then opened again) starts with
 *  nothing in the hand and no press under way; only its cursor is kept. */
function readHand(): Hand {
  const h = {} as Hand;
  for (const k of HAND_KEYS) h[k] = kit.memory(`~h.${k}`, 0);
  if (h.seen !== 0 && kit.tick() + 1 - h.seen > 1) {
    for (const k of HAND_KEYS) if (k !== "cx" && k !== "cy") h[k] = 0;
  }
  return h;
}

function writeHand(h: Hand): void {
  h.seen = kit.tick() + 1;
  for (const k of HAND_KEYS) kit.remember(`~h.${k}`, h[k]);
}

const LONG_PRESS = 30;
const DOUBLE = 20;

/** Pointer travel before a press becomes a drag. */
function threshold(): number {
  return frame().touch ? 10 : 5;
}

/** The pointer ended this tick a press that never moved (a click), inside `r`. */
function pointerClicked(r: Api.UiRect | null): boolean {
  const p = pointer();
  return p.released && inside(r, p.x, p.y);
}

// ── geometry ────────────────────────────────────────────────────────────

interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

function footprintOf(it: Api.GridItem): Box {
  const w = Math.max(1, it.w ?? 1);
  const h = Math.max(1, it.h ?? 1);
  return it.rotated ? { x: it.x ?? 0, y: it.y ?? 0, w: h, h: w } : { x: it.x ?? 0, y: it.y ?? 0, w, h };
}

const hits = (a: Box, b: Box): boolean => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

/** The cell under a point, from the cells' laid-out rectangle (mirrored under right to left). */
function cellAt(b: Api.UiRect | null, cs: number, x: number, y: number): { cx: number; cy: number } | null {
  if (!b || !inside(b, x, y)) return null;
  const lx = rtl() ? b.x + b.width - x : x - b.x;
  return { cx: Math.floor(lx / cs), cy: Math.floor((y - b.y) / cs) };
}

type Drop = "valid" | "merge" | "swap" | "invalid";

// ── an item's tile ──────────────────────────────────────────────────────

/** The tile's look: a well in the tier's colour, its art or silhouette, pips, a count, marks and a charge bar. */
function tile(it: Api.GridItem, wpx: number, hpx: number, cs: number, opts: { lifted?: boolean; dim?: boolean; lit?: boolean; selected?: boolean; raised?: boolean }): void {
  const tier = rarityColor(it.rarity);
  const base = tier ?? role("lineStrong");
  const r = Math.max(2, Math.round(frame().t.small));
  // The well: the tier's colour rising from the foot, an edge in it (two px once epic), so the tier reads at a glance.
  kit.setDraw({
    shape: { kind: "rect", radii: [r, r, r, r] },
    fills: [
      { kind: "solid", color: token("well") },
      { kind: "gradient", type: "linear", from: { x: 0.5, y: 1 }, to: { x: 0.5, y: 0 }, stops: [{ offset: 0, color: paint(base, tier ? 0.42 : 0.12) }, { offset: 1, color: paint(base, tier ? 0.08 : 0.02) }] },
      ...(it.unusable ? [{ kind: "solid", color: paint(role("danger"), 0.22) } as Api.Fill] : []),
    ],
    strokes: [line(rarityPips(it.rarity) >= 4 ? 2 : 1, paint(tier ? mix(base, role("ink"), 0.1) : role("line"), opts.dim ? 0.4 : 0.95))],
    ...(opts.raised ? { effects: [shadow(u(10), u(18), 0.55)] } : opts.selected ? { effects: [shadow(u(4), u(10), 0.6)] } : {}),
  });
  // The art, turned with the item when it lies turned: it is laid out unturned and rotated a quarter.
  const aw = it.rotated ? hpx : wpx;
  const ah = it.rotated ? wpx : hpx;
  artSlot("Art", "Inventory.art", {
    texture: it.image ?? null,
    kind: "item",
    icon: it.icon,
    rarity: it.rarity,
    radius: 0,
    width: px(Math.max(1, aw - 6)),
    height: px(Math.max(1, ah - 6)),
  });
  kit.props(overProps({ insets: { centerX: true, centerY: true }, transform: { rotate: it.rotated ? Math.PI / 2 : 0 }, inert: true, opacity: opts.lifted ? 0.35 : opts.dim ? 0.45 : 1 }));
  const mk = Math.max(12, Math.round(cs * 0.26));
  if (rarityPips(it.rarity) > 0) {
    kit.open("Pips", { direction: "row" });
    pips("Row", it.rarity, Math.max(5, Math.round(cs * 0.1)));
    kit.close();
    over({ insets: { start: 4, top: 4 }, inert: true });
  }
  const marks: Array<[string, () => void]> = [];
  if (it.locked) marks.push(["Lock", () => symbol("Sym", "lock", mk, role("ink"), { icon: "lock" })]);
  if (it.equipped) marks.push(["Equipped", () => symbol("Sym", "check", mk, role("success"))]);
  if (it.fresh) marks.push(["Fresh", () => symbol("Sym", "dot", Math.round(mk * 0.6), role("warn"))]);
  if (marks.length > 0) {
    kit.open("Marks", { direction: "row", gap: 2, padding: xy(3, 2), radius: 4, background: token("shadow", 0.6) });
    for (const [id, draw] of marks) {
      kit.open(id, { direction: "row" });
      draw();
      kit.close();
    }
    kit.close();
    over({ insets: { end: 3, top: 3 }, inert: true });
  }
  const stacks = (it.maxStack ?? 0) > 1;
  const count = it.count ?? 0;
  if (count >= 2 || (stacks && count >= 1)) {
    kit.open("Count", { direction: "row", padding: xy(4, 0), radius: 4, background: token("shadow", 0.72), part: "Inventory.count", partClass: "badge" });
    text("Text", count.toString(), { step: st(12), nowrap: true, font: "mono", color: count >= (it.maxStack ?? Infinity) ? "warn" : "ink" });
    kit.close();
    over({ insets: { end: 3, bottom: it.charge !== undefined && it.charge !== null ? 7 : 3 }, inert: true });
  }
  if (it.charge !== undefined && it.charge !== null) {
    const c = clamp(it.charge, 0, 1);
    kit.open("Charge", { width: grow(), height: px(3), radius: 2, background: token("shadow", 0.6), part: "Inventory.charge", partClass: "bar" });
    kit.open("Fill", { width: px(Math.max(1, Math.round((wpx - 8) * c))), height: px(3), radius: 2, background: token(c < 0.25 ? "danger" : "success") });
    kit.close();
    kit.close();
    over({ insets: { start: 4, end: 4, bottom: 3 }, inert: true });
  }
  if (opts.lit) {
    kit.open("Lit", { width: grow(), height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [line(2, token("highlight"))], effects: [{ kind: "innerGlow", blur: 10, spread: 0, color: token("highlight", 0.45) }] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
  }
  if (opts.selected) {
    // Selected: an edge lit in the focus ink with a halo fading outward, the art left clear (the tile also lifts on
    // its shadow, above).
    kit.open("Selected", { width: grow(), height: grow(), part: "Inventory.selected", partClass: "slot" });
    kit.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [line(2, token("focus")), line(2, token("focus", 0.45), "outside"), line(6, token("focus", 0.14), "outside")] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
  }
  if (opts.lifted) {
    kit.open("Lifted", { width: grow(), height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, strokes: [line(2, token("inkMuted"), "inside", { dash: 5, gap: 4 })] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
  }
}

/** What a screen reader says an item is. */
function itemName(it: Api.GridItem, d: { lockedLabel?: string; unusableLabel?: string }): string {
  const bits = [it.name ?? it.key];
  if ((it.count ?? 0) >= 2) bits.push(`×${it.count}`);
  if (it.rarity && it.rarity !== "none") bits.push(it.rarity);
  if (it.locked) bits.push(d.lockedLabel ?? "Locked");
  if (it.unusable) bits.push(d.unusableLabel ?? "Cannot use");
  return bits.join(", ");
}

/** The item's state groups (docs/ui.md §7.3): selection, carry and highlight, each one that shows. */
function itemStates(lifted: boolean, hl: "normal" | "lit" | "dim"): Api.ElementProps {
  return {
    states: [
      { name: "carry", states: [vstate("resting"), vstate("lifted", [{ prop: "scale", value: 0.94 }])], transitions: [] },
      { name: "highlight", states: [vstate("normal"), vstate("lit", [{ prop: "z", value: 1 }]), vstate("dim", [{ prop: "opacity", value: 0.55 }])], transitions: [] },
    ],
    state: [
      { group: "carry", state: lifted ? "lifted" : "resting" },
      { group: "highlight", state: hl },
    ],
  };
}

/** What a drop would do, as the carried item says it: its tone, its mark and the mark's words. */
export interface Verdict {
  drop: Drop;
  /** A merge's count (the +n it adds to the stack). */
  n?: number;
}

/**
 * A verdict's look on what is carried (the ghost sits over its footprint, so the footprint alone is hidden): an edge in
 * its tone and a badge at the top end corner that differs for each: a tick (valid), +n (merge), the passing arrows
 * (swap), ⊘ over a danger shade (invalid).
 */
export function verdictMarks(v: Verdict, w: number, h: number, cs: number): void {
  const tone: Api.ThemeColor = v.drop === "valid" ? "success" : v.drop === "merge" ? "gold" : v.drop === "swap" ? "info" : "danger";
  kit.open("Verdict", { width: px(w), height: px(h), part: `Inventory.verdict${flags(v.drop)}`, partClass: "slot" });
  kit.setDraw({ shape: { kind: "rect", radii: [4, 4, 4, 4] }, fills: [{ kind: "solid", color: token(v.drop === "invalid" ? "danger" : tone, v.drop === "invalid" ? 0.34 : 0.1) }], strokes: [line(3, token(tone)), line(1, token("shadow", 0.7), "outside")] });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true, z: 3 });
  const b = Math.max(22, Math.round(cs * 0.48));
  kit.open("Badge", { direction: "row", height: px(b), width: v.drop === "merge" ? null : px(b), padding: xy(v.drop === "merge" ? Math.round(b * 0.28) : 0, 0), align: "center", cross: "center", part: `Inventory.verdictBadge${flags(v.drop)}`, partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token(tone) }], strokes: [line(2, token("shadow", 0.75), "outside")], effects: [shadow(2, 6, 0.5)] });
  // Gold is light in most styles: the merge's count is in the dark ink that reads on it.
  const ink = role(v.drop === "merge" ? "inkInverse" : "onPrimary");
  if (v.drop === "valid") symbol("Mark", "check", Math.round(b * 0.62), ink);
  else if (v.drop === "swap") symbol("Mark", "swap", Math.round(b * 0.7), ink);
  else if (v.drop === "invalid") symbol("Mark", "ban", Math.round(b * 0.66), ink);
  else text("Count", `+${v.n ?? 1}`, { step: st(Math.max(12, b * 0.5)), color: ink, font: "mono", nowrap: true });
  kit.close();
  over({ insets: { end: -Math.round(b * 0.3), top: -Math.round(b * 0.3) }, inert: true, z: 4 });
  kit.semantics({ role: "status", name: v.drop === "merge" ? `merge +${v.n ?? 1}` : v.drop });
}

/** The carried item under the pointer: drawn at the root over everything, lifted and shadowed, with what a drop would do. */
function ghost(it: Api.GridItem, rot: boolean, count: number, cs: number, gx: number, gy: number, at: { x: number; y: number }, root: Api.UiRect | null, verdict: Verdict | null = null, outside = false): void {
  if (!root) return;
  // Off its grid (over a paperdoll's slot, another grid) it is carried as a one-cell icon centred on the pointer, so
  // what it is over shows round it: the slot's accept or reject, the other grid's footprint.
  const shown = outside ? { ...it, rotated: false, w: 1, h: 1, count } : { ...it, rotated: rot, count };
  const f = footprintOf(shown);
  const w = f.w * cs;
  const h = f.h * cs;
  const lx = rtl() ? root.x + root.width - at.x : at.x - root.x;
  const start = lx - (outside ? 0.5 : gx + 0.5) * cs;
  const top = at.y - root.y - (outside ? 0.5 : gy + 0.5) * cs;
  kit.open("Ghost", { width: px(w), height: px(h), part: `Inventory.ghost${flags(`rarity=${it.rarity ?? "none"}`)}`, partClass: "slot" });
  tile(shown, w, h, cs, { raised: true });
  if (verdict) verdictMarks(verdict, w, h, cs);
  kit.close();
  kit.props(overProps({ insets: { start, top }, inert: true, z: 50, transform: { scaleX: still() ? 1 : 1.06, scaleY: still() ? 1 : 1.06, rotate: still() ? 0 : -0.035 } }));
}

/**
 * A locked cell or slot in the open element, `w`×`h`: sunk below the
 * empty cells (darker, a shadow along its top inner edge), hatched, and a
 * lock large enough to read.
 */
export function lockedCell(w: number, h: number): void {
  kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("shadow"), 0.55)) }], strokes: [line(1, token("shadow", 0.6))] });
  // The hatch: diagonals one path, its joins along the edges (under the frame's line).
  const g = Math.max(6, Math.round(Math.min(w, h) / 5));
  const pts: Api.Vector2[] = [];
  for (let d = g, k = 0; d < w + h; d += g, k++) {
    const a = { x: Math.min(d, w), y: Math.max(0, d - w) };
    const b = { x: Math.max(0, d - h), y: Math.min(d, h) };
    pts.push(...(k % 2 ? [b, a] : [a, b]));
  }
  kit.open("Hatch", { width: px(w), height: px(h) });
  kit.setDraw({ shape: { kind: "path", points: pts, width: 1, cap: "butt", join: "miter" }, fills: [{ kind: "solid", color: token("inkFaint", 0.16) }] });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  kit.open("Inset", { width: px(w), height: px(Math.max(3, Math.round(h * 0.12))) });
  kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("shadow", 0.55)], [1, token("shadow", 0)]])] });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  symbol("Lock", "lock", Math.round(Math.min(w, h) * 0.44), role("inkMuted"), { icon: "lock" });
}

// ── the grid ────────────────────────────────────────────────────────────

/**
 * A grid inventory (Diablo, Path of Exile, Escape from Tarkov, Resident
 * Evil 4): see `fn inventoryGrid` in crates/og-api/api/ui.ogt.
 */
export function inventoryGrid(id: string, desc: Api.InventoryGridDesc): Api.InventoryGridResult {
  const { touch, viewport } = frame();
  const cols = Math.max(1, desc.columns ?? 10);
  const rows = Math.max(1, desc.rows ?? 6);
  const items = desc.items ?? [];
  const cs = Math.max(touch ? 44 : 0, desc.cellSize && desc.cellSize > 0 ? desc.cellSize : u(touch ? 48 : 52));
  const cap = desc.capacity && desc.capacity > 0 ? Math.min(desc.capacity, cols * rows) : cols * rows;
  const keys = items.map((i) => i.key ?? "");
  const byKey = new Map(items.map((i) => [i.key ?? "", i]));
  const out: Mut<Api.InventoryGridResult> = { selected: null, used: null, menu: null, menuAt: { x: 0, y: 0 }, quick: null, move: null, carried: null, hovered: null, hoveredRect: null, refused: false };
  const p = pointer();
  const tick = kit.tick();

  kit.open(id, { direction: "column", gap: u(8), cross: "start", part: "Inventory.frame", partClass: "panel" });
  const rootPath = recallPath(`${id}~root`);
  const h = readHand();

  // ── this grid's own carry, and one from elsewhere ──
  let own = keyOf(keys, h.key);
  if (h.key && !own) h.key = 0; // the item left the viewmodel
  if (desc.pick && byKey.has(desc.pick.key ?? "")) {
    const it = byKey.get(desc.pick.key!)!;
    if (!it.locked) {
      own = it.key ?? "";
      h.key = keyHash(own);
      h.count = desc.pick.count && desc.pick.count > 0 ? Math.min(desc.pick.count, it.count || 1) : it.count || 1;
      h.split = h.count < (it.count || 1) ? 1 : 0;
      h.rot = it.rotated ? 1 : 0;
      h.byFocus = p.down ? 0 : 1;
      h.gx = 0;
      h.gy = 0;
      h.cx = it.x ?? 0;
      h.cy = it.y ?? 0;
      kit.feedback("select", -1);
    }
  }
  const foreign = !own && desc.carry && desc.carry.source !== id ? desc.carry : null;
  const carriedItem = own ? byKey.get(own)! : null;
  if (own && kit.action("rotate")) {
    h.rot = h.rot ? 0 : 1;
    kit.feedback("toggle", -1);
  }

  const cellsKey = `${id}~cells`;
  const cellsRect = kit.bounds(recallPath(cellsKey));
  const gridFocused = lastInteraction(`${id}~grid`).focused;
  const carrying = !!own || !!foreign;
  const byFocus = own ? h.byFocus === 1 : !!foreign?.byFocus;
  const occupant = (x: number, y: number): Api.GridItem | null => items.find((i) => i.key !== own && hits(footprintOf(i), { x, y, w: 1, h: 1 })) ?? null;

  // The carried footprint's size (turned when carried turned).
  const carrySize = (): { w: number; h: number } => {
    const held = own ? byKey.get(own) : undefined;
    if (held) {
      const w = Math.max(1, held.w ?? 1);
      const hh = Math.max(1, held.h ?? 1);
      return h.rot ? { w: hh, h: w } : { w, h: hh };
    }
    return { w: Math.max(1, foreign!.w ?? 1), h: Math.max(1, foreign!.h ?? 1) };
  };
  // Keys and pads carry by the grid's cursor: the footprint's top start cell, kept inside the grid.
  if (carrying && byFocus) {
    const s = carrySize();
    h.cx = clamp(h.cx, 0, Math.max(0, cols - s.w));
    h.cy = clamp(h.cy, 0, Math.max(0, rows - s.h));
  }
  h.cx = clamp(h.cx, 0, cols - 1);
  h.cy = clamp(h.cy, 0, rows - 1);

  let fp: Box | null = null;
  if (carrying) {
    const s = carrySize();
    if (byFocus) {
      if (gridFocused || own) fp = { x: h.cx, y: h.cy, w: s.w, h: s.h };
    } else {
      const c = cellAt(cellsRect, cs, own ? p.x : foreign!.x, own ? p.y : foreign!.y);
      if (c) fp = { x: c.cx - (own ? h.gx : Math.floor(s.w / 2)), y: c.cy - (own ? h.gy : Math.floor(s.h / 2)), w: s.w, h: s.h };
    }
  }

  /** Whether the carried item may land at `b`, and how. */
  const judge = (b: Box): { drop: Drop; other: Api.GridItem | null } => {
    if (b.x < 0 || b.y < 0 || b.x + b.w > cols || b.y + b.h > rows) return { drop: "invalid", other: null };
    for (let y = b.y; y < b.y + b.h; y++) for (let x = b.x; x < b.x + b.w; x++) if (y * cols + x >= cap) return { drop: "invalid", other: null };
    const under = items.filter((i) => i.key !== own && hits(footprintOf(i), b));
    if (under.length === 0) return { drop: "valid", other: null };
    if (under.length > 1) return { drop: "invalid", other: null };
    const o = under[0]!;
    const tag = own ? carriedItem!.tag : foreign!.tag;
    if (o.tag && o.tag === tag && (o.maxStack ?? 0) > 1 && (o.count ?? 1) < (o.maxStack ?? 0)) return { drop: "merge", other: o };
    if (o.locked) return { drop: "invalid", other: o };
    return { drop: "swap", other: o };
  };
  const verdict = fp ? judge(fp) : null;
  const verdictOf = (): Verdict | null => {
    if (!verdict) return null;
    if (verdict.drop !== "merge") return { drop: verdict.drop };
    const o = verdict.other!;
    const carried = own ? h.count || carriedItem!.count || 1 : foreign!.count ?? 1;
    return { drop: "merge", n: Math.min(carried, (o.maxStack ?? 0) - (o.count ?? 1)) };
  };

  /** Land the carried item at the footprint: a move, or a refusal. */
  const land = (): void => {
    if (!fp || !verdict || verdict.drop === "invalid") {
      out.refused = true;
      h.rt = tick;
      kit.feedback("error", -1);
      // A drop let go there goes home; keys and pads keep it in hand to choose again.
      if (own && !h.byFocus) h.key = 0;
      return;
    }
    const key = own ?? foreign!.key ?? "";
    const rotated = own ? h.rot === 1 : !!foreign!.rotated;
    // A carry that lands where it lay, unturned and whole, is no move.
    if (own && verdict.drop === "valid" && fp.x === carriedItem!.x && fp.y === carriedItem!.y && rotated === !!carriedItem!.rotated && !h.split) {
      h.key = 0;
      kit.feedback("cancel", -1);
      return;
    }
    out.move = {
      key,
      from: own ? id : foreign!.source ?? "",
      fromSlot: own ? "" : foreign!.slot ?? "",
      x: fp.x,
      y: fp.y,
      rotated,
      count: own ? h.count : foreign!.count ?? 1,
      split: own ? h.split === 1 : false,
      mergeInto: verdict.drop === "merge" ? verdict.other!.key ?? null : null,
      swapWith: verdict.drop === "swap" ? verdict.other!.key ?? null : null,
    };
    h.key = 0;
    kit.feedback("confirm", -1);
  };

  // The cursor's place without a carry: the item under it, else its cell.
  const cursorItem = !carrying || !byFocus ? occupant(h.cx, h.cy) : null;
  const cursorBox: Box = carrying && byFocus && fp ? fp : cursorItem ? footprintOf(cursorItem) : { x: h.cx, y: h.cy, w: 1, h: 1 };
  // The arrows the grid keeps while its cursor can still move that way; at an edge the focus leaves it.
  const arrows: Api.ArrowClaim = carrying && byFocus
    ? { left: cursorBox.x > 0, right: cursorBox.x + cursorBox.w < cols, up: cursorBox.y > 0, down: cursorBox.y + cursorBox.h < rows }
    : { left: cursorBox.x > 0, right: cursorBox.x + cursorBox.w < cols, up: cursorBox.y > 0, down: cursorBox.y + cursorBox.h < rows };

  // ── the grid ──
  const W = cols * cs;
  const H = rows * cs;
  const room = viewport.width - viewport.safeArea.left - viewport.safeArea.right - u(32);
  const scrolls = W > room;
  if (scrolls) kit.openScroll("View", { width: px(Math.max(cs * 4, room)), axes: "horizontal", padding: xy(0, 0), gap: 0 });
  const used = new Set<number>();
  for (const it of items) {
    const f = footprintOf(it);
    for (let y = f.y; y < f.y + f.h; y++) for (let x = f.x; x < f.x + f.w; x++) used.add(y * cols + x);
  }
  const cursorWords = cursorItem ? itemName(cursorItem, desc) : h.cx + h.cy * cols >= cap ? desc.lockedLabel ?? "Locked" : desc.emptyLabel ?? "Empty";
  kit.open("Cells", {
    width: px(W),
    height: px(H),
    interactive: true,
    focusable: true,
    arrows,
    part: `Inventory.grid${flags(gridFocused && "focused", carrying && "carrying")}`,
    partClass: "slot",
    radius: frame().t.small,
  });
  let grid: Api.Response;
  try {
    // The ground: one well; the cells' hairlines are one path (a box per cell would cost an element each).
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("well") }], strokes: [line(1, token("lineStrong"))] });
    const pts: Api.Vector2[] = [];
    for (let x = 1; x < cols; x++) pts.push(...(x % 2 ? [{ x: x * cs, y: 0 }, { x: x * cs, y: H }] : [{ x: x * cs, y: H }, { x: x * cs, y: 0 }]));
    // Along the top or bottom edge to the end corner, then the rows back and forth (each join runs on an edge, under the frame).
    pts.push({ x: W, y: pts.length > 0 ? pts[pts.length - 1]!.y : 0 });
    for (let y = 1; y < rows; y++) pts.push(...(y % 2 ? [{ x: W, y: y * cs }, { x: 0, y: y * cs }] : [{ x: 0, y: y * cs }, { x: W, y: y * cs }]));
    kit.open("Lines", { width: px(W), height: px(H) });
    kit.setDraw({ shape: { kind: "path", points: pts, width: 1, cap: "butt", join: "miter" }, fills: [{ kind: "solid", color: token("line", 0.8) }] });
    kit.close();
    over({ insets: { start: 0, top: 0 }, inert: true });
    // Cells past the capacity: locked, a lock each.
    for (let i = cap; i < cols * rows; i++) {
      const cx = i % cols;
      const cy = Math.floor(i / cols);
      kit.open(kit.idi("Locked", i), { width: px(cs), height: px(cs), align: "center", cross: "center", part: "Inventory.cell[locked]", partClass: "slot" });
      lockedCell(cs, cs);
      kit.close();
      over({ insets: { start: cx * cs, top: cy * cs }, inert: true });
      kit.semantics({ role: "gridcell", name: desc.lockedLabel ?? "Locked", disabled: true });
    }
    // Items: the pointer and a finger reach each; keys and pads move the grid's cursor over them.
    for (const it of items) {
      const key = it.key ?? "";
      const f = footprintOf(it);
      const wpx = f.w * cs;
      const hpx = f.h * cs;
      const lifted = own === key;
      const selected = (desc.selected ?? "") === key;
      const hl: "normal" | "lit" | "dim" = desc.highlight ? (it.tag === desc.highlight ? "lit" : "dim") : "normal";
      const shaking = h.rt > 0 && tick - h.rt < 18 && h.pk === keyHash(key);
      const shake = shaking && !still() ? Math.sin((tick - h.rt) * 1.6) * (18 - (tick - h.rt)) * 0.5 : 0;
      const r = surface(key, {
        part: "Inventory.item",
        cls: "slot",
        flags: [`rarity=${it.rarity ?? "none"}`, selected ? "selected" : "", lifted ? "lifted" : "", it.locked ? "locked" : "", it.unusable ? "unusable" : "", hl === "dim" ? "dim" : ""],
        selected,
        fill: null,
        washBy: 0.1,
        focusable: false,
        width: px(wpx),
        height: px(hpx),
        radius: frame().t.small,
        semantics: { role: "gridcell", name: itemName(it, desc), selected },
        layer: { insets: { start: f.x * cs, top: f.y * cs }, transform: { x: shake }, ...itemStates(lifted, hl) },
      }, () => tile(it, wpx, hpx, cs, { lifted, dim: hl === "dim", lit: hl === "lit", selected }));
      const path = kit.lastPath();
      const ix = lastInteraction(key);
      if (ix.hovered && !lifted) {
        out.hovered = key;
        out.hoveredRect = kit.bounds(path);
      }
      // A press begins: remember where and when, and whether Shift splits it.
      if (r.pressed && p.pressed) {
        h.pk = keyHash(key);
        h.px = p.x;
        h.py = p.y;
        h.pt = tick;
        h.shift = p.shift ? 1 : 0;
        h.menu = 0;
        h.cx = f.x;
        h.cy = f.y;
      }
      if (r.clicked && !carrying) {
        if (p.shift) out.quick = key;
        else if (h.ck === keyHash(key) && tick - h.ct <= DOUBLE) {
          out.used = key;
          h.ck = 0;
        } else {
          out.selected = key;
          h.ck = keyHash(key);
          h.ct = tick;
          kit.feedback("select", -1);
        }
      }
    }
    // The footprint: where the carried item would land, in what it would do there.
    if (fp && verdict) {
      const tone: Api.ThemeColor = verdict.drop === "valid" ? "success" : verdict.drop === "merge" ? "gold" : verdict.drop === "swap" ? "info" : "danger";
      const fx = clamp(fp.x, 0, cols - 1);
      const fy = clamp(fp.y, 0, rows - 1);
      const fw = Math.max(1, Math.min(fp.w, cols - fx));
      const fh = Math.max(1, Math.min(fp.h, rows - fy));
      const shake = h.rt > 0 && tick - h.rt < 18 && !still() ? Math.sin((tick - h.rt) * 1.6) * (18 - (tick - h.rt)) * 0.5 : 0;
      kit.open("Footprint", { width: px(fw * cs), height: px(fh * cs), align: "center", cross: "center", part: `Inventory.footprint${flags(verdict.drop)}`, partClass: "slot" });
      kit.setDraw({ shape: { kind: "rect", radii: [3, 3, 3, 3] }, fills: [{ kind: "solid", color: token(tone, 0.3) }], strokes: [line(2, token(tone))] });
      // Its marks where nothing carried covers it (keys and pads, or a carry from another grid or the doll).
      if (byFocus || !own) verdictMarks(verdictOf()!, fw * cs, fh * cs, cs);
      kit.close();
      kit.props(overProps({
        insets: { start: fx * cs, top: fy * cs },
        inert: true,
        z: 20,
        transform: { x: shake },
        states: [{ name: "drop", states: [vstate("valid"), vstate("merge"), vstate("swap"), vstate("invalid")], transitions: [] }],
        state: [{ group: "drop", state: verdict.drop }],
      }));
      kit.semantics({ role: "status", name: verdict.drop });
    }
    // The cursor of keys and pads: a ring in the focus ink around the item or cell it is on.
    if (gridFocused && !(carrying && byFocus)) {
      const b = cursorBox;
      kit.open("Cursor", { width: px(b.w * cs), height: px(b.h * cs), part: "Inventory.cursor", partClass: "slot" });
      // A ring and a soft halo outside it, both strokes: a glow effect on a shape with no fill floods the item under it.
      kit.setDraw({ shape: { kind: "rect", radii: [4, 4, 4, 4] }, strokes: [line(3, token("focus"), "inside"), line(3, token("focus", 0.35), "outside"), line(7, token("focus", 0.12), "outside")] });
      kit.close();
      kit.props(overProps({ insets: { start: b.x * cs, top: b.y * cs }, inert: true, z: 30, layoutMotion: { kind: "token", token: "ui" } }));
    }
  } finally {
    grid = kit.close() as Api.Response;
  }
  const cellsPath = kit.lastPath();
  rememberPath(cellsKey, cellsPath);
  kit.remember(`${id}~grid~ix`, (grid.hovered ? 1 : 0) | (grid.pressed ? 2 : 0) | (grid.focused ? 4 : 0));
  kit.props({
    states: [{ name: "interaction", states: [vstate("normal"), vstate("hover"), vstate("pressed"), vstate("focused"), vstate("disabled")], transitions: [] }],
  });
  kit.semantics({ role: "grid", name: `${desc.label ?? ""}${desc.label ? ", " : ""}${used.size} / ${cap}`, value: `${cursorWords}, ${cursorBox.x + 1}, ${cursorBox.y + 1}` });
  if (scrolls) kit.close();

  // ── keys and pads: the cursor moves; confirm picks up and places; details asks for the menu ──
  if (grid.focused) {
    const step = (dx: number, dy: number): void => {
      const dir = rtl() ? -dx : dx;
      if (carrying && byFocus) {
        h.cx += dir;
        h.cy += dy;
        return;
      }
      const b = cursorBox;
      if (dir > 0) h.cx = b.x + b.w;
      else if (dir < 0) h.cx = b.x - 1;
      if (dy > 0) h.cy = b.y + b.h;
      else if (dy < 0) h.cy = b.y - 1;
      // Inside an item the cursor keeps its own row or column of the move.
      if (dir !== 0) h.cy = clamp(h.cy, b.y, b.y + b.h - 1);
      if (dy !== 0) h.cx = clamp(h.cx, b.x, b.x + b.w - 1);
    };
    for (let k = 0; k < Math.abs(grid.adjust); k++) step(Math.sign(grid.adjust), 0);
    for (let k = 0; k < Math.abs(grid.adjustY ?? 0); k++) step(0, Math.sign(grid.adjustY ?? 0));
    if (grid.adjust || grid.adjustY) kit.feedback("focus", -1);
    if (cursorItem && !carrying) {
      out.hovered = out.hovered ?? cursorItem.key ?? null;
      if (out.hovered === cursorItem.key) out.hoveredRect = cellsRect ? { x: cellsRect.x + (rtl() ? cols - cursorBox.x - cursorBox.w : cursorBox.x) * cs, y: cellsRect.y + cursorBox.y * cs, width: cursorBox.w * cs, height: cursorBox.h * cs } : null;
    }
    const keyed = grid.clicked && !p.released;
    if (keyed) {
      if (carrying) land();
      else if (cursorItem) {
        if (cursorItem.locked) {
          out.selected = cursorItem.key ?? null;
          out.refused = true;
          h.rt = tick;
          h.pk = keyHash(cursorItem.key ?? "");
          kit.feedback("error", -1);
        } else {
          own = cursorItem.key ?? "";
          h.key = keyHash(own);
          h.count = cursorItem.count || 1;
          h.split = 0;
          h.rot = cursorItem.rotated ? 1 : 0;
          h.byFocus = 1;
          h.cx = cursorItem.x ?? 0;
          h.cy = cursorItem.y ?? 0;
          kit.feedback("select", -1);
        }
      }
    }
    if (!carrying && cursorItem && kit.action("details")) {
      out.menu = cursorItem.key ?? null;
      const b = out.hoveredRect;
      out.menuAt = b ? { x: rtl() ? b.x : b.x + b.width, y: b.y + b.height } : { x: 0, y: 0 };
    }
  }

  // ── the pointer's drag, the long press, letting go ──
  const held = h.pk ? keyOf(keys, h.pk) : null;
  if (held && !own && p.down && h.pt) {
    const moved = Math.hypot(p.x - h.px, p.y - h.py);
    const it = byKey.get(held)!;
    if (moved > threshold()) {
      if (it.locked) {
        if (h.rt < h.pt) {
          out.refused = true;
          h.rt = tick;
          kit.feedback("error", -1);
        }
      } else {
        // Picked up by the cell it was grabbed at, so it stays under the finger.
        const g = cellAt(cellsRect, cs, h.px, h.py);
        const f = footprintOf(it);
        own = held;
        h.key = keyHash(held);
        const whole = it.count || 1;
        h.split = h.shift && whole > 1 ? 1 : 0;
        h.count = h.split ? Math.floor(whole / 2) : whole;
        h.rot = it.rotated ? 1 : 0;
        h.byFocus = 0;
        h.gx = g ? clamp(g.cx - f.x, 0, f.w - 1) : 0;
        h.gy = g ? clamp(g.cy - f.y, 0, f.h - 1) : 0;
        kit.feedback("select", -1);
      }
      h.pt = 0;
    } else if (!h.menu && tick - h.pt >= LONG_PRESS) {
      out.menu = held;
      out.menuAt = { x: p.x, y: p.y };
      h.menu = 1;
      h.pt = 0;
    }
  }
  if (own && !h.byFocus && p.released && h.key) {
    if (cellAt(cellsRect, cs, p.x, p.y)) land();
    else {
      // Let go outside: whoever is under the pointer takes it (it is in no one's hand next tick).
      out.carried = carryOf(id, byKey.get(own)!, h, p, true);
      h.key = 0;
    }
  }
  // A carry from elsewhere let go over this grid lands here.
  if (foreign && foreign.released && !foreign.byFocus && cellAt(cellsRect, cs, foreign.x ?? -1, foreign.y ?? -1)) land();
  if (h.key && h.byFocus && kit.back()) {
    h.key = 0;
    kit.feedback("cancel", -1);
  }
  if (!p.down && !p.released) h.pk = h.rt === tick ? h.pk : 0;
  if (p.released) h.pt = 0;

  // ── what is carried: the hand's report, the ghost and the touch rotate ──
  const still0 = h.key ? keyOf(keys, h.key) : null;
  if (still0) {
    const it = byKey.get(still0)!;
    out.carried = carryOf(id, it, h, p, false);
    if (h.byFocus && cellsRect) {
      const s = carrySize();
      const x0 = rtl() ? cellsRect.x + (cols - h.cx - s.w) * cs : cellsRect.x + h.cx * cs;
      out.carried = { ...out.carried, x: x0 + (s.w * cs) / 2, y: cellsRect.y + (h.cy + s.h / 2) * cs };
    }
    if (!h.byFocus) ghost(it, h.rot === 1, h.count, cs, h.gx, h.gy, { x: p.x, y: p.y }, kit.bounds(rootPath), verdictOf(), !fp);
    if (touch || h.byFocus) {
      const r = btn("Rotate", desc.rotateLabel ?? "Rotate", { size: "sm", sym: "rotate", part: "Inventory.rotate" });
      if (r.clicked) {
        h.rot = h.rot ? 0 : 1;
        kit.feedback("toggle", -1);
      }
    }
  }
  writeHand(h);
  kit.close();
  rememberPath(`${id}~root`, kit.lastPath());
  return out;
}

/**
 * The silhouette an empty slot shows: its icon's name when it names a
 * drawn kind, else the kind its key and accepted tags say (a helmet slot
 * a helm, a ring slot a ring), else none (a plain socket, never a letter).
 */
function slotGlyph(s: Api.PaperdollSlot): SymbolKind | null {
  const words = [s.icon ?? "", s.key ?? "", ...(s.accept ?? [])].join(" ").toLowerCase();
  const kinds: Array<[RegExp, SymbolKind]> = [
    [/helm|head|hat|hood/, "helm"],
    [/amulet|neck|pendant/, "amulet"],
    [/shield|off/, "shield"],
    [/weapon|sword|blade|main/, "sword"],
    [/ring/, "gemRing"],
    [/glove|gauntlet|hands?\b|wrist/, "glove"],
    [/feet|boot|shoe|leg/, "boot"],
    [/chest|armou?r|body|torso|robe/, "armor"],
    [/relic|trinket|charm|artifact/, "star"],
  ];
  return kinds.find(([re]) => re.test(words))?.[1] ?? null;
}

/** The hand's report for the other places on the screen. */
function carryOf(source: string, it: Api.GridItem, h: Hand, p: Api.UiPointer, released: boolean, slot = ""): Api.InventoryCarry {
  const f = footprintOf({ ...it, rotated: h.rot === 1 });
  return { key: it.key ?? "", source, slot, tag: it.tag ?? "", w: f.w, h: f.h, rotated: h.rot === 1, count: h.count || it.count || 1, x: p.x, y: p.y, byFocus: h.byFocus === 1, released };
}

// ── the paperdoll ───────────────────────────────────────────────────────

/**
 * A paperdoll (Diablo IV, Baldur's Gate 3, World of Warcraft): see `fn
 * paperdoll` in crates/og-api/api/ui.ogt.
 */
export function paperdoll(id: string, desc: Api.PaperdollDesc, figure: Api.UiBuild): Api.PaperdollResult {
  const { t, phone, touch, viewport } = frame();
  const out: Mut<Api.PaperdollResult> = { equip: null, selected: null, used: null, carried: null, hovered: null, hoveredRect: null, refused: false };
  const slots = desc.slots ?? [];
  const p = pointer();
  const tick = kit.tick();
  const portrait = viewport.breakpoint === "phonePortrait" || !!desc.stack;
  kit.open(id, { direction: portrait ? "column" : "row", gap: u(20), cross: portrait ? "stretch" : "start", width: desc.width ?? null, part: "Paperdoll.frame", partClass: "panel" });
  const rootPath = recallPath(`${id}~root`);
  const h = readHand();
  const slotKeys = slots.map((s) => s.key ?? "");
  const own = h.key ? slotKeys.find((k) => keyHash(`${k}`) === h.key && slots.find((s) => s.key === k)?.item) ?? null : null;
  if (h.key && !own) h.key = 0;
  const carry = own ? null : desc.carry ?? null;
  const takes = (s: Api.PaperdollSlot, tag: string): boolean => !s.locked && ((s.accept ?? []).length === 0 || (s.accept ?? []).includes(tag));

  // The figure's box and the slots placed around it by fractions of it.
  const aspect = desc.figureAspect && desc.figureAspect > 0 ? desc.figureAspect : 0.62;
  const roomH = viewport.height - viewport.safeArea.top - viewport.safeArea.bottom;
  const bh = viewport.breakpoint === "phonePortrait" ? Math.min(u(400), Math.round(roomH * 0.5)) : desc.stack ? Math.min(u(400), Math.round(roomH * 0.5)) : Math.min(u(460), Math.round(roomH * (phone ? 0.78 : 0.62)));
  const roomW = viewport.width - viewport.safeArea.left - viewport.safeArea.right - u(32);
  const sizeOf = (s: Api.PaperdollSlot) => Math.max(touch ? 48 : 0, s.size && s.size > 0 ? s.size : u(phone ? 56 : 64));
  const maxSlot = slots.reduce((m, s) => Math.max(m, sizeOf(s)), 0);
  const bw = Math.min(viewport.breakpoint === "phonePortrait" ? roomW : roomW * 0.45, Math.round(bh * aspect) + maxSlot * 2);
  kit.column("Doll", { width: px(bw), gap: u(8), cross: "center" }, () => {
    if (desc.title || desc.rating) {
      kit.row("Header", { width: grow(), gap: u(12), cross: "center" }, () => {
        kit.column("Names", { width: grow(), gap: 2 }, () => {
          if (desc.title) text("Title", desc.title, { step: st(20), heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
          if (desc.subtitle) text("Subtitle", desc.subtitle, { step: st(13), color: "inkMuted", maxLines: 2, ellipsis: true, width: grow() });
        });
        if (desc.rating) {
          kit.column("Rating", { gap: 0, cross: "end", part: "Paperdoll.rating", partClass: "badge" }, () => {
            text("Value", desc.rating!, { step: st(24), font: "mono", color: "highlight", nowrap: true });
            if (desc.ratingLabel) text("Label", desc.ratingLabel, { step: st(12), color: "inkMuted", nowrap: true });
          });
        }
      });
    }
    kit.open("Slots", { width: px(bw), height: px(bh), part: "Paperdoll.figure", partClass: "slot", radius: t.panel });
    kit.setDraw({
      shape: { kind: "rect" },
      fills: [
        { kind: "solid", color: token("well") },
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.42 }, to: { x: 0.55, y: 0.6 }, stops: [{ offset: 0, color: token("primarySoft") }, { offset: 1, color: token("well", 0) }] },
      ],
      strokes: [line(1, token("line"))],
    });
    kit.open("Figure", { width: px(Math.round(bh * aspect * 0.92)), height: px(Math.round(bh * 0.92)), direction: "column", align: "center", cross: "center" });
    figure();
    kit.close();
    over({ insets: { centerX: true, centerY: true } });
    for (const s of slots) {
      const key = s.key ?? "";
      const sz = sizeOf(s);
      // A narrow slot (a two-handed weapon's) is never under a finger's 44 CSS px.
      const sw = Math.max(touch ? 44 : 0, s.aspect && s.aspect > 0 ? Math.round(sz * s.aspect) : sz);
      const sh = sz;
      const sx = clamp((s.x ?? 0.5) * bw - sw / 2, 4, bw - sw - 4);
      const sy = clamp((s.y ?? 0.5) * bh - sh / 2, 4, bh - sh - 4);
      const it = s.item ?? null;
      const selected = (desc.selected ?? "") === key;
      // While something is carried: the slot under it (or the keys' cursor) says yes in success or no in danger; the
      // others hint (a quiet success edge where it would fit) or step back.
      const wasAt = kit.bounds(recallPath(`${id}~slot~${key}`));
      const under = !!carry && !!wasAt && inside({ x: wasAt.x - 8, y: wasAt.y - 8, width: wasAt.width + 16, height: wasAt.height + 16 }, carry.x ?? -1, carry.y ?? -1);
      const fits = !!carry && takes(s, carry.tag ?? "");
      const target: "none" | "hint" | "accepts" | "rejects" | "dim" = !carry ? "none" : under ? (fits ? "accepts" : "rejects") : fits ? "hint" : "dim";
      const lifted = own === key;
      const shake = h.rt > 0 && tick - h.rt < 18 && h.pk === keyHash(key) ? Math.sin((tick - h.rt) * 1.6) * (18 - (tick - h.rt)) * 0.5 : 0;
      const r = surface(key, {
        part: "Paperdoll.slot",
        cls: "slot",
        flags: [it ? "filled" : "empty", target === "none" ? "" : target, s.locked ? "locked" : "", selected ? "selected" : ""],
        selected,
        fill: null,
        washBy: 0.12,
        disabled: !!s.locked && !carry,
        width: px(sw),
        height: px(sh),
        align: "center",
        cross: "center",
        radius: t.small,
        semantics: { role: "button", selected, name: [s.label ?? key, it ? it.name ?? it.key : s.locked ? desc.lockedLabel ?? "Locked" : desc.emptyLabel ?? "Empty"].join(", "), ...(s.locked && s.lockedHint ? { description: s.lockedHint } : {}) },
        layer: {
          insets: { start: sx, top: sy },
          transform: { x: shake },
          states: [
            { name: "fill", states: [vstate("empty"), vstate("filled")], transitions: [] },
            { name: "target", states: [vstate("none"), vstate("hint"), vstate("accepts", [{ prop: "scale", value: 1.08 }]), vstate("rejects"), vstate("dim", [{ prop: "opacity", value: 0.5 }])], transitions: [] },
          ],
          state: [{ group: "fill", state: it ? "filled" : "empty" }, { group: "target", state: target }],
          motion: { kind: "token", token: "ui" },
        },
      }, () => {
        if (it) tile({ ...it, rotated: false, w: 1, h: 1 }, sw, sh, Math.min(sw, sh), { lifted, selected });
        else {
          if (s.locked) lockedCell(sw, sh);
          else {
            // An open socket: recessed into the doll's ground (a shade falling from its top edge), its kind's
            // silhouette faint in it.
            kit.setDraw({
              shape: { kind: "rect" },
              fills: [{ kind: "solid", color: token("well") }, vgrad([[0, token("shadow", 0.5)], [0.45, token("shadow", 0)]])],
              strokes: [line(1.5, token(target === "hint" || target === "accepts" ? "success" : "line")), line(1, paint(role("ink"), 0.08), "outside")],
              effects: [{ kind: "innerShadow", offset: { x: 0, y: 2 }, blur: 6, spread: 0, color: token("shadow", 0.8) }],
            });
            const glyph = Math.round(Math.min(sw, sh) * 0.56);
            if (s.icon && kit.hasIcon(s.icon)) kit.icon("Icon", s.icon, { size: glyph, tint: token("inkFaint"), label: "" });
            else {
              const kind = slotGlyph(s);
              if (kind) symbol("Glyph", kind, glyph, mix(role("well"), role("ink"), 0.34));
            }
          }
        }
        if (target === "accepts" || target === "rejects") {
          // Under the carry: a bright success edge and glow (it fits), or a danger edge, a shade and ⊘ (it does not).
          const yes = target === "accepts";
          // Drawn round the slot (what is carried covers its middle): a ring a little outside it, the inside shaded when
          // it refuses, and a badge on its top end corner, a tick or ⊘.
          const o = u(5);
          kit.open(yes ? "Accepts" : "Rejects", { width: px(sw + o * 2), height: px(sh + o * 2), part: `Paperdoll.target${flags(target)}`, partClass: "slot" });
          kit.setDraw({ shape: { kind: "rect", radii: [t.small + o, t.small + o, t.small + o, t.small + o] }, fills: yes ? [] : [{ kind: "solid", color: token("danger", 0.32) }], strokes: [line(3, token(yes ? "success" : "danger")), line(1, token("shadow", 0.7), "outside")], effects: [{ kind: "outerGlow", blur: 12, spread: 0, color: token(yes ? "success" : "danger", 0.6) }] });
          kit.close();
          kit.props(overProps({ insets: { start: -o, top: -o }, inert: true, z: 4, overlaps: ["*"] }));
          const bd = Math.max(22, u(26));
          kit.open(yes ? "Yes" : "No", { width: px(bd), height: px(bd), align: "center", cross: "center", part: `Paperdoll.targetBadge${flags(target)}`, partClass: "badge" });
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token(yes ? "success" : "danger") }], strokes: [line(2, token("shadow", 0.75), "outside")], effects: [shadow(2, 6, 0.5)] });
          symbol("Mark", yes ? "check" : "ban", Math.round(bd * 0.64), role("onPrimary"));
          kit.close();
          kit.props(overProps({ insets: { end: -Math.round(bd * 0.75), top: -Math.round(bd * 0.75) }, inert: true, z: 6, overlaps: ["*"] }));
          kit.semantics({ role: "status", name: yes ? "accepts" : "refuses" });
        }
      });
      const path = kit.lastPath();
      rememberPath(`${id}~slot~${key}`, path);
      const b = kit.bounds(path);
      const ix = lastInteraction(key);
      if ((ix.hovered || ix.focused) && it && !lifted) {
        out.hovered = key;
        out.hoveredRect = b;
      }
      // A label under an empty slot where there is room (a desktop).
      const labelH = Math.round(u(14) * Math.max(1, frame().theme.textScale ?? 1));
      if (!it && !phone && s.label && sw >= 56 && sy + sh + 2 + labelH <= bh) {
        text(`${key}Label`, s.label, { step: st(11), color: "inkMuted", nowrap: true, ellipsis: true, width: px(sw + 24), align: "center" });
        kit.props(overProps({ insets: { start: sx - 12, top: sy + sh + 2 }, inert: true }));
      }
      if (r.pressed && p.pressed) {
        h.pk = keyHash(key);
        h.px = p.x;
        h.py = p.y;
        h.pt = tick;
      }
      const viaPointer = p.released && inside(b, p.x, p.y);
      const equip = (from: Api.InventoryCarry): void => {
        if (!takes(s, from.tag ?? "")) {
          out.refused = true;
          h.rt = tick;
          h.pk = keyHash(key);
          kit.feedback("error", -1);
          return;
        }
        out.equip = { slot: key, key: from.key ?? "", from: from.source ?? "", fromSlot: from.slot ?? "", swapped: it ? it.key ?? null : null };
        kit.feedback("confirm", -1);
      };
      if (carry && carry.released && inside(b, carry.x ?? -1, carry.y ?? -1)) equip(carry);
      if (r.clicked) {
        if (carry && (carry.byFocus || !viaPointer)) equip(carry);
        else if (lifted && h.byFocus) h.key = 0;
        else if (!carry && viaPointer) {
          if (h.ck === keyHash(key) && tick - h.ct <= DOUBLE && it) {
            out.used = key;
            h.ck = 0;
          } else {
            out.selected = key;
            h.ck = keyHash(key);
            h.ct = tick;
            kit.feedback("select", -1);
          }
        } else if (!carry && it) {
          h.key = keyHash(key);
          h.count = it.count || 1;
          h.byFocus = 1;
          h.rot = 0;
          kit.feedback("select", -1);
        }
      }
      rememberPath(`${id}~slot~${key}`, path);
    }
    kit.close(); // Slots
  });

  // Dragging an item out of its slot.
  const held = h.pk ? slotKeys.find((k) => keyHash(k) === h.pk) ?? null : null;
  const heldSlot = held ? slots.find((s) => s.key === held) : undefined;
  if (held && heldSlot?.item && !own && p.down && h.pt && Math.hypot(p.x - h.px, p.y - h.py) > threshold()) {
    h.key = keyHash(held);
    h.count = heldSlot.item.count || 1;
    h.byFocus = 0;
    h.rot = 0;
    h.pt = 0;
    kit.feedback("select", -1);
  }
  const ownNow = h.key ? slots.find((s) => keyHash(s.key ?? "") === h.key && s.item) : undefined;
  if (ownNow?.item) {
    const it = ownNow.item;
    const report = { ...carryOf(id, { ...it, w: 1, h: 1 }, h, p, false, ownNow.key ?? "") };
    if (!h.byFocus && p.released) {
      // Dropped on another slot of this doll that takes it: a move between slots; elsewhere: the others take it.
      const onto = slots.find((s) => s.key !== ownNow.key && inside(kit.bounds(recallPath(`${id}~slot~${s.key}`)), p.x, p.y));
      if (onto && takes(onto, it.tag ?? "")) {
        out.equip = { slot: onto.key ?? "", key: it.key ?? "", from: id, fromSlot: ownNow.key ?? "", swapped: onto.item?.key ?? null };
        kit.feedback("confirm", -1);
      } else if (!inside(kit.bounds(recallPath(`${id}~slot~${ownNow.key}`)), p.x, p.y)) out.carried = { ...report, released: true };
      h.key = 0;
    } else {
      out.carried = report;
      if (!h.byFocus) {
        const sz = Math.round(u(56) * 1);
        ghost({ ...it, w: 1, h: 1, rotated: false }, false, h.count, sz, 0, 0, { x: p.x, y: p.y }, kit.bounds(rootPath));
      } else if (kit.back()) {
        h.key = 0;
        kit.feedback("cancel", -1);
      }
    }
  }
  if (!p.down && !p.released) h.pk = 0;

  // The stats, with the compared item's changes beside their values.
  if ((desc.stats ?? []).length > 0) {
    kit.column("Stats", { width: portrait ? px(bw) : px(Math.max(u(240), Math.min(u(320), roomW - bw - u(20)))), gap: u(4), part: "Paperdoll.stats", partClass: "panel", padding: xy(u(16), u(14)), radius: t.panel, background: token("panelRaised") }, () => {
      if (desc.statsTitle) text("Title", desc.statsTitle, { step: st(14), color: "inkMuted", font: "display", nowrap: true, ellipsis: true, width: grow() });
      (desc.stats ?? []).forEach((s, k) => statLine(kit.idi("Stat", k), s, desc.compare ?? false));
    });
  }
  writeHand(h);
  kit.close();
  rememberPath(`${id}~root`, kit.lastPath());
  kit.semantics({ role: "group", name: desc.title ?? "" });
  return out;
}

/** A stat's line: its label, its value and, when compared, the change in its own column (never over the value). */
export function statLine(id: string, s: Api.PaperdollStat, compare: boolean): void {
  const d = s.delta ?? 0;
  const good = s.lowerIsBetter ? d < 0 : d > 0;
  const tone: Api.ThemeColor = d === 0 ? "inkMuted" : good ? "success" : "danger";
  kit.row(id, { width: grow(), gap: u(8), cross: "center", padding: xy(0, u(3)), part: "Paperdoll.stat", partClass: "panel" }, () => {
    // At 200 % text a long label wraps to a second line rather than losing its end.
    text("Label", s.label ?? "", { step: st(14), color: "inkMuted", maxLines: 2, ellipsis: true, width: grow() });
    text("Value", s.value ?? "", { step: st(14), font: "mono", nowrap: true });
    if (compare) {
      kit.row("Delta", { width: px(u(64)), gap: 0, cross: "center", align: "end", part: `Paperdoll.delta${flags(d > 0 ? "up" : d < 0 ? "down" : "")}`, partClass: "badge" }, () => {
        if (d === 0) return;
        // A signed number is one left-to-right run in every script (+48, −6 %): its sign (the real minus,
        // U+2212) before its digits, built the other way round right to left so the mirrored row still
        // reads sign first (the bidi algorithm would put a lone sign after the digits).
        const digits = signedDigits(s.deltaText || `${Math.abs(d)}`);
        const mark = () => text("Sign", d > 0 ? "+" : "\u2212", { step: st(13), color: tone, font: "mono", nowrap: true });
        const num = () => text("Text", digits, { step: st(13), color: tone, font: "mono", nowrap: true });
        if (rtl()) {
          num();
          mark();
        } else {
          mark();
          num();
        }
      });
    }
  });
  if (compare && d !== 0) kit.semantics({ role: "text", name: `${s.label}: ${s.value}, ${d > 0 ? "plus" : "minus"} ${signedDigits(s.deltaText || `${Math.abs(d)}`)}` });
}

/** A delta's digits without its sign (set on its own): "−6%" and "+48" give "6%" and "48". */
export function signedDigits(t: string): string {
  return t.trim().replace(/^[+\-\u2212\u2013±]\s*/, "");
}
