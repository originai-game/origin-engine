// The garage (docs/ui.md §11 genre patterns, Appendix B: garage, tuning,
// livery editor). Read against Forza Horizon's and Gran Turismo's (the car
// large on its lit turntable, a rail of categories, the category's parts
// with their tier pips and prices, a performance card whose bars light the
// selected part's change, the class badge and rating) and B-racing-garage.
// The livery editor is its layers list over og.ui.colorPicker: the layers,
// their order and colours are the game's; the paint on the car is the
// game's view. The car, the parts, the purse and the fitting are the game's.

import type * as Api from "@origingame/engine";
import {
  artSlot, btn, clamp, eyebrow, flags, frame, glow, grouped, grow, keyHash, kit, lastInteraction, line, mix, over, pct, priceTag, px, rarityColor,
  rememberRoom, role, safeHeight, safeWidth, shadow, spring, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut,
} from "./core.ts";
import { chevron } from "../containers/base.ts";
import { colorPicker } from "../controls/pickers.ts";

const toHex = (c: Api.PickedColor | undefined): string => {
  const h = (v: number | undefined) => Math.round(clamp(v ?? 0, 0, 1) * 255).toString(16).padStart(2, "0");
  return `#${h(c?.r)}${h(c?.g)}${h(c?.b)}`;
};

/** A part's upgrade pips: `n` of `max` lit in the accent, the rest wells. */
function pips(id: string, n: number, max: number, size: number): void {
  kit.row(id, { gap: Math.max(2, Math.round(size * 0.45)), cross: "center", part: "Garage.pips", partClass: "badge" }, () => {
    for (let i = 0; i < max; i++) {
      const on = i < n;
      kit.open(kit.idi("Pip", i), { width: px(size), height: px(size) });
      kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0.16) }], ...(on ? { effects: [glow(u(4), token("primary", 0.45), 0)] } : {}) });
      kit.close();
    }
  });
}

/** A figure's bar: the value now in the accent, the preview's change after it in success (or the fall in danger), the rest the track. */
function statRow(s: Api.GarageStat, compact: boolean, words: { label: string }): void {
  const max = (s.max ?? 0) > 0 ? s.max! : 10;
  const now = clamp((s.value ?? 0) / max, 0, 1);
  const has = (s.preview ?? -1) >= 0;
  const pre = has ? clamp(s.preview! / max, 0, 1) : now;
  const d = has ? (s.preview! - (s.value ?? 0)) : 0;
  const fmt = (v: number) => (Number.isInteger(v) ? String(v) : v.toFixed(1));
  kit.column(s.key ?? "", { width: grow(), gap: u(compact ? 3 : 6), part: "Garage.stat", partClass: "badge" }, () => {
    kit.row("Head", { width: grow(), gap: u(8), cross: "center" }, () => {
      text("Label", (s.label ?? "").toLocaleUpperCase(), { step: st(compact ? 12 : 14), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      text("Value", fmt(s.value ?? 0), { step: st(compact ? 14 : 17), font: "mono", nowrap: true });
      if (d !== 0) text("Delta", `${d > 0 ? "+" : "\u2212"}${fmt(Math.abs(d))}`, { step: st(compact ? 13 : 15), font: "mono", nowrap: true, color: d > 0 ? "success" : "danger" });
    });
    const h = Math.max(4, u(compact ? 5 : 7));
    kit.open("Bar", { direction: "row", width: grow(), height: px(h), part: "Garage.bar", partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect", radii: [h / 2, h / 2, h / 2, h / 2] }, fills: [{ kind: "solid", color: token("ink", 0.12) }], clip: "shape" } as Api.DrawDesc);
    // The segments flow from the start: the value now (up to the lower of now and the preview), then the gain or the fall.
    const seg = (id: string, f: number, c: Api.Paint, lit = false): void => {
      if (f <= 0) return;
      kit.open(id, { width: pct(f), height: px(h) });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: c }], ...(lit ? { effects: [glow(u(5), c, 0)] } : {}) });
      kit.close();
    };
    seg("Now", Math.min(now, pre), token("primary"));
    if (pre > now) seg("Gain", pre - now, token("success"), true);
    if (pre < now) seg("Loss", now - pre, token("danger", 0.85));
    kit.close();
  });
  kit.semantics({ role: "meter", name: [s.label ?? "", fmt(s.value ?? 0), d !== 0 ? `${words.label} ${d > 0 ? "+" : "-"}${fmt(Math.abs(d))}` : ""].filter(Boolean).join(" "), value: fmt(s.value ?? 0) });
}

