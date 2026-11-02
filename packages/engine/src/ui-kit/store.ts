// Collections: the shop, the inventory (drag and drop between slots), the
// level select and the leaderboard. Each is a dialog-shaped screen with a
// focus scope; Back closes it.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { all as edges, hudBox, fit, fmt, frame, grow, iconOr, px, token, xy } from "./util.ts";
import { artSlot } from "../ui/foundations/media.ts";
import { over } from "../ui/containers/base.ts";
import { rarityColor } from "../ui/feedback/core.ts";

/**
 * A slot's ground in its rarity's colour (the style's rarity roles): a glow
 * from the centre under the art, so a cut-out item stands on its tier and no
 * square of its own. Built first in the cell, under what follows.
 */
export function rarityGround(id: string, rarity: Api.Rarity | undefined): void {
  const c = rarityColor(rarity);
  if (!c) return;
  const ink = (alpha: number): Api.Paint => ({ color: { r: c.r, g: c.g, b: c.b }, alpha });
  kit.open(id, { width: grow(), height: grow() });
  kit.setDraw({ shape: { kind: "rect", radii: [6, 6, 6, 6] }, clip: "shape", fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.55 }, to: { x: 1.05, y: 1.05 }, stops: [{ offset: 0, color: ink(0.55) }, { offset: 1, color: ink(0.12) }] }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: ink(0.7) } }] } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 0 });
}

/** A cut-out item (square art on transparent ground): no frame of its own, the ground under it shows through. */
function cutout(id: string, image: Api.Texture, width: Api.Size, label = ""): void {
  kit.image(id, { texture: image, width, aspect: 1, label });
}

/** The detail panel's head: the item's cut-out large on its rarity's ground. */
function showcase(id: string, image: Api.Texture, rarity: Api.Rarity | undefined): void {
  kit.column(id, { width: grow(), aspect: 1.25, align: "center", cross: "center", padding: edges(8) }, () => {
    rarityGround("Ground", rarity);
    cutout("Image", image, grow());
  });
}

/** A portrait in the style's card frame, its lower edge fading into the panel's ground. */
function portrait(id: string, art: Api.Texture, aspect: number, label: string): void {
  const { t } = frame();
  kit.column(id, { width: grow(), padding: edges(0), radius: t.panel, frame: "card" }, () => {
    artSlot("Art", { texture: art, kind: "portrait", aspect, focal: { x: 0.5, y: 0.25 }, radius: t.panel, width: grow(), label });
    kit.open("Vignette", { width: grow(), height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, clip: "shape", fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0.5 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: token("panel", 0) }, { offset: 1, color: token("panel", 0.85) }] }] } as Api.DrawDesc);
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
  });
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
      const gw = 3 * (cw + t.sm) + t.sm;
      const g = kit.grid("Items", { count: items.length, cellWidth: cw, cellHeight: ch, columns: wide ? 3 : 0, height: fit(0, phone ? 290 : 340), width: wide ? px(gw) : grow(), selected: sel }, (i) => {
        const it = items[i]!;
        rarityGround("Ground", it.rarity);
        artOr("Picture", it.image, it.icon, it.name ?? "", phone ? 64 : 84, 36);
        // A product's name is never cut: two lines, a step smaller on a phone.
        kit.label("Name", it.name ?? "", { step: phone ? "meta" : "base", maxLines: 2, width: px(cw - 12), align: "center" });
        if (it.owned) kit.badge("Owned", desc.ownedLabel ?? "Owned", "success");
        else price("Price", it.price ?? 0, balance >= (it.price ?? 0) ? "ink" : "inkFaint", "meta", 14);
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
      const g = kit.grid("Slots", { count: slots, cellWidth: cell, cellHeight: cell, columns, height: px(phone ? 300 : pane ? 420 : 340), width: grow(), selected: sel >= 0 ? sel : null }, (i) => {
        const it = bySlot.get(i);
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
