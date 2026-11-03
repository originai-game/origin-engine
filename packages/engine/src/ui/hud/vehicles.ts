// og.ui.speedometer and og.ui.flightHud: Appendix C's "speedometer" and
// "flight ladder". Read against Forza Horizon's and Gran Turismo's dials (a
// 270° sweep of ticks, the needle on a spring, the tach's red zone and a
// shift light, the speed in large tabular digits with the gear on a badge)
// and the head-up displays of Ace Combat and DCS (one phosphor ink with a
// dark edge, a pitch ladder of rungs every 5° dashed below the horizon and
// turning with roll, a flight path marker, a heading tape across the top,
// speed and altitude tapes with boxed readouts at either side).

import type * as Api from "@origingame/engine";
import {
  TAU, a11y, fixed, anchorOf, box, bp, clamp01, ellipseShape, frame, glow, groups, hudPlace, kit, leaf, lit, mix, paintRgba, pct, pulse, px,
  reduced, role, sel, solid, springTo, stroke, text, token, ts, u, vgrad, type Rgba, ut, tsFit,
  stepSize, textScale,
} from "./core.ts";
import { arcadeSpeedo } from "./arcade.ts";

const START = (3 * Math.PI) / 4;
const SWEEP = (3 * Math.PI) / 2;

/** A nice tick step for a dial to `max`: about ten numbered ticks. */
function niceStep(max: number): number {
  const raw = max / 10;
  const p = 10 ** Math.floor(Math.log10(raw));
  for (const m of [1, 2, 2.5, 5, 10]) if (m * p >= raw) return m * p;
  return 10 * p;
}

