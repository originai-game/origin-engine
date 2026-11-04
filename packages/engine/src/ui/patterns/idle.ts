// Idle (docs/ui.md §11 genre patterns, Appendix B: management and idle
// generators): a generator's row and the prestige panel. The game holds the
// numbers and writes them in its notation (og.ui.formatNumber); the row's
// held-buy timing is the component's own memory.

import type * as Api from "@origingame/engine";
import { holdButton } from "../controls/buttons.ts";
import {
  artSlot, btn, clamp, fill, frame, glow, grow, hex, kit, lastInteraction, line, mix, over, paint, pct, px, radial, role, shadow, solid, st, still,
  symbol, text, token, u, vgrad, vstate, xy, type Mut,
} from "./core.ts";
import { STAR_GOLD } from "./stars.ts";

/** Ticks a held Buy waits before it repeats, then between repeats. */
const HOLD_DELAY = 24;
const HOLD_EVERY = 6;

/** One generator of an idle game: see `fn idleGenerator` in crates/og-api/api/ui.ogt. */
export function idleGenerator(id: string, desc: Api.IdleGeneratorDesc): Api.IdleGeneratorResult {
  const { phone } = frame();
  const out: Mut<Api.IdleGeneratorResult> = { bought: false, short: false, held: false };
  const tick = kit.tick();
  const moving = !still();
  const locked = !!desc.locked;
  const can = !locked && !!desc.affordable;
  const art = u(phone ? 56 : 72);
  const name = desc.name ?? "";
  const n = Math.max(1, desc.buyCount ?? 1);
  const bought = kit.memory(`${id}~pop`, -1000);
  const pop = moving && tick - bought < 14 ? 1 + 0.06 * Math.sin(((tick - bought) / 14) * Math.PI) : 1;

  // On a phone the Buy button takes its own line under the art and the words, the row's width: beside them it left
  // the words too little room once its cost and label fitted (a pixel face, large text).
  kit.open(id, { direction: phone ? "column" : "row", width: grow(), gap: u(phone ? 10 : 14), cross: phone ? "stretch" : "center", padding: xy(u(10), u(10)), radius: u(12), part: `Idle.generator${locked ? "[locked]" : ""}${can ? "[affordable]" : ""}`, partClass: "panel" });
  // Dark glass with a quiet rim; a row the purse can buy from carries a thin accent along its start edge.
  kit.setDraw({
    shape: { kind: "rect", radii: [u(12), u(12), u(12), u(12)] },
    fills: [vgrad([[0, paint(mix(role("panelRaised"), role("shadow"), 0.35), 0.9)], [1, paint(mix(role("panel"), role("shadow"), 0.5), 0.9)]])],
    strokes: [line(1, paint(mix(role("ink"), role("panel"), 0.7), 0.55))],
    effects: [shadow(u(3), u(10), 0.5)],
  } as Api.DrawDesc);
  kit.props({ transform: { scaleX: pop, scaleY: pop }, states: [{ name: "purse", states: [vstate("afford"), vstate("short")], transitions: [] }], state: [{ group: "purse", state: can ? "afford" : "short" }] });
  if (phone) kit.open("Top", { direction: "row", width: grow(), gap: u(10), cross: "center" });
  // The art, darkened and locked until it is unlocked.
  kit.open("Art", { width: px(art), height: px(art), part: "Idle.art", partClass: "badge" });
  if (desc.image) artSlot("Pic", "Idle.pic", { texture: desc.image, kind: "item", width: px(art), height: px(art), radius: u(8) });
  else {
    kit.setDraw({ shape: { kind: "rect", radii: [u(8), u(8), u(8), u(8)] }, fills: [vgrad([[0, paint(mix(role("info"), role("panel"), 0.55))], [1, paint(mix(role("info"), role("shadow"), 0.7))]])] } as Api.DrawDesc);
    symbol("Pic", "bolt", Math.round(art * 0.55), mix(role("info"), hex("#ffffff"), 0.3), { icon: desc.icon || undefined });
  }
  if (locked) {
    kit.open("Shade", { width: px(art), height: px(art), align: "center", cross: "center", radius: u(8), background: token("shadow", 0.62), part: "Idle.lock", partClass: "badge" });
    symbol("Lock", "lock", Math.round(art * 0.36), mix(role("ink"), role("inkFaint"), 0.3), { icon: "lock" });
    kit.close();
    over({ insets: { start: 0, top: 0 }, inert: true, z: 2 });
  }
  kit.close();
  // Its words: name over the level and owned, the cycle's bar and the rate.
  kit.open("Info", { direction: "column", width: grow(), gap: u(4) });
  text("Name", name, { step: st(phone ? 16 : 18), font: "display", nowrap: true, ellipsis: true, width: grow(), color: locked ? "inkMuted" : "ink" });
  if (locked) {
    if (desc.lockedHint) text("Hint", desc.lockedHint, { step: st(13), color: "warn", maxLines: 2, width: grow() });
  } else {
    // Both lines wrap at large text rather than cut what they say.
    kit.row("Meta", { gap: u(10), cross: "center", width: grow(), wrap: true }, () => {
      text("Level", fill(desc.levelLabel ?? "Lv. {level}", { level: desc.level ?? 0 }), { step: st(13), color: role("info"), nowrap: true, font: "display" });
      if (desc.ownedText) text("Owned", fill(desc.ownedLabel ?? "Owned: {owned}", { owned: desc.ownedText }), { step: st(13), color: "inkMuted", nowrap: true });
    });
    kit.row("Line", { gap: u(10), cross: "center", width: grow(), wrap: true }, () => {
      const secs = desc.cycleSeconds ?? 0;
      if (secs > 0) {
        // Under a quarter second a cycle is too fast to read as a fill: the bar stays full and a light runs along it.
        const fast = secs < 0.25;
        const w = u(phone ? 90 : 120);
        const h = u(6);
        kit.open("Cycle", { width: px(w), height: px(h), radius: 99, part: `Idle.cycle${fast ? "[fast]" : ""}`, partClass: "bar" });
        kit.setDraw({ shape: { kind: "rect", radii: [h, h, h, h] }, fills: [solid(role("shadow"), 0.8)] } as Api.DrawDesc);
        kit.open("Fill", { width: pct(fast ? 1 : clamp(desc.cycle ?? 0, 0, 1)), height: grow(), radius: 99, part: "Idle.cycleFill", partClass: "bar" });
        const sweep = moving && fast ? (tick % 40) / 40 : -1;
        kit.setDraw({
          shape: { kind: "rect", radii: [h, h, h, h] },
          fills: [
            solid(role("info")),
            ...(sweep >= 0 ? [{ kind: "gradient", type: "linear", from: { x: sweep - 0.25, y: 0.5 }, to: { x: sweep + 0.25, y: 0.5 }, stops: [{ offset: 0, color: paint(hex("#ffffff"), 0) }, { offset: 0.5, color: paint(hex("#ffffff"), 0.55) }, { offset: 1, color: paint(hex("#ffffff"), 0) }] } as Api.Fill] : []),
          ],
          effects: [glow(u(4), paint(role("info"), 0.5), 0)],
        } as Api.DrawDesc);
        kit.close();
        kit.close();
        kit.props({ states: [{ name: "cycle", states: [vstate("slow"), vstate("fast")], transitions: [] }], state: [{ group: "cycle", state: fast ? "fast" : "slow" }] });
        kit.semantics({ role: "meter", name: name, valueMin: 0, valueMax: 100, valueNow: Math.round((fast ? 1 : desc.cycle ?? 0) * 100) });
      }
      if (desc.rateText) text("Rate", desc.rateText, { step: st(13), color: role("info"), nowrap: true, font: "display" });
    });
  }
  kit.close();
  if (phone) kit.close();
  // Buy: lit (the primary's raised face) while the purse pays, glass when it does not; its cost over its label.
  const was = lastInteraction("Buy");
  const label = fill(desc.buyLabel ?? "Buy ×{n}", { n });
  const r = btn("Buy", "", {
    variant: can ? "primary" : "glass",
    size: "lg",
    disabled: locked,
    // As wide as its cost and label need (a pixel face, large text), never narrower than a steady column of buttons.
    width: phone ? grow() : ({ kind: "fit", min: u(128) } as Api.Size),
    part: `Idle.buy${can ? "[affordable]" : "[short]"}`,
    name: `${label}, ${desc.costText ?? ""}${can || locked ? "" : ", short"}`,
    before: () => {
      kit.column("Stack", { gap: 0, cross: "center" }, () => {
        // The cost leads, in the button's own ink (dark on the lit face, the ink on glass), the currency's art before it.
        const ink = can ? hex("#0b1220") : role("ink");
        kit.row("Cost", { gap: u(5), cross: "center", part: "Idle.cost", partClass: "badge" }, () => {
          const ic = u(phone ? 16 : 18);
          if (desc.currencyIcon) artSlot("Icon", "Idle.currency", { texture: desc.currencyIcon, kind: "currency", width: px(ic), height: px(ic), radius: u(4) });
          else symbol("Icon", "coin", ic, STAR_GOLD);
          text("Amount", desc.costText ?? "", { step: st(16), color: ink, nowrap: true, font: "display" });
        });
        text("Label", label, { step: st(13), color: can ? mix(ink, hex("#3a5a6a"), 0.35) : "inkMuted", nowrap: true, font: "display" });
      });
    },
  });
  // Held, it repeats after a beat while the purse pays.
  const heldFrom = kit.memory(`${id}~held`, -1);
  if (r.pressed && was.pressed && !locked) {
    out.held = true;
    if (heldFrom < 0) kit.remember(`${id}~held`, tick);
    else if (can && tick - heldFrom >= HOLD_DELAY && (tick - heldFrom - HOLD_DELAY) % HOLD_EVERY === 0) out.bought = true;
  } else if (heldFrom >= 0) kit.remember(`${id}~held`, -1);
  if (r.clicked && !locked) {
    if (can) out.bought = true;
    else {
      out.short = true;
      kit.feedback("error", -1);
    }
  }
  if (out.bought) kit.remember(`${id}~pop`, tick);
  kit.close();
  kit.semantics({ role: "group", name });
  return out;
}

