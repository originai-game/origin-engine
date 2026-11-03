// og.ui.minimapView, og.ui.compass and og.ui.objectiveTracker: Appendix C's
// "minimap and compass" and "objective tracker". Read against League of
// Legends' and Apex Legends' minimaps (north fixed and the player's arrow
// turning, or the map turning under a fixed arrow; objectives and pings
// kept on the rim pointing out when off the map), Skyrim's and Apex's
// compass strips (ticks and letters sliding under a centre notch, points
// of interest with their distance, held at the ends when behind) and the
// trackers of The Witcher 3 and World of Warcraft (a title over steps that
// tick with a pop, counts, a countdown, the tracked one marked).

import type * as Api from "@origingame/engine";
import { TAU, textScale, a11y, arc, inner, absolute, fixed, anchorOf, box, bp, chip, clamp01, close, edge, ellipseShape, fillDraw, frame, glow, groups, hudPlace, hudRoom, icon, keyId, kit, leaf, lit, msToTicks, now, open, paintRgba, pct, pop, pulse, px, rarityColor, reduced, role, sel, shapeOf, solid, sound, stroke, teamColor, text, token, trackDraw, ts, u, vgrad, countdown, type Rgba, apply, type Props, tsFit, sideSpan, stepSize, isolate, drop, minText } from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

/** An angle wrapped into (-π, π]. */
const wrap = (a: number): number => {
  let x = a % TAU;
  if (x > Math.PI) x -= TAU;
  if (x <= -Math.PI) x += TAU;
  return x;
};

function markerInk(kind: Api.MarkerKind, color: Api.Paint | null | undefined, rarity?: Api.Rarity): Rgba {
  if (color) return paintRgba(color, WHITE);
  switch (kind) {
    case "player": return role("highlight");
    case "ally": return teamColor(0);
    case "enemy": return role("danger");
    case "objective": return role("gold");
    case "ping": return role("warn");
    case "loot": return rarityColor(rarity) ?? WHITE;
    default: return WHITE;
  }
}

/** A marker's mark, centred in a box of side `s`. */
function mark(id: string, kind: Api.MarkerKind, ink: Rgba, s: number, heading: number, iconName: string, part: string, label: string, place: Props, diamond = false): void {
  const outline = stroke(1.5, token("shadow", 0.9), "outside");
  if (kind === "player" || kind === "ally") {
    // An arrow by heading: a narrow triangle with a notch (a dart).
    const c = s / 2;
    const pts = [[0, -0.5], [0.36, 0.42], [0, 0.22], [-0.36, 0.42]].map(([x, y]) => {
      const cs = Math.cos(heading);
      const sn = Math.sin(heading);
      return { x: c + (x! * cs - y! * sn) * s, y: c + (x! * sn + y! * cs) * s };
    });
    leaf(id, { width: px(s), height: px(s), part }, {
      shape: { kind: "path", points: pts, closed: true, width: 0 },
      fills: [solid(lit(ink))],
      strokes: [outline],
    }, { ...place, ...(label ? { a11y: a11y("image", label) } : {}) });
  } else if (kind === "poi" && iconName) {
    box(id, { width: px(s), height: px(s), align: "center", cross: "center", part }, undefined, () => icon("Icon", iconName, s, lit(ink)), { ...place, ...(label ? { a11y: a11y("image", label) } : {}) });
  } else if (diamond && kind === "objective") {
    // A compass's objective: a crisp diamond with a dark core (a waypoint's mark), apart from an enemy's rounded one.
    box(id, { width: px(s), height: px(s), align: "center", cross: "center", part }, {
      shape: { kind: "polygon", sides: 4 },
      fills: [solid(lit(ink))],
      strokes: [outline],
      effects: [glow(4, lit(ink, 0.6))],
    }, () => leaf("Core", { width: px(Math.round(s * 0.42)), height: px(Math.round(s * 0.42)) }, { shape: { kind: "polygon", sides: 4 }, fills: [solid(lit({ r: 0.05, g: 0.06, b: 0.08, a: 1 }, 0.85))] }), { ...place, ...(label ? { a11y: a11y("image", label) } : {}) });
  } else {
    const shape: Api.DrawShape = kind === "enemy" ? { kind: "polygon", sides: 4, rounding: 1 }
      : kind === "objective" ? { kind: "polygon", sides: 5, inner: 0.45, rounding: 0.5 }
      : kind === "loot" ? { kind: "rect", radii: [2, 2, 2, 2] }
      : ellipseShape();
    const sz = kind === "loot" ? Math.round(s * 0.6) : s;
    leaf(id, { width: px(sz), height: px(sz), part }, {
      shape,
      fills: [solid(lit(ink))],
      strokes: [outline],
      effects: kind === "objective" ? [glow(4, lit(ink, 0.6))] : [],
    }, { ...place, ...(label ? { a11y: a11y("image", label) } : {}) });
  }
}