export function speedometer(id: string, desc: Api.SpeedometerDesc = {}): void {
  // The dial grows with the player's text size (to 1.4×), so its numerals keep their spacing at 200 %.
  let size = Math.round(u(desc.size || (bp().phone ? 132 : 180)) * Math.min(1.4, textScale()));
  const max = Math.max(1, desc.maxSpeed ?? 300);
  const speed = Math.max(0, desc.speed ?? 0);
  const rpm = clamp01(desc.rpm ?? 0);
  const red = clamp01(desc.redline ?? 0.85);
  const over = rpm >= red;
  const style: Api.SpeedoStyle = desc.style ?? "dial";
  const unit = desc.unit ?? "km/h";
  if (style === "tach") {
    // A tach never takes more than a third of the screen's short side.
    const v = frame().viewport;
    size = Math.min(size, Math.round(0.34 * Math.min(v.width, v.height)));
  }
  hudPlace(id, anchorOf(desc, "bottomRight"), () => {
    if (style === "digital") digital(id, speed, rpm, red, unit, desc, size);
    else if (style === "arcade") arcadeSpeedo(id, speed, unit, desc, size);
    else if (style === "tach") tach(id, speed, rpm, red, unit, desc, size);
    else {
      box(id, { width: px(size), height: px(size), part: sel("Speedometer", "dial") }, {
        shape: ellipseShape(),
        fills: [vgrad([[0, token("hud", 0.55)], [1, token("hud", 0.9)]])],
        strokes: [stroke(Math.max(2, size / 60), token("lineStrong", 0.8)), stroke(1, token("shadow", 0.9), "outside")],
        effects: [{ kind: "dropShadow", offset: { x: 0, y: 3 }, blur: 12, spread: 0, color: token("shadow", 0.55) }],
      }, () => {
        const c = size / 2;
        const R = c - Math.max(3, size / 30);
        // The tach: an inner band filling to the revs, red past the redline.
        const band = Math.max(4, Math.round(size / 26));
        const tachD = (R - size * 0.02) * 2;
        const to = (R - size * 0.02);
        leaf("TachTrack", { width: px(tachD), height: px(tachD), part: sel("Speedometer", "tach") }, {
          shape: { kind: "arc", start: START, sweep: SWEEP, thickness: band, cap: "butt" },
          fills: [solid(token("shadow", 0.6))],
        }, fixed({ start: c - to, top: c - to }));
        leaf("TachRed", { width: px(tachD), height: px(tachD), part: sel("Speedometer", "tach", "red") }, {
          shape: { kind: "arc", start: START + SWEEP * red, sweep: SWEEP * (1 - red), thickness: band, cap: "butt" },
          fills: [solid(lit(role("danger"), 0.45))],
        }, fixed({ start: c - to, top: c - to }));
        const shownRpm = reduced() ? rpm : clamp01(springTo("rpm", rpm, 140));
        if (shownRpm > 0) {
          leaf("Tach", { width: px(tachD), height: px(tachD), part: sel("Speedometer", "tach", over ? "red" : null) }, {
            shape: { kind: "arc", start: START, sweep: SWEEP * shownRpm, thickness: band, cap: "butt" },
            fills: [solid(lit(over ? role("danger") : mix(role("info"), role("warn"), shownRpm)))],
            effects: over ? [glow(6, lit(role("danger"), 0.6))] : [],
          }, fixed({ start: c - to, top: c - to }));
        }
        // Ticks and numbers.
        // At a large text size the dial grows less than its text: the numerals ring is capped smaller and pushed out,
        // and the readout dropped a little, so the readout sits clear under the hub and of the numerals.
        const big = textScale() >= 1.5;
        const speedStep = tsFit(size >= 150 ? 30 : 22, size * 0.15, true);
        const unitStep = tsFit(10, size * 0.065);
        const readoutHalf = Math.max(String(Math.round(max)).length * stepSize(speedStep).size * 0.62, unit.length * stepSize(unitStep).size * 0.62 + size * 0.16) / 2 + size * 0.02;
        const readoutTop = c + size * (big ? 0.07 : 0.05);
        const step = niceStep(max);
        const n = Math.floor(max / step + 1e-6);
        // Numerals placed so far: one that would touch its neighbour (the dial's top at a large text size) is left out.
        const placedNums: Array<[number, number, number, number]> = [];
        for (let i = 0; i <= n * 2; i++) {
          const v = (i * step) / 2;
          if (v > max + 1e-6) break;
          const major = i % 2 === 0;
          const a = START + (v / max) * SWEEP;
          const r0 = R - band - (major ? size * 0.075 : size * 0.04);
          const r1 = R - band - size * 0.012;
          leaf(kit.idi("Tick", i), { width: px(size), height: px(size), part: sel("Speedometer", "tick", major ? "major" : null) }, {
            shape: { kind: "path", points: [{ x: c + Math.cos(a) * r0, y: c + Math.sin(a) * r0 }, { x: c + Math.cos(a) * r1, y: c + Math.sin(a) * r1 }], width: major ? 2 : 1, cap: "butt" },
            fills: [solid(major ? token("ink") : token("inkMuted"))],
          }, fixed({ start: 0, top: 0 }));
          const rt = r0 - size * (big ? 0.065 : 0.08);
          const lw = size * 0.16;
          const numStep = tsFit(10, size * (big ? 0.058 : 0.075));
          const nx = c + Math.cos(a) * rt;
          const ny = c + Math.sin(a) * rt - size * 0.035;
          // A numeral that would reach the readout under the hub (large text, the dial's lowest marks) is left out.
          const numW = String(Math.round(v)).length * stepSize(numStep).size * 0.62;
          // Against the readout's own top and the numeral's laid-out box, as the overlap check measures them.
          const hitsReadout = ny + stepSize(numStep).line > readoutTop && Math.abs(nx - c) - Math.max(numW, lw) / 2 < readoutHalf;
          // The box it is laid out in (lw), so two placed numerals never overlap as the check measures them.
          const half = Math.max(numW / 2 + 2, lw / 2);
          const nb: [number, number, number, number] = [nx - half, nx + half, ny, ny + stepSize(numStep).line];
          const crowded = placedNums.some((q) => nb[0] < q[1] && nb[1] > q[0] && nb[2] < q[3] && nb[3] > q[2]);
          if (major && size >= 120 && !hitsReadout && !crowded) {
            placedNums.push(nb);
            text(kit.idi("Num", i), String(Math.round(v)), { step: numStep, font: "mono", color: token("inkMuted"), nowrap: true, width: px(lw), align: "center" },
              fixed({ start: nx - lw / 2, top: ny }));
          }
        }
        // The needle on a spring with a little bounce, so it reads as mechanical.
        const frac = clamp01(speed / max);
        const nf = reduced() ? frac : springTo("needle", frac, 220, 0.12);
        const a = START + nf * SWEEP;
        const nr = R - band - size * 0.02;
        leaf("Needle", { width: px(size), height: px(size), part: sel("Speedometer", "needle") }, {
          shape: { kind: "path", points: [{ x: c - Math.cos(a) * size * 0.06, y: c - Math.sin(a) * size * 0.06 }, { x: c + Math.cos(a) * nr, y: c + Math.sin(a) * nr }], width: Math.max(2, size / 60), cap: "round" },
          fills: [solid(lit(role("danger")))],
          strokes: [stroke(1, token("shadow", 0.7), "outside")],
          effects: [glow(4, lit(role("danger"), 0.5))],
        }, fixed({ start: 0, top: 0 }));
        leaf("Hub", { width: px(size * 0.09), height: px(size * 0.09) }, {
          shape: ellipseShape(),
          fills: [solid(token("panelRaised"))],
          strokes: [stroke(1.5, token("lineStrong"))],
        }, fixed({ start: c - size * 0.045, top: c - size * 0.045 }));
        // Under the hub, one column that cannot collide with itself: the
        // speed, then its unit beside the gear.
        box("Readout", { direction: "column", cross: "center", gap: 0, width: pct(1) }, undefined, () => {
          text("Speed", String(Math.round(speed)), { step: speedStep, font: "display", color: token("ink"), nowrap: true, outline: 1 });
          kit.row("Meta", { gap: Math.round(size * 0.03), cross: "center" }, () => {
            text("Unit", unit, { step: unitStep, font: "mono", color: token("inkMuted"), nowrap: true });
            if (desc.gear) {
              // The box holds its numeral at any text size: it grows past its share of the dial rather than let the gear spill.
              const gearStep = tsFit(15, size * 0.075, true);
              const gearBox = Math.max(size * 0.13, stepSize(gearStep).line + 2, Math.ceil((desc.gear ?? "").length * stepSize(gearStep).size * 0.62) + 8);
              box("Gear", { width: px(gearBox), height: px(gearBox), align: "center", cross: "center", part: sel("Speedometer", "gear") }, {
                shape: { kind: "rect", radii: [3, 3, 3, 3] },
                fills: [solid(over ? lit(role("danger"), 0.4 + 0.6 * pulse(6)) : token("shadow", 0.7))],
                strokes: [stroke(1.5, over ? lit(role("danger")) : token("lineStrong"))],
              }, () => text("Text", desc.gear ?? "", { step: gearStep, font: "display", color: token("ink"), nowrap: true }));
            }
          });
        }, fixed({ start: 0, top: readoutTop }));
        if (over) {
          leaf("Shift", { width: px(size * 0.1), height: px(size * 0.1), part: sel("Speedometer", "shift") }, {
            shape: ellipseShape(),
            fills: [solid(lit(role("danger"), reduced() ? 1 : 0.35 + 0.65 * pulse(8)))],
            effects: [glow(8, lit(role("danger"), 0.8))],
          }, fixed({ centerX: true, top: size * 0.2 }));
        }
        if (desc.boost !== null && desc.boost !== undefined) {
          const bd = size * 0.32;
          leaf("BoostTrack", { width: px(bd), height: px(bd), part: sel("Speedometer", "boost") }, {
            shape: { kind: "arc", start: Math.PI * 0.75, sweep: Math.PI * 0.5, thickness: Math.max(3, band * 0.7), cap: "round" },
            fills: [solid(token("shadow", 0.6))],
          }, fixed({ start: c - bd / 2, top: c - bd / 2 + size * 0.02 }));
          leaf("Boost", { width: px(bd), height: px(bd) }, {
            shape: { kind: "arc", start: Math.PI * 0.75, sweep: Math.max(0.0001, Math.PI * 0.5 * clamp01(desc.boost)), thickness: Math.max(3, band * 0.7), cap: "round" },
            fills: [solid(lit(role("info")))],
          }, fixed({ start: c - bd / 2, top: c - bd / 2 + size * 0.02 }));
        }
      }, {
        a11y: a11y("meter", "Speed", { value: `${Math.round(speed)} ${unit}${desc.gear ? `, gear ${desc.gear}` : ""}`, valueMin: 0, valueMax: max, valueNow: speed }),
        ...groups({ rev: over ? "redline" : "normal" }, { rev: ["normal", "redline"] }),
      });
    }
  });
}

