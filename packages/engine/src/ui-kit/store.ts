// Collections: the shop, the inventory (drag and drop between slots), the
// level select and the leaderboard. Each is a dialog-shaped screen with a
// focus scope; Back closes it.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { capsule, fit, fmt, frame, grow, iconOr, px, token } from "./util.ts";

/** A screen panel: a dialog over the game, wide on desktop, full width on a phone. */
function screen(id: string, title: string, width: number, build: () => void): boolean {
  const { phone, viewport } = frame();
  const m = kit.modal(id, true, { title, width: phone ? grow() : px(Math.min(width, viewport.width - 48)), ignoreBackdrop: true }, build);
  return m.dismissed;
}

/** An item's picture: its icon, else its initial on the slot. */
function picture(id: string, icon: string | undefined, name: string, size: number): void {
  if (icon && kit.hasIcon(icon)) kit.icon(id, icon, { size });
  else kit.label(id, name.slice(0, 1).toUpperCase(), { step: "xl2", font: "display", color: token("fgMuted"), nowrap: true });
}

export function shopScreen(id: string, desc: Api.ShopDesc = {}): Api.ShopResult {
  const { t, phone } = frame();
  const out: { bought: string | null; refused: string | null; closed: boolean } = { bought: null, refused: null, closed: false };
  const cats = desc.categories ?? [];
  let cat = Math.min(kit.memory(`${id}.cat`, 0), Math.max(0, cats.length - 1));
  const all = desc.items ?? [];
  const items = cats.length === 0 ? all : all.filter((i) => (i.category || cats[0]) === cats[cat]);
  let sel = Math.min(kit.memory(`${id}.sel`, 0), Math.max(0, items.length - 1));
  const balance = desc.balance ?? 0;
  out.closed = screen(id, desc.title ?? "Shop", 760, () => {
    kit.row("Bar", { width: grow(), gap: t.sm, cross: "center", align: "spaceBetween" }, () => {
      if (cats.length > 0) {
        const r = kit.tabs("Categories", cats, cat, {});
        if (r.response.changed) sel = 0;
        cat = r.selected;
      } else {
        kit.spacer("Gap", null);
      }
      capsule("Balance", () => {
        if (desc.currencyIcon) iconOr("Icon", desc.currencyIcon, 18, token("accent"));
        kit.counter("Amount", balance, { font: "mono", step: "base" });
        kit.label("Unit", desc.currency ?? "coins", { step: "base", color: token("fgMuted"), nowrap: true });
      });
    });
    const body = (): void => {
      const g = kit.grid("Items", { count: items.length, cellWidth: phone ? 96 : 112, cellHeight: phone ? 112 : 124, height: fit(0, phone ? 260 : 320), width: grow(), selected: sel }, (i) => {
        const it = items[i]!;
        picture("Picture", it.icon, it.name ?? "", 36);
        kit.label("Name", it.name ?? "", { step: "base", nowrap: true });
        if (it.owned) kit.badge("Owned", desc.ownedLabel ?? "Owned", "success");
        else kit.label("Price", fmt(it.price ?? 0), { step: "meta", font: "mono", color: token(balance >= (it.price ?? 0) ? "accentInk" : "fgSubtle"), nowrap: true });
      });
      if (g.clicked !== null && g.clicked !== undefined) sel = g.clicked;
      const it = items[sel];
      kit.column("Detail", { width: phone ? grow() : px(220), gap: t.sm }, () => {
        if (!it) {
          kit.label("Empty", desc.emptyLabel ?? "Nothing for sale here", { color: token("fgMuted") });
          return;
        }
        kit.label("Name", it.name ?? "", { step: "lg", width: grow() });
        if (it.description) kit.label("Description", it.description, { step: "meta", color: token("fgMuted"), width: grow() });
        kit.keyValue("Price", desc.priceLabel ?? "Price", `${fmt(it.price ?? 0)} ${desc.currency ?? "coins"}`, true);
        const short = balance < (it.price ?? 0);
        // Short of the price, Buy still answers: it refuses, shaking with the error sound.
        const r = kit.button("Buy", it.owned ? (desc.ownedLabel ?? "Owned") : short ? (desc.shortLabel ?? "Not enough") : (desc.buyLabel ?? "Buy"), { variant: short ? "secondary" : "primary", width: grow(), disabled: it.owned }) as Api.Response;
        if (r.clicked && !it.owned && short) {
          kit.shake({ sound: true });
          out.refused = it.id ?? null;
        }
        if (r.clicked && !it.owned && !short) out.bought = it.id ?? null;
      });
    };
    if (phone) kit.column("Body", { width: grow(), gap: t.md }, body);
    else kit.row("Body", { width: grow(), gap: t.lg }, body);
  });
  kit.remember(`${id}.cat`, cat);
  kit.remember(`${id}.sel`, sel);
  if (out.bought !== null) desc.onBuy?.(out.bought);
  return out;
}

