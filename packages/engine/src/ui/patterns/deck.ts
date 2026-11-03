// A deck builder over the collection (docs/ui.md §11 genre patterns, Appendix B: deck builder, collection). Read
// against Hearthstone's, Marvel Snap's and Legends of Runeterra's (the filters by cost, tier and kind with the search
// and the sort; the collection's grid of card faces with the copies owned; the deck's list by cost with its count, the
// mana curve and the kinds' counts; Done) and B-cards-deck. Each card is the face og.ui.cardHand draws (cards.ts). The
// collection, the deck and what they hold are the game's: this filters and sorts what it is given, draws it, and reports
// what was asked.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, clamp, eyebrow, flags, frame, glow, grow, keyHash, kit, lastInteraction, line, mix, over, pct, pointer, px, rarityColor, rememberRoom,
  role, safeWidth, shadow, st, still, stIn, surface, symbol, text, token, u, vgrad, xy, type Mut,
} from "./core.ts";
import { rtl } from "../containers/base.ts";
import { cardFace } from "./cards.ts";
import { inputField } from "../controls/fields.ts";
import { dropdown } from "../controls/pickers.ts";
import { tps } from "../feedback/core.ts";

const RANK: Record<string, number> = { mythic: 6, legendary: 5, epic: 4, rare: 3, uncommon: 2, common: 1, none: 0 };
const COSTS = 8;

/** A card as og.ui.cardHand's face draws it. */
const face = (c: Api.CollectionCard): Api.HandCard => ({
  key: c.key ?? "",
  name: c.name ?? "",
  cost: c.cost ?? null,
  typeLine: c.typeLine ?? "",
  text: c.text ?? "",
  image: c.image ?? null,
  rarity: c.rarity ?? "none",
  playable: true,
});

/**
 * A deck builder (Hearthstone, Marvel Snap, Legends of Runeterra): see `fn deckBuilder` in crates/og-api/api/ui.ogt.
 */