const TACH_START = (5 * Math.PI) / 6;
const TACH_SWEEP = (4 * Math.PI) / 3;

/**
 * The tach (the racing HUD brief): a 240° arc of the revs with its numbers
 * (thousands) outside it, so no number can reach the readout; the last
 * stretch from the redline in the danger role, the fill turning from the
 * accent to danger as it crosses; a row of shift lights over the readout
 * filling green, accent, danger and all flashing at the shift point; the
 * speed the largest numeral, condensed italic in a fixed three-digit box,
 * its unit under it; the gear a square badge beside it (N and R in warn);
 * boost a thin inner arc. At a large text size the numbers go before the
 * readout shrinks.
 */
function tach(id: string, speed: number, rpm: number, red: number, unit: string, desc: Api.SpeedometerDesc, size: number): void {
  const maxRpm = Math.max(1, desc.maxRpm || 8000);
  const rpmStep = Math.max(1, desc.rpmStep || 1000);
  const shiftAt = clamp01(desc.shiftAt || red);
  const over = rpm >= red;
  const shifting = rpm >= shiftAt;
  const c = size / 2;
  // Numbers at caption size round the outside; dropped where they would be
  // larger than a tenth of the dial (200 % text on a small dial).
  const numStep = tsFit(10, size * 0.1);
  const numLine = stepSize(numStep).line;
  const showNums = size >= 120 && stepSize(numStep).size <= size * 0.075 + 0.5;
  const ring = showNums ? numLine + Math.max(2, size * 0.01) : Math.max(2, size * 0.02);
  const Ro = c - ring;
  const band = Math.max(5, Math.round(size / 20));
  const accent = role("primary");
  const danger = role("danger");
  const arc = (rid: string, r: number, from: number, sweep: number, thick: number, paint: Api.Paint, part: string, effects: Api.Effect[] = []) =>
    leaf(rid, { width: px(r * 2), height: px(r * 2), part }, {
      shape: { kind: "arc", start: TACH_START + TACH_SWEEP * from, sweep: Math.max(0.0001, TACH_SWEEP * sweep), thickness: thick, cap: "butt" },
      fills: [solid(paint)],
      effects,
    }, fixed({ start: c - r, top: c - r }));
  box(id, { width: px(size), height: px(size), part: sel("Speedometer", "dial", "tach") }, {
    shape: ellipseShape(),
    fills: [vgrad([[0, token("hud", 0.5)], [1, token("hud", 0.88)]])],
    strokes: [stroke(1, token("shadow", 0.9), "outside")],
    effects: [{ kind: "dropShadow", offset: { x: 0, y: 3 }, blur: 12, spread: 0, color: token("shadow", 0.5) }],
  }, () => {
    arc("TachTrack", Ro, 0, 1, band, token("shadow", 0.65), sel("Speedometer", "tach"));
    arc("TachRed", Ro, red, 1 - red, band, lit(danger, 0.35), sel("Speedometer", "tach", "red"));
    const shown = reduced() ? rpm : clamp01(springTo("rpm", rpm, 120));
    if (shown > 0) arc("Tach", Ro, 0, Math.min(shown, red), band, lit(accent), sel("Speedometer", "tach"));
    if (shown > red) arc("TachOver", Ro, red, shown - red, band, lit(danger), sel("Speedometer", "tach", "red"), [glow(6, lit(danger, 0.6))]);
    // Ticks across the band every rpmStep (and half-steps), numbers outside.
    const n = Math.floor(maxRpm / rpmStep + 1e-6);
    for (let i = 0; i <= n * 2; i++) {
      const v = (i * rpmStep) / 2;
      if (v > maxRpm + 1e-6) break;
      const major = i % 2 === 0;
      const f = v / maxRpm;
      const a = TACH_START + f * TACH_SWEEP;
      const r0 = Ro - band - (major ? Math.max(3, size * 0.03) : 0);
      const r1 = major ? Ro : Ro - band * 0.45;
      leaf(kit.idi("Tick", i), { width: px(size), height: px(size), part: sel("Speedometer", "tick", major ? "major" : null) }, {
        shape: { kind: "path", points: [{ x: c + Math.cos(a) * r0, y: c + Math.sin(a) * r0 }, { x: c + Math.cos(a) * r1, y: c + Math.sin(a) * r1 }], width: major ? 2 : 1, cap: "butt" },
        fills: [solid(f >= red - 1e-6 ? lit(danger) : major ? token("ink") : token("inkMuted"))],
      }, fixed({ start: 0, top: 0 }));
      if (major && showNums) {
        const rt = Ro + ring / 2;
        const lw = numLine * 1.6;
        text(kit.idi("Num", i), String(Math.round(v / 1000)), { step: numStep, font: "mono", color: f >= red - 1e-6 ? lit(danger) : token("inkMuted"), nowrap: true, width: px(lw), align: "center" },
          fixed({ start: c + Math.cos(a) * rt - lw / 2, top: c + Math.sin(a) * rt - numLine / 2 }));
      }
    }
    // Boost: a thin arc inside the ticks.
    if (desc.boost !== null && desc.boost !== undefined) {
      const rb = Ro - band - Math.max(5, size * 0.05);
      const th = Math.max(2, Math.round(band * 0.4));
      arc("BoostTrack", rb, 0, 1, th, token("shadow", 0.6), sel("Speedometer", "boost"));
      if (desc.boost > 0) arc("Boost", rb, 0, clamp01(desc.boost), th, lit(role("info")), sel("Speedometer", "boost"));
    }
    // The readout: the speed, the largest numeral, in a fixed box of three
    // digits (condensed and slanted), its unit under it, the gear beside it.
    const speedStep = tsFit(40, size * 0.27, true);
    const sp = stepSize(speedStep);
    const unitStep = tsFit(10, size * 0.07);
    const digitsW = Math.ceil(3 * sp.size * 0.62 * 0.88);
    const gearStep = tsFit(12, sp.size * 0.45, true);
    const gearSide = Math.ceil(Math.max(stepSize(gearStep).line, (desc.gear ?? "").length * stepSize(gearStep).size * 0.62 + 6));
    const gap = Math.max(3, Math.round(size * 0.02));
    // The slant leans the speed's top right by a fifth of its line: the gear sits past that.
    const lean = Math.ceil(sp.line * 0.2);
    const rowW = digitsW + (desc.gear ? lean + gap + gearSide : 0);
    const speedTop = c - sp.line * 0.6;
    const left = c - rowW / 2;
    text("Speed", String(Math.min(999, Math.round(speed))), { step: speedStep, font: "display", color: token("ink"), nowrap: true, outline: 1.2, width: px(digitsW), align: "end" },
      fixed({ start: left, top: speedTop }, { transform: { scaleX: 0.88, skewX: -0.2, pivotX: 1, pivotY: 1 } }));
    // The unit fits its text (a translation may be longer than "km/h"), centred under the digits.
    box("UnitLine", { width: px(size), cross: "center", align: "center" }, undefined, () =>
      text("Unit", unit, { step: unitStep, font: "mono", color: token("inkMuted"), nowrap: true }, { transform: { x: left + digitsW / 2 - c } }),
    fixed({ start: 0, top: speedTop + sp.line }));
    if (desc.gear) {
      const g = desc.gear;
      const neutral = g === "N" || g === "R";
      // A square for one character; a longer gear (a translation's) widens it to the right, never over the speed.
      const gearPad = Math.max(3, Math.round((gearSide - stepSize(gearStep).size * 0.62) / 2));
      box("Gear", { height: px(gearSide), padding: { left: gearPad, right: gearPad, top: 0, bottom: 0 }, align: "center", cross: "center", part: sel("Speedometer", "gear", neutral ? "neutral" : null) }, {
        shape: { kind: "rect", radii: [3, 3, 3, 3] },
        fills: [solid(token("shadow", 0.7))],
        strokes: [stroke(1.5, neutral ? lit(role("warn")) : token("lineStrong"))],
      }, () => text("Text", g, { step: gearStep, font: "display", color: neutral ? lit(role("warn")) : token("ink"), nowrap: true }),
      fixed({ start: left + digitsW + lean + gap, top: speedTop + (sp.line - sp.size * 0.72) / 2 }));
    }
    // Shift lights over the readout: green, accent, danger, filling to the shift point; all flash there.
    const lights = size >= 150 ? 7 : 5;
    const d = Math.max(5, Math.round(size * 0.04));
    const lgap = Math.max(2, Math.round(d * 0.5));
    const from = Math.max(0, shiftAt - 0.25);
    const flash = shifting && !reduced() ? 0.35 + 0.65 * pulse(8) : 1;
    box("Lights", { direction: "row", gap: lgap, part: sel("Speedometer", "shift") }, undefined, () => {
      for (let k = 0; k < lights; k++) {
        const on = shifting || rpm >= from + ((k + 1) / lights) * (shiftAt - from) - 1e-6;
        const hue = k < lights * 0.4 ? role("success") : k < lights * 0.75 ? accent : danger;
        leaf(kit.idi("Light", k), { width: px(d), height: px(d), part: sel("Speedometer", "light", on ? "on" : null) }, {
          shape: ellipseShape(),
          fills: [solid(on ? lit(hue, flash) : token("shadow", 0.6))],
          effects: on ? [glow(d * 0.8, lit(hue, 0.5 * flash))] : [],
        });
      }
    }, fixed({ start: c - (lights * d + (lights - 1) * lgap) / 2, top: speedTop - d - Math.max(4, size * 0.05) }));
  }, {
    a11y: a11y("meter", "Speed", { value: `${Math.round(speed)} ${unit}${desc.gear ? `, gear ${desc.gear}` : ""}`, valueMin: 0, valueMax: desc.maxSpeed ?? 300, valueNow: speed }),
    ...groups({ rev: over ? "redline" : "normal" }, { rev: ["normal", "redline"] }),
  });
}

