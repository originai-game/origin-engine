// og.ui.objectiveTracker: Appendix C's "objective tracker", read against
// the trackers of The Witcher 3 and World of Warcraft (a title over steps
// that tick with a pop, counts, a countdown, the tracked one marked). The
// minimap and compass are component documents now (src/ui/components).

import type * as Api from "@origingame/engine";
import { decimalNumber } from "../data/format.ts";
import { textScale, a11y, absolute, anchorOf, box, clamp01, edge, fillDraw, frame, groups, hudPlace, hudRoom, icon, keyId, kit, leaf, lit, msToTicks, now, open, pct, pop, px, role, sel, solid, sound, stroke, text, token, trackDraw, ts, u, vgrad, countdown, apply, sideSpan, stepSize, isolate } from "./core.ts";

export function objectiveTracker(id: string, desc: Api.ObjectiveTrackerDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const t = now();
  // A phone at a large text size has room for the focused quest only.
  // Folded when its side's stack outgrew the room its side leaves last
  // tick: first one quest, then its steps cut to two open ones, then one.
  const anchor = anchorOf(desc, "right");
  const vp0 = f.viewport;
  const roomH = vp0.height - vp0.safeArea.top - vp0.safeArea.bottom - tk.gutter * 2;
  const foldKey = `${id}~fold/${Math.round(roomH)}`;
  let fold = kit.memory(foldKey, 0);
  // The room is what its side leaves between the pieces at its top and bottom edges.
  const fits = anchor === "left" || anchor === "right" ? sideSpan(anchor).room : roomH;
  if (anchor && kit.memory(`~hudStack/${anchor}/total`, 0) > fits + 0.5 && fold < 3) fold += 1;
  kit.remember(foldKey, fold);
  const max = fold > 0 ? 1 : desc.max || (f.phone ? (textScale() >= 1.5 ? 1 : 2) : 3);
  const width = Math.min(u(desc.width || 300), hudRoom());
  // Under about 280 design px the head's title and its timer cannot share a line.
  const narrow = width < u(280);
  const quests = (desc.quests ?? []).filter((q) => q.completedAt === null || q.completedAt === undefined || t - q.completedAt < msToTicks(2400)).slice(0, max);
  hudPlace(id, anchor, () => {
    box(id, { direction: "column", gap: tk.md, width: px(width), padding: { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm }, part: sel("ObjectiveTracker", "frame"), partClass: "hud" }, {
      // A soft scrim from the edge it sits on, so steps read over any scene.
      fills: [vgrad([[0, token("sceneShadow", 0)], [0.25, token("sceneShadow", 0.42)], [1, token("sceneShadow", 0.5)]], kit.frameDirection() === "rtl" ? -90 : 90)],
    }, () => {
      for (const q of quests) {
        const done = q.completedAt !== null && q.completedAt !== undefined;
        const since = done ? t - (q.completedAt ?? t) : 0;
        const e = edge(`${keyId(q.key ?? "")}~done`, done ? 1 : 0);
        sound("reward", e.changed && done);
        const leave = done ? clamp01((since - msToTicks(1800)) / msToTicks(600)) : 0;
        box(keyId(q.key ?? q.title ?? ""), {
          direction: "column", gap: u(4), width: pct(1),
          part: sel("ObjectiveTracker", "quest", q.focused ? "focused" : null, done ? "complete" : null),
        }, undefined, () => {
          const meta = (): void => {
            if (q.timer !== null && q.timer !== undefined) {
              text("Timer", countdown(q.timer, false), { step: ts(13), font: "mono", color: q.timer < 10 ? lit(role("danger")) : token("ink"), onScene: true, nowrap: true, outline: 1 });
            }
            if (q.distance !== null && q.distance !== undefined) {
              text("Distance", `${decimalNumber(Math.round(q.distance), 0)}${desc.units ?? "m"}`, { step: ts(12), font: "mono", color: token("inkMuted"), onScene: true, nowrap: true, outline: 1 });
            }
          };
          // The timer and distance share the title's line only when both fit whole beside it (a measured guess:
          // display letters at 0.6 em, mono at 0.62); else they take their own line rather than crowd or cut it.
          const metaText = [q.timer != null ? countdown(q.timer, false) : "", q.distance != null ? `${decimalNumber(Math.round(q.distance), 0)}${desc.units ?? "m"}` : ""].filter(Boolean);
          const metaW = metaText.reduce((w, m, i) => w + m.length * stepSize(ts(13)).size * 0.62 + (i > 0 ? tk.md : 0), 0);
          const lead = (q.focused ? u(3) + tk.sm : 0) + (q.icon ? u(18) + tk.sm : 0);
          const titleW = (q.title ?? "").length * stepSize(ts(15, true)).size * 0.6;
          const apart = narrow || (metaText.length > 0 && lead + titleW + tk.sm + metaW > width - tk.md * 2);
          kit.row("Head", { gap: tk.sm, cross: "center", width: pct(1) }, () => {
            if (q.focused) leaf("Mark", { width: px(u(3)), height: px(u(18)), part: sel("ObjectiveTracker", "focus") }, { fills: [solid(token("highlight"))] });
            if (q.icon) icon("Icon", q.icon, u(18), q.focused ? token("highlight") : lit(role("gold")));
            text("Title", q.title ?? "", { step: ts(15), font: "display", color: q.focused ? token("highlight") : token("ink"), onScene: true, width: { kind: "grow" }, maxLines: 2, ellipsis: true, outline: 1 });
            if (!apart) meta();
          });
          // A narrow tracker (a phone's) gives the title the whole line and
          // the timer and distance their own line under it.
          if (apart && metaText.length > 0) kit.row("Meta", { gap: tk.md, cross: "center" }, meta);
          if (done) {
            const s = pop(since, 0.2, 400);
            text("Complete", q.completeLabel ?? "Complete", { step: ts(14), font: "display", color: lit(role("success")), onScene: true, nowrap: true, outline: 1 }, { transform: { scaleX: s, scaleY: s, pivotX: 0 } });
          }
          // Open steps first when the list is cut, so what is left to do shows.
          const all = q.steps ?? [];
          const cap = desc.maxSteps || (fold > 2 ? 1 : fold > 1 || (f.phone && textScale() >= 1.5) ? 2 : 0);
          const shown = cap > 0 && all.length > cap ? [...all.filter((st) => !st.done && !st.failed), ...all.filter((st) => st.done || st.failed)].slice(0, cap) : all;
          for (const st of all) if (shown.includes(st)) step(keyId(st.key ?? st.text ?? ""), st, tk);
          if (shown.length < all.length) text("More", `+${all.length - shown.length} more`, { step: ts(12), color: token("inkMuted"), onScene: true, nowrap: true, outline: 0.8 });
        }, { opacity: 1 - leave, a11y: a11y("group", q.title ?? "") });
      }
    }, { a11y: a11y("list", "Objectives") });
  });
}

