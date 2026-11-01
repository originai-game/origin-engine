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
import { TAU, a11y, arc, inner, absolute, fixed, anchorOf, box, bp, chip, clamp01, close, edge, ellipseShape, fillDraw, frame, glow, groups, hudPlace, icon, keyId, kit, leaf, lit, msToTicks, now, open, paintRgba, pct, pop, pulse, px, rarityColor, reduced, role, sel, shapeOf, solid, sound, stroke, teamColor, text, token, trackDraw, ts, u, vgrad, countdown, type Rgba, apply, type Props } from "./core.ts";

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
function mark(id: string, kind: Api.MarkerKind, ink: Rgba, s: number, heading: number, iconName: string, part: string, label: string, place: Props): void {
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

export function minimapView(id: string, desc: Api.MinimapViewDesc = {}): Api.MinimapResult {
  const size = u(desc.size || (bp().phone ? 144 : 200));
  const shape: Api.MapShape = desc.shape ?? "circle";
  const zoom = Math.max(0.01, desc.zoom ?? 0.3);
  const cx = desc.centerX ?? 0.5;
  const cy = desc.centerY ?? 0.5;
  const heading = desc.heading ?? 0;
  const turn = desc.rotate ? -heading : 0;
  const R = size / 2;
  const rim = Math.max(3, Math.round(size / 40));
  let result: Api.MinimapResult = { clicked: false };
  hudPlace(id, anchorOf(desc, "topRight"), () => {
    kit.column(id, { gap: u(4), cross: "center" }, () => {
      const { radius, corner } = shapeOf("hud");
      const outline: Api.DrawShape = shape === "circle" ? ellipseShape() : { kind: "rect", radii: [radius, radius, radius, radius], corner };
      const path = open("Map", { width: px(size), height: px(size), interactive: true, part: sel("Minimap", "frame", shape), partClass: "hud" }, {
        shape: outline,
        fills: [solid(token("well", 0.92))],
        strokes: [stroke(rim, token("lineStrong")), stroke(1, token("shadow", 0.9), "outside")],
        effects: [inner(0, size * 0.08, token("shadow", 0.6))],
        clip: "shape",
      });
      // The map: a square larger than the frame, so it covers the corners as it turns.
      const cover = size * (desc.rotate ? Math.SQRT2 : 1);
      const span = zoom * (cover / size);
      if (desc.texture) {
        kit.image("Texture", { texture: desc.texture, uv: [cx - span / 2, cy - span / 2, cx + span / 2, cy + span / 2], width: px(cover), height: px(cover), label: "" });
        apply(fixed({ start: (size - cover) / 2, top: (size - cover) / 2 }, { transform: { rotate: turn } }));
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
      // Markers: in range at their place; objectives and pings held on the rim.
      const ms = Math.max(10, Math.round(size / 16));
      const edgeR = R - rim - ms * 0.7;
      const cs = Math.cos(turn);
      const sn = Math.sin(turn);
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
        mark(k, kind, ink, s, hd, m.icon ?? "", sel("Minimap", "marker", kind, clamped ? "clamped" : null), m.label ?? "",
          fixed({ start: R + dx - s / 2, top: R + dy - s / 2 }, clamped ? { opacity: 0.75 + 0.25 * pulse(1.2) } : {}));
      }
      // North on the rim.
      const nAng = -Math.PI / 2 + turn;
      const nr = R - rim - u(8);
      box("North", { width: px(u(16)), height: px(u(16)), align: "center", cross: "center", part: sel("Minimap", "north") }, {
        shape: ellipseShape(),
        fills: [solid(token("hud", 0.95))],
        strokes: [stroke(1, token("lineStrong"))],
      }, () => text("Letter", desc.north ?? "N", { step: ts(10), font: "display", color: lit(role("danger")), nowrap: true }),
        fixed({ start: R + Math.cos(nAng) * nr - u(8), top: R + Math.sin(nAng) * nr - u(8) }));
      const r = close({ a11y: a11y("image", desc.zone || "Minimap") });
      result = { clicked: r.clicked };
      if (desc.zone) text("Zone", desc.zone, { step: ts(12), font: "display", color: token("ink"), nowrap: true, outline: 1 });
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
  const h = u(30);
  const pxPerRad = width / fov;
  const names = (desc.points ?? []).length === 8 ? desc.points! : POINTS;
  const deg = Math.round((((heading * 180) / Math.PI) % 360 + 360) % 360);
  const xOf = (bearing: number) => width / 2 + wrap(bearing - heading) * pxPerRad;
  hudPlace(id, anchorOf(desc, "top"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      box("Strip", { width: px(width), height: px(h + u(22)), part: sel("Compass", "strip") }, {
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
          const ink = markerInk(kind, p.color);
          const s = u(16);
          box(keyId(p.key ?? ""), { direction: "column", cross: "center", gap: 0, width: px(u(48)), part: sel("Compass", "poi", kind, clamped ? "clamped" : null) }, undefined, () => {
            mark("Mark", kind, ink, s, 0, p.icon ?? "", sel("Compass", "mark", kind), "", {});
            if (p.distance !== null && p.distance !== undefined) {
              text("Distance", `${Math.round(p.distance)}${desc.units ?? "m"}`, { step: ts(10), font: "mono", color: token("ink"), nowrap: true, outline: 1 });
            }
          }, fixed({ start: x - u(24), top: u(22) - s / 2 }, { a11y: a11y("image", p.label ?? ""), ...(clamped ? { opacity: 0.8 } : {}) }));
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
  const max = desc.max || (f.phone ? 2 : 3);
  const width = u(desc.width || 300);
  const quests = (desc.quests ?? []).filter((q) => q.completedAt === null || q.completedAt === undefined || t - q.completedAt < msToTicks(2400)).slice(0, max);
  hudPlace(id, anchorOf(desc, "right"), () => {
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
          kit.row("Head", { gap: tk.sm, cross: "center", width: pct(1) }, () => {
            if (q.focused) leaf("Mark", { width: px(u(3)), height: px(u(18)), part: sel("ObjectiveTracker", "focus") }, { fills: [solid(token("highlight"))] });
            if (q.icon) icon("Icon", q.icon, u(18), q.focused ? token("highlight") : lit(role("gold")));
            text("Title", q.title ?? "", { step: ts(15), font: "display", color: q.focused ? token("highlight") : token("ink"), width: { kind: "grow" }, maxLines: 2, outline: 1 });
            if (q.timer !== null && q.timer !== undefined) {
              text("Timer", countdown(q.timer, false), { step: ts(13), font: "mono", color: q.timer < 10 ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1 });
            }
            if (q.distance !== null && q.distance !== undefined) {
              text("Distance", `${Math.round(q.distance)}${desc.units ?? "m"}`, { step: ts(12), font: "mono", color: token("inkMuted"), nowrap: true, outline: 1 });
            }
          });
          if (done) {
            const s = pop(since, 0.2, 400);
            text("Complete", q.completeLabel ?? "Complete", { step: ts(14), font: "display", color: lit(role("success")), nowrap: true, outline: 1 }, { transform: { scaleX: s, scaleY: s, pivotX: 0 } });
          }
          for (const st of q.steps ?? []) step(keyId(st.key ?? st.text ?? ""), st, tk);
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
      text("Text", `${st.text ?? ""}${st.optional ? " (Optional)" : ""}`, { step: ts(13), color: ink, width: { kind: "grow" }, maxLines: 2, outline: 0.8 });
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

