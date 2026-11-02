// Collections: the shop, the inventory (drag and drop between slots), the
// level select and the leaderboard. Each is a dialog-shaped screen with a
// focus scope; Back closes it.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { all as edges, hudBox, fit, fmt, frame, grow, iconOr, px, token, xy } from "./util.ts";
import { artSlot } from "../ui/foundations/media.ts";
import { over } from "../ui/containers/base.ts";
import { luminance, rarityColor, role } from "../ui/feedback/core.ts";

/**
 * A slot's ground for its rarity, as commercial inventories draw it: the
 * slot's own ground stays, a common item adds nothing, and a higher tier
 * adds a glow in its hue (the style's rarity roles) rising from the bottom
 * centre, and a tier line. One stroke per cell: where the style dresses the
 * slot by its tier (`Slot.frame[rarity=…]`, og.ui.partAttr) its own frame
 * carries the tier and the glow is all this adds; else a hairline, 1 px at
 * 60 % on a dark panel, the full tier colour at 1.5 px on a light one, where
 * the glow is stronger too so the tier still reads on parchment. `line`
 * false (the detail's showcase) draws the glow alone. Built first in the
 * cell, under what follows.
 */
export function rarityGround(id: string, rarity: Api.Rarity | undefined, line = true): void {
  const c = rarityColor(rarity);
  if (!c || rarity === "common") return;
  const dressed = line && kit.partAttr("rarity", rarity!);
  const dark = luminance(role("panel")) < 0.36;
  const top = rarity === "legendary" || rarity === "mythic";
  const glow = dark ? (top ? 0.35 : 0.25) : top ? 0.4 : 0.28;
  const ink = (alpha: number): Api.Paint => ({ color: { r: c.r, g: c.g, b: c.b }, alpha });
  const r = frame().t.small;
  kit.open(id, { width: grow(), height: grow() });
  kit.setDraw({
    shape: { kind: "rect", radii: [r, r, r, r] }, clip: "shape",
    // A radial's `to` is its radii: wide, and up to the top of the slot.
    fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 1 }, to: { x: 0.8, y: 1 }, stops: [{ offset: 0, color: ink(glow) }, { offset: 0.6, color: ink(glow * 0.35) }, { offset: 1, color: ink(0) }] }],
    strokes: line && !dressed ? [{ width: dark ? 1 : 1.5, align: "inside", paint: { kind: "solid", color: ink(dark ? 0.6 : 1) } }] : [],
  } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 0 });
}

/**
 * The selected tile (docs/ui.md §15.5): a soft light rim at 55 % white 2 px
 * outside the slot's edge and a glow round it to 8 px, over the lift og-ui's
 * grid already gives it. Outside, so a pack's own frame (gilded's gold) stays
 * whole and the selection adds to it, in every style.
 */
function selectedRim(id: string): void {
  const r = frame().t.small + RIM;
  const white = (alpha: number): Api.Paint => ({ color: { r: 1, g: 1, b: 1 }, alpha });
  kit.open(id, { width: grow(), height: grow() });
  kit.setDraw({
    shape: { kind: "rect", radii: [r, r, r, r] },
    // The glow as rings outside the rim, fading out to 8 px from the slot: an
    // outerGlow is a blurred copy of the whole shape and would veil the art.
    strokes: [
      ...[[8 - RIM, 0.05], [5, 0.07], [2.5, 0.1]].map(([width, alpha]) => ({ width, align: "outside", paint: { kind: "solid", color: white(alpha) } })),
      { width: 1.5, align: "inside", paint: { kind: "solid", color: white(0.55) } },
    ],
  } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: -RIM, top: -RIM, end: -RIM, bottom: -RIM }, inert: true, z: 3 });
}

/**
 * A grid's height in whole rows: every row when they fit under `limit`, else
 * as many whole rows as fit and a third of the next (`more`), which the grid
 * fades out under a soft scrim, its scrollbar in a gutter of its own. Never a
 * height that cuts through a cell's frame.
 */
