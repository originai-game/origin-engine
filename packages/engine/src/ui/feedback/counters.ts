// og.ui.numberCounter and og.ui.formatNumber: numbers that count (docs/ui.md
// §9 formatting, Appendix E "Count-ups, rolling digits, escalating
// cascades"). Read against Cookie Clicker's and Melvor Idle's numbers past
// trillions (letter notation that never runs out), Balatro's chip count
// and the odometers of racing and casino games: a change counts on an ease
// out, each digit rolls into the next, a rise throws its +n off the number,
// and the number pops when it lands.

import type * as Api from "@origingame/engine";
import { a11y, absolute, box, clamp01, close, easeOut, frame, formatNumber, groups, icon, kit, lit, mix, msToTicks, now, open, pop, px, reduced, role, sel, solid, stepSize, text, token, xy, ts, apply } from "./core.ts";

export { formatNumber };

export function numberCounter(id: string, value: number, desc: Api.NumberCounterDesc = {}): number {
  let shown = value;
  if (desc.anchor) {
    const f = frame();
    kit.anchor(id, { anchor: desc.anchor, direction: "row" }, () => {
      shown = counter("Counter", value, desc, true, f.t.control);
    });
  } else shown = counter(id, value, desc, false, 0);
  return shown;
}

function counter(id: string, value: number, desc: Api.NumberCounterDesc, hud: boolean, radius: number): number {
  const f = frame();
  const tk = f.t;
  const format: Api.NumberFormatDesc = desc.format ?? {};
  const step: Api.TypeStepName = desc.step ?? ts(26);
  const ink = desc.color ?? token("ink");
  const t = now();
  let shown = value;
  const path = open(id, {
    direction: "row",
    gap: tk.sm,
    cross: "center",
    padding: hud ? xy(tk.md, tk.xs) : { left: 0, top: 0, right: 0, bottom: 0 },
    part: sel("NumberCounter", "frame", hud ? "hud" : null),
    partClass: "hud",
  }, hud ? {
    shape: { kind: "rect", radii: [radius, radius, radius, radius] },
    fills: [solid(token("hud"))],
  } : undefined);
  // Where the count runs from and since when: the UI's memory.
  const target = kit.memory("target", value);
  let from = kit.memory("from", value);
  let start = kit.memory("start", -1e9);
  const was = kit.memory("shown", value);
  if (target !== value) {
    from = was;
    start = t;
    kit.remember("target", value);
    kit.remember("from", from);
    kit.remember("start", start);
    if (value > target && desc.delta !== false) {
      kit.remember("delta", value - target);
      kit.remember("deltaAt", t);
    }
    if (value < target) kit.remember("fellAt", t);
  }
  const span = Math.abs(value - from);
  const big = Math.abs(value) > 0 && span / Math.abs(value) > 0.1;
  const dur = msToTicks(desc.durationMs || (big ? 1200 : 600));
  const x = reduced() ? 1 : clamp01((t - start) / dur);
  shown = x >= 1 ? value : from + (value - from) * easeOut(x);
  kit.remember("shown", shown);
  const counting = x < 1;
  if (desc.ticks && counting && (t - start) % 3 === 0) kit.feedback("counterTick", Math.floor(x * 8));
  const landed = t - (start + dur);
  const scale = landed >= 0 ? pop(landed, 0.12, 300) : 1;
  const fellAt = kit.memory("fellAt", -1e9);
  const fall = clamp01(1 - (t - fellAt) / msToTicks(500));
  const inkNow = fall > 0 && ink.token === "ink" ? lit(mix(role("ink"), role("danger"), fall)) : ink;

  if (desc.icon) icon("Icon", desc.icon, Math.round(stepSize(step).size * 0.8), desc.icon === "coin" ? token("gold") : token("ink"));
  kit.column("Body", { gap: 0, cross: desc.align === "end" ? "end" : desc.align === "center" ? "center" : "start" }, () => {
    const roll = desc.roll !== false && counting && (format.notation ?? "grouped") === "grouped" && (format.exponent ?? 0) === 0 && Math.abs(value) < 1e15 && Math.abs(from) < 1e15;
    if (roll) digits("Value", shown, format, step, desc.font ?? "mono", inkNow, hud);
    else text("Value", formatNumber(shown, format), { step, font: desc.font ?? "mono", color: inkNow, nowrap: true, outline: hud ? 1 : false });
    apply({ transform: { scaleX: scale, scaleY: scale, pivotX: desc.align === "end" ? 1 : desc.align === "center" ? 0.5 : 0 } });
    if (desc.label) text("Label", desc.label, { step: ts(13), color: token("inkMuted"), nowrap: true, outline: hud ? 0.7 : false });
  });
  // The +n of a rise flies up off the number and fades.
  const dAt = kit.memory("deltaAt", -1e9);
  const dx = (t - dAt) / msToTicks(900);
  if (dx >= 0 && dx < 1) {
    const d = kit.memory("delta", 0);
    text("Delta", formatNumber(d, { ...format, sign: true }), { step: ts(18), font: "mono", nowrap: true, color: token("success"), outline: 1 }, absolute({ end: 0, top: -8 }, {
      transform: { y: reduced() ? 0 : -28 * easeOut(dx) },
      opacity: 1 - dx * dx,
      z: 2,
      overlaps: [path],
    }));
  }
  close({
    a11y: a11y("text", desc.label || "", { value: formatNumber(value, format) }),
    ...groups({ change: !counting ? "still" : value >= from ? "rising" : "falling" }, { change: ["still", "rising", "falling"] }),
  });
  return shown;
}

/**
 * The number with each digit in a clipped cell that rolls to its next
 * digit as the count passes it, the way an odometer's wheels do: a wheel
 * turns only while every wheel to its right is passing 9.
 */
function digits(id: string, shown: number, format: Api.NumberFormatDesc, step: Api.TypeStepName, font: Api.FontRole, ink: Api.Paint, hud: boolean): void {
  const line = stepSize(step).line;
  const textNow = formatNumber(Math.trunc(shown), { ...format, decimals: 0 });
  const whole = Math.abs(shown);
  // Wheel k (units 0) shows floor(whole / 10^k) mod 10, turning by the
  // fraction the lower wheels carry into it.
  const frac = whole - Math.floor(whole);
  let k = textNow.replace(/[^0-9]/g, "").length - 1;
  kit.row(id, { gap: 0, cross: "center" }, () => {
    let i = 0;
    for (const ch of textNow) {
      if (ch < "0" || ch > "9") {
        text(kit.idi("Sep", i++), ch, { step, font, color: ink, nowrap: true, outline: hud ? 1 : false });
        continue;
      }
      const place = 10 ** k;
      const below = Math.floor(whole) % place;
      const carry = below === place - 1 ? frac : 0;
      const d = Number(ch);
      box(kit.idi("Digit", i++), { direction: "column", height: px(line), part: sel("NumberCounter", "digit") }, { clip: "shape" }, () => {
        kit.column("Wheel", { gap: 0 }, () => {
          text("D", String(d), { step, font, color: ink, nowrap: true, outline: hud ? 1 : false });
          text("N", String((d + 1) % 10), { step, font, color: ink, nowrap: true, outline: hud ? 1 : false });
        });
        apply({ transform: { y: -carry * line } });
      });
      k -= 1;
    }
  });
}