/** `bare`: a zone of a panel that frames it (rtsConsole): no bezel of its own. */
export function minimapView(id: string, desc: Api.MinimapViewDesc = {}, bare = false): Api.MinimapResult {
  const vp0 = frame().viewport;
  // A phone held sideways keeps the map within 30 % of its height, so at a large text size it shrinks before it meets
  // the instrument in the other corner.
  const size = Math.min(u(desc.size || (bp().phone ? 144 : 200)), !bare && vp0.breakpoint === "phoneLandscape" ? Math.round(vp0.height * 0.3) : Infinity);
  const shape: Api.MapShape = desc.shape ?? "circle";
  const zoom = Math.max(0.01, desc.zoom ?? 0.3);
  const cx = desc.centerX ?? 0.5;
  const cy = desc.centerY ?? 0.5;
  const heading = desc.heading ?? 0;
  // Player-up for a racing track unless the game says otherwise; north-up for every other map.
  const rotate = desc.rotate ?? (desc.track ?? []).length >= 3;
  const turn = rotate ? -heading : 0;
  const R = size / 2;
  const rim = Math.max(3, Math.round(size / 40));
  let result: Api.MinimapResult = { clicked: false };
  // A dark bezel round the map, the rear mirror's family: rounded, a soft drop under it. A square map's caption sits
  // in the bezel as a bottom plate; a round one's under it, outlined over the scene.
  const bezel = bare ? 0 : Math.max(5, u(7));
  const plate = shape !== "circle" && !!desc.zone;
  const capStep = ts(11);
  // The north badge straddles the bezel's top edge, a tenth of the map across.
  // It grows with the text: at least the caption step's cap height and its padding, so the letter never shrinks to a dot.
  // And its letter (drawn at most 0.62 of the badge) never under the smallest text the checks allow (§15.2).
  const nb = Math.max(u(14), Math.round(size * 0.1), Math.ceil(stepSize(capStep).size * 0.72) + u(8), Math.ceil(minText() / 0.62));
  hudPlace(id, anchorOf(desc, "topRight"), () => {
    // Bare (a console's zone): no room above it, so the north badge sits inside the map's top edge.
    kit.column(id, { gap: bare ? 0 : u(4), cross: "center", padding: { top: bare ? 0 : Math.ceil(nb / 2), bottom: 0, left: 0, right: 0 } }, () => {
      const { radius, corner } = shapeOf("hud");
      const outline: Api.DrawShape = shape === "circle" ? ellipseShape() : { kind: "rect", radii: [radius, radius, radius, radius], corner };
      const ro = radius + bezel;
      box("Bezel", { direction: "column", gap: 0, cross: "center", width: px(size + bezel * 2), padding: { left: bezel, right: bezel, top: bezel, bottom: plate ? 0 : bezel }, part: sel("Minimap", "bezel") }, bare ? undefined : {
        shape: shape === "circle" ? ellipseShape() : { kind: "rect", radii: [ro, ro, ro, ro], corner },
        fills: [solid(lit({ r: 0.043, g: 0.05, b: 0.063, a: 1 }, 0.96)), vgrad([[0, lit(WHITE, 0.07)], [0.5, lit(WHITE, 0)]])],
        strokes: [stroke(1, lit(WHITE, 0.08)), stroke(1, lit({ r: 0, g: 0, b: 0, a: 1 }, 0.6), "outside")],
        effects: [drop(u(4), u(16), token("shadow", 0.55))],
      }, () => {
      const path = open("Map", { width: px(size), height: px(size), interactive: true, part: sel("Minimap", "frame", shape), partClass: "hud" }, {
        shape: outline,
        fills: [solid(token("well", 0.92))],
        strokes: [stroke(1, lit(WHITE, 0.06))],
        // Bare, the zone's well is the map itself, edge to edge: a hairline inset, not a vignette that reads as a dish.
        effects: [inner(0, bare ? u(3) : size * 0.08, token("shadow", bare ? 0.5 : 0.6))],
        clip: "shape",
      });
      // The map: a square larger than the frame, so it covers the corners as it turns.
      const cover = size * (rotate ? Math.SQRT2 : 1);
      const span = zoom * (cover / size);
      if (desc.texture) {
        kit.image("Texture", { texture: desc.texture, uv: [cx - span / 2, cy - span / 2, cx + span / 2, cy + span / 2], width: px(cover), height: px(cover), label: "" });
        apply(fixed({ start: (size - cover) / 2, top: (size - cover) / 2 }, { transform: { rotate: turn } }));
      } else if (desc.pitch) {
        // A pitch under the markers, in map coordinates (so it pans, zooms and turns with them): touchlines, halfway
        // line, centre circle and both penalty boxes as hairlines at about 20 % white.
        const turnC = Math.cos(turn);
        const turnS = Math.sin(turn);
        const at = (x: number, y: number): Api.Vector2 => {
          const dx = ((x - cx) / zoom) * size;
          const dy = ((y - cy) / zoom) * size;
          return { x: R + dx * turnC - dy * turnS, y: R + dx * turnS + dy * turnC };
        };
        const [x0, x1, y0, y1] = [0.04, 0.96, 0.04, 0.96];
        const len = x1 - x0;
        const wid = y1 - y0;
        // A real pitch's proportions along its length and width: a box 16.5 m deep and 40.3 m wide on 105 by 68 m.
        const boxD = len * (16.5 / 105);
        const boxW = wid * (40.3 / 68);
        const ring = len * (9.15 / 105);
        const ink = solid(lit(WHITE, 0.2));
        const lw = Math.max(1, u(1));
        box("Pitch", { width: px(size), height: px(size), part: sel("Minimap", "pitch") }, undefined, () => {
          const line = (lid: string, pts: Api.Vector2[], closed = false) =>
            leaf(lid, { width: px(size), height: px(size) }, { shape: { kind: "path", points: pts, closed, width: lw, cap: "butt", join: "miter" }, fills: [ink] }, fixed({ start: 0, top: 0 }));
          line("Touchlines", [at(x0, y0), at(x1, y0), at(x1, y1), at(x0, y1)], true);
          line("Halfway", [at(0.5, y0), at(0.5, y1)]);
          const mid = (y0 + y1) / 2;
          line("BoxL", [at(x0, mid - boxW / 2), at(x0 + boxD, mid - boxW / 2), at(x0 + boxD, mid + boxW / 2), at(x0, mid + boxW / 2)]);
          line("BoxR", [at(x1, mid - boxW / 2), at(x1 - boxD, mid - boxW / 2), at(x1 - boxD, mid + boxW / 2), at(x1, mid + boxW / 2)]);
          const n = 48;
          line("Circle", Array.from({ length: n }, (_, i) => at(0.5 + ring * Math.cos((i / n) * Math.PI * 2), mid + ring * Math.sin((i / n) * Math.PI * 2))), true);
        }, fixed({ start: 0, top: 0 }));
      } else {
        // No art: a grid that moves with the player, so motion still reads.
        const cells = 6;
        const step = cover / cells;
        const ox = (((-cx / zoom) * size) % step + step) % step;
        const oy = (((-cy / zoom) * size) % step + step) % step;
        box("Grid", { width: px(cover + step), height: px(cover + step), part: sel("Minimap", "map") }, undefined, () => {
          for (let i = 0; i <= cells; i++) {
            leaf(kit.idi("V", i), { width: px(1), height: pct(1) }, { fills: [solid(token("line", 0.5))] }, fixed({ start: i * step + ox, top: 0 }));
            leaf(kit.idi("H", i), { width: pct(1), height: px(1) }, { fills: [solid(token("line", 0.5))] }, fixed({ start: 0, top: i * step + oy }));
          }
        }, fixed({ start: (size - cover) / 2 - step / 2, top: (size - cover) / 2 - step / 2 }, { transform: { rotate: turn } }));
      }
      const cs = Math.cos(turn);
      const sn = Math.sin(turn);
      const track = desc.track ?? [];
      if (track.length >= 3) {
        // A circuit: its centre line through every point (Catmull-Rom as cubic Béziers), a dark bed under a light line,
        // and the start line across it at the first point.
        const closed = !desc.trackOpen;
        const pts = track.map((p) => {
          const dx = (((p.x ?? 0) - cx) / zoom) * size;
          const dy = (((p.y ?? 0) - cy) / zoom) * size;
          return { x: R + dx * cs - dy * sn, y: R + dx * sn + dy * cs };
        });
        const n = pts.length;
        const at = (i: number) => (closed ? pts[((i % n) + n) % n]! : pts[Math.max(0, Math.min(n - 1, i))]!);
        const curve: Api.Vector2[] = [pts[0]!];
        for (let i = 0; i < (closed ? n : n - 1); i++) {
          const p0 = at(i - 1), p1 = at(i), p2 = at(i + 1), p3 = at(i + 2);
          curve.push({ x: p1.x + (p2.x - p0.x) / 6, y: p1.y + (p2.y - p0.y) / 6 }, { x: p2.x - (p3.x - p1.x) / 6, y: p2.y - (p3.y - p1.y) / 6 }, { x: p2.x, y: p2.y });
        }
        const lw = Math.max(2.5, size / 60);
        leaf("TrackBed", { width: px(size), height: px(size) }, { shape: { kind: "path", points: curve, curves: true, width: lw * 2.6, cap: "round", join: "round" }, fills: [solid(lit({ r: 0, g: 0, b: 0, a: 1 }, 0.55))] }, fixed({ start: 0, top: 0 }));
        leaf("Track", { width: px(size), height: px(size), part: sel("Minimap", "track") }, { shape: { kind: "path", points: curve, curves: true, width: lw, cap: "round", join: "round" }, fills: [solid(lit(WHITE, 0.88))] }, fixed({ start: 0, top: 0 }));
        // Lines across the track: the start a checker of black and white cells, each sector a light tick.
        const across = (k: number): { p: Api.Vector2; nx: number; ny: number } => {
          const a = at(k - 1), b = at(k + 1), p = at(k);
          const tl = Math.hypot(b.x - a.x, b.y - a.y) || 1;
          return { p, nx: -(b.y - a.y) / tl, ny: (b.x - a.x) / tl };
        };
        const half = lw * 2.2;
        const s0 = across(0);
        const cellsN = 4;
        for (let c = 0; c < cellsN; c++) {
          const t0 = -half + (2 * half * c) / cellsN;
          const t1 = t0 + (2 * half) / cellsN;
          const cw = Math.max(2, lw * 1.1);
          leaf(c === 0 ? "Start" : kit.idi("Start", c), { width: px(size), height: px(size), part: sel("Minimap", "start") }, {
            shape: { kind: "path", points: [{ x: s0.p.x + s0.nx * t0, y: s0.p.y + s0.ny * t0 }, { x: s0.p.x + s0.nx * t1, y: s0.p.y + s0.ny * t1 }], width: cw, cap: "butt" },
            fills: [solid(lit(c % 2 === 0 ? WHITE : { r: 0, g: 0, b: 0, a: 1 }))],
          }, fixed({ start: 0, top: 0 }));
        }
        (desc.sectors ?? []).forEach((frac, k) => {
          const f = clamp01(frac) * (closed ? n : n - 1);
          const i0 = Math.floor(f);
          const q = across(i0), q1 = across(i0 + 1);
          const t = f - i0;
          const p = { x: q.p.x + (q1.p.x - q.p.x) * t, y: q.p.y + (q1.p.y - q.p.y) * t };
          const sh = half * 0.8;
          leaf(kit.idi("Sector", k), { width: px(size), height: px(size), part: sel("Minimap", "sector") }, {
            shape: { kind: "path", points: [{ x: p.x - q.nx * sh, y: p.y - q.ny * sh }, { x: p.x + q.nx * sh, y: p.y + q.ny * sh }], width: Math.max(1.5, lw * 0.6), cap: "round" },
            fills: [solid(lit(role("gold")))],
          }, fixed({ start: 0, top: 0 }));
        });
      }
      // Markers: in range at their place; objectives and pings held on the rim.
      const ms = Math.max(10, Math.round(size / 16));
      const edgeR = R - rim - ms * 0.7;
      for (const m of desc.markers ?? []) {
        const kind: Api.MarkerKind = m.kind ?? "poi";
        let dx = (((m.x ?? 0) - cx) / zoom) * size;
        let dy = (((m.y ?? 0) - cy) / zoom) * size;
        [dx, dy] = [dx * cs - dy * sn, dx * sn + dy * cs];
        const dist = shape === "circle" ? Math.hypot(dx, dy) : Math.max(Math.abs(dx), Math.abs(dy));
        const keep = kind === "objective" || kind === "ping" || kind === "player";
        let clamped = false;
        if (dist > edgeR) {
          if (!keep) continue;
          const k = edgeR / dist;
          dx *= k;
          dy *= k;
          clamped = true;
        }
        const ink = markerInk(kind, m.color, m.rarity);
        const s = kind === "player" ? Math.round(ms * 1.4) : ms;
        const hd = kind === "player" || kind === "ally" ? (m.heading ?? (kind === "player" ? heading : 0)) + turn : 0;
        const k = keyId(m.key ?? `${kind}${m.x}${m.y}`);
        if (kind === "ping" && !reduced()) {
          const p = (kit.seconds() * 0.8) % 1;
          const pr = s * (1 + 1.6 * p);
          leaf(`${k}Pulse`, { width: px(pr * 2), height: px(pr * 2), part: sel("Minimap", "ping") }, {
            shape: arc(1, 2),
            fills: [solid(lit(ink, 1 - p))],
          }, fixed({ start: R + dx - pr, top: R + dy - pr }));
        }
        if (m.ringed) {
          // A rival just ahead or behind: a ring in its colour round it.
          const rr = Math.round(s * 0.95);
          leaf(`${k}Ring`, { width: px(rr * 2), height: px(rr * 2), part: sel("Minimap", "ring") }, {
            shape: ellipseShape(),
            strokes: [stroke(Math.max(1.5, s / 7), lit(ink)), stroke(1, token("shadow", 0.7), "outside")],
          }, fixed({ start: R + dx - rr, top: R + dy - rr }, { overlaps: [`${path}/${k}`] }));
        }
        mark(k, kind, ink, s, hd, m.icon ?? "", sel("Minimap", "marker", kind, clamped ? "clamped" : null), m.label ?? "",
          fixed({ start: R + dx - s / 2, top: R + dy - s / 2 }, clamped ? { opacity: 0.75 + 0.25 * pulse(1.2) } : {}));
      }
      const r = close({ a11y: a11y("image", desc.zone || "Minimap") });
      result = { clicked: r.clicked };
      if (plate) {
        // A long name at a large text size takes a second line rather than an ellipsis.
        box("Plate", { width: pct(1), padding: { left: u(4), right: u(4), top: u(2), bottom: u(2) }, align: "center", cross: "center", part: sel("Minimap", "plate") }, undefined, () =>
          text("Zone", desc.zone ?? "", { step: capStep, font: "display", color: token("ink", 0.9), align: "center", width: pct(1), maxLines: 2, ellipsis: true }));
      }
      // North on the rim: centred on the bezel's ring, half outside the map, so it covers nothing on the map.
      const nAng = -Math.PI / 2 + turn;
      const ring = bare ? R - nb / 2 - u(3) : R + bezel / 2;
      const nStep = tsFit(10, u(10));
      // The letter fits the badge at any text size (the badge keeps its share of the map).
      const fitK = Math.min(1, (nb * 0.62) / Math.max(1, stepSize(nStep).size));
      box("North", { width: px(nb), height: px(nb), align: "center", cross: "center", part: sel("Minimap", "north") }, {
        shape: ellipseShape(),
        fills: [solid(lit({ r: 0.043, g: 0.05, b: 0.063, a: 1 }))],
        strokes: [stroke(1, lit(WHITE, 0.18))],
        effects: [drop(u(1), u(4), token("shadow", 0.5))],
      }, () => text("Letter", desc.north ?? "N", { step: nStep, font: "display", color: lit(role("danger")), nowrap: true }, fitK < 1 ? { transform: { scaleX: fitK, scaleY: fitK } } : undefined),
        // It straddles the map's rim on purpose (inside the top edge when bare).
        fixed({ start: bezel + R + Math.cos(nAng) * ring - nb / 2, top: bezel + R + Math.sin(nAng) * ring - nb / 2 }, { overlaps: [path] }));
      });
      // A round map's caption: the square map's plate as a pill centred under the ring, so it reads on any ground.
      if (desc.zone && !plate) {
        box("ZonePlate", { padding: { left: u(10), right: u(10), top: u(3), bottom: u(3) }, align: "center", cross: "center", part: sel("Minimap", "plate") }, {
          shape: { kind: "rect", radii: [999, 999, 999, 999] },
          fills: [solid(lit({ r: 0.043, g: 0.05, b: 0.063, a: 1 }, 0.92))],
          strokes: [stroke(1, lit(WHITE, 0.08))],
          effects: [drop(u(2), u(8), token("shadow", 0.45))],
        }, () => text("Zone", desc.zone ?? "", { step: capStep, font: "display", color: token("ink", 0.92), nowrap: true }));
      }
    });
  });
  return result;
}