export function deckBuilder(id: string, desc: Api.DeckBuilderDesc): Api.DeckBuilderResult {
  const out: Mut<Api.DeckBuilderResult> = { add: null, remove: null, refused: null, costFilter: null, rarityFilter: null, kindFilter: null, search: null, sort: null, view: null, done: false };
  const { viewport: vp, t, phone, touch } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const W = kit.memory(`~db.w.${id}`, 0) || firstRoom().w;
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  // Read and written outside the pattern's element (a memory key is the open element's): the last refusal, the room's place.
  const rt = kit.memory(`~db.rt.${id}`, -1);
  const rk = kit.memory(`~db.rk.${id}`, 0);
  const rwhy = kit.memory(`~db.rw.${id}`, 0);
  const rx = kit.memory(`~db.x.${id}`, 0);
  const ry = kit.memory(`~db.y.${id}`, 0);
  const rw = kit.memory(`~db.rw2.${id}`, 0);
  const rh = kit.memory(`~db.rh.${id}`, 0);
  const cards = desc.cards ?? [];
  const size = desc.deckSize ?? 30;
  const inDeck = cards.reduce((n, c) => n + (c.count ?? 0), 0);
  const allowed = (c: Api.CollectionCard): number => Math.min(c.owned ?? 0, (c.limit ?? 0) > 0 ? c.limit! : desc.copyLimit ?? 2);
  let why = 0;
  const showDeck = portrait && desc.view === "deck";
  // The deck's column beside the collection on a desktop.
  const deckW = Math.round(Math.min(u(360), W * 0.27) * Math.min(1.25, ts));

  // The collection through the filters, in the sort's order.
  const q = (desc.search ?? "").trim().toLocaleLowerCase();
  const cf = desc.costFilter ?? -1;
  const shown = cards.filter((c) => {
    if (cf >= 0 && (c.cost == null || (cf >= COSTS - 1 ? c.cost < COSTS - 1 : c.cost !== cf))) return false;
    if (desc.rarityFilter && desc.rarityFilter !== "none" && c.rarity !== desc.rarityFilter) return false;
    if (desc.kindFilter && c.kind !== desc.kindFilter) return false;
    return !q || (c.name ?? "").toLocaleLowerCase().includes(q);
  });
  const byName = (a: Api.CollectionCard, b: Api.CollectionCard) => (a.name ?? "").localeCompare(b.name ?? "");
  const byCost = (a: Api.CollectionCard, b: Api.CollectionCard) => (a.cost ?? 99) - (b.cost ?? 99) || byName(a, b);
  const sorts: Record<string, (a: Api.CollectionCard, b: Api.CollectionCard) => number> = {
    cost: byCost,
    name: byName,
    rarity: (a, b) => (RANK[b.rarity ?? "none"] ?? 0) - (RANK[a.rarity ?? "none"] ?? 0) || byCost(a, b),
    owned: (a, b) => (b.owned ?? 0) - (a.owned ?? 0) || byCost(a, b),
  };
  const order = sorts[desc.sort ?? ""];
  if (order) shown.sort(order);

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 8 : 14), part: "DeckBuilder.frame", partClass: "panel" });

  // A filter's chip: its mark and word; the chosen one lit. A whole touch target under a finger.
  const chip = (key: string, label: string, on: boolean, mark?: () => void): boolean => {
    const r = surface(key, {
      part: `DeckBuilder.chip${flags(on && "selected")}`,
      cls: "chip",
      fill: on ? { ...role("primary"), a: 0.24 } : { ...role("shadow"), a: 0.5 },
      washBy: 0.14,
      gap: u(6),
      align: "center",
      cross: "center",
      // A whole touch target under a finger.
      ...(touch ? { width: { kind: "fit", min: 44 } as Api.Size } : {}),
      height: { kind: "fit", min: touch ? 44 : u(30) },
      padding: xy(u(label.length <= 2 ? 8 : 10), u(3)),
      radius: Math.max(4, t.small),
      semantics: { role: "radio", name: label, checked: on } as never,
    }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [], strokes: [line(1, on ? token("primary", 0.9) : token("ink", 0.14))] } as never);
      mark?.();
      text("Label", label, { step: st(13), nowrap: true, color: on ? "ink" : "inkMuted" });
    });
    if (r.clicked && !on) kit.feedback("select", -1);
    return r.clicked && !on;
  };

  // The filters: cost, tier and kind, each with All; the search and the sort.
  const costs = (): void => {
    kit.row("Costs", { gap: u(4), cross: "center" }, () => {
      if (chip("-1", desc.allLabel ?? "All", cf < 0)) out.costFilter = -1;
      for (let n = 0; n < COSTS; n++) if (chip(String(n), n === COSTS - 1 ? `${n}+` : String(n), cf === n)) out.costFilter = n;
    });
    kit.semantics({ role: "radiogroup", name: "Cost" });
  };
  const rarities = (): void => {
    if ((desc.rarities ?? []).length === 0) return;
    kit.row("Rarities", { gap: u(4), cross: "center" }, () => {
      const rf = desc.rarityFilter ?? "none";
      if (chip("none", desc.allLabel ?? "All", rf === "none")) out.rarityFilter = "none";
      for (const r of desc.rarities ?? []) {
        const tier = rarityColor(r.rarity) ?? role("inkMuted");
        if (chip(r.rarity ?? "none", r.label ?? "", rf === r.rarity, () => symbol("Mark", "gem", u(12), tier))) out.rarityFilter = r.rarity ?? "none";
      }
    });
    kit.semantics({ role: "radiogroup", name: "Rarity" });
  };
  const kinds = (): void => {
    if ((desc.kinds ?? []).length === 0) return;
    kit.row("Kinds", { gap: u(4), cross: "center" }, () => {
      const kf = desc.kindFilter ?? "";
      if (chip("all", desc.allLabel ?? "All", kf === "")) out.kindFilter = "";
      for (const k of desc.kinds ?? []) {
        if (chip(k.key ?? "", k.label ?? "", kf === k.key, k.image ? () => artSlot("Mark", "DeckBuilder.kindMark", { texture: k.image, kind: "icon", width: px(u(16)), height: px(u(16)), radius: 2, label: "" }) : undefined)) out.kindFilter = k.key ?? "";
      }
    });
    kit.semantics({ role: "radiogroup", name: "Kind" });
  };
  const search = (w: Api.Size): void => {
    const r = inputField("Search", desc.search ?? "", { placeholder: desc.searchLabel ?? "Search cards", icon: "search", clearable: true, width: w });
    if (r.changed) out.search = r.value;
  };
  const sort = (w: Api.Size): void => {
    const list = desc.sorts ?? [];
    if (list.length === 0) return;
    const i = list.findIndex((s) => s.key === desc.sort);
    const r = dropdown("Sort", { options: list.map((s) => ({ label: `${desc.sortLabel ?? "Sort"}: ${s.label ?? s.key ?? ""}` })), selected: i, width: w, searchable: false });
    if (r.changed && r.selected >= 0) out.sort = list[r.selected]?.key ?? "";
  };
  const filters = (): void => {
    if (portrait) {
      kit.column("Filters", { width: grow(), gap: u(6) }, () => {
        kit.openScroll("Chips", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal" } as never);
        kit.open("Row", { direction: "row", gap: u(10), cross: "center" });
        costs();
        rarities();
        kinds();
        kit.close();
        kit.close();
        // Large text stacks the search over the sort: side by side they no longer fit a phone's width.
        const stack = ts > 1.25;
        kit.open("Find", { direction: stack ? "column" : "row", width: grow(), gap: u(8), cross: stack ? "stretch" : "center" });
        search(grow());
        sort(stack ? grow() : px(Math.round(u(150) * Math.min(1.3, ts))));
        kit.close();
      });
    } else {
      kit.row("Filters", { width: grow(), gap: u(14), cross: "center", wrap: true } as never, () => {
        costs();
        rarities();
        kinds();
        kit.spacer("Gap", grow());
        search(px(Math.round(u(220) * Math.min(1.3, ts))));
        sort(px(Math.round(u(190) * Math.min(1.3, ts))));
      });
    }
  };

  // A collection card: the face, a mark of the copies in the deck at its top end, its copies owned in their own row under it.
  let preview: { c: Api.CollectionCard; r: Api.UiRect; cw: number } | null = null;
  const tile = (c: Api.CollectionCard, cw: number, ch: number): void => {
    const k = c.key ?? "";
    const owned = c.owned ?? 0;
    const n = c.count ?? 0;
    const unowned = owned === 0;
    const full = !unowned && n >= allowed(c);
    const ix = lastInteraction(k);
    const shaking = rt >= 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.6 : 0;
    const pad = Math.max(4, Math.round(cw * 0.045));
    const rr = Math.max(6, Math.round(cw * 0.07));
    const r = surface(k, {
      part: `DeckBuilder.card${flags(unowned && "unowned", full && "full")}`,
      cls: "card",
      fill: null,
      washBy: 0.04,
      direction: "column",
      gap: u(4),
      cross: "center",
      width: px(cw),
      lift: unowned ? 0 : u(4),
      semantics: { role: "button", name: [c.name ?? k, c.cost != null ? `${c.cost}` : "", c.typeLine ?? "", `×${owned}`, n > 0 ? `${n}/${allowed(c)}` : ""].filter(Boolean).join(", ") },
      props: shaking ? { transform: { x: shaking } } : {},
    }, () => {
      kit.open("Face", { direction: "column", width: px(cw), height: px(ch), padding: xy(pad, pad), gap: Math.max(3, Math.round(cw * 0.03)), cross: "stretch", part: `Card.frame${flags(`rarity=${c.rarity ?? "none"}`)}`, partClass: "card" });
      cardFace(face(c), cw, ch, { playable: true, ready: false, rules: false, focused: ix.focused });
      if (unowned || full) {
        kit.open("Dim", { width: px(cw), height: px(ch), part: "DeckBuilder.dim", partClass: "panel" });
        kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("shadow", unowned ? 0.62 : 0.34) }] });
        kit.close();
        over({ insets: { start: 0, top: 0 }, inert: true, z: 7 });
      }
      if (n > 0) {
        const g = Math.round(Math.max(22, cw * 0.17) * Math.min(1.4, ts));
        kit.open("InDeck", { width: px(g), height: px(g), align: "center", cross: "center", part: "DeckBuilder.inDeck", partClass: "badge" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("primary") }], strokes: [line(Math.max(1.5, u(2)), token("shadow", 0.7), "outside")], effects: [glow(u(6), token("primary", 0.5), 0)] });
        text("Value", String(n), { step: stIn(14, Math.round(g * 0.62)), font: "display", nowrap: true, color: "onPrimary" });
        kit.close();
        over({ insets: { end: Math.max(3, Math.round(pad * 0.5)), top: Math.max(3, Math.round(pad * 0.5)) }, inert: true, z: 8 });
      }
      kit.close();
      text("Owned", `×${owned}`, { step: st(14), font: "mono", nowrap: true, align: "center", color: unowned ? "inkFaint" : full ? "inkMuted" : "ink" });
    });
    const rect = kit.bounds(kit.lastPath());
    // The card hovered or focused stands large over the grid with its rules (not under a finger: a tap adds it).
    if (!touch && (ix.hovered || ix.focused) && rect) preview = { c, r: rect, cw };
    if (r.clicked) {
      const reason = unowned ? 3 : inDeck >= size ? 1 : n >= allowed(c) ? 2 : 0;
      if (reason > 0) {
        out.refused = k;
        why = reason;
        kit.feedback("error", -1);
      } else {
        out.add = k;
        kit.feedback("confirm", -1);
      }
    }
  };

  // The collection's grid: whole rows that scroll, and the count of the cards owned.
  const collection = (w: Api.Size): void => {
    kit.column("Collection", { width: w, height: grow(), gap: u(8) }, () => {
      // The grid's width: the room less the deck beside it (never last tick's grid, which its own rows would hold open).
      const CW = Math.max(200, portrait ? W : W - deckW - u(16));
      const gap = u(phone ? 8 : 14);
      const want = u(portrait ? 104 : 150) * Math.min(1.3, ts);
      const cols = Math.max(2, Math.floor((CW - u(8) + gap) / (want + gap)));
      const cw = Math.floor((CW - u(8) - gap * (cols - 1)) / cols);
      const ch = Math.round(cw * 1.4);
      kit.openScroll("Grid", { width: grow(), height: grow(), gap, padding: xy(u(4), u(8)), fade: 32, fadeInto: "shadow" } as never);
      if (shown.length === 0) text("Empty", desc.emptyLabel ?? "No cards match", { step: st(16), color: "inkMuted", width: grow(), align: "center" });
      for (let r0 = 0; r0 < shown.length; r0 += cols) {
        kit.row(kit.idi("Line", r0 / cols), { gap }, () => shown.slice(r0, r0 + cols).forEach((c) => tile(c, cw, ch)));
      }
      kit.close();
      kit.semantics({ role: "grid", name: desc.collectionLabel ?? "Cards" });
      const ownedN = cards.filter((c) => (c.owned ?? 0) > 0).length;
      if (!portrait) text("Count", `${ownedN} / ${cards.length} ${desc.collectionLabel ?? "Cards"}`, { step: st(14), font: "mono", color: "inkMuted", nowrap: true });
    });
  };

  // A deck row: its cost gem, its tier's mark, its name over a strip of its art fading in, its copies. A press takes one out.
  const deckRow = (c: Api.CollectionCard): void => {
    const k = c.key ?? "";
    const tier = rarityColor(c.rarity) ?? role("inkMuted");
    const h = Math.round(u(phone ? 44 : 40) * Math.min(1.4, ts));
    const rr = Math.max(4, t.small);
    const r = surface(k, {
      part: "DeckBuilder.row",
      cls: "listItem",
      fill: null,
      gap: u(8),
      cross: "center",
      width: grow(),
      height: px(Math.max(44, h)),
      padding: xy(u(6), 0),
      radius: rr,
      semantics: { role: "button", name: [c.name ?? k, c.cost != null ? `${c.cost}` : "", `×${c.count ?? 0}`].filter(Boolean).join(", ") },
    }, () => {
      const ix = lastInteraction(k);
      kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("panel", 0.88) }], strokes: [line(1, ix.hovered || ix.focused ? token("ink", 0.4) : token("ink", 0.08))], clip: "shape" } as Api.DrawDesc);
      // The strip of its art at the end, fading in from the row's ground under the name.
      artSlot("Strip", "DeckBuilder.strip", { texture: c.image ?? null, kind: "card", width: pct(0.55), height: grow(), radius: 0, label: "", focal: { x: 0.5, y: 0.35 } }, () => {
        kit.open("Fade", { width: grow(), height: grow() });
        kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("panel", 0.96)], [0.55, token("panel", 0.5)], [1, token("panel", 0.1)]], rtl() ? 270 : 90)] });
        kit.close();
      });
      over({ insets: { end: 0, top: 0, bottom: 0 }, inert: true });
      // The edge in its tier's colour.
      kit.open("Edge", { width: px(Math.max(2, u(3))), height: grow() });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { ...tier, a: 0.9 } as never }] });
      kit.close();
      over({ insets: { start: 0, top: 0, bottom: 0 }, inert: true });
      const g = Math.round(u(26) * Math.min(1.4, ts));
      kit.open("Cost", { width: px(g), height: px(g), align: "center", cross: "center", part: "DeckBuilder.cost", partClass: "badge" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "gradient", type: "radial", from: { x: 0.36, y: 0.3 }, to: { x: 0.62, y: 0.62 }, stops: [{ offset: 0, color: token("highlight") }, { offset: 1, color: { ...mix(role("highlight"), role("well"), 0.55) } as never }] }], strokes: [line(1, token("shadow", 0.7), "outside")] } as Api.DrawDesc);
      text("Value", c.cost != null ? String(c.cost) : "–", { step: stIn(14, Math.round(g * 0.62)), font: "display", nowrap: true });
      kit.close();
      symbol("Tier", "gem", u(12), tier);
      text("Name", c.name ?? k, { step: st(15), nowrap: true, ellipsis: true, width: grow() });
      text("Copies", String(c.count ?? 0), { step: st(15), font: "mono", nowrap: true, color: (c.count ?? 0) >= allowed(c) ? "highlight" : "ink" });
    });
    if (r.clicked) {
      out.remove = k;
      kit.feedback("confirm", -1);
    }
  };

  // The mana curve: a bar per cost (7 and up the last), its count over it.
  const curve = (): void => {
    const counts = Array.from({ length: COSTS }, () => 0);
    for (const c of cards) if ((c.count ?? 0) > 0 && c.cost != null) counts[Math.min(COSTS - 1, c.cost)]! += c.count!;
    const top = Math.max(1, ...counts);
    const barH = u(phone ? 36 : 40);
    kit.column("Curve", { width: grow(), gap: u(6), padding: xy(u(12), u(10)), part: "DeckBuilder.curve", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }], strokes: [line(1, token("ink", 0.1))] });
      eyebrow("Title", desc.curveLabel ?? "Mana curve");
      kit.row("Bars", { width: grow(), gap: u(4), cross: "end" }, () => {
        counts.forEach((n, i) => {
          kit.column(kit.idi("Cost", i), { width: grow(), gap: u(2), cross: "center", part: "DeckBuilder.bar", partClass: "badge" }, () => {
            text("Count", String(n), { step: st(12), font: "mono", nowrap: true, color: n > 0 ? "ink" : "inkFaint" });
            kit.open("Bar", { width: pct(0.7), height: px(Math.max(2, Math.round((barH * n) / top))) });
            kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 0, 0] }, fills: [vgrad([[0, token("primary")], [1, token("primary", 0.55)]])], ...(n > 0 ? { effects: [glow(u(4), token("primary", 0.35), 0)] } : {}) });
            kit.close();
            text("Label", i === COSTS - 1 ? `${i}+` : String(i), { step: st(12), font: "mono", nowrap: true, color: "inkMuted" });
          });
        });
      });
    });
    kit.semantics({ role: "image", name: `${desc.curveLabel ?? "Mana curve"}: ${counts.map((n, i) => `${i === COSTS - 1 ? `${i}+` : i}: ${n}`).join(", ")}` });
  };

  // The kinds' counts.
  const kindCounts = (): void => {
    const list = desc.kinds ?? [];
    if (list.length === 0) return;
    kit.column("Kinds", { width: grow(), gap: u(4), padding: xy(u(12), u(10)), part: "DeckBuilder.kinds", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }], strokes: [line(1, token("ink", 0.1))] });
      eyebrow("Title", desc.kindsLabel ?? "Types");
      for (const k of list) {
        const n = cards.filter((c) => c.kind === k.key).reduce((s, c) => s + (c.count ?? 0), 0);
        kit.row(k.key ?? "", { width: grow(), gap: u(8), cross: "center" }, () => {
          if (k.image) artSlot("Mark", "DeckBuilder.kindMark", { texture: k.image, kind: "icon", width: px(u(18)), height: px(u(18)), radius: 2, label: "" });
          text("Label", k.label ?? "", { step: st(13), nowrap: true, ellipsis: true, width: grow(), color: "inkMuted" });
          text("Count", String(n), { step: st(13), font: "mono", nowrap: true });
        });
      }
    });
  };

  // A refusal's reason, for a while after it.
  const refusal = (): void => {
    const recent = rt >= 0 && tick - rt < Math.round(2.2 * tps());
    if (!recent || rwhy <= 0) return;
    const words = rwhy === 1 ? desc.fullLabel ?? "The deck is full" : rwhy === 2 ? desc.limitLabel ?? "No more copies allowed" : desc.unownedLabel ?? "Not owned";
    kit.row("Refusal", { width: grow(), gap: u(6), cross: "center", part: "DeckBuilder.refusal", partClass: "badge" }, () => {
      symbol("Mark", "ban", u(14), role("danger"));
      text("Text", words, { step: st(13), color: "danger", width: grow() });
    });
    kit.semantics({ role: "status", name: words });
  };

  // The deck: its count and a refusal's reason, its name over its art, its rows by cost, the curve, the counts, Done.
  const deck = (w: Api.Size): void => {
    kit.column("Deck", { width: w, height: grow(), gap: u(10), padding: xy(u(12), u(12)), part: "DeckBuilder.deck", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.72) }], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(6), u(24), 0.5)] });
      kit.row("Head", { width: grow(), gap: u(8), cross: "center" }, () => {
        eyebrow("Title", desc.deckLabel ?? "My deck");
        kit.spacer("Gap", grow());
        text("Count", `${inDeck}/${size}`, { step: st(16), font: "mono", nowrap: true, color: inDeck > size ? "danger" : inDeck === size ? "success" : "ink" });
      });
      kit.semantics({ role: "status", name: `${desc.deckLabel ?? "My deck"} ${inDeck}/${size}` });
      refusal();
      if (desc.deckName) {
        const nh = Math.round(u(52) * Math.min(1.4, ts));
        kit.row("Name", { width: grow(), height: px(nh), cross: "center", padding: xy(u(12), 0), part: "DeckBuilder.name", partClass: "badge" }, () => {
          const rr = Math.max(4, t.small);
          kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("well", 0.9) }], strokes: [line(1, token("primary", 0.5))], clip: "shape" } as Api.DrawDesc);
          if (desc.deckImage) {
            artSlot("Art", "DeckBuilder.deckArt", { texture: desc.deckImage, kind: "card", width: pct(0.55), height: grow(), radius: 0, label: "", focal: { x: 0.5, y: 0.3 } }, () => {
              kit.open("Fade", { width: grow(), height: grow() });
              kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, token("well", 0.96)], [1, token("well", 0.05)]], rtl() ? 270 : 90)] });
              kit.close();
            });
            over({ insets: { end: 0, top: 0, bottom: 0 }, inert: true });
          }
          text("Text", desc.deckName ?? "", { step: st(19), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
        });
      }
      const rows = cards.filter((c) => (c.count ?? 0) > 0).sort(byCost);
      // At a large text size the curve and the counts scroll under the rows: standing fixed, they leave the list no room.
      const big = ts > 1.25;
      const stats = (): void => {
        if (big) {
          curve();
          kindCounts();
        } else kit.row("Stats", { width: grow(), gap: u(8) }, () => {
          curve();
          kindCounts();
        });
      };
      kit.openScroll("List", { width: grow(), height: grow(), gap: u(4), padding: xy(0, 0), fade: 24, fadeInto: "shadow" } as never);
      rows.forEach(deckRow);
      if (big) stats();
      kit.close();
      kit.semantics({ role: "list", name: desc.deckName || desc.deckLabel || "Deck" });
      if (!big) stats();
      if (btn("Done", desc.doneLabel ?? "Done", { variant: "primary", size: "lg", width: grow(), raised: true }).clicked) {
        out.done = true;
        kit.feedback("confirm", -1);
      }
    });
  };

  // A phone in portrait: the two halves are tabs, the collection and the deck with its count.
  const views = (): void => {
    kit.row("Views", { width: grow(), gap: u(6) }, () => {
      const tab = (key: string, label: string, on: boolean): void => {
        const r = surface(key, {
          part: `DeckBuilder.view${flags(on && "selected")}`,
          cls: "chip",
          fill: on ? { ...role("primary"), a: 0.26 } : { ...role("shadow"), a: 0.55 },
          washBy: 0.14,
          width: grow(),
          height: { kind: "fit", min: 44 },
          align: "center",
          cross: "center",
          padding: xy(u(10), u(6)),
          radius: 999,
          semantics: { role: "tab", name: label, selected: on },
        }, () => text("Label", label, { step: st(15), font: "display", nowrap: true, ellipsis: true, color: on ? "ink" : "inkMuted" }));
        if (r.clicked && !on) {
          out.view = key;
          kit.feedback("tab", -1);
        }
      };
      tab("cards", desc.collectionLabel ?? "Cards", !showDeck);
      tab("deck", `${desc.deckLabel ?? "My deck"} ${inDeck}/${size}`, showDeck);
    });
    kit.semantics({ role: "tablist", name: desc.title || "Deck builder" });
  };

  if (portrait) {
    views();
    if (showDeck) deck(grow());
    else {
      filters();
      refusal();
      collection(grow());
    }
  } else {
    // The deck stands the whole height beside the filters and the grid, so its list keeps its rows.
    kit.row("Body", { width: grow(), height: grow(), gap: u(16) }, () => {
      kit.column("Cards", { width: grow(), height: grow(), gap: u(12) }, () => {
        filters();
        collection(grow());
      });
      deck(px(deckW));
    });
  }

  // The card hovered or focused, large with its rules, over the grid beside where it stands.
  const pv = preview as { c: Api.CollectionCard; r: Api.UiRect; cw: number } | null;
  if (pv && rw > 0 && !pointer().down) {
    const bw = Math.round(Math.max(pv.cw * 1.45, u(220)) * Math.min(1.3, ts));
    const bh = Math.round(bw * 1.4);
    const cx = pv.r.x + pv.r.width / 2 - rx;
    const cy = pv.r.y + pv.r.height / 2 - ry;
    const x = clamp(cx - bw / 2, 0, Math.max(0, rw - bw));
    const y = clamp(cy - bh / 2, 0, Math.max(0, rh - bh));
    const pad = Math.max(4, Math.round(bw * 0.045));
    kit.open("Preview", { direction: "column", width: px(bw), height: px(bh), padding: xy(pad, pad), gap: Math.max(3, Math.round(bw * 0.03)), cross: "stretch", part: `Card.frame${flags(`rarity=${pv.c.rarity ?? "none"}`, "inspect")}`, partClass: "card" });
    cardFace(face(pv.c), bw, bh, { playable: true, ready: false, rules: true, big: true });
    kit.close();
    over({ insets: { start: rtl() ? rw - x - bw : x, top: y }, inert: true, z: 20 });
    kit.semantics({ role: "tooltip", name: [pv.c.name ?? "", pv.c.typeLine ?? "", pv.c.text ?? ""].filter(Boolean).join(". ") } as never);
  }

  kit.close();
  const self = kit.bounds(kit.lastPath());
  rememberRoom("db", id);
  if (self && self.width > 0) {
    kit.remember(`~db.x.${id}`, Math.round(self.x));
    kit.remember(`~db.y.${id}`, Math.round(self.y));
    kit.remember(`~db.rw2.${id}`, Math.round(self.width));
    kit.remember(`~db.rh.${id}`, Math.round(self.height));
  }
  if (out.refused) {
    kit.remember(`~db.rt.${id}`, tick);
    kit.remember(`~db.rk.${id}`, keyHash(out.refused));
    kit.remember(`~db.rw.${id}`, why);
  } else {
    kit.remember(`~db.rt.${id}`, rt);
    kit.remember(`~db.rk.${id}`, rk);
    kit.remember(`~db.rw.${id}`, rwhy);
  }
  return out;
}
