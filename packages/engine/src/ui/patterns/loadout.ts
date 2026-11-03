// A shooter's loadout (docs/ui.md §11 genre patterns, Appendix B: loadout). Read against Call of Duty's, Apex Legends'
// and The Division's (a rail of slots with their items' art, the selected weapon large with its attachment points
// called out on leader lines, its figures with what the attachments change, the attachments' list with their tiers and
// a remove button, the loadouts as tabs) and B-shooter-loadout. The items, the attachments' choices and the purse are
// the game's: this draws them and reports what was asked.

import type * as Api from "@origingame/engine";
import {
  artSlot, clamp, eyebrow, flags, frame, glow, grow, keyHash, kit, lastInteraction, line, mix, over, pct, priceTag, px, rarityColor,
  rememberRoom, role, safeWidth, shadow, spring, st, still, surface, symbol, text, token, u, vgrad, xy, type Mut,
} from "./core.ts";
import { chevron, rtl } from "../containers/base.ts";

/** A figure's row: its name, a bar (the attachments' gain lit at its end, or the loss shown past it), the value and the change. */
function statRow(s: Api.LoadoutStat, compact: boolean): void {
  const max = (s.max ?? 0) > 0 ? s.max! : 100;
  const v = s.value ?? 0;
  const d = s.delta ?? 0;
  const now = clamp(v / max, 0, 1);
  const base = clamp((v - d) / max, 0, 1);
  const fmt = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(1));
  kit.row(s.key ?? "", { width: grow(), gap: u(compact ? 8 : 12), cross: "center", part: "Loadout.stat", partClass: "badge" }, () => {
    text("Label", (s.label ?? "").toLocaleUpperCase(), { step: st(compact ? 12 : 13), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: px(u(compact ? 92 : 110) * Math.min(1.4, Math.max(1, frame().theme.textScale ?? 1))) });
    const h = Math.max(4, u(compact ? 6 : 8));
    kit.open("Bar", { direction: "row", width: grow(), height: px(h) });
    kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink", 0.12) }], clip: "shape" } as Api.DrawDesc);
    const seg = (id: string, f: number, c: Api.Paint, lit = false): void => {
      if (f <= 0) return;
      kit.open(id, { width: pct(f), height: px(h) });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: c }], ...(lit ? { effects: [glow(u(5), c, 0)] } : {}) });
      kit.close();
    };
    seg("Base", Math.min(now, base), token("primary"));
    if (now > base) seg("Gain", now - base, token("success"), true);
    if (now < base) seg("Loss", base - now, token("danger", 0.55));
    kit.close();
    text("Value", fmt(v), { step: st(compact ? 14 : 16), font: "mono", nowrap: true });
    kit.row("Delta", { gap: u(2), cross: "center", width: px(u(compact ? 44 : 56)) }, () => {
      if (d === 0) return;
      text("Text", `${d > 0 ? "+" : "\u2212"}${fmt(Math.abs(d))}`, { step: st(compact ? 13 : 14), font: "mono", nowrap: true, color: d > 0 ? "success" : "danger" });
      chevron("Arrow", d > 0 ? "up" : "down", u(12), d > 0 ? role("success") : role("danger"));
    });
  });
  kit.semantics({ role: "meter", name: [s.label ?? "", fmt(v), d !== 0 ? `${d > 0 ? "+" : "-"}${fmt(Math.abs(d))}` : ""].filter(Boolean).join(" "), value: fmt(v) });
}

/**
 * A shooter's loadout (Call of Duty, Apex Legends, The Division): see `fn loadout` in crates/og-api/api/ui.ogt.
 */
