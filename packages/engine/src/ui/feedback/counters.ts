// og.ui.numberCounter and og.ui.formatNumber: numbers that count (docs/ui.md
// §6 formatting, Appendix E "Count-ups, rolling digits, escalating
// cascades"). Read against Cookie Clicker's and Melvor Idle's numbers past
// trillions (letter notation that never runs out), Balatro's chip count
// and the odometers of racing and casino games: a change counts on an ease
// out, each digit rolls into the next, a rise throws its +n off the number,
// and the number pops when it lands.

import type * as Api from "@origingame/engine";
import { a11y, absolute, box, clamp01, close, easeOut, frame, formatNumber, groups, icon, kit, lit, mix, msToTicks, now, open, pop, px, reduced, role, sel, solid, stepSize, text, token, xy, ts, apply, fitWidth } from "./core.ts";

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
  let step: Api.TypeStepName = desc.step ?? ts(26);
  const ink = desc.color ?? token("ink");
  const t = now();
  let shown = value;
  // At most as wide as what holds it (last tick): the +n of a rise folds
  // under the number when there is no room beside it. A holder sized by
  // the counter alone (a HUD anchor around it) sets no limit, or the count
  // could never grow a digit.
  const holder = kit.openPath();
  const held = holder ? kit.bounds(holder)?.width ?? 0 : 0;
  const self = holder ? kit.bounds(`${holder}/${id}`)?.width ?? 0 : 0;
  // Once capped it stays capped: a capped counter is as wide as its
  // holder, which would otherwise read as the anchor case on the next tick.
  const capKey = `${id}.capped`;
  const room = held > 0 && (Math.abs(held - self) > 1 || kit.memory(capKey, 0) > 0) ? held : 0;
  kit.remember(capKey, room > 0 ? 1 : 0);
  const path = open(id, {
    direction: "row",
    gap: tk.sm,
    cross: "center",
    wrap: room > 0,
    width: room > 0 ? { kind: "fit", max: room } : null,
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

  // The box is as wide as the wider end of the count (an invisible copy
  // sizes it), so a count that grows a digit never outgrows its place, and
  // it keeps room on the side the landing pop grows towards. A count wider
  // than what holds it (large text, a long number) steps its numerals down
  // until it fits: digits never wrap.
  const ends = [formatNumber(value, format), formatNumber(counting ? from : value, format)];
  const widest = ends[0]!.length >= ends[1]!.length ? ends[0]! : ends[1]!;
  // The icon (0.8 em and a gap) shares the line with the numerals.
  if (room > 0) step = fitWidth(step, widest.length * 0.62 * 1.12 + (desc.icon ? 0.8 : 0), room - (desc.icon ? tk.sm : 0));
  if (desc.icon) icon("Icon", desc.icon, Math.round(stepSize(step).size * 0.8), desc.icon === "coin" ? token("gold") : token("ink"));
  const popRoom = Math.ceil(widest.length * stepSize(step).size * 0.62 * 0.12);
  const popPad = desc.align === "end" ? { left: popRoom, right: 0 } : desc.align === "center" ? { left: Math.ceil(popRoom / 2), right: Math.ceil(popRoom / 2) } : { left: 0, right: popRoom };
  kit.column("Body", { gap: 0, cross: desc.align === "end" ? "end" : desc.align === "center" ? "center" : "start", padding: { top: 0, bottom: 0, ...popPad } }, () => {
    const roll = desc.roll !== false && counting && (format.notation ?? "grouped") === "grouped" && (format.exponent ?? 0) === 0 && Math.abs(value) < 1e15 && Math.abs(from) < 1e15;
    text("Sizer", widest, { step, font: desc.font ?? "mono", nowrap: true, outline: 1 }, { opacity: 0, inert: true });
    const at = desc.align === "end" ? { end: 0, top: 0 } : desc.align === "center" ? { centerX: true, top: 0 } : { start: 0, top: 0 };
    if (roll) digits("Value", shown, format, step, desc.font ?? "mono", inkNow, hud);
    else text("Value", formatNumber(shown, format), { step, font: desc.font ?? "mono", color: inkNow, nowrap: true, outline: 1 });
    apply(absolute(at, { transform: { scaleX: scale, scaleY: scale, pivotX: desc.align === "end" ? 1 : desc.align === "center" ? 0.5 : 0 } }));
    // Over a scene as often as on a card: outlined like the progress labels, so it reads at 4.5:1 on either.
    if (desc.label) text("Label", desc.label, { step: ts(13), color: token("ink", 0.88), nowrap: true, outline: 0.9 });
  });
  // The +n of a rise, in its own place beside the number: it rises a little and fades.
  const dAt = kit.memory("deltaAt", -1e9);
  const dx = (t - dAt) / msToTicks(900);
  if (dx >= 0 && dx < 1) {
    const d = kit.memory("delta", 0);
    text("Delta", formatNumber(d, { ...format, sign: true }), { step: ts(18), font: "mono", nowrap: true, color: token("success"), outline: 1 }, {
      transform: { y: reduced() ? 0 : -10 * easeOut(dx) },
      opacity: 1 - dx * dx,
    });
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
        text(kit.idi("Sep", i++), ch, { step, font, color: ink, nowrap: true, outline: 1 });
        continue;
      }
      const place = 10 ** k;
      const below = Math.floor(whole) % place;
      const carry = below === place - 1 ? frac : 0;
      const d = Number(ch);
      // The wheel shows through a window the height of a numeral's caps
      // (and its outline), centred on the line: a digit in motion is never
      // seen above the others' tops or below their baseline. The next digit
      // sits one window below the current one, so they roll edge to edge.
      const win = Math.min(line, Math.ceil(stepSize(step).size * 0.8));
      const top = Math.floor((line - win) / 2);
      // Placed by layout (padding and insets), not transforms: a clip holds what layout puts in it.
      box(kit.idi("Digit", i++), { direction: "column", height: px(line), padding: { left: 0, right: 0, top, bottom: 0 }, part: sel("NumberCounter", "digit") }, undefined, () => {
        box("Window", { direction: "column", height: px(win) }, { clip: "shape" }, () => {
          // An invisible numeral gives the window its width.
          text("W", String(d), { step, font, nowrap: true, outline: 1 }, { opacity: 0, inert: true });
          text("D", String(d), { step, font, color: ink, nowrap: true, outline: 1 }, absolute({ start: 0, top: -top - carry * win }));
          text("N", String((d + 1) % 10), { step, font, color: ink, nowrap: true, outline: 1 }, absolute({ start: 0, top: -top + (1 - carry) * win }));
        });
      });
      k -= 1;
    }
  });
}

