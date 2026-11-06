import type { ComboDesc, JudgementDesc } from "../types.ts";
// og.ui.scorebug, og.ui.comboCounter and og.ui.judgement. Read against EA
// FC's and NBA 2K's broadcast scorebugs, Devil May Cry's and Hi-Fi Rush's
// combo counters and the judgements of osu!, Project Sekai and Beat Saber.
// The scoreboard, resource bar and turn order moved into the shooter,
// strategy and RPG kits (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { luminance, a11y, absolute, anchorOf, box, chip, close, fitLevel, hudRoom, open, clamp01, countdown, easeBack, easeOut, edge, ellipseShape, fillDraw, formatNumber, frame, glow, groups, hudPlace, icon, inkOn, keyId, kit, leaf, lit, mix, msToTicks, now, paintRgba, pct, pop, pulse, px, reduced, role, sel, shapeOf, solid, sound, spring, stroke, teamColor, text, token, trackDraw, ts, u, vgrad, type Rgba, apply, type Props, ut, tsFit, stepSize, textScale } from "@origingame/engine/ui-parts/hud";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

/**
 * The bright end of a HUD readout's ink: white over a dark HUD ground, the
 * pack's ink over a light one (a light pack's `hud` plate, where white and a
 * white-mixed gradient lose their contrast).
 */
function hudLight(): Rgba {
  return luminance(role("hud")) > 0.36 ? role("ink") : WHITE;
}

const MILESTONES = [10, 25, 50, 100];

export function comboCounter(id: string, desc: ComboDesc = {}): void {
  const count = desc.count ?? 0;
  const t = now();
  let comboProps: Props = {};
  // The whole counter (its growth room, the count, its label and its decay bar) keeps to about a third of the
  // screen's short side, as the tach does: at 200 % on a sideways phone the count takes a smaller step.
  const vp = frame().viewport;
  const short = Math.min(vp.width, vp.height);
  const want = stepSize(ts(48, true));
  const underLine = stepSize(ts(14)).line + (desc.decay !== null && desc.decay !== undefined ? u(4) + u(4) : 0);
  // Its share of the short side grows half as fast as the text (as a slot does), so at 150 % on a phone the count
  // still reaches its step (a fixed third held 48 px to 54).
  const ks = Math.min(2, textScale());
  const countMax = Math.max(stepSize(ts(14)).size, ((short * 0.34 * (1 + (ks - 1) / 2) - underLine) / 1.45) * (want.size / want.line));
  const countStep = tsFit(48, countMax, true);
  const cl = stepSize(countStep).line;
  hudPlace(id, anchorOf(desc, "right"), () => {
    // The count grows up from its baseline (its pop and its growth with the count), never down onto its label: room
    // above it for the growth, and a gap under it for its tilt.
    box(id, { direction: "column", gap: Math.ceil(cl * 0.1), cross: "center", padding: { left: 0, right: 0, top: Math.round(cl * 0.35), bottom: 0 } }, undefined, () => {
      const e = edge("count", count);
      // A break: the last count shakes and falls off.
      const brokeAt = kit.memory("brokeAt", -1e9);
      const lastLive = kit.memory("last", 0);
      if (e.changed && count < 2 && e.prev >= 2) {
        kit.remember("brokeAt", t);
        kit.remember("last", e.prev);
      }
      const ms = (desc.milestones ?? []).length > 0 ? desc.milestones! : MILESTONES;
      const hitMilestone = e.changed && count > e.prev && (ms.includes(count) || (count > 100 && count % 100 === 0));
      if (hitMilestone) kit.remember("mileAt", t);
      sound("reward", hitMilestone);
      const mileAt = kit.memory("mileAt", -1e9);
      const mile = t - mileAt < msToTicks(700);
      const broken = count < 2 && t - brokeAt < msToTicks(600);
      const state = broken ? "broken" : count < 2 ? "idle" : mile ? "milestone" : "live";
      if (count >= 2 || broken) {
        const shown = broken ? lastLive : count;
        const grow = 1 + Math.min(0.35, Math.log10(Math.max(1, shown)) * 0.15);
        const s = broken ? 1 : (mile ? pop(t - mileAt, 0.45, 520) : pop(e.since, 0.22, 260)) * grow;
        const bx = broken ? (t - brokeAt) / msToTicks(600) : 0;
        // Broken, the count goes red and the word says so: the break reads in
        // a still frame, not only in the shake.
        const ink = broken ? role("danger") : mile ? role("gold") : shown >= 50 ? role("danger") : shown >= 25 ? role("warn") : hudLight();
        // Late in the fall the whole group drops and fades together.
        const fall = broken ? { transform: { y: reduced() ? 0 : Math.max(0, bx - 0.5) * 2 * u(24) }, opacity: Math.min(1, 2 - 2 * bx) } : {};
        text("Count", String(shown), { step: countStep, font: "display", color: lit(ink), nowrap: true, outline: 1.6 }, {
          transform: broken ? { x: reduced() ? 0 : Math.sin(bx * 40) * u(6) * (1 - bx), y: reduced() ? 0 : bx * bx * u(40), rotate: reduced() ? 0 : bx * 0.3 } : { scaleX: s, scaleY: s, rotate: reduced() ? 0 : -0.06, pivotY: 1 },
          opacity: broken ? 1 - bx * bx : 1,
        });
        box("Under", { direction: "row", gap: u(6), cross: "center" }, undefined, () => {
          text("Label", broken ? desc.breakLabel || "Break" : desc.label ?? "Combo", { step: ts(14), font: "display", color: broken ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1 });
          if ((desc.multiplier ?? 0) > 1 && !broken) {
            box("Multiplier", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, part: sel("Combo", "multiplier") }, {
              shape: { kind: "rect", radii: [999, 999, 999, 999] },
              fills: [solid(lit(role("gold")))],
            }, () => text("Text", `×${formatNumber(desc.multiplier ?? 1, { decimals: Number.isInteger(desc.multiplier) ? 0 : 1 })}`, { step: ts(13), font: "display", color: lit(inkOn(role("gold"))), nowrap: true }));
          }
        }, fall);
        if (desc.decay !== null && desc.decay !== undefined && !broken) {
          box("Decay", { width: px(u(96)), height: px(u(4)), part: sel("Combo", "decay") }, trackDraw(2), () => {
            leaf("Fill", { width: pct(clamp01(desc.decay ?? 0)), height: pct(1) }, fillDraw(clamp01(desc.decay ?? 0) < 0.25 ? role("danger") : ink, 2));
          });
        }
        if (mile && !reduced()) {
          const x = (t - mileAt) / msToTicks(700);
          leaf("Burst", { width: px(u(140)), height: px(u(140)), part: sel("Combo", "burst") }, {
            shape: { kind: "polygon", sides: 12, inner: 0.55 },
            fills: [solid(lit(role("gold"), 0.5 * (1 - x)))],
          }, absolute({ centerX: true, top: -u(30) }, { transform: { scaleX: 0.5 + easeOut(x), scaleY: 0.5 + easeOut(x), rotate: x * 0.8 }, z: -1 }));
        }
      }
      comboProps = {
        a11y: a11y("status", desc.label ?? "Combo", { value: count >= 2 ? String(count) : "" }),
        ...groups({ combo: state }, { combo: ["idle", "live", "milestone", "broken"] }),
      };
    });
    apply(comboProps);
  });
}