/** An idle game's prestige: see `fn prestigePanel` in crates/og-api/api/ui.ogt. */
export function prestigePanel(id: string, desc: Api.PrestigeDesc): Api.PrestigeResult {
  const { phone } = frame();
  const ready = !!desc.available;
  const tick = kit.tick();
  const moving = !still();
  const em = u(phone ? 72 : 96);
  kit.open(id, { direction: "column", width: grow(), gap: u(12), padding: xy(u(18), u(16)), radius: u(14), part: `Prestige.frame${ready ? "[available]" : ""}`, partClass: "panel" });
  kit.setDraw({
    shape: { kind: "rect", radii: [u(14), u(14), u(14), u(14)] },
    fills: [vgrad([[0, paint(mix(role("panelRaised"), role("shadow"), 0.3), 0.92)], [1, paint(mix(role("panel"), role("shadow"), 0.55), 0.92)]])],
    strokes: [line(1, paint(ready ? STAR_GOLD : mix(role("ink"), role("panel"), 0.7), ready ? 0.55 : 0.45))],
    effects: [shadow(u(4), u(14), 0.55)],
  } as Api.DrawDesc);
  kit.props({ states: [{ name: "availability", states: [vstate("ready"), vstate("waiting")], transitions: [] }], state: [{ group: "availability", state: ready ? "ready" : "waiting" }] });
  text("Title", (desc.title ?? "Prestige"), { uppercase: true, step: st(14), font: "display", color: "inkMuted", nowrap: true, heading: true });
  kit.row("Body", { width: grow(), gap: u(16), cross: "center" }, () => {
    // The emblem, glowing in gold while a reset gives something.
    const glowK = ready ? (moving ? 0.55 + 0.25 * Math.sin(tick * 0.06) : 0.65) : 0;
    kit.open("Emblem", { width: px(em), height: px(em), align: "center", cross: "center", part: "Prestige.emblem", partClass: "badge" });
    // A soft gold halo behind it (a radial wash, not a glow on an empty disc, which paints the disc whole).
    if (glowK > 0) kit.setDraw({ shape: { kind: "ellipse" }, fills: [radial([[0, paint(STAR_GOLD, glowK * 0.55)], [0.6, paint(STAR_GOLD, glowK * 0.18)], [1, paint(STAR_GOLD, 0)]])] } as Api.DrawDesc);
    const inner = Math.round(em * 0.78);
    if (desc.image) artSlot("Pic", "Prestige.pic", { texture: desc.image, kind: "emblem", width: px(inner), height: px(inner), radius: Math.round(inner / 2) });
    else symbol("Pic", "crown", inner, ready ? STAR_GOLD : mix(role("inkFaint"), role("ink"), 0.3));
    kit.close();
    kit.column("Words", { width: grow(), gap: u(4) }, () => {
      if (desc.lead) text("Lead", desc.lead, { step: st(14), color: "inkMuted", maxLines: 2, width: grow() });
      text("Reward", desc.rewardText ?? "", { step: st(phone ? 24 : 30), font: "display", color: ready ? STAR_GOLD : role("inkMuted"), maxLines: 2, width: grow() });
      (desc.points ?? []).forEach((pt, k) => {
        kit.row(kit.idi("Point", k), { gap: u(6), cross: "center", width: grow() }, () => {
          symbol("Tick", "check", u(14), ready ? role("success") : role("inkFaint"));
          text("Text", pt, { step: st(13), color: "ink", maxLines: 2, width: grow() });
        });
      });
    });
  });
  // The reset is held to confirm, so a run is never thrown away by a slip.
  const h = holdButton("Action", desc.actionLabel ?? "Ascend", { variant: "primary", size: "large", holdMs: desc.holdMs ?? 1200, disabled: !ready, width: grow(), ...(ready ? {} : desc.lockedHint ? { hint: desc.lockedHint } : {}) } as Api.HoldButtonDesc);
  kit.close();
  kit.semantics({ role: "region", name: desc.title ?? "Prestige" });
  return { confirmed: h.confirmed, holding: h.holding };
}