function digital(id: string, speed: number, rpm: number, red: number, unit: string, desc: Api.SpeedometerDesc, size: number): void {
  const over = rpm >= red;
  const segs = 20;
  box(id, { direction: "column", cross: "end", gap: u(4), part: sel("Speedometer", "dial", "digital") }, undefined, () => {
    box("Tach", { direction: "row", gap: u(2), part: sel("Speedometer", "tach") }, undefined, () => {
      for (let i = 0; i < segs; i++) {
        const at = (i + 1) / segs;
        const on = rpm >= at - 1e-6;
        const zone = at > red ? role("danger") : at > red - 0.2 ? role("warn") : role("success");
        leaf(kit.idi("Seg", i), { width: px(Math.round(size / 26)), height: px(u(8 + i * 0.6)) }, {
          shape: { kind: "rect", radii: [1, 1, 1, 1] },
          fills: [solid(on ? lit(zone, over && !reduced() ? 0.5 + 0.5 * pulse(8) : 1) : token("shadow", 0.55))],
        });
      }
    });
    kit.row("Body", { gap: u(8), cross: "end" }, () => {
      if (desc.gear) text("Gear", desc.gear, { step: ts(28, true), font: "display", color: over ? lit(role("danger")) : token("highlight"), nowrap: true, outline: 1.2 });
      text("Speed", String(Math.round(speed)), { step: ts(48, true), font: "display", color: token("ink"), nowrap: true, outline: 1.4 });
      text("Unit", unit, { step: ts(12), font: "mono", color: token("inkMuted"), nowrap: true, outline: 0.8 });
    });
  }, {
    a11y: a11y("meter", "Speed", { value: `${Math.round(speed)} ${unit}`, valueMin: 0, valueMax: desc.maxSpeed ?? 300, valueNow: speed }),
    ...groups({ rev: over ? "redline" : "normal" }, { rev: ["normal", "redline"] }),
  });
}