function tierInk(tier: number): Rgba {
  return [role("gold"), role("success"), role("info"), role("warn")][tier] ?? role("danger");
}

export function judgement(id: string, desc: JudgementDesc = {}): void {
  const t = now();
  const ev = desc.last ?? null;
  let judgeProps: Props = {};
  hudPlace(id, anchorOf(desc, "center"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      const age = ev ? t - (ev.at ?? 0) : 1e9;
      const life = msToTicks(600);
      const tier = ev?.tier ?? 0;
      const miss = tier >= 4;
      const state = !ev || age >= life || age < 0 ? "none" : tier === 0 ? "best" : miss ? "miss" : "good";
      if (state !== "none" && ev) {
        const x = age / life;
        const ink = tierInk(tier);
        const s = reduced() ? 1 : x < 0.15 ? 1.35 - 0.35 * easeBack(x / 0.15) : 1;
        const y = reduced() ? 0 : miss ? u(18) * easeOut(x) : -u(10) * easeOut(x);
        const grad: Api.Gradient | undefined = tier === 0 ? {
          type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 },
          stops: [{ offset: 0, color: lit(hudLight() === WHITE ? mix(ink, WHITE, 0.6) : ink) }, { offset: 0.55, color: lit(ink) }, { offset: 1, color: lit(mix(ink, { r: 0.6, g: 0.3, b: 0, a: 1 }, 0.5)) }],
        } : undefined;
        text("Word", ev.text ?? "", { step: ts(30, true), font: "display", color: lit(miss ? mix(ink, { r: 0.5, g: 0.5, b: 0.5, a: 1 }, 0.4) : ink), nowrap: true, outline: 1.4, ...(grad ? { gradient: grad } : {}) }, {
          transform: { scaleX: s, scaleY: s, y },
          opacity: x < 0.7 ? 1 : 1 - (x - 0.7) / 0.3,
        });
        const off = ev.offsetMs ?? 0;
        if (off !== 0 && !miss) {
          text("Timing", `${off < 0 ? desc.earlyLabel ?? "Early" : desc.lateLabel ?? "Late"} ${Math.abs(Math.round(off))}ms`, {
            step: ts(12), font: "mono", color: lit(off < 0 ? role("info") : role("warn")), nowrap: true, outline: 1,
          }, { transform: { y }, opacity: x < 0.7 ? 1 : 1 - (x - 0.7) / 0.3 });
        }
      }
      judgeProps = {
        a11y: a11y("status", state === "none" ? "" : ev?.text ?? ""),
        ...groups({ judge: state }, { judge: ["none", "best", "good", "miss"] }),
      };
    });
    apply(judgeProps);
  }, { offsetY: desc.offsetY ?? 0 });
}