/**
 * A garage (Forza Horizon, Gran Turismo): see `fn garage` in
 * crates/og-api/api/ui.ogt.
 */
export function garage(id: string, desc: Api.GarageDesc): Api.GarageResult {
  const out: Mut<Api.GarageResult> = { category: null, selected: null, equip: null, buy: null, refused: null, layer: null, layerColor: null, layerMove: null, layerVisible: null, addLayer: false, deleteLayer: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const W = kit.memory(`~ga.w.${id}`, 0) || safeWidth();
  const H = kit.memory(`~ga.h.${id}`, 0) || safeHeight();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const rt = kit.memory("~ga.rt", 0);
  const rk = kit.memory("~ga.rk", 0);
  const cats = desc.categories ?? [];
  const cat = cats.find((c) => c.key === desc.category) ?? cats[0] ?? null;
  const livery = !!cat?.livery;
  const parts = desc.parts ?? [];
  const part = parts.find((p) => p.key === desc.selected) ?? null;
  const layers = desc.layers ?? [];
  const layer = layers.find((l) => l.key === desc.layer) ?? null;
  const balance = desc.balance ?? 0;
  const carHash = keyHash(`${desc.selected ?? ""}|${desc.category ?? ""}`);
  const carChanged = kit.memory("~ga.car", 0) !== carHash;
  kit.remember("~ga.car", carHash);
  const refuse = (k: string): void => {
    out.refused = k;
    kit.feedback("error", -1);
  };

  // The pad's shoulders (the tab actions) step the categories, wrapping.
  const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
  if (step !== 0 && cats.length > 1) {
    const i = Math.max(0, cats.findIndex((c) => c.key === cat?.key));
    out.category = cats[(i + step + cats.length) % cats.length]!.key ?? null;
    kit.feedback("tab", -1);
  }

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 10 : 18), part: "Garage.frame", partClass: "panel" });

  // The head: the title, and the purse at the end.
  kit.row("Head", { width: grow(), gap: u(16), cross: "center" }, () => {
    if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
    else kit.spacer("Gap", grow());
    kit.row("Purse", { gap: u(8), cross: "center", padding: xy(u(12), u(6)), part: "Garage.purse", partClass: "chip" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [u(18), u(18), u(18), u(18)] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
      priceTag("Balance", { amount: balance, currency: "coins", image: desc.currencyImage ?? null, size: "lg" });
    });
  });

  // The rail: a category per entry, its art and its name (a row of chips over the car on a phone in portrait).
  const rail = (): void => {
    const entry = (c: Api.GarageCategory): void => {
      const k = c.key ?? "";
      const on = !!cat && cat.key === k;
      const r = surface(k, {
        part: `Garage.category${flags(on && "selected")}`,
        cls: portrait ? "chip" : "listItem",
        fill: on ? { ...role("primary"), a: 0.16 } : portrait ? { ...role("shadow"), a: 0.55 } : null,
        washBy: 0.12,
        gap: u(portrait ? 8 : 12),
        cross: "center",
        width: portrait ? null : grow(),
        padding: xy(u(portrait ? 10 : 12), u(portrait ? 6 : 10)),
        radius: portrait ? 999 : Math.max(4, t.small),
        semantics: { role: "tab", name: [c.name ?? k, c.count ?? ""].filter(Boolean).join(", "), selected: on },
      }, () => {
        if (on && !portrait) {
          kit.open("Edge", { width: px(u(3)), height: grow() });
          kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("primary") }], effects: [glow(u(6), token("primary", 0.5), 0)] });
          kit.close();
          over({ insets: { start: 0, top: u(6), bottom: u(6) }, inert: true });
        }
        const z = u(portrait ? 28 : 44);
        artSlot("Art", "Garage.categoryArt", { texture: c.image ?? null, kind: "item", width: px(z), height: px(z), radius: Math.max(3, t.small - 2) });
        text("Name", (c.name ?? k).toLocaleUpperCase(), { step: st(portrait ? 13 : 15), font: "display", nowrap: true, ellipsis: !portrait, ...(portrait ? {} : { width: grow() }), color: on ? "ink" : "inkMuted" });
      });
      if (r.clicked && !on) {
        out.category = k;
        kit.feedback("tab", -1);
      }
    };
    if (portrait) {
      kit.openScroll("Rail", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal", part: "Garage.rail", partClass: "panel" } as never);
      kit.open("Chips", { direction: "row", gap: u(6) });
      cats.forEach(entry);
      kit.close();
      kit.close();
    } else {
      kit.openScroll("Rail", { width: px(Math.round(Math.min(u(230), W * 0.15))), height: grow(), gap: u(4), padding: xy(u(6), u(6)), part: "Garage.rail" } as never);
      cats.forEach(entry);
      kit.close();
    }
    kit.semantics({ role: "tablist", name: desc.title || "Garage" });
  };

  // A part's row: its art on a tier-rimmed tile, its name, its pips, its price (a lock and why, or the fitted tick).
  const partRow = (p: Api.GaragePart, rowW: Api.Size): void => {
    const k = p.key ?? "";
    const on = !!part && part.key === k;
    const ix = lastInteraction(k);
    const short = !p.owned && !p.locked && (p.price ?? 0) > balance;
    const state = p.locked ? "locked" : p.equipped ? "equipped" : on ? "selected" : "open";
    const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
    const tierMax = (p.tierMax ?? 0) > 0 ? p.tierMax! : 3;
    const z = u(phone ? 56 : 72);
    const r = surface(k, {
      part: `Garage.part${flags(on && "selected", p.equipped && "equipped", p.locked && "locked")}`,
      cls: "listItem",
      fill: null,
      gap: u(12),
      cross: "center",
      width: rowW,
      padding: xy(u(8), u(8)),
      radius: Math.max(6, t.panel),
      semantics: { role: "button", name: [p.name ?? k, `${p.tier ?? 0}/${tierMax}`, p.locked ? p.lockedHint || desc.lockedLabel || "Locked" : p.equipped ? desc.equippedLabel ?? "Equipped" : p.owned ? "" : (p.price ?? 0) > 0 ? `${grouped(p.price!)}${short ? `, ${desc.shortLabel ?? "not enough"}` : ""}` : desc.freeLabel ?? "Free"].filter(Boolean).join(", "), selected: on },
      props: {
        states: [{ name: "part", states: [vstate("open"), vstate("selected"), vstate("equipped"), vstate("locked")], transitions: [] }],
        state: [{ group: "part", state }],
        ...(shaking ? { transform: { x: shaking } } : {}),
      },
    }, () => {
      const rr = Math.max(6, t.panel);
      kit.setDraw({
        shape: { kind: "rect", radii: [rr, rr, rr, rr] },
        fills: [on ? vgrad([[0, token("panelRaised", 0.96)], [1, token("panel", 0.96)]]) : { kind: "solid", color: token("panel", 0.78) }],
        strokes: on ? [line(u(2), token("focus"))] : ix.hovered || ix.focused ? [line(u(1.5), token("ink", 0.35))] : [line(1, token("ink", 0.08))],
        effects: on ? [glow(u(8), token("focus", 0.35), 0), shadow(u(4), u(12), 0.45)] : [],
      });
      artSlot("Art", "Garage.partArt", { texture: p.image ?? null, kind: "item", rarity: p.rarity, width: px(z), height: px(z), radius: Math.max(4, t.small) }, () => {
        if (p.locked) {
          kit.open("Dim", { width: grow(), height: grow() });
          kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
          kit.close();
        }
      });
      kit.column("Words", { width: grow(), gap: u(4) }, () => {
        text("Name", p.name ?? k, { step: st(phone ? 15 : 17), font: "display", nowrap: true, ellipsis: true, width: grow(), color: p.locked ? "inkMuted" : "ink" });
        pips("Pips", p.tier ?? 0, tierMax, Math.max(5, u(phone ? 8 : 10)));
        kit.row("Price", { gap: u(6), cross: "center", width: grow() }, () => {
          if (p.locked) {
            symbol("Lock", "lock", u(14), role("inkMuted"), { icon: "lock" });
            text("Hint", p.lockedHint || (desc.lockedLabel ?? "Locked"), { step: st(13), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
          } else if (p.equipped) {
            symbol("Tick", "check", u(14), role("success"));
            text("Word", (desc.equippedLabel ?? "Equipped").toLocaleUpperCase(), { step: st(13), font: "display", color: "success", nowrap: true });
          } else if (!p.owned) priceTag("Tag", { amount: p.price ?? 0, currency: "coins", image: desc.currencyImage ?? null, affordable: !short, free: desc.freeLabel ?? "Free", size: "sm", shortLabel: desc.shortLabel ?? "not enough" });
        });
      });
    });
    if (r.clicked) {
      if (!on) {
        out.selected = k;
        kit.feedback("select", -1);
      }
    } else if (ix.focused && !on && kit.memory(`~ga.f.${id}`, 0) !== keyHash(k)) out.selected = k;
    if (ix.focused) kit.remember(`~ga.f.${id}`, keyHash(k));
  };

  // A livery layer's row: its swatch, name and kind, its eye, and up and down.
  const layerRow = (l: Api.LiveryLayer, i: number, rowW: Api.Size): void => {
    const k = l.key ?? "";
    const on = !!layer && layer.key === k;
    const hidden = l.visible === false;
    const small = (key: string, label: string, build: () => void, disabled = false): boolean =>
      surface(key, { part: "Garage.layerButton", cls: "button", fill: null, washBy: 0.14, disabled, width: px(Math.max(44, u(32))), height: px(Math.max(44, u(32))), align: "center", cross: "center", radius: 999, semantics: { role: "button", name: `${label} ${l.name ?? k}` } }, build).clicked;
    const r = surface(k, {
      part: `Garage.layer${flags(on && "selected", hidden && "hidden")}`,
      cls: "listItem",
      fill: on ? { ...role("primary"), a: 0.14 } : null,
      gap: u(10),
      cross: "center",
      width: rowW,
      padding: xy(u(10), u(4)),
      radius: Math.max(6, t.panel),
      semantics: { role: "option", name: [l.name ?? k, l.kind ?? "", hidden ? "hidden" : ""].filter(Boolean).join(", "), selected: on },
    }, () => {
      const z = u(phone ? 28 : 32);
      kit.open("Swatch", { width: px(z), height: px(z), part: "Garage.swatch", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect", radii: [u(6), u(6), u(6), u(6)] }, fills: [{ kind: "solid", color: { r: l.color?.r ?? 0, g: l.color?.g ?? 0, b: l.color?.b ?? 0, a: hidden ? 0.35 : 1 } as never }], strokes: [line(1, token("ink", 0.4))] });
      kit.close();
      kit.column("Words", { width: grow(), gap: 0 }, () => {
        text("Name", l.name ?? k, { step: st(15), font: "display", nowrap: true, ellipsis: true, width: grow(), color: hidden ? "inkMuted" : "ink" });
        text("Kind", [l.kind ?? "", toHex(l.color).toUpperCase()].filter(Boolean).join(" · "), { step: st(12), font: "mono", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      });
      if (l.locked) symbol("Lock", "lock", u(14), role("inkMuted"), { icon: "lock" });
      if (small("Eye", desc.visibleLabel ?? "Visible", () => symbol("Mark", "eye", u(16), hidden ? role("inkFaint") : role("ink")))) {
        out.layerVisible = k;
        kit.feedback("toggle", -1);
      }
      if (small("Up", desc.upLabel ?? "Move up", () => chevron("Mark", "up", u(14), role("ink")), i === 0 || !!l.locked)) out.layerMove = { key: k, by: -1 };
      if (small("Down", desc.downLabel ?? "Move down", () => chevron("Mark", "down", u(14), role("ink")), i === layers.length - 1 || !!l.locked)) out.layerMove = { key: k, by: 1 };
    });
    if (r.clicked && !on) {
      out.layer = k;
      kit.feedback("select", -1);
    }
  };

  // The list: the category's parts or the livery's layers, with its heading and count; whole rows scroll.
  const list = (w: Api.Size, h: Api.Size): void => {
    kit.column(livery ? "Layers" : "Parts", { width: w, height: h, gap: u(8), part: "Garage.parts", partClass: "panel" }, () => {
      kit.row("Head", { width: grow(), gap: u(8), cross: "center", padding: xy(u(4), 0) }, () => {
        text("Title", (livery ? desc.layersLabel ?? "Layers" : cat?.name ?? "").toLocaleUpperCase(), { step: st(16), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
        if (!livery && cat?.count) text("Count", cat.count, { step: st(14), font: "mono", color: "inkMuted", nowrap: true });
      });
      kit.openScroll("List", { width: grow(), height: grow(), gap: u(6), padding: xy(0, 0), fade: 32, fadeInto: "shadow" } as never);
      if (livery) layers.forEach((l, i) => layerRow(l, i, grow()));
      else parts.forEach((p) => partRow(p, grow()));
      kit.close();
      kit.semantics({ role: livery ? "listbox" : "list", name: livery ? desc.layersLabel ?? "Layers" : cat?.name ?? "" });
    });
  };

  // The stage: the car on its turntable, a ring of light under it.
  const stage = (w: Api.Size, h: Api.Size): void => {
    kit.column("Stage", { width: w, height: h, align: "end", cross: "center", part: "Garage.stage", partClass: "badge" }, () => {
      // The ring under the car's wheels, laid over the stage's foot.
      kit.open("Ring", { width: grow(), height: px(u(phone ? 40 : 84)) });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 1, y: 1 }, stops: [{ offset: 0, color: token("primary", 0.16) }, { offset: 1, color: token("primary", 0) }] }], strokes: [line(u(2), token("primary", 0.55))], effects: [glow(u(12), token("primary", 0.4), 0)] } as Api.DrawDesc);
      kit.close();
      over({ insets: { start: u(phone ? 8 : 24), end: u(phone ? 8 : 24), bottom: u(phone ? 4 : 12) }, inert: true });
      if (desc.vehicle) {
        artSlot("Car", "Garage.car", { texture: desc.vehicle, kind: "vehicle", fit: "contain", width: grow(), height: pct(0.82), label: "" });
        if (carChanged && !still()) kit.props({ enter: { values: [{ prop: "opacity", value: 0.4 }, { prop: "scale", value: 0.97 }], transition: spring("panel") } });
      }
    });
  };

  // The performance card: the class badge and rating, the figures and the legend.
  const performance = (compact: boolean): void => {
    kit.column("Performance", { width: grow(), gap: u(compact ? 6 : 12), padding: xy(u(16), u(compact ? 10 : 16)), part: "Garage.performance", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }, vgrad([[0, token("sheen", 0.06)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(6), u(20), 0.45)] });
      kit.row("Head", { width: grow(), gap: u(12), cross: "center" }, () => {
        eyebrow("Title", desc.performanceLabel ?? "Performance");
        kit.spacer("Gap", grow());
        if (desc.ratingClass) {
          const b = u(compact ? 32 : 44);
          kit.open("Badge", { width: px(b), height: px(b), align: "center", cross: "center", part: "Garage.badge", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [u(6), u(6), u(6), u(6)] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }], strokes: [line(u(2), token("primary"))], effects: [glow(u(8), token("primary", 0.45), 0)] });
          text("Class", desc.ratingClass, { step: st(compact ? 16 : 22), font: "display", nowrap: true });
          kit.close();
          kit.column("Rating", { gap: 0, cross: "end" }, () => {
            const d = (desc.ratingPreview ?? 0) > 0 ? desc.ratingPreview! - (desc.rating ?? 0) : 0;
            kit.row("Value", { gap: u(6), cross: "end" }, () => {
              text("Now", String(desc.rating ?? 0), { step: st(compact ? 22 : 34), font: "mono", nowrap: true });
              if (d !== 0) text("Delta", `${d > 0 ? "+" : "\u2212"}${Math.abs(d)}`, { step: st(compact ? 13 : 16), font: "mono", nowrap: true, color: d > 0 ? "success" : "danger" });
            });
            if (!compact) text("Caption", (desc.classLabel ?? "Class").toLocaleUpperCase(), { step: "caption", font: "display", color: "inkMuted", nowrap: true });
          });
          kit.semantics({ role: "status", name: `${desc.classLabel ?? "Class"} ${desc.ratingClass} ${desc.rating ?? 0}${(desc.ratingPreview ?? 0) > 0 ? ` → ${desc.ratingPreview}` : ""}` });
        }
      });
      kit.column("Stats", { width: grow(), gap: u(compact ? 6 : 12) }, () => {
        for (const s of desc.stats ?? []) statRow(s, compact, { label: desc.previewLabel ?? "Preview" });
      });
      if (!compact) kit.row("Legend", { gap: u(16), cross: "center" }, () => {
        const key = (k: string, c: Api.Paint, w: string): void => void kit.row(k, { gap: u(6), cross: "center" }, () => {
          kit.open("Chip", { width: px(u(14)), height: px(u(6)), radius: 2, background: c });
          kit.close();
          text("Word", w.toLocaleUpperCase(), { step: "caption", font: "display", color: "inkMuted", nowrap: true });
        });
        key("Current", token("primary"), desc.currentLabel ?? "Current");
        key("Preview", token("success"), desc.previewLabel ?? "Preview");
      });
    });
  };

  // The action on the selected part: Equip, Buy, or Equipped inert; a locked or unaffordable one is refused (it shakes).
  const action = (w: Api.Size): void => {
    if (!part) return;
    const k = part.key ?? "";
    const short = !part.owned && !part.locked && (part.price ?? 0) > balance;
    const label = part.equipped ? desc.equippedLabel ?? "Equipped" : part.locked ? desc.lockedLabel ?? "Locked" : part.owned ? desc.equipLabel ?? "Equip" : desc.buyLabel ?? "Buy";
    const r = btn("Action", label, { variant: part.equipped ? "secondary" : "primary", size: "lg", width: w, disabled: !!part.equipped, part: `Garage.action${flags(part.owned ? "equip" : "buy")}`, raised: !part.equipped, ...(part.locked ? { sym: "lock" as const } : part.equipped ? { sym: "check" as const } : {}) });
    if (r.clicked) {
      if (part.locked || short) refuse(k);
      else if (part.owned) {
        out.equip = k;
        kit.feedback("confirm", -1);
      } else {
        out.buy = k;
        kit.feedback("confirm", -1);
      }
    }
  };

  // The selected part's detail: its name in the display face, its tier word, its paragraph, its price and the action.
  const detail = (compact: boolean): void => {
    if (!part) return;
    const tier = rarityColor(part.rarity);
    kit.column("Detail", { width: grow(), gap: u(compact ? 6 : 10), padding: xy(u(16), u(compact ? 10 : 16)), part: "Garage.detail", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.92) }], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(6), u(20), 0.45)] });
      text("Name", (part.name ?? "").toLocaleUpperCase(), { step: st(compact ? 18 : 24), font: "display", heading: true, maxLines: 2, width: grow() });
      if (part.rarityLabel) text("Tier", part.rarityLabel.toLocaleUpperCase(), { step: st(13), font: "display", nowrap: true, color: tier ? mix(tier, role("ink"), 0.25) : "inkMuted" });
      if (!compact && part.description) text("Description", part.description, { step: st(15), color: "inkMuted", width: grow(), maxLines: 4 });
      if (!part.owned && !part.locked) priceTag("Price", { amount: part.price ?? 0, currency: "coins", image: desc.currencyImage ?? null, affordable: (part.price ?? 0) <= balance, free: desc.freeLabel ?? "Free", size: "md", shortLabel: desc.shortLabel ?? "not enough" });
    });
    kit.semantics({ role: "region", name: part.name ?? "" });
  };

  // The livery's side: the selected layer's colour, Add layer and Delete.
  const liverySide = (): void => {
    kit.column("Livery", { width: grow(), gap: u(10), padding: xy(u(16), u(14)), part: "Garage.detail", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.92) }], strokes: [line(1, token("ink", 0.1))] });
      if (layer) {
        eyebrow("Title", `${desc.colorLabel ?? "Colour"} · ${layer.name ?? ""}`);
        const c = colorPicker("Picker", layer.color ?? { r: 1, g: 1, b: 1, a: 1 }, { swatches: desc.swatches ?? [], hex: !phone, label: desc.colorLabel ?? "Colour", width: grow(), disabled: !!layer.locked });
        if (c.changed && !layer.locked) out.layerColor = c.color;
      }
      kit.row("Actions", { width: grow(), gap: u(8) }, () => {
        if (btn("Add", desc.addLayerLabel ?? "Add layer", { variant: "secondary", width: grow(), sym: "plus" }).clicked) {
          out.addLayer = true;
          kit.feedback("confirm", -1);
        }
        if (btn("Delete", desc.deleteLabel ?? "Delete", { variant: "danger", width: grow(), disabled: !layer || !!layer.locked }).clicked && layer) {
          out.deleteLayer = layer.key ?? null;
          kit.feedback("confirm", -1);
        }
      });
    });
  };

  if (portrait) {
    rail();
    stage(grow(), px(Math.round(Math.min(W * 0.62, H * 0.26))));
    kit.openScroll("Body", { width: grow(), height: grow(), gap: u(10), padding: xy(0, 0), fade: 40, fadeInto: "shadow" } as never);
    if (livery) {
      layers.forEach((l, i) => layerRow(l, i, grow()));
      liverySide();
    } else {
      kit.column("Parts", { width: grow(), gap: u(6) }, () => parts.forEach((p) => partRow(p, grow())));
      performance(true);
      detail(true);
    }
    kit.close();
    if (!livery) action(grow());
  } else {
    kit.row("Body", { width: grow(), height: grow(), gap: u(16) }, () => {
      rail();
      const listW = Math.round(Math.min(u(340), W * 0.22) * Math.min(1.25, ts));
      list(px(listW), grow());
      stage(grow(), grow());
      // The side scrolls its cards; the part's action stands at its foot, always in reach.
      kit.column("Side", { width: px(Math.round(Math.min(u(380), W * 0.25) * Math.min(1.2, ts))), height: grow(), gap: u(12) }, () => {
        kit.openScroll("Cards", { width: grow(), height: grow(), gap: u(12), padding: xy(0, 0), fade: 32, fadeInto: "shadow" } as never);
        if (livery) liverySide();
        else {
          performance(false);
          detail(false);
        }
        kit.close();
        if (!livery) action(grow());
      });
    });
  }
  if (out.refused) {
    kit.remember("~ga.rt", tick);
    kit.remember("~ga.rk", keyHash(out.refused));
  }
  kit.close();
  kit.props({ states: [{ name: "garage", states: [vstate("parts"), vstate("livery")], transitions: [] }], state: [{ group: "garage", state: livery ? "livery" : "parts" }] });
  rememberRoom("ga", id);
  return out;
}

