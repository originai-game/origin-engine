import type { FlightHudDesc } from "../types.ts";
// og.ui.flightHud: Appendix C's "flight ladder", read against the head-up
// displays of Ace Combat and DCS (one phosphor ink with a dark edge, a
// pitch ladder of rungs every 5° dashed below the horizon and turning with
// roll, a flight path marker, a heading tape across the top, speed and
// altitude tapes with boxed readouts at either side). The speedometer moved
// into the racing kit (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import {
  TAU, a11y, fixed, anchorOf, box, bp, clamp01, ellipseShape, frame, glow, groups, hudPlace, kit, leaf, lit, mix, paintRgba, pct, pulse, px,
  reduced, role, sel, solid, springTo, stroke, text, token, ts, u, vgrad, type Rgba, ut, tsFit,
  stepSize, textScale,
} from "@origingame/engine/ui-parts/hud";

const DEG = Math.PI / 180;

/**
 * The flight HUD, read against Ace Combat 7's and DCS's head-up displays
 * and drawn for a game screen rather than a cockpit glass: a ladder of few
 * rungs (every 10°, numbered in a face that reads at a glance), the
 * waterline and the flight path marker heaviest as the focal pair, quiet
 * tapes with large boxed readouts, all in one ink from the style.
 */
export function flightHud(id: string, desc: FlightHudDesc = {}): void {
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

function tapes(W: number, H: number, desc: FlightHudDesc, paint: Api.Paint, quiet: Api.Paint, edgeStroke: Api.Stroke, num: Num, narrow: boolean): void {
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