function step(id: string, st: Api.ObjectiveStep, tk: ReturnType<typeof frame>["t"]): void {
  const state = st.failed ? "failed" : st.done ? "done" : "open";
  const e = edge(`${id}~done`, st.done ? 1 : 0);
  sound("success", e.changed && !!st.done && e.prev === 0);
  const c = edge(`${id}~count`, st.count ?? 0);
  const s = st.done && e.prev === 0 ? pop(e.since, 0.3, 360) : 1;
  const cs = !st.done && c.prev < (st.count ?? 0) ? pop(c.since, 0.25, 300) : 1;
  const ink = st.failed ? lit(role("danger")) : st.done ? token("inkFaint") : st.optional ? token("inkMuted") : token("ink");
  kit.column(id, { gap: u(3), width: pct(1) }, () => {
    kit.row("Row", { gap: tk.sm, cross: "center", width: pct(1) }, () => {
      const box0 = u(13);
      box("Check", { width: px(box0), height: px(box0), align: "center", cross: "center", part: sel("ObjectiveTracker", "check", st.done ? "done" : st.failed ? "failed" : null) }, {
        shape: { kind: "rect", radii: [2, 2, 2, 2] },
        fills: [solid(st.done ? lit(role("success")) : st.failed ? lit(role("danger"), 0.3) : token("shadow", 0.4))],
        strokes: [stroke(1.5, st.done ? lit(role("success")) : st.failed ? lit(role("danger")) : token("ink", 0.8))],
      }, () => {
        if (st.done) {
          leaf("Tick", { width: px(box0), height: px(box0) }, {
            shape: { kind: "path", points: [{ x: box0 * 0.22, y: box0 * 0.52 }, { x: box0 * 0.42, y: box0 * 0.72 }, { x: box0 * 0.78, y: box0 * 0.3 }], width: 2, cap: "round", join: "round" },
            fills: [solid(token("shadow", 0.95))],
          }, absolute({ start: 0, top: 0 }));
        } else if (st.failed) {
          leaf("Cross", { width: px(box0), height: px(box0) }, {
            shape: { kind: "path", points: [{ x: box0 * 0.25, y: box0 * 0.25 }, { x: box0 * 0.75, y: box0 * 0.75 }], width: 2, cap: "round" },
            fills: [solid(lit(role("danger")))],
          }, absolute({ start: 0, top: 0 }));
        }
      }, { transform: { scaleX: s, scaleY: s } });
      text("Text", st.optional ? `${isolate(st.text ?? "")} (Optional)` : (st.text ?? ""), { step: ts(13), color: ink, onScene: true, width: { kind: "grow" }, maxLines: 2, ellipsis: true, outline: 0.8 });
      if ((st.target ?? 0) > 0) {
        text("Count", `${Math.min(st.count ?? 0, st.target ?? 0)}/${st.target}`, { step: ts(13), font: "mono", color: st.done ? token("inkFaint") : token("ink"), onScene: true, nowrap: true, outline: 0.8 },
          { transform: { scaleX: cs, scaleY: cs, pivotX: 1 } });
      }
    });
    if (st.progress !== null && st.progress !== undefined && !st.done) {
      box("Progress", { width: pct(1), height: px(u(4)), part: sel("ObjectiveTracker", "progress") }, trackDraw(2), () => {
        leaf("Fill", { width: pct(clamp01(spring01(st.progress ?? 0))), height: pct(1) }, fillDraw(role("gold"), 2));
      });
    }
  });
  apply({
    // A list item, not a checkbox: the tracker reports, a tap does nothing, so no role promises a control (a
    // checkbox is a touch target the checks hold to 44 px). Its state is in its value.
    a11y: a11y("listitem", st.text ?? "", { value: [st.failed ? "failed" : st.done ? "done" : "", (st.target ?? 0) > 0 ? `${st.count ?? 0} of ${st.target}` : ""].filter(Boolean).join(", ") }),
    ...groups({ step: state }, { step: ["open", "done", "failed"] }),
  });
}

/** The progress on the ui spring (under the open element). */
function spring01(v: number): number {
  return kit.animate("progress", clamp01(v), { kind: "token", token: "ui" });
}
