// The store (docs/ui.md §11 genre patterns, Appendix B: store, bundles and
// rotations): the purse at its head, then sections of offers on rotation,
// each offer a tile of art in its tier's rim. The game holds the offers,
// the purse and what the player owns; a press answers what the player
// asked for (chosen, viewed, refused, a top-up) for the game to confirm
// and charge. A refusal's shake is the component's own state (og.ui.memory).

import type * as Api from "@origingame/engine";
import {
  artSlot, countdown, fill, frame, grouped, grow, hex, keyHash, kit, lastInteraction, line, mix, over, paint, pct, priceTag, px, rarityColor, role,
  scrimFill, shadow, glow, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, isPhone, safeWidth, type Mut, type Rgba, type TimeWords,
} from "./core.ts";
import { cornerTag } from "./rewards.ts";

/** What a press on an offer would do. */
type Availability = "available" | "short" | "owned" | "soldOut";

function availability(o: Api.StoreOffer): Availability {
  if (o.owned) return "owned";
  if ((o.limit ?? 0) > 0 && (o.bought ?? 0) >= (o.limit ?? 0)) return "soldOut";
  if (o.affordable === false) return "short";
  return "available";
}

/** The purse: each currency a glass pill (its art or icon, the amount), a plus after one that can be bought. */
function wallet(desc: Api.StoreFrontDesc, out: Mut<Api.StoreFrontResult>): void {
  const entries = desc.wallet ?? [];
  if (entries.length === 0) return;
  kit.open("Wallet", { direction: "row", gap: u(10), cross: "center", wrap: true, part: "Store.wallet", partClass: "badge" });
  for (const w of entries) {
    const key = w.key || w.name || "currency";
    kit.open(key, { direction: "row", gap: u(8), cross: "center", padding: { left: u(6), top: u(4), right: w.topUp ? u(4) : u(14), bottom: u(4) }, part: "Store.currency", partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.58) }, vgrad([[0, paint(hex("#ffffff"), 0.1)], [1, paint(hex("#ffffff"), 0.02)]])], effects: [shadow(u(2), u(6), 0.35)] });
    priceTag("Amount", { amount: w.amount ?? 0, currency: w.name ?? "", image: w.image ?? null, size: "md" });
    if (w.topUp) {
      const r = surface("TopUp", {
        part: "Store.topUp",
        cls: "badge",
        fill: mix(role("panelRaised"), role("ink"), 0.16),
        // A whole touch target under a finger (26 px was a third short of one).
        width: px(Math.max(u(26), frame().touch ? 44 : 0)),
        height: px(Math.max(u(26), frame().touch ? 44 : 0)),
        align: "center",
        cross: "center",
        radius: 999,
        pressScale: 0.92,
        semantics: { role: "button", name: fill(desc.topUpLabel ?? "Get more {name}", { name: w.name ?? key }) },
      }, () => {
        symbol("Plus", "plus", u(12), role("ink"), { weight: Math.max(2, u(2.4)) });
      });
      if (r.clicked) {
        out.topUp = key;
        kit.feedback("select", -1);
      }
    }
    kit.close();
    kit.semantics({ role: "text", name: `${grouped(w.amount ?? 0)} ${w.name ?? ""}`.trim() });
  }
  kit.close();
}

