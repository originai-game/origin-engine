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
  reduced, role, sel, solid, springTo, stroke, text, token, ts, u, vgrad, type Rgba,
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
          text("Speed", String(Math.round(speed)), { step: ts(size >= 150 ? 30 : 22, true), font: "display", color: token("ink"), nowrap: true, outline: 1 });
          text("Unit", unit, { step: ts(10), font: "mono", color: token("inkMuted"), nowrap: true });
        }, fixed({ start: 0, top: c + size * 0.08 }));
        if (desc.gear) {
          box("Gear", { width: px(size * 0.16), height: px(size * 0.16), align: "center", cross: "center", part: sel("Speedometer", "gear") }, {
            shape: { kind: "rect", radii: [3, 3, 3, 3] },
            fills: [solid(over ? lit(role("danger"), 0.4 + 0.6 * pulse(6)) : token("shadow", 0.7))],
            strokes: [stroke(1.5, over ? lit(role("danger")) : token("lineStrong"))],
          }, () => text("Text", desc.gear ?? "", { step: ts(15, true), font: "display", color: token("ink"), nowrap: true }), fixed({ centerX: true, bottom: size * 0.04 }));
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

export function flightHud(id: string, desc: Api.FlightHudDesc = {}): void {
  const f = frame();
  const ink: Rgba = desc.color ? paintRgba(desc.color, { r: 0.3, g: 1, b: 0.5, a: 1 }) : { r: 0.3, g: 1, b: 0.5, a: 1 };
  const paint = lit(ink);
  const edgeStroke = stroke(1, token("shadow", 0.75), "outside");
  const W = Math.min(u(560), Math.round(f.viewport.width * 0.7));
  const H = Math.min(u(420), Math.round(f.viewport.height * 0.7));
  const fovDeg = (desc.fov || 40 * DEG) / DEG;
  const ppd = H / fovDeg;
  const pitchDeg = (desc.pitch ?? 0) / DEG;
  const roll = desc.roll ?? 0;
  const c = { x: W / 2, y: H / 2 };
  const line = (lid: string, pts: Array<[number, number]>, width = 2, part = sel("FlightHud", "line")) =>
    leaf(lid, { width: px(W), height: px(H), part }, { shape: { kind: "path", points: pts.map(([x, y]) => ({ x, y })), width, cap: "butt", join: "miter" }, fills: [solid(paint)], strokes: [edgeStroke] }, fixed({ start: 0, top: 0 }));
  const label = (lid: string, s: string, x: number, y: number, align: "start" | "end" | "center" = "center") => {
    const w = u(40);
    text(lid, s, { step: ts(11), font: "mono", color: paint, nowrap: true, width: px(w), align, outline: 0.7 }, fixed({ start: align === "end" ? x - w : align === "center" ? x - w / 2 : x, top: y - u(7) }));
  };
  kit.anchor(`${id}Hud`, { anchor: desc.anchor ?? "center", margin: 0 }, () => {
    box(id, { width: px(W), height: px(H), part: sel("FlightHud", "frame") }, undefined, () => {
      // The ladder turns with roll about the boresight; the frame clips it.
      box("Ladder", { width: pct(1), height: pct(1), part: sel("FlightHud", "ladder") }, { clip: "shape" }, () => {
        box("Rungs", { width: pct(1), height: pct(1) }, undefined, () => {
          const lo = Math.ceil((pitchDeg - fovDeg / 2 - 5) / 5) * 5;
          const hi = Math.floor((pitchDeg + fovDeg / 2 + 5) / 5) * 5;
          for (let d = Math.max(-90, lo); d <= Math.min(90, hi); d += 5) {
            const y = c.y + (pitchDeg - d) * ppd;
            const rid = `R${d < 0 ? "m" : ""}${Math.abs(d)}`;
            if (d === 0) {
              line(rid, [[c.x - W * 0.42, y], [c.x - W * 0.08, y]], 2, sel("FlightHud", "rung", "horizon"));
              line(`${rid}b`, [[c.x + W * 0.08, y], [c.x + W * 0.42, y]], 2, sel("FlightHud", "rung", "horizon"));
              continue;
            }
            const half = W * 0.16;
            const gap = W * 0.06;
            const tip = d > 0 ? u(6) : -u(6);
            if (d > 0) {
              line(rid, [[c.x - gap - half, y], [c.x - gap, y], [c.x - gap, y + tip]], 1.5, sel("FlightHud", "rung", "up"));
              line(`${rid}b`, [[c.x + gap + half, y], [c.x + gap, y], [c.x + gap, y + tip]], 1.5, sel("FlightHud", "rung", "up"));
            } else {
              // Below the horizon: dashed, two pieces a side, the tips pointing up to it.
              const piece = half / 2 - u(3);
              line(rid, [[c.x - gap - half, y], [c.x - gap - half + piece, y]], 1.5, sel("FlightHud", "rung", "down"));
              line(`${rid}c`, [[c.x - gap - piece, y], [c.x - gap, y], [c.x - gap, y + tip]], 1.5, sel("FlightHud", "rung", "down"));
              line(`${rid}b`, [[c.x + gap + half, y], [c.x + gap + half - piece, y]], 1.5, sel("FlightHud", "rung", "down"));
              line(`${rid}d`, [[c.x + gap + piece, y], [c.x + gap, y], [c.x + gap, y + tip]], 1.5, sel("FlightHud", "rung", "down"));
            }
            label(`${rid}L`, String(Math.abs(d)), c.x - gap - half - u(4), y, "end");
            label(`${rid}N`, String(Math.abs(d)), c.x + gap + half + u(4), y, "start");
          }
        }, fixed({ start: 0, top: 0 }, { transform: { rotate: -roll, pivotX: 0.5, pivotY: 0.5 } }));
      }, fixed({ start: 0, top: 0 }));
      // The waterline: the aircraft's nose, fixed.
      line("Waterline", [[c.x - u(26), c.y], [c.x - u(12), c.y], [c.x - u(6), c.y + u(7)], [c.x, c.y], [c.x + u(6), c.y + u(7)], [c.x + u(12), c.y], [c.x + u(26), c.y]], 2, sel("FlightHud", "waterline"));
      // The flight path marker: where the aircraft is going.
      const fx = c.x + ((desc.pathX ?? 0) / DEG) * ppd;
      const fy = c.y + ((desc.pathY ?? 0) / DEG) * ppd;
      const fr = u(7);
      box("Path", { width: px(fr * 2 + u(28)), height: px(fr * 2 + u(12)), part: sel("FlightHud", "path") }, undefined, () => {
        leaf("Ring", { width: px(fr * 2), height: px(fr * 2) }, { shape: { kind: "arc", start: 0, sweep: TAU, thickness: 2, cap: "butt" }, fills: [solid(paint)], strokes: [edgeStroke] }, fixed({ start: u(14), top: u(6) }));
        const w0 = fr * 2 + u(28);
        leaf("Wings", { width: px(w0), height: px(fr * 2 + u(12)) }, {
          shape: { kind: "path", points: [{ x: 0, y: u(6) + fr }, { x: u(14), y: u(6) + fr }], width: 2 },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
        leaf("Wings2", { width: px(w0), height: px(fr * 2 + u(12)) }, {
          shape: { kind: "path", points: [{ x: w0 - u(14), y: u(6) + fr }, { x: w0, y: u(6) + fr }, ], width: 2 },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
        leaf("Fin", { width: px(w0), height: px(fr * 2 + u(12)) }, {
          shape: { kind: "path", points: [{ x: w0 / 2, y: u(6) }, { x: w0 / 2, y: 0 }], width: 2 },
          fills: [solid(paint)], strokes: [edgeStroke],
        }, fixed({ start: 0, top: 0 }));
      }, fixed({ start: fx - fr - u(14), top: fy - fr - u(6) }));
      tapes(W, H, desc, paint, edgeStroke, label);
    }, { a11y: a11y("group", "Flight") });
  });
}

function tapes(W: number, H: number, desc: Api.FlightHudDesc, paint: Api.Paint, edgeStroke: Api.Stroke, label: (id: string, s: string, x: number, y: number, a?: "start" | "end" | "center") => void): void {
  const headingDeg = ((((desc.heading ?? 0) / DEG) % 360) + 360) % 360;
  // Heading tape across the top: ticks every 5°, numbers every 10° in tens (09 is 090).
  const tw = W * 0.5;
  const tx = (W - tw) / 2;
  const ppd = tw / 40;
  box("Heading", { width: px(tw), height: px(u(34)), part: sel("FlightHud", "heading") }, { clip: "shape" }, () => {
    for (let d = Math.floor((headingDeg - 22) / 5) * 5; d <= headingDeg + 22; d += 5) {
      const x = tw / 2 + (d - headingDeg) * ppd;
      const major = d % 10 === 0;
      leaf(`T${d}`, { width: px(2), height: px(major ? u(8) : u(4)) }, { fills: [solid(paint)], strokes: [edgeStroke] }, fixed({ start: x - 1, top: u(18) }));
      if (major) {
        const v = ((d % 360) + 360) % 360;
        label(`N${d}`, String(Math.round(v / 10)).padStart(2, "0"), x, u(8));
      }
    }
  }, fixed({ start: tx, top: 0 }));
  readout("Heading", String(Math.round(headingDeg) % 360).padStart(3, "0"), W / 2, u(46), paint, edgeStroke, "center");
  // Speed at the left, altitude at the right: a tape of ticks and a boxed readout at the boresight.
  const tape = (tid: "Speed" | "Altitude", value: number, unitLabel: string, x: number, step: number, side: "start" | "end") => {
    const th = H * 0.6;
    const per = th / (step * 8);
    box(tid, { width: px(u(60)), height: px(th), part: sel("FlightHud", tid === "Speed" ? "speed" : "altitude") }, { clip: "shape" }, () => {
      for (let v = Math.floor((value - step * 4) / step) * step; v <= value + step * 4; v += step / 2) {
        const y = th / 2 - (v - value) * per;
        const major = Math.abs(v / step - Math.round(v / step)) < 1e-6;
        const tw0 = major ? u(10) : u(5);
        leaf(`T${Math.round(v * 10)}`, { width: px(tw0), height: px(2) }, { fills: [solid(paint)], strokes: [edgeStroke] }, fixed(side === "start" ? { end: 0, top: y - 1 } : { start: 0, top: y - 1 }));
        if (major && Math.abs(y - th / 2) > u(14)) label(`N${Math.round(v)}`, String(Math.round(v)), side === "start" ? u(60) - u(14) : u(14), y, side === "start" ? "end" : "start");
      }
    }, fixed({ start: x, top: (H - th) / 2 }));
    readout(tid, String(Math.round(value)), side === "start" ? x + u(60) - u(2) : x + u(2), H / 2, paint, edgeStroke, side === "start" ? "end" : "start");
    label(`${tid}Unit`, unitLabel, side === "start" ? x + u(60) : x, (H - th) / 2 - u(10), side === "start" ? "end" : "start");
  };
  tape("Speed", desc.speed ?? 0, desc.speedUnit ?? "kt", W * 0.06, 10, "start");
  tape("Altitude", desc.altitude ?? 0, desc.altitudeUnit ?? "ft", W * 0.94 - u(60), 100, "end");
  const vs = desc.verticalSpeed ?? 0;
  if (vs !== 0) label("Climb", `${vs > 0 ? "+" : ""}${Math.round(vs)}`, W * 0.94 - u(60), H * 0.8 + u(12), "start");
}

function readout(rid: string, value: string, x: number, y: number, paint: Api.Paint, edgeStroke: Api.Stroke, align: "start" | "end" | "center"): void {
  const w = u(58);
  const h = u(22);
  box(`${rid}Box`, { width: px(w), height: px(h), align: "center", cross: "center", part: sel("FlightHud", "box") }, {
    shape: { kind: "rect", radii: [0, 0, 0, 0] },
    fills: [solid(token("shadow", 0.55))],
    strokes: [stroke(1.5, paint), edgeStroke],
  }, () => text("Text", value, { step: ts(13), font: "mono", color: paint, nowrap: true }), fixed({ start: align === "end" ? x - w : align === "center" ? x - w / 2 : x, top: y - h / 2 }, { a11y: a11y("text", rid, { value }) }));
}