const POINTS = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];

export function compass(id: string, desc: Api.CompassDesc = {}): void {
  const f = frame();
  const width = desc.width || (f.phone ? Math.round(f.viewport.width / 3) * 2 : u(560));
  const fov = desc.fov || Math.PI;
  const heading = desc.heading ?? 0;
  // The strip's rows grow with the player's text size: the point names,
  // the markers hung from their baseline, and the distances under them.
  const k = Math.max(1, stepSize(ts(16)).size / u(16));
  const s = u(16);
  const markTop = u(2) + u(20) * k - s / 2;
  const stripH = Math.max(u(52), markTop + s + stepSize(ts(10)).size * 1.3 + u(2));
  const pxPerRad = width / fov;
  const names = (desc.points ?? []).length === 8 ? desc.points! : POINTS;
  const deg = Math.round((((heading * 180) / Math.PI) % 360 + 360) % 360);
  const xOf = (bearing: number) => width / 2 + wrap(bearing - heading) * pxPerRad;
  hudPlace(id, anchorOf(desc, "top"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      box("Strip", { width: px(width), height: px(stripH), part: sel("Compass", "strip") }, {
        // The ends fade, so the strip has no hard edge over the game.
        fills: [vgrad([[0, token("shadow", 0)], [0.18, token("shadow", 0.45)], [0.82, token("shadow", 0.45)], [1, token("shadow", 0)]], 90)],
        clip: "shape",
      }, () => {
        for (let d = 0; d < 360; d += 15) {
          const b = (d * Math.PI) / 180;
          const x = xOf(b);
          if (x < -10 || x > width + 10) continue;
          const fade = 1 - Math.abs(x - width / 2) / (width / 2);
          const cardinal = d % 90 === 0;
          const named = d % 45 === 0;
          if (named) {
            text(`P${d}`, names[d / 45]!, { step: cardinal ? ts(16) : ts(12), font: "display", color: d === 0 ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1 },
              fixed({ start: x, top: u(2) }, { transform: { x: -u(cardinal ? 6 : 9) }, opacity: clamp01(fade * 1.6) }));
          } else {
            leaf(`T${d}`, { width: px(2), height: px(u(8)), part: sel("Compass", "tick", named ? "major" : null) }, { fills: [solid(token("ink", 0.7))] },
              fixed({ start: x - 1, top: u(6) }, { opacity: clamp01(fade * 1.6) }));
          }
        }
        // Where each marker sits, then which distances show: a label that
        // would collide with one already placed thins out, objectives and
        // pings first, then the nearest (a strip that names every marker
        // reads as none).
        const placed: Array<{ p: Api.CompassPoi; kind: Api.MarkerKind; x: number; clamped: boolean }> = [];
        for (const p of desc.pois ?? []) {
          const kind: Api.MarkerKind = p.kind ?? "poi";
          const keep = kind === "objective" || kind === "ping";
          let x = xOf(p.bearing ?? 0);
          let clamped = false;
          if (x < u(10) || x > width - u(10)) {
            if (!keep) continue;
            x = x < width / 2 ? u(10) : width - u(10);
            clamped = true;
          }
          placed.push({ p, kind, x, clamped });
        }
        const rank = (q: (typeof placed)[number]): number => (q.kind === "objective" ? 0 : q.kind === "ping" ? 1 : 2) * 1e9 + (q.p.distance ?? 1e8);
        const labelled = new Set<Api.CompassPoi>();
        const spans: Array<[number, number]> = [];
        for (const q of [...placed].sort((a, b) => rank(a) - rank(b))) {
          if (q.p.distance === null || q.p.distance === undefined) continue;
          const half = (`${Math.round(q.p.distance)}${desc.units ?? "m"}`.length * stepSize(ts(10)).size * 0.62) / 2 + u(3);
          if (spans.some(([a, b]) => q.x + half > a && q.x - half < b)) continue;
          spans.push([q.x - half, q.x + half]);
          labelled.add(q.p);
        }
        for (const { p, kind, x, clamped } of placed) {
          const ink = markerInk(kind, p.color);
          const bw = Math.max(u(48), Math.ceil(`${Math.round(p.distance ?? 0)}${desc.units ?? "m"}`.length * stepSize(ts(10)).size * 0.62) + u(4));
          box(keyId(p.key ?? ""), { direction: "column", cross: "center", gap: 0, width: px(bw), part: sel("Compass", "poi", kind, clamped ? "clamped" : null) }, undefined, () => {
            mark("Mark", kind, ink, s, 0, p.icon ?? "", sel("Compass", "mark", kind), "", {}, true);
            if (p.distance !== null && p.distance !== undefined && labelled.has(p)) {
              text("Distance", `${Math.round(p.distance)}${desc.units ?? "m"}`, { step: ts(10), font: "mono", color: token("ink"), nowrap: true, outline: 1 });
            }
          }, fixed({ start: x - bw / 2, top: markTop }, { a11y: a11y("image", p.label ?? ""), ...(clamped ? { opacity: 0.8 } : {}) }));
        }
        // The notch at the centre.
        leaf("Notch", { width: px(u(12)), height: px(u(8)), part: sel("Compass", "notch") }, {
          shape: { kind: "polygon", sides: 3, rotation: Math.PI },
          fills: [solid(token("ink"))],
          strokes: [stroke(1, token("shadow", 0.8), "outside")],
        }, fixed({ start: width / 2 - u(6), top: 0 }));
      });
      box("Bearing", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, part: sel("Compass", "bearing"), partClass: "chip" }, chip(3, true), () => {
        text("Text", String(deg).padStart(3, "0"), { step: ts(12), font: "mono", color: token("ink"), nowrap: true });
      });
    });
    apply({ a11y: a11y("status", "Heading", { value: `${deg}°, ${names[Math.round(deg / 45) % 8]}` }) });
  });
}

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
      fills: [vgrad([[0, token("shadow", 0)], [0.25, token("shadow", 0.42)], [1, token("shadow", 0.5)]], kit.frameDirection() === "rtl" ? -90 : 90)],
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
              text("Timer", countdown(q.timer, false), { step: ts(13), font: "mono", color: q.timer < 10 ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1 });
            }
            if (q.distance !== null && q.distance !== undefined) {
              text("Distance", `${Math.round(q.distance)}${desc.units ?? "m"}`, { step: ts(12), font: "mono", color: token("inkMuted"), nowrap: true, outline: 1 });
            }
          };
          // The timer and distance share the title's line only when both fit whole beside it (a measured guess:
          // display letters at 0.6 em, mono at 0.62); else they take their own line rather than crowd or cut it.
          const metaText = [q.timer != null ? countdown(q.timer, false) : "", q.distance != null ? `${Math.round(q.distance)}${desc.units ?? "m"}` : ""].filter(Boolean);
          const metaW = metaText.reduce((w, m, i) => w + m.length * stepSize(ts(13)).size * 0.62 + (i > 0 ? tk.md : 0), 0);
          const lead = (q.focused ? u(3) + tk.sm : 0) + (q.icon ? u(18) + tk.sm : 0);
          const titleW = (q.title ?? "").length * stepSize(ts(15, true)).size * 0.6;
          const apart = narrow || (metaText.length > 0 && lead + titleW + tk.sm + metaW > width - tk.md * 2);
          kit.row("Head", { gap: tk.sm, cross: "center", width: pct(1) }, () => {
            if (q.focused) leaf("Mark", { width: px(u(3)), height: px(u(18)), part: sel("ObjectiveTracker", "focus") }, { fills: [solid(token("highlight"))] });
            if (q.icon) icon("Icon", q.icon, u(18), q.focused ? token("highlight") : lit(role("gold")));
            text("Title", q.title ?? "", { step: ts(15), font: "display", color: q.focused ? token("highlight") : token("ink"), width: { kind: "grow" }, maxLines: 2, ellipsis: true, outline: 1 });
            if (!apart) meta();
          });
          // A narrow tracker (a phone's) gives the title the whole line and
          // the timer and distance their own line under it.
          if (apart && metaText.length > 0) kit.row("Meta", { gap: tk.md, cross: "center" }, meta);
          if (done) {
            const s = pop(since, 0.2, 400);
            text("Complete", q.completeLabel ?? "Complete", { step: ts(14), font: "display", color: lit(role("success")), nowrap: true, outline: 1 }, { transform: { scaleX: s, scaleY: s, pivotX: 0 } });
          }
          // Open steps first when the list is cut, so what is left to do shows.
          const all = q.steps ?? [];
          const cap = desc.maxSteps || (fold > 2 ? 1 : fold > 1 || (f.phone && textScale() >= 1.5) ? 2 : 0);
          const shown = cap > 0 && all.length > cap ? [...all.filter((st) => !st.done && !st.failed), ...all.filter((st) => st.done || st.failed)].slice(0, cap) : all;
          for (const st of all) if (shown.includes(st)) step(keyId(st.key ?? st.text ?? ""), st, tk);
          if (shown.length < all.length) text("More", `+${all.length - shown.length} more`, { step: ts(12), color: token("inkMuted"), nowrap: true, outline: 0.8 });
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
      text("Text", st.optional ? `${isolate(st.text ?? "")} (Optional)` : (st.text ?? ""), { step: ts(13), color: ink, width: { kind: "grow" }, maxLines: 2, ellipsis: true, outline: 0.8 });
      if ((st.target ?? 0) > 0) {
        text("Count", `${Math.min(st.count ?? 0, st.target ?? 0)}/${st.target}`, { step: ts(13), font: "mono", color: st.done ? token("inkFaint") : token("ink"), nowrap: true, outline: 0.8 },
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
    a11y: a11y("checkbox", st.text ?? "", { checked: !!st.done, value: (st.target ?? 0) > 0 ? `${st.count ?? 0} of ${st.target}` : "" }),
    ...groups({ step: state }, { step: ["open", "done", "failed"] }),
  });
}

/** The progress on the ui spring (under the open element). */
function spring01(v: number): number {
  return kit.animate("progress", clamp01(v), { kind: "token", token: "ui" });
}