function rowsView(count: number, columns: number, cell: number, gap: number, limit: number): { height: number; more: boolean } {
  const rows = Math.ceil(count / Math.max(1, columns));
  const stride = cell + gap;
  const all = rows * stride - gap;
  if (all <= limit) return { height: all, more: false };
  const whole = Math.max(1, Math.floor((limit - cell / 3 + gap) / stride));
  return { height: whole * stride + cell / 3, more: true };
}

/** How far outside the slot the selected rim's outer edge sits (2 px clear of the frame, the rim 1.5 px). */
const RIM = 3.5;
/** A grid's padding, so the selected rim of an edge cell is not cut by the grid's clip. */
const EDGE = 4;

/** The scrollbar's gutter beside a grid with more rows than it shows: its bar's width and a margin either side, no more (a phone's bag keeps six touch-sized slots a row). */
const gutterOf = (t: { xs: number; xxs: number }): number => t.xs + 2 * t.xxs;

/** A grid in its view: with more rows than show, the last third-row fades out (an alpha mask, the panel's own ground showing through). */
function gridView(id: string, more: boolean, width: Api.Size, build: () => void): void {
  kit.open(id, { width });
  if (more) kit.setDraw({ clip: "mask", mask: { gradient: { type: "linear", from: { x: 0.5, y: 0.78 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { color: { r: 1, g: 1, b: 1 }, alpha: 1 } }, { offset: 1, color: { color: { r: 1, g: 1, b: 1 }, alpha: 0 } }] } } } as Api.DrawDesc);
  build();
  kit.close();
}

/** A cut-out item (square art on transparent ground): no frame of its own, the ground under it shows through. */
function cutout(id: string, image: Api.Texture, width: Api.Size, label = ""): void {
  kit.image(id, { texture: image, width, aspect: 1, label });
}

/** The detail panel's head: the item's cut-out large on its rarity's ground. */
function showcase(id: string, image: Api.Texture, rarity: Api.Rarity | undefined): void {
  kit.column(id, { width: grow(), aspect: 1.25, align: "center", cross: "center", padding: edges(8) }, () => {
    rarityGround("Ground", rarity, false);
    cutout("Image", image, grow());
  });
}

/**
 * A portrait in the style's card frame, faded out from 70 % of its height
 * down: an alpha mask over the frame and the art, so what shows through at
 * the foot is the panel's own ground (a painted parchment too), not a colour.
 */
function portrait(id: string, art: Api.Texture, aspect: number, label: string): void {
  const { t } = frame();
  kit.open(id, { width: grow() });
  kit.setDraw({ clip: "mask", mask: { gradient: { type: "linear", from: { x: 0.5, y: 0.7 }, to: { x: 0.5, y: 1 }, stops: [[0, 1], [0.3, 0.85], [0.65, 0.35], [1, 0]].map(([offset, alpha]) => ({ offset, color: { color: { r: 1, g: 1, b: 1 }, alpha } })) } } } as Api.DrawDesc);
  kit.column("Card", { width: grow(), padding: edges(0), radius: t.panel, frame: "card" }, () => {
    artSlot("Art", { texture: art, kind: "portrait", aspect, focal: { x: 0.5, y: 0.25 }, radius: t.panel, width: grow(), label });
  });
  kit.close();
}

/** A screen panel: a dialog over the game, wide on desktop, full width on a phone. With `scrim` a press on the backdrop closes it too. */
function screen(id: string, title: string, width: number, build: () => void, scrim = false): boolean {
  const { phone, viewport } = frame();
  const m = kit.modal(id, true, { title, width: phone ? grow() : px(Math.min(width, viewport.width - 48)), ignoreBackdrop: !scrim }, build);
  return m.dismissed;
}

