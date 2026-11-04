// Puzzle goals (docs/ui.md §11 genre patterns, Appendix B: puzzle HUD
// goals): a level's goals as chips (the score toward its stars is
// og.ui.starBar, stars.ts). The game holds the counts; which goals were
// already shown met is the component's own memory, so a crossing pops once.

import type * as Api from "@origingame/engine";
import {
  artSlot, clamp, fill, frame, glow, grouped, grow, hex, kit, line, mix, over, paint, pct, px, role, shadow, solid, spring, st, still, symbol, text,
  token, u, vgrad, vstate, xy,
} from "./core.ts";
import { STAR_GOLD } from "./stars.ts";

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