const DEG = Math.PI / 180;

/**
 * The flight HUD, read against Ace Combat 7's and DCS's head-up displays
 * and drawn for a game screen rather than a cockpit glass: a ladder of few
 * rungs (every 10°, numbered in a face that reads at a glance), the
 * waterline and the flight path marker heaviest as the focal pair, quiet
 * tapes with large boxed readouts, all in one ink from the style.
 */
export function flightHud(id: string, desc: Api.FlightHudDesc = {}): void {
  const f = frame();
  // The style's primary, lifted toward white so thin lines read over sky and ground.
  const base = mix(role("primary"), { r: 1, g: 1, b: 1, a: 1 }, 0.35);
  const ink: Rgba = desc.color ? paintRgba(desc.color, base) : base;
  const paint = lit(ink);
  const quiet = lit(ink, 0.72);
  const edgeStroke = stroke(1.5, token("shadow", 0.6), "outside");
  const vp = f.viewport;
  const roomW = vp.hud!.width;
  const roomH = vp.height - vp.safeArea.top - vp.safeArea.bottom;
  const W = Math.min(u(620), Math.round(roomW * 0.86));
  const H = Math.min(u(440), Math.round(roomH * 0.78));
  const fovDeg = (desc.fov || 40 * DEG) / DEG;
  const ppd = H / fovDeg;
  const pitchDeg = (desc.pitch ?? 0) / DEG;
  const roll = desc.roll ?? 0;
  const c = { x: W / 2, y: H / 2 };
  const line = (lid: string, pts: Array<[number, number]>, width: number, ink0: Api.Paint, part: string) =>
    leaf(lid, { width: px(W), height: px(H), part }, { shape: { kind: "path", points: pts.map(([x, y]) => ({ x, y })), width, cap: "round", join: "round" }, fills: [solid(ink0)], strokes: [edgeStroke] }, fixed({ start: 0, top: 0 }));
  const num = (lid: string, s: string, x: number, y: number, align: "start" | "end" | "center", step: Api.TypeStepName, ink0: Api.Paint = paint, more: { opacity?: number } = {}) => {
    const w = ut(56);
    const lh = ut(22);
    text(lid, s, { step, font: "mono", color: ink0, nowrap: true, width: px(w), align, outline: 1.25 }, fixed({ start: align === "end" ? x - w : align === "center" ? x - w / 2 : x, top: y - lh / 2 }, more));
  };
  const narrow = W < u(420);
  kit.anchor(`${id}Hud`, { anchor: desc.anchor ?? "center", margin: 0 }, () => {
    box(id, { width: px(W), height: px(H), part: sel("FlightHud", "frame") }, undefined, () => {
      // The ladder turns with roll about the boresight, clipped to its central window.
      const lw = W * (narrow ? 0.62 : 0.56);
      // A rung's numeral is left out where it would land on a boxed readout
      // or the heading tape (large text, a steep pitch): the ladder turns
      // about the boresight, so its place is turned the same way first.
      const boxW = ut(76);
      const boxH = ut(32);
      const keepOut: Array<[number, number, number, number]> = [
        [0, W, 0, ut(40) + ut(16) + boxH / 2],
        [0, boxW, H / 2 - boxH / 2, H / 2 + boxH / 2],
        [W - boxW, W, H / 2 - boxH / 2, H / 2 + boxH / 2],
      ];
      const rungClear = (lx: number, ly: number, label: string, dir: 1 | -1): boolean => {
        const sz = stepSize(ts(14)).size;
        const tw = label.length * sz * 0.62;
        const x0 = lx + (W - lw) / 2 + (dir > 0 ? tw / 2 : -tw / 2) - c.x;
        const y0 = ly + H * 0.13 - c.y;
        const cs = Math.cos(-roll);
        const sn = Math.sin(-roll);
        const x = c.x + x0 * cs - y0 * sn;
        const y = c.y + x0 * sn + y0 * cs;
        const r = [x - tw / 2 - 2, x + tw / 2 + 2, y - sz * 0.7, y + sz * 0.7];
        return !keepOut.some((k) => r[0]! < k[1] && r[1]! > k[0] && r[2]! < k[3] && r[3]! > k[2]);
      };
      box("Ladder", { width: px(lw), height: px(H * 0.74), part: sel("FlightHud", "ladder") }, { clip: "shape" }, () => {
        const lc = { x: lw / 2, y: (H * 0.74) / 2 };
        box("Rungs", { width: pct(1), height: pct(1) }, undefined, () => {
          const lo = Math.ceil((pitchDeg - fovDeg / 2) / 10) * 10;
          const hi = Math.floor((pitchDeg + fovDeg / 2) / 10) * 10;
          const half = lw * 0.2;
          const gap = lw * 0.1;
          const L = (lid: string, pts: Array<[number, number]>, width: number, ink0: Api.Paint, part: string) =>
            leaf(lid, { width: pct(1), height: pct(1), part }, { shape: { kind: "path", points: pts.map(([x, y]) => ({ x, y })), width, cap: "round", join: "round" }, fills: [solid(ink0)], strokes: [edgeStroke] }, fixed({ start: 0, top: 0 }));
          for (let d = Math.max(-90, lo); d <= Math.min(90, hi); d += 10) {
            const y = lc.y + (pitchDeg - d) * ppd;
            const rid = `R${d < 0 ? "m" : ""}${Math.abs(d)}`;
            if (d === 0) {
              // The horizon: one long line either side of a wide gap.
              L(rid, [[0, y], [lc.x - gap, y]], 2.5, paint, sel("FlightHud", "rung", "horizon"));
              L(`${rid}b`, [[lc.x + gap, y], [lw, y]], 2.5, paint, sel("FlightHud", "rung", "horizon"));
              continue;
            }
            const tip = d > 0 ? u(8) : -u(8);
            const part = sel("FlightHud", "rung", d > 0 ? "up" : "down");
            if (d > 0) {
              L(rid, [[lc.x - gap - half, y], [lc.x - gap, y], [lc.x - gap, y + tip]], 2, quiet, part);
              L(`${rid}b`, [[lc.x + gap + half, y], [lc.x + gap, y], [lc.x + gap, y + tip]], 2, quiet, part);
            } else {
              // Below the horizon: broken in two a side, the tips pointing up to it.
              const piece = half * 0.42;
              L(rid, [[lc.x - gap - half, y], [lc.x - gap - half + piece, y]], 2, quiet, part);
              L(`${rid}c`, [[lc.x - gap - piece, y], [lc.x - gap, y], [lc.x - gap, y + tip]], 2, quiet, part);
              L(`${rid}b`, [[lc.x + gap + half, y], [lc.x + gap + half - piece, y]], 2, quiet, part);
              L(`${rid}d`, [[lc.x + gap + piece, y], [lc.x + gap, y], [lc.x + gap, y + tip]], 2, quiet, part);
            }
            const label = String(Math.abs(d));
            if (!narrow && rungClear(lc.x - gap - half - u(6), y, label, -1)) num(`${rid}L`, label, lc.x - gap - half - u(6), y, "end", ts(14), paint);
            if (rungClear(lc.x + gap + half + u(6), y, label, 1)) num(`${rid}N`, label, lc.x + gap + half + u(6), y, "start", ts(14), paint);
          }
        }, fixed({ start: 0, top: 0 }, { transform: { rotate: -roll, pivotX: 0.5, pivotY: 0.5 } }));
      }, fixed({ start: (W - lw) / 2, top: H * 0.13 }));
      // The waterline: the aircraft's nose, fixed, the heaviest line.
      const wl = u(30);
      line("Waterline", [[c.x - wl, c.y], [c.x - u(14), c.y], [c.x - u(7), c.y + u(8)], [c.x, c.y], [c.x + u(7), c.y + u(8)], [c.x + u(14), c.y], [c.x + wl, c.y]], 3, paint, sel("FlightHud", "waterline"));
      // The flight path marker: where the aircraft is going.
      const fx = c.x + ((desc.pathX ?? 0) / DEG) * ppd;
      const fy = c.y + ((desc.pathY ?? 0) / DEG) * ppd;
      const fr = u(8);
      const pw = fr * 2 + u(32);
      const ph = fr * 2 + u(14);
      box("Path", { width: px(pw), height: px(ph), part: sel("FlightHud", "path") }, undefined, () => {
        leaf("Ring", { width: px(fr * 2), height: px(fr * 2) }, { shape: { kind: "arc", start: 0, sweep: TAU, thickness: 2.5, cap: "butt" }, fills: [solid(paint)], strokes: [edgeStroke] }, fixed({ start: u(16), top: u(7) }));
        leaf("Wings", { width: px(pw), height: px(ph) }, {
          shape: { kind: "path", points: [{ x: 0, y: u(7) + fr }, { x: u(16), y: u(7) + fr }], width: 2.5, cap: "round" },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
        leaf("Wings2", { width: px(pw), height: px(ph) }, {
          shape: { kind: "path", points: [{ x: pw - u(16), y: u(7) + fr }, { x: pw, y: u(7) + fr }], width: 2.5, cap: "round" },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
        leaf("Fin", { width: px(pw), height: px(ph) }, {
          shape: { kind: "path", points: [{ x: pw / 2, y: u(7) }, { x: pw / 2, y: 0 }], width: 2.5, cap: "round" },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
      }, fixed({ start: fx - pw / 2, top: fy - fr - u(7) }));
      tapes(W, H, desc, paint, quiet, edgeStroke, num, narrow);
    }, { a11y: a11y("group", "Flight", { value: `Pitch ${Math.round(pitchDeg)}°, heading ${Math.round((((desc.heading ?? 0) / DEG) % 360 + 360) % 360)}°, speed ${Math.round(desc.speed ?? 0)}, altitude ${Math.round(desc.altitude ?? 0)}` }) });
  });
}

type Num = (id: string, s: string, x: number, y: number, a: "start" | "end" | "center", step: Api.TypeStepName, ink?: Api.Paint, more?: { opacity?: number }) => void;

function tapes(W: number, H: number, desc: Api.FlightHudDesc, paint: Api.Paint, quiet: Api.Paint, edgeStroke: Api.Stroke, num: Num, narrow: boolean): void {
  const headingDeg = ((((desc.heading ?? 0) / DEG) % 360) + 360) % 360;
  // Heading across the top: a tick every 10°, a number every 30° (N E S W at the cardinals), fading at the ends.
  const tw = W * (narrow ? 0.6 : 0.44);
  const tx = (W - tw) / 2;
  const span = 60;
  const ppd = tw / span;
  const cardinal: Record<number, string> = { 0: "N", 90: "E", 180: "S", 270: "W" };
  box("Heading", { width: px(tw), height: px(ut(40)), part: sel("FlightHud", "heading") }, { clip: "shape" }, () => {
    for (let d = Math.floor((headingDeg - span / 2) / 10) * 10; d <= headingDeg + span / 2; d += 10) {
      const x = tw / 2 + (d - headingDeg) * ppd;
      const v = ((d % 360) + 360) % 360;
      const major = v % 30 === 0;
      const fade = 1 - Math.abs((x - tw / 2) / (tw / 2)) ** 3;
      leaf(`T${d}`, { width: px(2), height: px(major ? u(10) : u(5)) }, { fills: [solid(quiet)], strokes: [edgeStroke] }, fixed({ start: x - 1, top: ut(40) - u(12) - (major ? u(10) : u(5)) }, { opacity: clamp01(fade) }));
      if (major) num(`N${d}`, cardinal[v] ?? String(v / 10).padStart(2, "0"), x, ut(10), "center", ts(14), paint, { opacity: clamp01(fade) });
    }
  }, fixed({ start: tx, top: 0 }));
  readout("Heading", String(Math.round(headingDeg) % 360).padStart(3, "0"), W / 2, ut(40) + ut(16), paint, edgeStroke, "center");
  // Speed at the left, altitude at the right: a quiet tape and a large boxed readout at the boresight's height.
  const tapeW = u(64);
  const tape = (tid: "Speed" | "Altitude", value: number, unitLabel: string, x: number, step: number, side: "start" | "end") => {
    const th = H * 0.56;
    const per = th / (step * 6);
    box(tid, { width: px(tapeW), height: px(th), part: sel("FlightHud", tid === "Speed" ? "speed" : "altitude") }, { clip: "shape" }, () => {
      for (let v = Math.floor((value - step * 3) / step) * step; v <= value + step * 3; v += step) {
        const y = th / 2 - (v - value) * per;
        const fade = clamp01(1 - Math.abs((y - th / 2) / (th / 2)) ** 3);
        leaf(`T${Math.round(v)}`, { width: px(u(10)), height: px(2) }, { fills: [solid(quiet)], strokes: [edgeStroke] }, fixed(side === "start" ? { end: 0, top: y - 1 } : { start: 0, top: y - 1 }, { opacity: fade }));
        // Clear of the boxed readout (its half height and a numeral's, at any text size), and at full ink:
        // the haloed numerals read over sky and ground.
        // Nor at the tape's ends, where the unit (above) and the climb rate (below) sit outside it.
        const off = Math.abs(y - th / 2);
        if (off > ut(16) + ut(13) && off < th / 2 - ut(11)) num(`N${Math.round(v)}`, String(Math.round(v)), side === "start" ? tapeW - u(14) : tapeW, y, "end", ts(13), paint, { opacity: fade });
      }
    }, fixed({ start: x, top: (H - th) / 2 }));
    // The box sits over the tape at its outer edge, inside the HUD's frame.
    readout(tid, String(Math.round(value)), side === "start" ? x : x + tapeW, H / 2, paint, edgeStroke, side === "start" ? "start" : "end");
    num(`${tid}Unit`, unitLabel, side === "start" ? x + tapeW : x, (H - th) / 2 - ut(14), side === "start" ? "end" : "start", ts(12), quiet);
  };
  tape("Speed", desc.speed ?? 0, desc.speedUnit ?? "kt", 0, 10, "start");
  tape("Altitude", desc.altitude ?? 0, desc.altitudeUnit ?? "ft", W - tapeW, 100, "end");
  const vs = desc.verticalSpeed ?? 0;
  if (vs !== 0) num("Climb", `${vs > 0 ? "+" : "-"}${Math.abs(Math.round(vs))}`, W - tapeW, H * 0.78 + ut(16), "start", ts(13), quiet);
}

function readout(rid: string, value: string, x: number, y: number, paint: Api.Paint, edgeStroke: Api.Stroke, align: "start" | "end" | "center"): void {
  const w = ut(76);
  const h = ut(32);
  box(`${rid}Box`, { width: px(w), height: px(h), align: "center", cross: "center", part: sel("FlightHud", "box") }, {
    shape: { kind: "rect", radii: [2, 2, 2, 2] },
    fills: [solid(token("shadow", 0.62))],
    strokes: [stroke(2, paint), edgeStroke],
  }, () => text("Text", value, { step: ts(20), font: "mono", color: paint, nowrap: true, outline: 1 }), fixed({ start: align === "end" ? x - w : align === "center" ? x - w / 2 : x, top: y - h / 2 }, { a11y: a11y("text", rid, { value }) }));
}