/** A section's heading: its title in the display face, its refresh timer after it. */
function heading(s: Api.StoreSection, desc: Api.StoreFrontDesc, words: TimeWords, small: boolean): void {
  kit.open("Heading", { direction: "row", width: grow(), gap: u(12), cross: "center", part: "Store.heading", partClass: "badge" });
  text("Title", s.title ?? "", { step: st(small ? 18 : 22), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
  if (s.endsIn !== undefined && s.endsIn !== null) {
    kit.open("Refresh", { direction: "row", gap: u(6), cross: "center", part: "Store.refresh", partClass: "badge" });
    symbol("Clock", "clock", u(14), role("inkMuted"), { icon: "clock" });
    if (!small) text("Label", desc.refreshLabel ?? "Refreshes in", { step: st(14), color: "inkMuted", nowrap: true });
    text("Time", countdown(s.endsIn, words), { step: st(14), color: s.endsIn < 3600 ? "warn" : "ink", nowrap: true, font: "display" });
    kit.close();
    kit.semantics({ role: "timer", name: `${desc.refreshLabel ?? "Refreshes in"} ${countdown(s.endsIn, words)}` });
  }
  kit.close();
}

/**
 * A store front (Fortnite's item shop, Genshin's, Clash Royale's): see
 * `fn storeFront` in crates/og-api/api/ui.ogt.
 */
export function storeFront(id: string, desc: Api.StoreFrontDesc): Api.StoreFrontResult {
  const { t } = frame();
  const small = isPhone();
  const out: Mut<Api.StoreFrontResult> = { chosen: null, viewed: null, refused: null, topUp: null, hovered: null };
  const words: TimeWords = { days: desc.daysLabel, hours: desc.hoursLabel };
  const tick = kit.tick();
  // The width the tiles share, as laid out last tick (the store's own: a side panel beside it takes the rest).
  const widthKey = `~st.w.${id}`;
  const laid = kit.memory(widthKey, 0);

  kit.open(id, { direction: "column", width: grow(), gap: u(small ? 18 : 26), part: "Store.frame", partClass: "badge" });
  const rt = kit.memory("~st.rt", 0);
  const rk = kit.memory("~st.rk", 0);
  let refusedAt = "";
  if (desc.title || (desc.wallet ?? []).length > 0) {
    kit.open("Head", { direction: "row", width: grow(), gap: u(16), cross: "center", wrap: true, part: "Store.head", partClass: "badge" });
    if (desc.title) text("Title", desc.title, { step: st(small ? 22 : 30), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
    else {
      kit.open("Spacer", { width: grow() });
      kit.close();
    }
    wallet(desc, out);
    kit.close();
  }

  // Tiles a row: as many 220 px tiles as the room takes (a featured one spans two), two on a phone.
  const room = Math.max(u(200), (laid || safeWidth() - u(small ? 24 : 96)));
  const gap = u(small ? 10 : 16);
  const cols = small ? 2 : Math.max(2, desc.columns && desc.columns > 0 ? desc.columns : Math.floor((room + gap) / (u(220) + gap)));
  const tileW = Math.floor((room - gap * (cols - 1)) / cols);
  const tileH = Math.round(tileW * (small ? 1.3 : 1.36));

  kit.open("Sections", { direction: "column", width: grow(), gap: u(small ? 22 : 32), part: "Store.sections", partClass: "badge" });
  for (const s of desc.sections ?? []) {
    const sk = s.key || s.title || "section";
    kit.open(sk, { direction: "column", width: grow(), gap: u(12), part: "Store.section", partClass: "badge" });
    heading(s, desc, words, small);
    kit.open("Offers", { direction: "row", width: grow(), gap, wrap: true, part: "Store.offers", partClass: "badge" });
    for (const o of s.offers ?? []) {
      const key = o.key || o.name || "offer";
      const av = availability(o);
      const tier = rarityColor(o.rarity);
      const rim = tier ?? role("lineStrong");
      const sel = !!desc.selected && desc.selected === key;
      const ix = lastInteraction(key);
      const hot = (ix.hovered || ix.focused) && av !== "soldOut";
      const span = o.featured ? Math.min(cols, 2) : 1;
      const w = tileW * span + gap * (span - 1);
      const shaking = !still() && rt > 0 && tick - rt < 18 && rk === keyHash(`${sk}/${key}`);
      const shake = shaking ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.6 : 0;
      const left = (o.limit ?? 0) > 0 ? Math.max(0, (o.limit ?? 0) - (o.bought ?? 0)) : -1;
      const off = o.original && o.price !== undefined && o.original > (o.price ?? 0) ? Math.round((1 - (o.price ?? 0) / o.original) * 100) : 0;
      const wallet0 = (desc.wallet ?? []).find((x) => x.key === o.currency);
      const priceWords = o.priceText || ((o.price ?? 0) === 0 && desc.freeLabel ? desc.freeLabel : `${grouped(o.price ?? 0)} ${wallet0?.name ?? o.currency ?? ""}`.trim());
      const stateWords = av === "owned" ? desc.ownedLabel ?? "Owned" : av === "soldOut" ? desc.soldOutLabel ?? "Sold out" : av === "short" ? `${priceWords}, ${desc.shortLabel ?? "not enough"}` : priceWords;
      const r = surface(key, {
        part: "Store.offer",
        cls: "card",
        flags: [`rarity=${o.rarity ?? "none"}`, o.featured ? "featured" : "", av === "owned" ? "owned" : "", av === "soldOut" ? "soldOut" : "", av === "short" ? "short" : "", sel ? "selected" : ""],
        fill: null,
        direction: "column",
        width: px(w),
        // A featured tile on a phone spans the row: as tall as it is wide, so its contents and price stay inside it. A
        // tile whose words need more (a bundle's contents, large text) grows to them rather than pushing its price
        // strip out under the next section.
        height: { kind: "fit", min: o.featured && small ? Math.round(w * 0.92) : tileH },
        padding: xy(0, 0),
        radius: t.panel,
        lift: av === "soldOut" ? 0 : -u(6),
        pressScale: 0.97,
        selected: sel,
        selectedFill: null,
        semantics: {
          role: "button",
          name: [o.name ?? key, o.kind ?? "", o.rarity && o.rarity !== "none" ? o.rarity : "", o.badge ?? "", stateWords, left > 0 ? fill(desc.leftLabel ?? "{left} left", { left, limit: o.limit ?? 0 }) : ""].filter(Boolean).join(", "),
          description: (o.contents ?? []).join(", "),
          selected: sel,
        },
        props: {
          transform: { x: shake },
          states: [{ name: "availability", states: [vstate("available"), vstate("short"), vstate("owned"), vstate("soldOut")], transitions: [] }],
          state: [{ group: "availability", state: av }],
        },
      }, () => {
        const radii: [number, number, number, number] = [t.panel, t.panel, t.panel, t.panel];
        // The tile's own face: the tier's colour rising from its foot over the art (a store's rarity wash); no rim at
        // rest (a coloured box round every tile read as debug); selected or hovered a light rim with a soft glow (the
        // selection grammar).
        kit.setDraw({
          shape: { kind: "rect", radii },
          fills: [vgrad([[0, paint(mix(role("panel"), rim, 0.22))], [1, paint(mix(role("panel"), rim, 0.5))]])],
          strokes: sel || hot ? [line(sel ? 2.5 : 1.5, paint(mix(rim, hex("#ffffff"), 0.55)))] : [line(1, token("shadow", 0.55), "outside")],
          effects: [shadow(u(hot ? 10 : 5), u(hot ? 22 : 12), 0.5), ...(sel || hot ? [glow(u(14), paint(mix(rim, hex("#ffffff"), 0.3), 0.45), 0)] : [])],
        });
        // The art over the whole tile, under a shade at the foot (and the start, on a featured tile) so the words read.
        artSlot("Art", "Store.art", { texture: o.image ?? null, kind: o.featured ? "splash" : "item", rarity: o.rarity, focal: o.focal ?? { x: 0.5, y: 0.42 }, width: grow(), height: grow(), radius: radii }, () => {
          // The tier as light rising from the tile's foot, behind the words (no box round the tile).
          if (tier) {
            kit.open("Tier", { width: grow(), height: pct(0.42), part: "Store.tier", partClass: "badge" });
            kit.setDraw({ shape: { kind: "rect", radii: [0, 0, radii[2], radii[3]] }, fills: [vgrad([[0, paint(tier, 0)], [0.45, paint(tier, 0.22)], [1, paint(tier, 0.6)]])] });
            kit.close();
            over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
          }
          if (av === "soldOut" || av === "owned") {
            kit.open("Dim", { width: grow(), height: grow(), part: "Store.dim", partClass: "badge" });
            kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.2), av === "soldOut" ? 0.68 : 0.32) }] });
            kit.close();
          }
        });
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
        kit.open("Shade", { width: grow(), height: grow(), part: "Store.shade", partClass: "badge" });
        kit.setDraw({ shape: { kind: "rect", radii }, fills: [scrimFill("bottom", 0.92), ...(o.featured ? [scrimFill("start", 0.7)] : [])] });
        kit.close();
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });

        // The words over the art: tags at the top, the name and what it holds at the foot, the price strip under them.
        kit.open("Body", { direction: "column", width: grow(), height: grow(), padding: xy(u(small ? 10 : 14), u(small ? 10 : 12)), gap: u(4) });
        kit.open("Top", { direction: "row", width: grow(), gap: u(6), cross: "start" });
        if (o.badge) cornerTag("Tag", o.badge, (o.badgeKind ?? "note") === "new" ? rarityColor(o.rarity) ?? role("highlight") : (o.badgeKind ?? "note") === "upgrade" ? role("success") : "highlight", (o.badgeKind ?? "note") === "upgrade");
        kit.open("Gap", { width: grow() });
        kit.close();
        kit.open("Ends", { direction: "column", gap: u(4), cross: "end" });
        if (off > 0 && av !== "owned") {
          // The saving: white on the danger ink, a ribbon read at a glance.
          kit.open("Ribbon", { direction: "row", padding: xy(u(8), u(3)), part: "Store.ribbon", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("danger") }], effects: [shadow(2, 4, 0.45)] });
          text("Save", fill(desc.saveLabel ?? "−{pct} %", { pct: off }), { step: st(13), color: hex("#ffffff"), nowrap: true, font: "display" });
          kit.close();
        }
        if (o.endsIn !== undefined && o.endsIn !== null && av !== "soldOut") {
          kit.open("Timer", { direction: "row", gap: u(4), cross: "center", padding: xy(u(7), u(3)), part: `Store.timer${o.endsIn < 3600 ? "[urgent]" : ""}`, partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: o.endsIn < 3600 ? token("danger", 0.9) : token("shadow", 0.62) }] });
          symbol("Clock", "clock", u(12), o.endsIn < 3600 ? hex("#ffffff") : role("inkMuted"), { icon: "clock" });
          text("Time", countdown(o.endsIn, words), { step: st(12), color: o.endsIn < 3600 ? hex("#ffffff") : "ink", nowrap: true, font: "display" });
          kit.close();
        }
        if (left > 0) {
          kit.open("Left", { direction: "row", padding: xy(u(7), u(3)), part: "Store.left", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.62) }] });
          text("Text", fill(desc.leftLabel ?? "{left} left", { left, limit: o.limit ?? 0 }), { step: st(12), color: "warn", nowrap: true, font: "display" });
          kit.close();
        }
        kit.close();
        kit.close();
        kit.open("Fill", { width: grow(), height: grow() });
        kit.close();
        if (o.featured && (o.contents ?? []).length > 0) {
          // What the bundle holds, on the shaded start side.
          kit.open("Contents", { direction: "column", gap: u(3), width: px(Math.round(w * 0.5)), part: "Store.contents", partClass: "badge" });
          for (const [i, c] of (o.contents ?? []).slice(0, 5).entries()) {
            kit.open(kit.idi("Line", i), { direction: "row", gap: u(6), cross: "start", width: grow() });
            symbol("Dot", "check", u(12), rim === role("lineStrong") ? role("success") : mix(rim, hex("#ffffff"), 0.35));
            // Wrapped, never cut: a pixel face or large text takes a second line.
            text("Text", c, { step: st(13), color: "ink", maxLines: 3, width: grow() });
            kit.close();
          }
          kit.close();
        }
        text("Name", o.name ?? "", { step: st(o.featured ? (small ? 18 : 24) : small ? 15 : 17), font: "display", maxLines: 2, width: grow(), color: tier ? mix(tier, hex("#ffffff"), 0.72) : "ink" });
        if (o.kind) text("Kind", o.kind, { step: st(small ? 12 : 13), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
        kit.close();

        // The price strip: a dark glass foot with a line of the tier's colour along its top (Fortnite's rarity band).
        kit.open("Price", { direction: "row", width: grow(), height: px(u(small ? 34 : 40)), align: "center", cross: "center", gap: u(6), part: `Store.price${av === "short" ? "[short]" : ""}`, partClass: "badge" });
        kit.setDraw({ shape: { kind: "rect", radii: [0, 0, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("shadow", 0.72) }], strokes: [{ width: 2, align: "inside", paint: { kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 0.08 }, stops: [{ offset: 0, color: paint(rim, 0.95) }, { offset: 1, color: paint(rim, 0) }] } }] } as Api.DrawDesc);
        if (av === "owned") {
          symbol("Check", "check", u(14), role("success"));
          text("State", desc.ownedLabel ?? "Owned", { step: st(14), color: "success", nowrap: true, font: "display" });
        } else if (av === "soldOut") {
          text("State", desc.soldOutLabel ?? "Sold out", { step: st(14), color: "inkMuted", nowrap: true, font: "display" });
        } else {
          priceTag("Tag", {
            amount: o.price ?? 0,
            currency: wallet0?.name ?? o.currency ?? "",
            image: wallet0?.image ?? null,
            original: o.original ?? 0,
            affordable: av !== "short",
            free: desc.freeLabel ?? "Free",
            text: o.priceText ?? "",
            size: small ? "sm" : "md",
            shortLabel: desc.shortLabel ?? "not enough",
          });
        }
        kit.close();
      });
      if (ix.hovered || ix.focused) out.hovered = key;
      if (r.clicked) {
        if (av === "owned") {
          out.viewed = key;
          kit.feedback("select", -1);
        } else if (av === "available") {
          out.chosen = key;
          kit.feedback("select", -1);
        } else {
          out.refused = key;
          refusedAt = `${sk}/${key}`;
          kit.feedback("error", -1);
        }
      }
    }
    kit.close();
    kit.close();
    kit.semantics({ role: "list", name: s.title ?? sk });
  }
  kit.close();
  // A refusal's shake, kept under the store where it is read (memory is keyed under the open element).
  if (refusedAt) {
    kit.remember("~st.rt", tick);
    kit.remember("~st.rk", keyHash(refusedAt));
  }
  kit.close();
  const box = kit.bounds(kit.lastPath());
  if (box && box.width > 0) kit.remember(widthKey, Math.round(box.width));
  kit.semantics({ role: "region", name: desc.label ?? "Store" });
  return out;
}