export function inventoryScreen(id: string, desc: Api.InventoryDesc = {}): Api.InventoryResult {
  const { t, phone } = frame();
  const slots = desc.slots ?? 24;
  const bySlot = new Map<number, Api.InventoryItem>();
  for (const it of desc.items ?? []) bySlot.set(it.slot ?? 0, it);
  let sel = kit.memory(`${id}.sel`, -1);
  const out: { selected: number | null; moved: Api.SlotMove | null; used: string | null; closed: boolean } = { selected: null, moved: null, used: null, closed: false };
  const cell = phone ? 64 : 72;
  const columns = desc.columns ?? 0;
  // Wide enough for the stated columns at the slot size (the grid shares
  // what is left between them if the screen is narrower): the dialog's
  // padding, the detail panel and the gap beside the grid, and room for
  // its scrollbar.
  const chrome = 2 * t.xl + 200 + t.lg;
  const width = columns > 0 ? Math.max(680, chrome + columns * (cell + t.sm) - t.sm + 2 * t.sm) : 680;
  out.closed = screen(id, desc.title ?? "Inventory", width, () => {
    const body = (): void => {
      const g = kit.grid("Slots", { count: slots, cellWidth: cell, cellHeight: cell, columns, height: px(phone ? 300 : 340), width: grow(), selected: sel >= 0 ? sel : null }, (i) => {
        const it = bySlot.get(i);
        if (!it) return;
        picture("Picture", it.icon, it.name ?? "", cell * 0.5);
        if ((it.count ?? 0) > 1) kit.label("Count", `${it.count}`, { step: "xs", font: "mono", color: token("fgMuted"), nowrap: true });
      });
      if (g.clicked !== null && g.clicked !== undefined) sel = g.clicked;
      if (g.dropFrom !== null && g.dropFrom !== undefined && g.dropTo !== null && g.dropTo !== undefined && bySlot.has(g.dropFrom)) {
        out.moved = { from: g.dropFrom, to: g.dropTo };
        sel = g.dropTo;
      }
      const it = sel >= 0 ? bySlot.get(sel) : undefined;
      kit.column("Detail", { width: phone ? grow() : px(200), gap: t.sm }, () => {
        if (!it) {
          kit.label("Hint", desc.hintLabel ?? "Select an item. Drag it to another slot to move it.", { step: "meta", color: token("fgMuted"), width: grow() });
          return;
        }
        kit.label("Name", it.name ?? "", { step: "lg", width: grow() });
        if (it.description) kit.label("Description", it.description, { step: "meta", color: token("fgMuted"), width: grow() });
        if ((it.count ?? 0) > 1) kit.keyValue("Count", desc.countLabel ?? "Count", `${it.count}`, true);
        if (desc.useLabel !== "") {
          if ((kit.button("Use", desc.useLabel ?? "Use", { variant: "primary", width: grow() }) as Api.Response).clicked) out.used = it.id ?? null;
        }
      });
    };
    if (phone) kit.column("Body", { width: grow(), gap: t.md }, body);
    else kit.row("Body", { width: grow(), gap: t.lg }, body);
  });
  kit.remember(`${id}.sel`, sel);
  out.selected = sel >= 0 ? sel : null;
  return out;
}

