// og.ui.starBar: a level's score toward its stars (docs/ui.md §2, Appendix
// B: puzzle HUD goals). The game holds the score; which stars were already
// shown lit is the component's own memory, so a crossing pops once. Split
// out of goals.ts unchanged when goalChips moved into the puzzle kit
// (docs/ui.md §5).

import type * as Api from "@origingame/engine";
import { clamp, fill, frame, glow, grouped, grow, hex, kit, line, mix, over, paint, pct, px, role, shadow, solid, spring, st, still, symbol, text, u, vgrad, vstate } from "./core.ts";

/** The star gold the stars are lit in on every style: a star is a reward, not a style's accent. */
export const STAR_GOLD = hex("#ffc93c");

/** The thresholds a bar uses: the game's, else a third, two thirds and all of max. */
function thresholds(desc: Api.StarBarDesc): number[] {
  const t = (desc.thresholds ?? []).filter((x) => Number.isFinite(x)).slice().sort((a, b) => a - b);
  if (t.length > 0) return t;
  const m = desc.max ?? 0;
  return [m / 3, (m * 2) / 3, m];
}

/** A level's score toward its stars: see `fn starBar` in crates/og-api/api/ui.ogt. */
export function starBar(id: string, desc: Api.StarBarDesc): Api.StarBarResult {
  const { phone } = frame();
  const th = thresholds(desc);
  const max = (desc.max ?? 0) > 0 ? desc.max! : th[th.length - 1] ?? 1;
  const value = Math.max(0, desc.value ?? 0);
  const earned = th.filter((x) => value >= x).length;
  const tick = kit.tick();
  const moving = !still();
  const was = kit.memory(`${id}~earned`, -1);
  const crossed = was >= 0 ? Math.max(0, earned - was) : 0;
  if (crossed > 0) {
    kit.remember(`${id}~pop`, tick);
    // The stars crossed this tick pop: those from the count shown before.
    kit.remember(`${id}~from`, was);
    kit.feedback("success", -1);
  }
  kit.remember(`${id}~earned`, earned);
  const popAt = kit.memory(`${id}~pop`, -1000);
  const popFrom = kit.memory(`${id}~from`, 0);
  const compact = !!desc.compact;
  const W = u(compact ? 150 : 220);
  const barH = u(compact ? 10 : 12);
  const sz = u(compact ? 22 : 30);
  const f = clamp(value / max, 0, 1);

  kit.open(id, { direction: compact ? "row" : "column", gap: u(compact ? 10 : 4), cross: "center", part: `StarBar.frame${compact ? "[compact]" : ""}`, partClass: "panel" });
  if (!compact) {
    if (desc.label) text("Label", desc.label.toLocaleUpperCase(), { step: st(13), color: "inkMuted", font: "display", nowrap: true });
    text("Value", desc.valueText || grouped(value), { step: st(phone ? 26 : 34), font: "display", nowrap: true });
  } else {
    text("Value", desc.valueText || grouped(value), { step: st(18), font: "display", nowrap: true });
  }
  // The bar with room over it for the stars standing on their thresholds.
  kit.open("Bar", { width: px(W), height: px(sz + barH - Math.round(sz * 0.35)), part: "StarBar.bar", partClass: "bar" });
  kit.open("Track", { width: px(W), height: px(barH), radius: barH, part: "StarBar.track", partClass: "bar" });
  kit.setDraw({ shape: { kind: "rect", radii: [barH, barH, barH, barH] }, fills: [solid(role("shadow"), 0.78)], strokes: [line(1, paint(mix(role("ink"), role("shadow"), 0.7), 0.6))] } as Api.DrawDesc);
  kit.open("Fill", { width: pct(f), height: grow(), radius: barH, part: "StarBar.fill", partClass: "bar" });
  // Warm gold with a light along its top, so it reads as a filled tube.
  kit.setDraw({ shape: { kind: "rect", radii: [barH, barH, barH, barH] }, fills: [vgrad([[0, paint(mix(STAR_GOLD, hex("#ffffff"), 0.35))], [0.5, paint(STAR_GOLD)], [1, paint(mix(STAR_GOLD, hex("#b06a00"), 0.5))]])], effects: [glow(u(6), paint(STAR_GOLD, 0.5), 0)] } as Api.DrawDesc);
  kit.props({ layoutMotion: spring("ui") });
  kit.close();
  kit.close();
  over({ insets: { start: 0, bottom: 0 }, inert: true });
  kit.open("Stars", { width: px(W), height: px(sz), part: "StarBar.stars", partClass: "badge" });
  th.forEach((x, k) => {
    const on = value >= x;
    const at = clamp(x / max, 0, 1) * W;
    const popping = on && moving && tick - popAt < 20 && k >= popFrom;
    const s = popping ? 1 + 0.35 * Math.sin(((tick - popAt) / 20) * Math.PI) : 1;
    kit.open(kit.idi("Star", k), { width: px(sz), height: px(sz), part: `StarBar.star${on ? "[lit]" : ""}`, partClass: "badge" });
    // A dark disc under each star so an outline reads over the fill and the scene alike.
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [solid(role("shadow"), on ? 0 : 0.55)], effects: on ? [glow(u(8), paint(STAR_GOLD, 0.65), 0)] : [] } as Api.DrawDesc);
    symbol("Mark", on ? "star" : "starOutline", sz, on ? STAR_GOLD : mix(role("ink"), role("inkFaint"), 0.35));
    kit.close();
    const x0 = Math.round(clamp(at - sz / 2, 0, W - sz));
    over({
      insets: { start: x0, top: 0 },
      transform: { scaleX: s, scaleY: s },
      states: [{ name: "lit", states: [vstate("off"), vstate("on")], transitions: [] }],
      state: [{ group: "lit", state: on ? "on" : "off" }],
    });
  });
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true, z: 2 });
  kit.close();
  kit.close();
  kit.semantics({ role: "meter", name: desc.label || "Score", value: desc.valueText || grouped(value), valueMin: 0, valueMax: max, valueNow: value, description: fill(desc.starsLabel ?? "{n} of {max} stars", { n: earned, max: th.length }) });
  return { earned, crossed };
}