export function loadout(id: string, desc: Api.LoadoutDesc): Api.LoadoutResult {
  const out: Mut<Api.LoadoutResult> = { loadout: null, slot: null, attachment: null, remove: null, refused: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const W = kit.memory(`~lo.w.${id}`, 0) || safeWidth();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const rt = kit.memory("~lo.rt", 0);
  const rk = kit.memory("~lo.rk", 0);
  const slots = desc.slots ?? [];
  const slot = slots.find((s) => s.key === desc.slot) ?? null;
  const atts = desc.attachments ?? [];
  const names = desc.loadouts ?? [];
  const current = clamp(desc.loadout ?? 0, 0, Math.max(0, names.length - 1));
  const itemHash = keyHash(`${desc.slot ?? ""}|${current}`);
  const itemChanged = kit.memory("~lo.item", 0) !== itemHash;
  kit.remember("~lo.item", itemHash);

  // The pad's shoulders (the tab actions) step the loadouts, wrapping.
  const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
  if (step !== 0 && names.length > 1) {
    out.loadout = (current + step + names.length) % names.length;
    kit.feedback("tab", -1);
  }

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 10 : 18), part: "Loadout.frame", partClass: "panel" });

  // A loadout's tab: its name, a lit underline under the one shown (a chip on a phone).
  const tab = (name: string, i: number): void => {
    const on = i === current;
    const r = surface(String(i), {
      part: `Loadout.tab${flags(on && "selected")}`,
      cls: portrait ? "chip" : "listItem",
      fill: portrait ? (on ? { ...role("primary"), a: 0.22 } : { ...role("shadow"), a: 0.55 }) : null,
      washBy: 0.12,
      direction: "column",
      gap: u(6),
      align: "center",
      cross: "center",
      ...(portrait ? { height: { kind: "fit", min: 44 } as Api.Size } : {}),
      padding: xy(u(portrait ? 12 : 16), u(portrait ? 8 : 6)),
      radius: portrait ? 999 : Math.max(4, t.small),
      semantics: { role: "tab", name, selected: on },
    }, () => {
      text("Label", name.toLocaleUpperCase(), { step: st(portrait ? 13 : 15), font: "display", nowrap: true, color: on ? "ink" : "inkMuted" });
      if (!portrait) {
        kit.open("Line", { width: grow(), height: px(Math.max(2, u(3))) });
        kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0) }], ...(on ? { effects: [glow(u(6), token("primary", 0.5), 0)] } : {}) });
        kit.close();
      }
    });
    if (r.clicked && !on) {
      out.loadout = i;
      kit.feedback("tab", -1);
    }
  };
  const tabs = (): void => {
    if (names.length < 2) return;
    kit.openScroll("Tabs", { width: portrait ? grow() : null, gap: 0, padding: xy(0, 0), axes: "horizontal" } as never);
    kit.open("Row", { direction: "row", gap: u(portrait ? 6 : 4), cross: "center" });
    names.forEach(tab);
    kit.close();
    kit.close();
    kit.semantics({ role: "tablist", name: desc.title || "Loadouts" });
  };
  const purse = (): void => {
    if (desc.balance == null) return;
    kit.row("Purse", { gap: u(8), cross: "center", padding: xy(u(12), u(6)), part: "Loadout.purse", partClass: "chip" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [u(18), u(18), u(18), u(18)] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
      priceTag("Balance", { amount: desc.balance ?? 0, currency: "coins", image: desc.currencyImage ?? null, size: "lg" });
    });
  };

  // The head: the title, the tabs, the purse at the end (on a phone the tabs under them).
  kit.row("Head", { width: grow(), gap: u(phone ? 10 : 32), cross: "center" }, () => {
    if (desc.title) text("Title", desc.title.toLocaleUpperCase(), { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: phone });
    if (!portrait) tabs();
    kit.spacer("Gap", grow());
    purse();
  });
  if (portrait) tabs();

  // A slot: its eyebrow, its item's name and detail, its art at the end; the selected one rimmed in the focus colour.
  const slotEntry = (s: Api.LoadoutSlot, tile: boolean): void => {
    const k = s.key ?? "";
    const on = !!slot && slot.key === k;
    const ix = lastInteraction(k);
    const empty = !s.name;
    const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
    const rr = Math.max(6, t.panel);
    const r = surface(k, {
      part: `Loadout.slot${flags(on && "selected", s.locked && "locked")}`,
      cls: "listItem",
      fill: null,
      direction: tile ? "column" : "row",
      gap: u(tile ? 6 : 12),
      cross: tile ? "start" : "center",
      width: tile ? px(Math.round(u(150) * Math.min(1.3, ts))) : grow(),
      padding: xy(u(tile ? 10 : 16), u(tile ? 10 : 12)),
      radius: rr,
      semantics: { role: "tab", name: [s.label ?? "", empty ? desc.emptyLabel ?? "Empty" : s.name ?? "", s.detail ?? "", s.locked ? desc.lockedLabel ?? "Locked" : ""].filter(Boolean).join(", "), selected: on },
      props: shaking ? { transform: { x: shaking } } : {},
    }, () => {
      kit.setDraw({
        shape: { kind: "rect", radii: [rr, rr, rr, rr] },
        fills: [on ? vgrad([[0, token("primary", 0.2)], [1, token("panel", 0.92)]], 90) : { kind: "solid", color: token("panel", 0.82) }],
        strokes: on ? [line(u(2), token("focus"))] : ix.hovered || ix.focused ? [line(u(1.5), token("ink", 0.35))] : [line(1, token("ink", 0.1))],
        effects: on ? [glow(u(10), token("focus", 0.4), 0)] : [],
      });
      const art = (): void => artSlot("Art", "Loadout.slotArt", { texture: s.image ?? null, kind: "item", rarity: s.rarity, fit: "contain", width: tile ? grow() : px(u(phone ? 72 : 112)), height: px(u(tile ? 56 : 60)), radius: Math.max(4, t.small), label: "" }, () => {
        if (s.locked) {
          kit.open("Dim", { width: grow(), height: grow(), align: "center", cross: "center" });
          kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
          symbol("Lock", "lock", u(18), role("inkMuted"), { icon: "lock" });
          kit.close();
        }
      });
      if (tile) art();
      kit.column("Words", { width: grow(), gap: u(2) }, () => {
        text("Label", (s.label ?? "").toLocaleUpperCase(), { step: st(12), font: "display", nowrap: true, ellipsis: true, width: grow(), color: on ? "primary" : "inkMuted" });
        text("Name", empty ? desc.emptyLabel ?? "Empty" : (s.name ?? "").toLocaleUpperCase(), { step: st(tile ? 14 : 17), font: "display", nowrap: true, ellipsis: true, width: grow(), color: empty || s.locked ? "inkMuted" : "ink" });
        if (s.detail && !tile) text("Detail", s.detail.toLocaleUpperCase(), { step: st(11), font: "mono", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      });
      if (!tile) art();
    });
    if (r.clicked) {
      if (s.locked) out.refused = k;
      else if (!on) {
        out.slot = k;
        kit.feedback("select", -1);
      }
      if (s.locked) kit.feedback("error", -1);
    } else if (ix.focused && !on && !s.locked && kit.memory(`~lo.f.${id}`, 0) !== keyHash(k)) out.slot = k;
    if (ix.focused) kit.remember(`~lo.f.${id}`, keyHash(k));
  };

  // The item's words: its name in the display face, the subtitle divided by rules, its line.
  const info = (compact: boolean): void => {
    kit.column("Info", { width: grow(), gap: u(compact ? 4 : 8) }, () => {
      text("Name", (desc.name ?? "").toLocaleUpperCase(), { step: st(compact ? 22 : 34), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow() });
      const words = desc.subtitle ?? [];
      if (words.length > 0) kit.row("Subtitle", { gap: u(10), cross: "center", wrap: true, width: grow() } as never, () => {
        const tier = rarityColor(desc.rarity);
        words.forEach((w, i) => {
          if (i > 0) {
            kit.open(kit.idi("Rule", i), { width: px(Math.max(1, u(1.5))), height: px(u(14)) });
            kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("primary", 0.7) }] });
            kit.close();
          }
          const isTier = i === (desc.rarityWord ?? -1) && tier;
          text(kit.idi("Word", i), w.toLocaleUpperCase(), { step: st(compact ? 12 : 14), font: "display", nowrap: true, color: isTier ? mix(tier!, role("ink"), 0.2) : "inkMuted" });
        });
      });
      if (desc.description && !compact) text("Description", desc.description, { step: st(15), color: "inkMuted", width: grow(), maxLines: 2 });
    });
  };

  // The stage: the item large over a floor light, its attachment points called out above and below it on a desktop.
  const stage = (w: Api.Size, h: Api.Size, callouts: boolean): void => {
    // Last tick's art box inside the stage, and the stage's size: the callouts and their leaders are laid from them.
    const SW = kit.memory(`~lo.sw.${id}`, 0);
    const SH = kit.memory(`~lo.sh.${id}`, 0);
    const ax = kit.memory(`~lo.ax.${id}`, 0);
    const ay = kit.memory(`~lo.ay.${id}`, 0);
    const aw = kit.memory(`~lo.aw.${id}`, 0);
    const ah = kit.memory(`~lo.ah.${id}`, 0);
    const shown = callouts ? atts : [];
    const top = shown.filter((a) => (a.y ?? 0.5) < 0.5);
    const bottom = shown.filter((a) => (a.y ?? 0.5) >= 0.5);
    const band = Math.round(u(84) * Math.min(1.5, ts));
    let artRect: Api.UiRect | null = null;
    kit.column("Stage", { width: w, height: h, gap: 0, padding: xy(u(phone ? 14 : 28), u(phone ? 12 : 22)), part: "Loadout.stage", partClass: "panel" }, () => {
      const rr = Math.max(8, t.panel);
      kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("panel", 0.55) }, vgrad([[0, token("sheen", 0.05)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.12))] });
      info(!callouts);
      if (top.length > 0) {
        kit.open("Above", { width: grow(), height: px(band) });
        kit.close();
      }
      kit.column("Room", { width: grow(), height: grow(), align: "center", cross: "center" }, () => {
        // The floor light under the item.
        kit.open("Floor", { width: grow(), height: px(u(phone ? 30 : 60)) });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.5 }, to: { x: 1, y: 1 }, stops: [{ offset: 0, color: token("primary", 0.22) }, { offset: 1, color: token("primary", 0) }] }] } as Api.DrawDesc);
        kit.close();
        over({ insets: { start: u(24), end: u(24), bottom: 0 }, inert: true });
        // The art at its own aspect inside the room, so the points land where they are drawn.
        artSlot("Art", "Loadout.art", { texture: desc.image ?? null, kind: "item", fit: "contain", width: grow(), height: grow(), label: desc.name ?? "", ...((desc.aspect ?? 0) > 0 ? { aspect: desc.aspect } : {}) });
        artRect = kit.bounds(kit.lastPath());
        if (itemChanged && !still()) kit.props({ enter: { values: [{ prop: "opacity", value: 0.3 }, { prop: "scale", value: 0.97 }], transition: spring("panel") } });
      });
      if (bottom.length > 0) {
        kit.open("Below", { width: grow(), height: px(band) });
        kit.close();
      }

      // The callouts: each a tile of its art, the point's eyebrow and what is on it, laid along its band without overlap.
      if (shown.length > 0 && SW > 0 && aw > 0) {
        const cw = Math.round(Math.min(u(210) * Math.min(1.35, ts), (SW - 2 * u(28)) / Math.max(1, Math.max(top.length, bottom.length)) - u(10)));
        const at = (a: Api.LoadoutAttachment) => ({ x: ax + (a.x ?? 0.5) * aw, y: ay + (a.y ?? 0.5) * ah });
        const lay = (list: Api.LoadoutAttachment[]): Map<string, number> => {
          const sorted = [...list].sort((p, q) => (p.x ?? 0) - (q.x ?? 0));
          const xs = sorted.map((a) => clamp(at(a).x - cw / 2, u(8), SW - cw - u(8)));
          for (let i = 1; i < xs.length; i++) xs[i] = Math.max(xs[i]!, xs[i - 1]! + cw + u(10));
          for (let i = xs.length - 1; i >= 0; i--) xs[i] = Math.min(xs[i]!, (i === xs.length - 1 ? SW - u(8) : xs[i + 1]! - u(10)) - cw);
          return new Map(sorted.map((a, i) => [a.key ?? "", Math.max(u(8), xs[i]!)]));
        };
        const xsTop = lay(top);
        const xsBottom = lay(bottom);
        const ch = Math.round(u(60) * Math.min(1.5, ts));
        kit.column("Callouts", { width: grow(), height: grow() }, () => {
          for (const a of shown) {
            const k = a.key ?? "";
            const above = (a.y ?? 0.5) < 0.5;
            const x = (above ? xsTop : xsBottom).get(k) ?? 0;
            const y = above ? ay - band + u(4) : ay + ah + band - ch - u(4);
            const p = at(a);
            // The leader: from the callout's near edge to the point, and a lit dot on it.
            const x0 = x + cw / 2;
            const y0 = above ? y + ch : y;
            const lx = Math.min(x0, p.x);
            const ly = Math.min(y0, p.y);
            const lw = Math.max(2, Math.abs(p.x - x0));
            const lh = Math.max(2, Math.abs(p.y - y0));
            kit.open(`${k}~Leader`, { width: px(lw), height: px(lh), part: "Loadout.leader", partClass: "badge" });
            kit.setDraw({ shape: { kind: "path", points: [{ x: x0 - lx, y: y0 - ly }, { x: x0 - lx, y: (y0 + p.y) / 2 - ly }, { x: p.x - lx, y: p.y - ly }], width: Math.max(1, u(1.25)), cap: "round", join: "round", closed: false }, fills: [{ kind: "solid", color: token("primary", 0.75) }] } as Api.DrawDesc);
            kit.close();
            over({ insets: { start: rtl() ? SW - lx - lw : lx, top: ly }, inert: true });
            const dz = u(8);
            kit.open(`${k}~Dot`, { width: px(dz), height: px(dz) });
            kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("primary") }], effects: [glow(u(6), token("primary", 0.7), 0)] });
            kit.close();
            over({ insets: { start: rtl() ? SW - p.x - dz / 2 : p.x - dz / 2, top: p.y - dz / 2 }, inert: true });
            const on = desc.attachment === k;
            const rimC = rarityColor(a.rarity);
            const r = surface(k, {
              part: `Loadout.callout${flags(on && "selected")}`,
              cls: "listItem",
              fill: null,
              gap: u(10),
              cross: "center",
              width: px(cw),
              height: px(ch),
              padding: xy(u(6), u(6)),
              radius: Math.max(4, t.small),
              semantics: { role: "button", name: [a.label ?? "", a.name || desc.emptyLabel || "Empty"].join(": "), selected: on },
            }, () => {
              const ix = lastInteraction(k);
              kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [{ kind: "solid", color: token("shadow", 0.72) }], strokes: [line(on ? u(2) : 1, on ? token("focus") : ix.hovered || ix.focused ? token("ink", 0.45) : rimC ? { ...rimC, a: 0.55 } as never : token("primary", 0.45))], effects: on ? [glow(u(8), token("focus", 0.4), 0)] : [] });
              artSlot("Art", "Loadout.calloutArt", { texture: a.image ?? null, kind: "item", rarity: a.rarity, width: px(ch - u(12)), height: px(ch - u(12)), radius: Math.max(3, t.small - 2), label: "" });
              kit.column("Words", { width: grow(), gap: u(2) }, () => {
                text("Label", (a.label ?? "").toLocaleUpperCase(), { step: st(12), font: "display", nowrap: true, ellipsis: true, width: grow() });
                text("Name", a.name || desc.emptyLabel || "Empty", { step: st(12), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
              });
            });
            over({ insets: { start: rtl() ? SW - x - cw : x, top: y } });
            if (r.clicked) {
              out.attachment = k;
              kit.feedback("select", -1);
            }
          }
        });
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 } });
      }
    });
    const sr = kit.bounds(kit.lastPath());
    const ar = artRect as Api.UiRect | null;
    if (sr && sr.width > 0) {
      kit.remember(`~lo.sw.${id}`, Math.round(sr.width));
      kit.remember(`~lo.sh.${id}`, Math.round(sr.height));
      if (ar && ar.width > 0) {
        // The art's box as drawn: contain at its aspect, centred in its slot.
        const asp = (desc.aspect ?? 0) > 0 ? desc.aspect! : ar.width / Math.max(1, ar.height);
        const dw = Math.min(ar.width, ar.height * asp);
        const dh = dw / asp;
        // Physical x from the stage's left edge (a right-to-left stage keeps the art unmirrored).
        kit.remember(`~lo.ax.${id}`, Math.round(ar.x - sr.x + (ar.width - dw) / 2));
        kit.remember(`~lo.ay.${id}`, Math.round(ar.y - sr.y + (ar.height - dh) / 2));
        kit.remember(`~lo.aw.${id}`, Math.round(dw));
        kit.remember(`~lo.ah.${id}`, Math.round(dh));
      }
    }
  };

  // The figures' card.
  const stats = (compact: boolean): void => {
    if ((desc.stats ?? []).length === 0) return;
    kit.column("Stats", { width: grow(), gap: u(compact ? 8 : 12), padding: xy(u(16), u(compact ? 12 : 16)), part: "Loadout.stats", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(6), u(20), 0.45)] });
      eyebrow("Title", desc.statsLabel ?? "Weapon stats");
      for (const s of desc.stats ?? []) statRow(s, compact);
    });
  };

  // An attachment's row: its art on a tier-rimmed tile, its name, its tier's band, and a remove button.
  const attachmentRow = (a: Api.LoadoutAttachment): void => {
    const k = a.key ?? "";
    const on = desc.attachment === k;
    const tier = rarityColor(a.rarity) ?? role("primary");
    let removed = false;
    const rr = Math.max(6, t.small);
    const r = surface(k, {
      part: `Loadout.attachment${flags(on && "selected")}`,
      cls: "listItem",
      fill: null,
      gap: u(10),
      cross: "center",
      width: grow(),
      padding: xy(u(6), u(6)),
      radius: rr,
      semantics: { role: "button", name: [a.label ?? "", a.name ?? "", a.rarityLabel ?? ""].filter(Boolean).join(", "), selected: on },
    }, () => {
      const ix = lastInteraction(k);
      kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [vgrad([[0, { ...tier, a: 0.22 } as never], [0.6, token("panel", 0.9)]], 90)], strokes: [line(on ? u(2) : 1, on ? token("focus") : ix.hovered || ix.focused ? token("ink", 0.4) : { ...tier, a: 0.45 } as never)] });
      const z = u(phone ? 48 : 52);
      artSlot("Art", "Loadout.attachmentArt", { texture: a.image ?? null, kind: "item", rarity: a.rarity, width: px(z), height: px(z), radius: Math.max(3, t.small - 2), label: "" });
      kit.column("Words", { width: grow(), gap: u(4) }, () => {
        text("Name", a.name ?? "", { step: st(15), nowrap: true, ellipsis: true, width: grow() });
        if (a.rarityLabel) kit.row("Band", { gap: 0, padding: xy(u(8), u(2)), width: grow() }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [vgrad([[0, { ...tier, a: 0.55 } as never], [1, { ...tier, a: 0 } as never]], 90)] });
          text("Tier", a.rarityLabel!.toLocaleUpperCase(), { step: st(11), font: "display", nowrap: true, color: mix(tier, role("ink"), 0.35) });
        });
      });
      const d = Math.max(44, u(36));
      removed = surface("Remove", {
        part: "Loadout.remove",
        cls: "button",
        fill: { ...role("shadow"), a: 0.4 },
        washBy: 0.16,
        width: px(d),
        height: px(d),
        align: "center",
        cross: "center",
        radius: Math.max(4, t.small),
        semantics: { role: "button", name: `${desc.removeLabel ?? "Remove"} ${a.name ?? ""}` },
      }, () => symbol("Mark", "cross", u(14), role("ink"))).clicked;
    });
    if (removed) {
      out.remove = k;
      kit.feedback("confirm", -1);
    } else if (r.clicked) {
      out.attachment = k;
      kit.feedback("select", -1);
    }
  };

  // The attachments' card: its heading and count, the rows.
  const attachments = (): void => {
    const filled = atts.filter((a) => !!a.name);
    const cap = (desc.capacity ?? 0) > 0 ? desc.capacity! : atts.length;
    kit.column("Attachments", { width: grow(), gap: u(8), padding: xy(u(16), u(14)), part: "Loadout.attachments", partClass: "panel" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.9) }], strokes: [line(1, token("ink", 0.1))], effects: [shadow(u(6), u(20), 0.45)] });
      kit.row("Head", { width: grow(), gap: u(8), cross: "center" }, () => {
        eyebrow("Title", desc.attachmentsLabel ?? "Attachments");
        kit.spacer("Gap", grow());
        text("Count", `${filled.length}/${cap}`, { step: st(14), font: "mono", nowrap: true, color: "inkMuted" });
      });
      kit.column("List", { width: grow(), gap: u(6) }, () => filled.forEach(attachmentRow));
      kit.semantics({ role: "list", name: desc.attachmentsLabel ?? "Attachments" });
    });
  };

  if (portrait) {
    kit.openScroll("Slots", { width: grow(), gap: 0, padding: xy(0, u(2)), axes: "horizontal", part: "Loadout.slots" } as never);
    kit.open("Row", { direction: "row", gap: u(8) });
    slots.forEach((s) => slotEntry(s, true));
    kit.close();
    kit.close();
    kit.semantics({ role: "tablist", name: desc.title || "Loadout" });
    kit.openScroll("Body", { width: grow(), height: grow(), gap: u(10), padding: xy(0, 0), fade: 40, fadeInto: "shadow" } as never);
    stage(grow(), px(Math.round(Math.max(W * 0.62, u(200) * Math.min(1.4, ts)))), false);
    stats(true);
    attachments();
    kit.close();
  } else {
    kit.row("Body", { width: grow(), height: grow(), gap: u(16) }, () => {
      kit.openScroll("Slots", { width: px(Math.round(Math.min(u(340), W * 0.24) * Math.min(1.25, ts))), height: grow(), gap: u(8), padding: xy(u(2), u(2)), fade: 24, fadeInto: "shadow", part: "Loadout.slots" } as never);
      slots.forEach((s) => slotEntry(s, false));
      kit.close();
      kit.semantics({ role: "tablist", name: desc.title || "Loadout" });
      // The callouts need the room's height: at a large text size the attachments' rows carry them alone.
      stage(grow(), grow(), !phone && ts <= 1.25);
      // The side scrolls as one: at a large text size its cards are taller than the room.
      kit.openScroll("Side", { width: px(Math.round(Math.min(u(380), W * 0.26) * Math.min(1.25, ts))), height: grow(), gap: u(12), padding: xy(0, 0), fade: 32, fadeInto: "shadow" } as never);
      stats(false);
      attachments();
      kit.close();
    });
  }
  if (out.refused) {
    kit.remember("~lo.rt", tick);
    kit.remember("~lo.rk", keyHash(out.refused));
  }
  kit.close();
  rememberRoom("lo", id);
  return out;
}
