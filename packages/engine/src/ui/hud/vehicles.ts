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
} from "./core.ts";

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
  const size = u(desc.size || (bp().phone ? 132 : 180));
  const max = Math.max(1, desc.maxSpeed ?? 300);
  const speed = Math.max(0, desc.speed ?? 0);
  const rpm = clamp01(desc.rpm ?? 0);
  const red = clamp01(desc.redline ?? 0.85);
  const over = rpm >= red;
  const style: Api.SpeedoStyle = desc.style ?? "dial";
  const unit = desc.unit ?? "km/h";
  hudPlace(id, anchorOf(desc, "bottomRight"), () => {
    if (style === "digital") digital(id, speed, rpm, red, unit, desc, size);
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
        const step = niceStep(max);
        const n = Math.floor(max / step + 1e-6);
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
          if (major && size >= 120) {
            const rt = r0 - size * 0.08;
            const lw = size * 0.16;
            text(kit.idi("Num", i), String(Math.round(v)), { step: ts(10), font: "mono", color: token("inkMuted"), nowrap: true, width: px(lw), align: "center" },
              fixed({ start: c + Math.cos(a) * rt - lw / 2, top: c + Math.sin(a) * rt - size * 0.035 }));
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
        // The speed and its unit under the hub, the gear at the bottom.
        box("Readout", { direction: "column", cross: "center", gap: 0, width: pct(1) }, undefined, () => {
          text("Speed", String(Math.round(speed)), { step: tsFit(size >= 150 ? 30 : 22, size * 0.2, true), font: "display", color: token("ink"), nowrap: true, outline: 1 });
          text("Unit", unit, { step: tsFit(10, size * 0.085), font: "mono", color: token("inkMuted"), nowrap: true });
        }, fixed({ start: 0, top: c + size * 0.08 }));
        if (desc.gear) {
          box("Gear", { width: px(size * 0.16), height: px(size * 0.16), align: "center", cross: "center", part: sel("Speedometer", "gear") }, {
            shape: { kind: "rect", radii: [3, 3, 3, 3] },
            fills: [solid(over ? lit(role("danger"), 0.4 + 0.6 * pulse(6)) : token("shadow", 0.7))],
            strokes: [stroke(1.5, over ? lit(role("danger")) : token("lineStrong"))],
          }, () => text("Text", desc.gear ?? "", { step: tsFit(15, size * 0.1, true), font: "display", color: token("ink"), nowrap: true }), fixed({ centerX: true, bottom: size * 0.04 }));
        }
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
  const roomW = vp.width - vp.safeArea.left - vp.safeArea.right;
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
    text(lid, s, { step, font: "mono", color: ink0, nowrap: true, width: px(w), align, outline: 1 }, fixed({ start: align === "end" ? x - w : align === "center" ? x - w / 2 : x, top: y - lh / 2 }, more));
  };
  const narrow = W < u(420);
  kit.anchor(`${id}Hud`, { anchor: desc.anchor ?? "center", margin: 0 }, () => {
    box(id, { width: px(W), height: px(H), part: sel("FlightHud", "frame") }, undefined, () => {
      // The ladder turns with roll about the boresight, clipped to its central window.
      const lw = W * (narrow ? 0.62 : 0.56);
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
            if (!narrow) num(`${rid}L`, String(Math.abs(d)), lc.x - gap - half - u(6), y, "end", ts(14), quiet);
            num(`${rid}N`, String(Math.abs(d)), lc.x + gap + half + u(6), y, "start", ts(14), quiet);
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
      if (major) num(`N${d}`, cardinal[v] ?? String(v / 10).padStart(2, "0"), x, ut(10), "center", ts(14), quiet, { opacity: clamp01(fade) });
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
        if (Math.abs(y - th / 2) > u(24)) num(`N${Math.round(v)}`, String(Math.round(v)), side === "start" ? tapeW - u(14) : tapeW, y, "end", ts(13), quiet, { opacity: fade });
      }
    }, fixed({ start: x, top: (H - th) / 2 }));
    // The box sits over the tape at its outer edge, inside the HUD's frame.
    readout(tid, String(Math.round(value)), side === "start" ? x : x + tapeW, H / 2, paint, edgeStroke, side === "start" ? "start" : "end");
    num(`${tid}Unit`, unitLabel, side === "start" ? x + tapeW : x, (H - th) / 2 - u(14), side === "start" ? "end" : "start", ts(12), quiet);
  };
  tape("Speed", desc.speed ?? 0, desc.speedUnit ?? "kt", 0, 10, "start");
  tape("Altitude", desc.altitude ?? 0, desc.altitudeUnit ?? "ft", W - tapeW, 100, "end");
  const vs = desc.verticalSpeed ?? 0;
  if (vs !== 0) num("Climb", `${vs > 0 ? "+" : "-"}${Math.abs(Math.round(vs))}`, W - tapeW, H * 0.78 + u(16), "start", ts(13), quiet);
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