/** An item's picture: its icon, else its initial on the slot. */
function picture(id: string, icon: string | undefined, name: string, size: number): void {
  if (icon && kit.hasIcon(icon)) kit.icon(id, icon, { size });
  else kit.label(id, name.slice(0, 1).toUpperCase(), { step: "xl2", font: "display", color: token("inkMuted"), nowrap: true });
}

/** A thing's art: its painted image filling a `size` square, else its icon (or initial) at `iconSize`. */
export function artOr(id: string, image: Api.Texture | null | undefined, icon: string | undefined, name: string, size: number, iconSize: number, label = ""): void {
  if (image) cutout(id, image, px(size), label);
  else picture(id, icon, name, iconSize);
}

/** A count on a dark plate in the bottom end corner of the element being built (a stack, a reward): it reads over any art. */
export function cornerCount(id: string, n: number): void {
  kit.row(id, { padding: xy(4, 0), radius: 4, background: token("scrim") }, () => {
    kit.label("Value", `${n}`, { step: "xs", font: "mono", color: { color: { r: 1, g: 1, b: 1 } }, nowrap: true });
  });
  over({ insets: { end: 4, bottom: 4 }, inert: true, z: 2 });
}

/**
 * Who a screen belongs to (the shop's merchant, the inventory's character):
 * on a desktop or tablet a column of `width` with the art at `aspect` and
 * the words under it, on a phone a row with a round portrait.
 */
