// Puzzle goals (docs/ui.md §11 genre patterns, Appendix B: puzzle HUD
// goals): a level's score toward its stars, and its goals as chips. The
// game holds the score and the counts; which stars and goals were already
// shown lit is the component's own memory, so a crossing pops once.

import type * as Api from "@origingame/engine";
import {
  artSlot, clamp, fill, frame, glow, grouped, grow, hex, kit, line, mix, over, paint, pct, px, role, shadow, solid, spring, st, still, symbol, text,
  token, u, vgrad, vstate, xy,
} from "./core.ts";
import { STAR_GOLD } from "./saga.ts";

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

/** A level's goals as chips: see `fn goalChips` in crates/og-api/api/ui.ogt. */
export function goalChips(id: string, desc: Api.GoalChipsDesc): void {
  const goals = desc.goals ?? [];
  const compact = !!desc.compact;
  const tick = kit.tick();
  const moving = !still();
  kit.open(id, { direction: "row", gap: u(compact ? 8 : 12), cross: "center", wrap: !compact, part: `Goals.frame${compact ? "[compact]" : ""}`, partClass: "panel" });
  goals.forEach((gl, i) => {
    const key = gl.key || `g${i}`;
    const need = Math.max(1, gl.need ?? 1);
    const have = Math.max(0, gl.have ?? 0);
    const done = have >= need;
    const last = kit.memory(`${id}~${key}`, -1);
    if (last >= 0 && have > last) {
      kit.remember(`${id}~${key}~pop`, tick);
      if (done && last < need) kit.feedback("success", -1);
    }
    kit.remember(`${id}~${key}`, have);
    const popAt = kit.memory(`${id}~${key}~pop`, -1000);
    const pop = moving && tick - popAt < 16 ? 1 + 0.22 * Math.sin(((tick - popAt) / 16) * Math.PI) : 1;
    const ink = done ? role("success") : role("ink");
    const art = u(compact ? 30 : 40);
    kit.open(key, { direction: "row", gap: u(compact ? 6 : 10), cross: "center", padding: xy(u(compact ? 8 : 12), u(compact ? 4 : 8)), radius: u(10), part: `Goals.chip${done ? "[done]" : ""}`, partClass: "panel" });
    // Dark glass with a rim; met, the rim and a soft wash turn to the success ink.
    kit.setDraw({
      shape: { kind: "rect", radii: [u(10), u(10), u(10), u(10)] },
      fills: [solid(mix(role("panel"), role("shadow"), 0.45), 0.84), ...(done ? [solid(role("success"), 0.12)] : [])],
      strokes: [line(1, paint(done ? role("success") : mix(role("ink"), role("panel"), 0.65), done ? 0.9 : 0.5))],
      effects: [shadow(u(2), u(8), 0.5)],
    } as Api.DrawDesc);
    kit.open("Art", { width: px(art), height: px(art), part: "Goals.art", partClass: "badge" });
    if (gl.image) artSlot("Pic", "Goals.pic", { texture: gl.image, kind: "item", width: px(art), height: px(art), radius: u(6) });
    else symbol("Pic", "gem", art, mix(role("info"), hex("#ffffff"), 0.15), { icon: gl.icon || undefined });
    if (done) {
      kit.open("Check", { width: px(u(18)), height: px(u(18)), align: "center", cross: "center", part: "Goals.check", partClass: "badge" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [solid(role("success"))], strokes: [line(1.5, token("shadow", 0.8), "outside")] } as Api.DrawDesc);
      symbol("Mark", "check", u(12), hex("#ffffff"));
      kit.close();
      over({ insets: { end: -u(4), bottom: -u(4) }, inert: true, z: 2 });
    }
    kit.close();
    kit.column("Text", { gap: u(3) }, () => {
      if (!compact && gl.label) text("Label", gl.label, { step: st(13), color: "inkMuted", nowrap: true });
      text("Count", `${grouped(Math.min(have, need))} / ${grouped(need)}`, { step: st(compact ? 15 : 17), font: "display", nowrap: true, color: ink });
      kit.props({ transform: { scaleX: pop, scaleY: pop } });
      if (!compact) {
        const w = u(120);
        kit.open("Track", { width: px(w), height: px(u(5)), radius: 99, background: token("shadow", 0.7), part: "Goals.track", partClass: "bar" });
        kit.open("Fill", { width: pct(clamp(have / need, 0, 1)), height: grow(), radius: 99, background: paint(done ? role("success") : role("info")), part: "Goals.fill", partClass: "bar" });
        kit.props({ layoutMotion: spring("ui") });
        kit.close();
        kit.close();
      }
    });
    kit.close();
    kit.props({ states: [{ name: "goal", states: [vstate("open"), vstate("done")], transitions: [] }], state: [{ group: "goal", state: done ? "done" : "open" }] });
    kit.semantics({ role: "listitem", name: [gl.label ?? key, fill(desc.progressLabel ?? "{have} of {need}", { have: Math.min(have, need), need }), done ? desc.doneLabel ?? "Done" : ""].filter(Boolean).join(", ") });
  });
  kit.close();
  kit.semantics({ role: "list", name: desc.label ?? "Goals" });
}