export function levelSelect(id: string, desc: Api.LevelSelectDesc = {}): Api.LevelSelectResult {
  const { t, phone } = frame();
  const levels = desc.levels ?? [];
  const earned = levels.reduce((n, l) => n + Math.min(3, l.stars ?? 0), 0);
  const out: { chosen: number | null; closed: boolean } = { chosen: null, closed: false };
  out.closed = screen(id, desc.title ?? "Levels", 720, () => {
    kit.row("Total", { gap: t.xs, cross: "center" }, () => {
      iconOr("Star", "star", 18, token("accent"));
      kit.label("Count", `${earned} / ${levels.length * 3}`, { font: "mono", color: token("fgMuted"), nowrap: true });
    });
    // A numeral is the card's face, in display type on a square card; a
    // name takes a wider card and a smaller step, wrapped to two lines (cut
    // with an ellipsis past them), so it is read, never clipped. With
    // `columns` the cards share the screen's width, so two columns are two
    // wide cards.
    const named = levels.some((l) => [...(l.name ?? "")].length > 3);
    const cell = phone ? 84 : 104;
    const g = kit.grid("Levels", { count: levels.length, cellWidth: named ? (phone ? 136 : 168) : cell, cellHeight: cell, columns: desc.columns ?? 0, height: fit(0, phone ? 420 : 400), width: grow() }, (i) => {
      const l = levels[i]!;
      if (l.locked) {
        iconOr("Lock", "lock", 28, token("fgSubtle"));
        return;
      }
      const name = l.name ?? `${i + 1}`;
      if ([...name].length > 3) kit.label("Name", name, { step: phone ? "base" : "lg", font: "display", align: "center", width: grow(), maxLines: 2, ellipsis: true });
      else kit.label("Name", name, { step: "xl3", font: "display", nowrap: true });
      kit.row("Stars", { gap: 2 }, () => {
        for (let k = 0; k < 3; k++) iconOr(kit.idi("Star", k), "star", 14, token(k < (l.stars ?? 0) ? "accent" : "fillStrong"));
      });
    });
    if (g.clicked !== null && g.clicked !== undefined && !levels[g.clicked]?.locked) out.chosen = g.clicked;
  });
  return out;
}

const MEDALS: readonly Api.ThemeColor[] = ["warn", "fgMuted", "accentInk"];

export function leaderboardScreen(id: string, desc: Api.LeaderboardDesc = {}): Api.LeaderboardResult {
  const { t, phone } = frame();
  const scopes = desc.scopes ?? [];
  let scope = Math.min(kit.memory(`${id}.scope`, 0), Math.max(0, scopes.length - 1));
  const out = { scope, changed: false, retry: false, closed: false };
  const entries = desc.entries ?? [];
  out.closed = screen(id, desc.title ?? "Leaderboard", 520, () => {
    if (scopes.length > 0) {
      const r = kit.tabs("Scopes", scopes, scope, { fill: true, width: grow() });
      out.changed = r.response.changed;
      scope = r.selected;
    }
    const height = px(phone ? 380 : 360);
    if (desc.loading) {
      // Honest loading: skeleton rows, never invented scores.
      kit.column("Loading", { width: grow(), gap: t.md, height }, () => {
        for (let i = 0; i < 6; i++) kit.skeleton(kit.idi("Row", i), { height: t.controlSm });
      });
    } else if (desc.error) {
      kit.column("Error", { width: grow(), gap: t.md, height, align: "center", cross: "center" }, () => {
        kit.label("Message", desc.error!, { color: token("danger"), align: "center", width: grow() });
        if ((kit.button("Retry", desc.retryLabel ?? "Retry", {}) as Api.Response).clicked) out.retry = true;
      });
    } else if (entries.length === 0) {
      kit.column("Empty", { width: grow(), height, align: "center", cross: "center" }, () => {
        kit.label("Message", desc.empty ?? "No scores yet", { color: token("fgMuted"), align: "center", width: grow() });
      });
    } else {
      const player = entries.findIndex((e) => e.player);
      kit.list("Rows", { count: entries.length, rowHeight: t.row, height, width: grow(), selected: player >= 0 ? player : null }, (i) => {
        const e = entries[i]!;
        const rank = e.rank ?? i + 1;
        kit.label("Rank", `${rank}`, { width: px(40), font: "mono", step: rank <= 3 ? "lg" : "body", color: token(rank <= 3 ? MEDALS[rank - 1]! : "fgMuted"), nowrap: true });
        kit.label("Name", e.name ?? "", { width: grow(), nowrap: true });
        kit.label("Score", e.score ?? "", { font: "mono", nowrap: true });
      });
    }
  });
  kit.remember(`${id}.scope`, scope);
  out.scope = scope;
  return out;
}