function figure(id: string, art: Api.Texture, aspect: number, width: number, name: string, line: string, after?: () => void, brief: readonly Api.StatRow[] = []): void {
  const { t, phone } = frame();
  if (phone) {
    // A phone: the bust large enough to carry the face, the words and the first stats beside it.
    kit.row(id, { width: grow(), gap: t.md, cross: "center" }, () => {
      kit.column("Bust", { width: px(76), height: px(76), padding: edges(0), radius: 38, frame: "card" }, () => {
        artSlot("Art", { texture: art, kind: "portrait", mask: "circle", width: px(76), height: px(76), focal: { x: 0.5, y: 0.2 }, label: name });
      });
      kit.column("Words", { width: grow(), gap: t.xxs }, () => {
        if (name) kit.label("Name", name, { step: "lg", font: "display", width: grow(), nowrap: true, ellipsis: true });
        if (line) kit.label("Line", line, { step: "meta", color: token("inkMuted"), width: grow(), maxLines: 2, ellipsis: true });
        if (brief.length > 0) {
          kit.row("Brief", { width: grow(), gap: t.md }, () => {
            brief.slice(0, 2).forEach((s, i) => kit.row(kit.idi("Stat", i), { gap: t.xs, cross: "baseline" }, () => {
              kit.label("Key", s.label ?? "", { step: "meta", color: token("inkMuted"), nowrap: true });
              kit.label("Value", s.value ?? "", { step: "meta", font: "mono", nowrap: true });
            }));
          });
        }
      });
    });
    return;
  }
  kit.column(id, { width: px(width), gap: t.sm }, () => {
    portrait("Portrait", art, aspect, name);
    kit.column("Words", { width: grow(), gap: t.xxs }, () => {
      if (name) kit.label("Name", name, { step: "xl", font: "display", width: grow() });
      if (line) kit.label("Line", line, { color: token("inkMuted"), width: grow() });
    });
    after?.();
  });
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
  const merchant = desc.merchant ?? null;
  // Painted items take larger cells, their art filling them; a merchant takes a column at the start.
  const painted = all.some((i) => i.image);
  const side = merchant && !phone ? 220 : 0;
  out.closed = screen(id, desc.title ?? "Shop", (painted ? 840 : 760) + (side ? side + t.lg : 0), () => {
    const bar = (): void => {
    // The balance wraps under the tabs where both do not fit (a phone, a large text size):
    // it never reaches past the panel's padding.
    kit.row("Bar", { width: grow(), gap: t.sm, cross: "center", align: "spaceBetween", wrap: true }, () => {
      if (cats.length > 0) {
        const r = kit.tabs("Categories", cats, cat, {});
        if (r.response.changed) sel = 0;
        cat = r.selected;
      } else {
        kit.spacer("Gap", null);
      }
      hudBox("Balance", () => {
        if (desc.currencyIcon) iconOr("Icon", desc.currencyIcon, 18, token("gold"));
        kit.counter("Amount", balance, { font: "mono", step: "base" });
        kit.label("Unit", desc.currency ?? "coins", { step: "base", color: token("inkMuted"), nowrap: true });
      });
    });
    };
    // A price is money: the currency's coin in gold before the numeral in full ink (the purse's way).
    const price = (pid: string, amount: number, ink: Api.ThemeColor, step: Api.TypeStepName, size: number, unit = ""): void => {
      kit.row(pid, { gap: t.xs, cross: "center" }, () => {
        iconOr("Coin", desc.currencyIcon || "coins", size, token("gold"));
        kit.label("Amount", `${fmt(amount)}${unit}`, { step, font: "mono", color: token(ink), nowrap: true });
      });
    };
    const body = (): void => {
      const cw = painted ? (phone ? 104 : 128) : phone ? 96 : 112;
      const ch = painted ? (phone ? 156 : 168) : phone ? 112 : 124;
      // Painted, the grid is three cells wide and the detail takes the rest (no dead band between them).
      const wide = painted && !phone;
      // Whole rows only: two rows of three show a category whole; more fade under a scrim, the scrollbar in its own gutter.
      const view = rowsView(items.length, 3, ch, t.sm, phone ? 330 : 352);
      const gutter = view.more ? gutterOf(t) : 0;
      const gw = 3 * (cw + t.sm) - t.sm + 2 * EDGE + gutter;
      let g: Api.ListResult = {} as Api.ListResult;
      gridView("ItemsView", view.more, wide ? fit() : grow(), () => {
      g = kit.grid("Items", { count: items.length, cellWidth: cw, cellHeight: ch, columns: 3, height: px(view.height + 2 * EDGE), width: wide ? px(gw) : grow(), padding: { left: EDGE, top: EDGE, right: EDGE + gutter, bottom: EDGE }, selected: sel }, (i) => {
        const it = items[i]!;
        rarityGround("Ground", it.rarity);
        if (i === sel) selectedRim("Selected");
        artOr("Picture", it.image, it.icon, it.name ?? "", phone ? 64 : 84, 36);
        // A product's name is never cut: two lines, a step smaller on a phone.
        kit.label("Name", it.name ?? "", { step: phone ? "meta" : "base", maxLines: 2, width: px(cw - 12), align: "center" });
        if (it.owned) kit.badge("Owned", desc.ownedLabel ?? "Owned", "success");
        else price("Price", it.price ?? 0, balance >= (it.price ?? 0) ? "ink" : "inkFaint", "meta", 14);
      });
      });
      if (g.clicked !== null && g.clicked !== undefined) sel = g.clicked;
      const it = items[sel];
      kit.column("Detail", { width: phone || painted ? grow() : px(220), gap: t.sm }, () => {
        if (!it) {
          kit.label("Empty", desc.emptyLabel ?? "Nothing for sale here", { color: token("inkMuted") });
          return;
        }
        // The item's art heads the panel, as large as the column (a row beside its name on a phone).
        if (it.image && !phone) showcase("Art", it.image, it.rarity);
        if (it.image && phone) {
          kit.row("Head", { width: grow(), gap: t.md, cross: "center" }, () => {
            kit.column("Art", { width: px(60), height: px(60), align: "center", cross: "center" }, () => {
              rarityGround("Ground", it.rarity);
              cutout("Image", it.image!, px(56));
            });
            kit.label("Name", it.name ?? "", { step: "lg", font: "display", width: grow() });
          });
        } else {
          kit.label("Name", it.name ?? "", { step: "lg", font: it.image ? "display" : "body", width: grow() });
        }
        if (it.description) kit.label("Description", it.description, { step: "meta", color: token("inkMuted"), width: grow() });
        kit.row("Price", { width: grow(), align: "spaceBetween", cross: "center", gap: t.md }, () => {
          kit.label("Key", desc.priceLabel ?? "Price", { color: token("inkMuted"), nowrap: true });
          price("Value", it.price ?? 0, "ink", "body", 18, ` ${desc.currency ?? "coins"}`);
        });
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
    const name = desc.merchantName ?? "";
    const line = desc.merchantLine ?? "";
    if (side) {
      kit.row("Stage", { width: grow(), gap: t.lg }, () => {
        figure("Merchant", merchant!, 0.8, side, name, line);
        kit.column("Main", { width: grow(), gap: t.md }, () => {
          bar();
          kit.row("Body", { width: grow(), gap: t.lg }, body);
        });
      });
    } else {
      if (merchant) figure("Merchant", merchant, 1, 0, name, line);
      bar();
      if (phone) kit.column("Body", { width: grow(), gap: t.md }, body);
      else kit.row("Body", { width: grow(), gap: t.lg }, body);
    }
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
  // The slot selected when it opens (InventoryDesc.selected), so the detail shows an item at once.
  let sel = kit.memory(`${id}.sel`, desc.selected ?? -1);
  const out: { selected: number | null; moved: Api.SlotMove | null; used: string | null; closed: boolean } = { selected: null, moved: null, used: null, closed: false };
  const cell = phone ? 64 : 72;
  const columns = desc.columns ?? 0;
  // Wide enough for the stated columns at the slot size (the grid shares
  // what is left between them if the screen is narrower): the dialog's
  // padding, the detail panel and the gap beside the grid, and room for
  // its scrollbar.
  const character = desc.character ?? null;
  const pane = character && !phone ? 200 : 0;
  const chrome = 2 * t.xl + 200 + t.lg + (pane ? pane + t.lg : 0);
  const width = columns > 0 ? Math.max(680, chrome + columns * (cell + t.sm) - t.sm + 2 * t.sm) : 680;
  out.closed = screen(id, desc.title ?? "Inventory", width, () => {
    const body = (): void => {
      // Beside a character pane the slots run as tall as it.
      // Whole rows only (rowsView); with no stated columns, as many as the panel's width holds.
      const across = columns > 0 ? columns : Math.max(1, Math.floor((frame().viewport.width - 2 * t.xl + t.sm) / (cell + t.sm)));
      const limit = phone ? 300 : pane ? 420 : 340;
      const gutter = rowsView(slots, across, cell, t.sm, limit).more ? gutterOf(t) : 0;
      // On a phone the slots are at the touch target already: the gutter comes out of the gaps between them, not the slots.
      const gap = phone ? Math.max(t.xxs, t.sm - (gutter + 2 * EDGE) / Math.max(1, across - 1)) : t.sm;
      const view = rowsView(slots, across, cell, gap, limit);
      let g: Api.ListResult = {} as Api.ListResult;
      gridView("SlotsView", view.more, grow(), () => {
      g = kit.grid("Slots", { count: slots, cellWidth: cell, cellHeight: cell, columns, height: px(view.height + 2 * EDGE), width: grow(), gap, padding: { left: EDGE, top: EDGE, right: EDGE + gutter, bottom: EDGE }, selected: sel >= 0 ? sel : null }, (i) => {
        const it = bySlot.get(i);
        if (i === sel) selectedRim("Selected");
        if (!it) return;
        rarityGround("Ground", it.rarity);
        if (it.image) {
          // The cut-out stands on the slot's ground; a stack's count sits on a plate in its corner.
          cutout("Picture", it.image, px(cell - 10));
          if ((it.count ?? 0) > 1) cornerCount("Count", it.count!);
          return;
        }
        picture("Picture", it.icon, it.name ?? "", cell * 0.5);
        if ((it.count ?? 0) > 1) kit.label("Count", `${it.count}`, { step: "xs", font: "mono", color: token("inkMuted"), nowrap: true });
      });
      });
      if (g.clicked !== null && g.clicked !== undefined) sel = g.clicked;
      if (g.dropFrom !== null && g.dropFrom !== undefined && g.dropTo !== null && g.dropTo !== undefined && bySlot.has(g.dropFrom)) {
        out.moved = { from: g.dropFrom, to: g.dropTo };
        sel = g.dropTo;
      }
      const it = sel >= 0 ? bySlot.get(sel) : undefined;
      kit.column("Detail", { width: phone ? grow() : px(200), gap: t.sm }, () => {
        if (!it) {
          kit.label("Hint", desc.hintLabel ?? "Select an item. Drag it to another slot to move it.", { step: "meta", color: token("inkMuted"), width: grow() });
          return;
        }
        if (it.image && !phone) showcase("Art", it.image, it.rarity);
        kit.label("Name", it.name ?? "", { step: "lg", font: it.image ? "display" : "body", width: grow() });
        if (it.description) kit.label("Description", it.description, { step: "meta", color: token("inkMuted"), width: grow() });
        if ((it.count ?? 0) > 1) kit.keyValue("Count", desc.countLabel ?? "Count", `${it.count}`, true);
        if (desc.useLabel !== "") {
          if ((kit.button("Use", desc.useLabel ?? "Use", { variant: "primary", width: grow() }) as Api.Response).clicked) out.used = it.id ?? null;
        }
      });
    };
    const stats = desc.stats ?? [];
    const attributes = (): void => {
      if (stats.length === 0) return;
      kit.column("Stats", { width: grow(), gap: 0 }, () => {
        stats.forEach((s, i) => kit.keyValue(kit.idi("Stat", i), s.label ?? "", s.value ?? "", true));
      });
    };
    const who = (): void => figure("Character", character!, 0.75, pane, desc.characterName ?? "", desc.characterTitle ?? "", attributes, [stats[0], stats[stats.length - 1]].filter((s): s is Api.StatRow => !!s && stats.length > 0).slice(0, stats.length > 1 ? 2 : 1));
    if (pane) {
      // The character at the start, the slots and the detail beside it.
      kit.row("Stage", { width: grow(), gap: t.lg }, () => {
        who();
        kit.row("Body", { width: grow(), gap: t.lg }, body);
      });
    } else {
      if (character) who();
      if (phone) kit.column("Body", { width: grow(), gap: t.md }, body);
      else kit.row("Body", { width: grow(), gap: t.lg }, body);
    }
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
  // A screen reader hears the stars as the desc words them ("2 of 3 stars"),
  // never the icons' English names: the first star carries it, the others
  // are decoration; a locked card is named by its level, not "lock".
  const starsLabel = (stars: number, max: number): string => (desc.starsLabel ?? "{stars} of {max} stars").replaceAll("{stars}", `${stars}`).replaceAll("{max}", `${max}`);
  let back = false;
  // Back from every input: the button, a press on the scrim, Escape, B.
  out.closed = screen(id, desc.title ?? "Levels", 720, () => {
    kit.row("Total", { gap: t.xs, cross: "center" }, () => {
      iconOr("Star", "star", 18, token("gold"), starsLabel(earned, levels.length * 3));
      kit.label("Count", `${earned} / ${levels.length * 3}`, { font: "mono", color: token("inkMuted"), nowrap: true });
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
      const name = l.name ?? `${i + 1}`;
      if (l.locked) {
        iconOr("Lock", "lock", 28, token("inkFaint"), (desc.lockedLabel ?? "{name}, locked").replaceAll("{name}", name));
        return;
      }
      if ([...name].length > 3) kit.label("Name", name, { step: phone ? "base" : "lg", font: "display", align: "center", width: grow(), maxLines: 2, ellipsis: true });
      else kit.label("Name", name, { step: "xl3", font: "display", nowrap: true });
      const stars = Math.min(3, l.stars ?? 0);
      kit.row("Stars", { gap: 2 }, () => {
        for (let k = 0; k < 3; k++) iconOr(kit.idi("Star", k), "star", 14, token(k < stars ? "gold" : "controlStrong"), k === 0 ? starsLabel(stars, 3) : "");
      });
    });
    if (g.clicked !== null && g.clicked !== undefined && !levels[g.clicked]?.locked) out.chosen = g.clicked;
    kit.row("Footer", { width: grow(), align: "end" }, () => {
      if ((kit.button("Back", desc.backLabel ?? "Back", {}) as Api.Response).clicked) back = true;
    });
  }, true);
  out.closed ||= back;
  return out;
}

// The award metals of packages/design's tokens (`metal.*`: highlight, mid and shade, sRGB): a
// medal is material, not a semantic role, so bronze never borrows warn and silver never reads as
// rank 4. Each disc is lit from above, highlight to shade, with a highlight rim on its top edge.
const MEDALS: ReadonlyArray<readonly [Api.Color, Api.Color, Api.Color]> = [
  [{ r: 1, g: 0.902, b: 0.502 }, { r: 1, g: 0.8, b: 0 }, { r: 0.722, g: 0.525, b: 0.043 }],
  [{ r: 0.957, g: 0.965, b: 0.973 }, { r: 0.722, g: 0.753, b: 0.8 }, { r: 0.478, g: 0.51, b: 0.557 }],
  [{ r: 0.953, g: 0.776, b: 0.627 }, { r: 0.788, g: 0.514, b: 0.302 }, { r: 0.541, g: 0.294, b: 0.133 }],
];

function medalDraw(rank: number): Api.DrawDesc {
  const [hi, mid, shade] = MEDALS[rank - 1]!;
  return {
    shape: { kind: "ellipse" },
    fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: { color: hi } }, { offset: 0.5, color: { color: mid } }, { offset: 1, color: { color: shade } }] }],
    effects: [{ kind: "innerShadow", offset: { x: 0, y: 1 }, blur: 0, spread: 0, color: { color: hi, alpha: 0.9 } }],
  } as Api.DrawDesc;
}

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
        kit.label("Message", desc.empty ?? "No scores yet", { color: token("inkMuted"), align: "center", width: grow() });
      });
    } else {
      const player = entries.findIndex((e) => e.player);
      kit.list("Rows", { count: entries.length, rowHeight: t.row, height, width: grow(), selected: player >= 0 ? player : null }, (i) => {
        const e = entries[i]!;
        const rank = e.rank ?? i + 1;
        // Ranks 1 to 3 on a filled metal disc with the numeral on it; the rest plain muted numerals.
        kit.row("Rank", { width: px(40), cross: "center" }, () => {
          if (rank <= 3) {
            kit.row("Medal", { width: px(26), height: px(26), align: "center", cross: "center" }, () => {
              kit.setDraw(medalDraw(rank));
              // Semibold near-black (#15151a) on the metal, so 1 and 2 read on their bright discs.
              kit.richText("Numeral", `[b]${rank}[/b]`, { font: "mono", step: "meta", color: { color: { r: 0.082, g: 0.082, b: 0.102 } }, nowrap: true });
            });
          } else {
            kit.label("Numeral", `${rank}`, { font: "mono", color: token("inkMuted"), nowrap: true });
          }
        });
        kit.label("Name", e.name ?? "", { width: grow(), nowrap: true });
        kit.label("Score", e.score ?? "", { font: "mono", nowrap: true });
      });
    }
  });
  kit.remember(`${id}.scope`, scope);
  out.scope = scope;
  return out;
}
