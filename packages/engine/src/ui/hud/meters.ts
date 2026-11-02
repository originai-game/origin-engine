// og.ui.statBar, og.ui.orbMeter and og.ui.radialMeter: the meters of
// Appendix C. Read against the bars they answer to: Overwatch's 25-point
// health blocks with armour and shields on top; Dark Souls' and the
// fighting games' ghost chunk that holds after a hit and then drains (and
// Street Fighter's grey recoverable health); Apex Legends' armour cells in
// their tier's colour; Diablo's glass health and mana globes with a liquid
// that sways; Breath of the Wild's stamina wheel, outer wheels for
// extended stamina, red and slow while exhausted, gone once full.

import type * as Api from "@origingame/engine";
import { TAU, a11y, absolute, box, clamp01, close, easeInOut, edge, ellipseShape, fillDraw, frame, glow, groups, hudPlace, icon, kindColor, kit, leaf, levelOf, lit, mix, msToTicks, now, open, paintRgba, pct, phase, pulse, px, radial, rarityColor, reduced, role, sel, solid, sound, spring, stroke, text, token, trackDraw, ts, u, vgrad, anchorOf, inner, drop, shapeOf, arc, type Level, type Rgba, apply, type Props, stepSize, WHITE, BLACK } from "./core.ts";

const GHOST_HOLD_MS = 450;
const GHOST_DRAIN_MS = 650;

/**
 * A ghost of a falling value under the open element: after a drop it holds
 * where the value was, then drains to it (the fighting games' chunk).
 */
function ghostOf(id: string, frac: number): number {
  const t = now();
  const last = kit.memory(`${id}~last`, frac);
  let g = kit.memory(`${id}~g`, frac);
  let at = kit.memory(`${id}~at`, -1e9);
  const hold = msToTicks(GHOST_HOLD_MS);
  const drain = msToTicks(GHOST_DRAIN_MS);
  const current = (): number => {
    const since = t - at;
    if (since < hold) return g;
    return g + (last - g) * easeInOut((since - hold) / drain);
  };
  if (frac < last) {
    // A new drop keeps the chunk where it shows now, so a combo's hits pile into one ghost.
    g = Math.max(current(), last);
    at = t;
    kit.remember(`${id}~g`, g);
    kit.remember(`${id}~at`, at);
  }
  if (frac !== last) kit.remember(`${id}~last`, frac);
  if (reduced()) return frac;
  const since = t - at;
  if (since < hold) return Math.max(g, frac);
  return Math.max(frac, g + (frac - g) * easeInOut((since - hold) / drain));
}

/** A fill that drops at once and rises on the ui spring. */
function riseOnly(id: string, frac: number): number {
  const s = spring(id, frac);
  return Math.min(frac, s);
}

export function statBar(id: string, desc: Api.StatBarDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const max = desc.max ?? 100;
  const value = desc.value ?? 0;
  const frac = max > 0 ? clamp01(value / max) : 0;
  const level = levelOf(frac, desc.low ?? 0.3, desc.critical ?? 0.15);
  const kind = desc.kind ?? "health";
  const ink = kindColor(kind, level, desc.color);
  const style: Api.StatBarStyle = desc.style ?? "flat";
  // A value inside the track: the track grows to hold its numerals (at 200 % too), the bar keeps its length.
  const inside = !!desc.showValue && !!desc.valueInside;
  const valueStep = ts(11);
  const h = Math.max(u(desc.thickness || 14), inside ? Math.ceil(stepSize(valueStep).size * 1.3) : 0);
  const radius = style === "pill" ? h / 2 : style === "slanted" ? 1 : Math.min(shapeOf("bar").radius, h / 3);
  const name = desc.name || desc.label || "Health";
  const valueText = desc.valueText || `${Math.round(value)} / ${Math.round(max)}`;
  const anchor = anchorOf(desc, null);
  const e = edge(`${id}~level`, ["normal", "low", "critical", "empty"].indexOf(level));
  sound("warning", e.changed && level === "critical" && e.prev < 2);
  hudPlace(id, anchor, () => {
    box(id, { direction: "column", gap: u(3), width: desc.width ?? px(u(240)), part: sel("StatBar", "frame", style, level), partClass: "hud" }, undefined, (path) => {
      if ((desc.shieldMax ?? 0) > 0) shieldRow(desc, h, radius, style);
      kit.row("Main", { gap: tk.sm, cross: "center", width: { kind: "grow" } }, () => {
        if (desc.icon) icon("Icon", desc.icon, Math.round(h * 1.4), lit(ink), "");
        // Over the scene, not a card: full ink a step up, with the HUD's outline and halo (4.5:1 on a bright ground).
        if (desc.label) text("Label", desc.label, { step: ts(14), font: "display", color: token("ink"), nowrap: true, outline: 1.2 });
        track(path, desc, frac, level, ink, h, radius, style, inside ? { text: valueText, step: valueStep, critical: level === "critical" } : null);
        if (desc.showValue && !inside) text("Value", valueText, { step: ts(14), font: "mono", color: level === "critical" ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1.2 });
      });
    }, {
      a11y: a11y("meter", name, { value: valueText, valueMin: 0, valueMax: max, valueNow: value }),
      ...groups({ level }, { level: ["normal", "low", "critical", "empty"] }),
    });
  });
}

function track(parent: string, desc: Api.StatBarDesc, frac: number, level: Level, ink: Rgba, h: number, radius: number, style: Api.StatBarStyle, inside: { text: string; step: Api.TypeStepName; critical: boolean } | null): void {
  const max = desc.max ?? 100;
  const path = open("Track", {
    width: { kind: "grow" },
    height: px(h),
    part: sel("StatBar", "track", style),
    partClass: "bar",
  }, trackDraw(radius));
  // A slanted bar leans its whole track; its text sits outside it, upright.
  const w = kit.bounds(path)?.width ?? kit.bounds(`${parent}/Main`)?.width ?? u(240);
  const side = (start: number, width: number, more: Props = {}): Props =>
    absolute(desc.reverse ? { end: start * w, top: 0 } : { start: start * w, top: 0 }, { ...more, ...(width <= 0 ? { opacity: 0 } : {}) });
  const shown = riseOnly("fill", frac);
  const ghost = desc.ghost === false ? shown : ghostOf("ghost", frac);
  const rec = clamp01((desc.recoverable ?? 0) / Math.max(1e-6, max));
  if (rec > shown) {
    leaf("Recoverable", { width: pct(rec), height: pct(1), part: sel("StatBar", "recoverable"), partClass: "bar" }, {
      shape: { kind: "rect", radii: [radius, radius, radius, radius] },
      fills: [solid(token("inkMuted", 0.55)), vgrad([[0, token("sheen", 0.18)], [1, token("sheen", 0)]])],
    }, side(0, rec));
  }
  if (ghost > shown + 0.001) {
    // The chunk: pale over the track, lit at its leading edge.
    leaf("Ghost", { width: pct(ghost), height: pct(1), part: sel("StatBar", "ghost"), partClass: "bar" }, {
      shape: { kind: "rect", radii: [radius, radius, radius, radius] },
      fills: [solid(lit(mix(ink, { r: 1, g: 0.95, b: 0.85, a: 1 }, 0.55), 0.85))],
    }, side(0, ghost));
  }
  const heal = clamp01((desc.heal ?? 0) / Math.max(1e-6, max));
  if (heal > 0 && shown < 1) {
    const hw = Math.min(heal, 1 - shown);
    leaf("Heal", { width: pct(hw), height: pct(1), part: sel("StatBar", "heal"), partClass: "bar" }, {
      fills: [solid(lit(role("success"), 0.45 + 0.2 * pulse(1.2)))],
    }, side(shown, hw));
  }
  const crit = level === "critical";
  const fd = fillDraw(ink, radius);
  const absorb = clamp01((desc.absorb ?? 0) / Math.max(1e-6, max));
  leaf("Fill", { width: pct(shown), height: pct(1), part: sel("StatBar", "fill", desc.kind ?? "health", level), partClass: "bar" }, {
    ...fd,
    effects: crit ? [glow(6 + 6 * pulse(2.2), lit(ink, 0.5 + 0.4 * pulse(2.2)))] : [],
  }, side(0, shown, crit && !reduced() ? { opacity: 0.78 + 0.22 * pulse(2.2) } : {}));
  if (absorb > 0) {
    // The absorb: white-blue past the fill; what overflows a full bar lies back over the fill's end, with a bright seam.
    const past = Math.min(absorb, 1 - shown);
    const over = Math.min(shown, absorb - past);
    const paint = (a: number): Api.Fill[] => [solid(lit({ r: 0.78, g: 0.9, b: 1, a: 1 }, a)), vgrad([[0, lit(WHITE, 0.35)], [0.5, lit(WHITE, 0)]])];
    if (past > 0) leaf("Absorb", { width: pct(past), height: pct(1), part: sel("StatBar", "absorb"), partClass: "bar" }, { fills: paint(0.62) }, side(shown, past));
    if (over > 0.001) {
      // Its seam only with the value outside the track: inside, a bright line would cross the figures.
      const seam = !(desc.showValue && desc.valueInside);
      leaf("Overabsorb", { width: pct(over), height: pct(1), part: sel("StatBar", "absorb", "over"), partClass: "bar" }, {
        fills: paint(0.5),
        strokes: [],
      }, side(shown - over, over));
      if (seam) leaf("Seam", { width: px(2), height: pct(1) }, { fills: [solid(lit(WHITE, 0.9))], effects: [glow(4, lit({ r: 0.7, g: 0.88, b: 1, a: 1 }, 0.8))] }, side(shown - over, 1));
    }
  }
  // Blocks: a dark notch at every segment's end (Overwatch), only when they stay readable.
  const seg = desc.segment ?? 0;
  if (seg > 0 && max / seg <= 40 && max / seg >= 2) {
    const n = Math.floor(max / seg - 1e-6);
    for (let i = 1; i <= n; i++) {
      const at = (i * seg) / max;
      leaf(kit.idi("Block", i), { width: px(Math.max(1, Math.round(h / 7))), height: pct(1), part: sel("StatBar", "block", at <= shown ? "filled" : null) }, {
        fills: [solid(token("shadow", 0.8))],
      }, side(at, 1, { transform: { x: desc.reverse ? 0.5 : -0.5 } }));
    }
  }
  // A highlight along the top third so the bar reads as a bar, not a flat rectangle.
  leaf("Gloss", { width: pct(1), height: pct(0.42) }, {
    fills: [vgrad([[0, token("sheen", 0.22)], [1, token("sheen", 0)]])],
  }, absolute({ start: 0, top: 0 }));
  if (inside) {
    // The value over the fill at the track's end, in the HUD's outline and its soft halo, so it reads over any fill or
    // the empty track with no box behind it (a scrim there read as the fill darkening toward its end).
    box("Value", { width: pct(1), height: pct(1), direction: "row", align: desc.reverse ? "start" : "end", cross: "center", padding: { left: u(6), right: u(6), top: 0, bottom: 0 } }, undefined, () => {
      text("Text", inside.text, { step: inside.step, font: "mono", color: inside.critical ? lit(mix(role("danger"), WHITE, 0.35)) : token("ink"), nowrap: true, outline: 1 });
    }, absolute({ start: 0, top: 0 }));
  }
  // A slanted bar leans its whole track; its text sits outside it, upright.
  close(style === "slanted" ? { transform: { skewX: desc.reverse ? 0.36 : -0.36 } } : undefined);
}

function shieldRow(desc: Api.StatBarDesc, h: number, radius: number, style: Api.StatBarStyle): void {
  const smax = desc.shieldMax ?? 0;
  const shield = Math.max(0, Math.min(desc.shield ?? 0, smax));
  const tier = rarityColor(desc.shieldTier) ?? role("info");
  const cellH = Math.max(4, Math.round(h * 0.5));
  const per = desc.shieldCell ?? 0;
  const cells = per > 0 ? Math.max(1, Math.ceil(smax / per - 1e-6)) : 1;
  box("Shield", { direction: "row", gap: per > 0 ? u(2) : 0, width: { kind: "grow" } }, undefined, () => {
    for (let i = 0; i < cells; i++) {
      const lo = per > 0 ? i * per : 0;
      const hi = per > 0 ? Math.min(smax, lo + per) : smax;
      const part = clamp01((shield - lo) / Math.max(1e-6, hi - lo));
      const r = Math.min(radius, cellH / 2);
      box(kit.idi("Cell", i), { width: { kind: "grow" }, height: px(cellH), part: sel("StatBar", "cell", part >= 1 ? "filled" : null, ["tier", desc.shieldTier ?? "none"]), partClass: "bar" },
        trackDraw(r), () => {
          if (part > 0) leaf("Fill", { width: pct(part), height: pct(1) }, fillDraw(tier, r), absolute(desc.reverse ? { end: 0, top: 0 } : { start: 0, top: 0 }));
        });
    }
  }, style === "slanted" ? { transform: { skewX: desc.reverse ? 0.36 : -0.36 } } : undefined);
}

export function orbMeter(id: string, desc: Api.OrbMeterDesc = {}): void {
  const max = desc.max ?? 100;
  const value = desc.value ?? 0;
  const frac = max > 0 ? clamp01(value / max) : 0;
  const level = levelOf(frac, 0.3, 0.15);
  const kind = desc.kind ?? "health";
  const ink = desc.color ? paintRgba(desc.color, role("health")) : kind === "health" ? role("health") : role(kind);
  const size = u(desc.size || 112);
  const name = desc.name || desc.label || (kind === "mana" ? "Mana" : "Health");
  const anchor = anchorOf(desc, null);
  hudPlace(id, anchor, () => {
    kit.column(id, { gap: u(4), cross: "center" }, () => {
      const e = edge("v", value);
      const flash = e.changed || e.since < msToTicks(260) ? (value < e.prev ? 1 - e.since / msToTicks(260) : 0) : 0;
      box("Orb", { width: px(size), height: px(size), part: sel("OrbMeter", "frame", level), partClass: "hud" }, {
        shape: ellipseShape(),
        fills: [solid(token("shadow", 0.9))],
        strokes: [stroke(Math.max(3, size / 22), token("lineStrong")), stroke(1.5, token("sheen", 0.35), "outside")],
        effects: [drop(3, 10, token("shadow", 0.6)), ...(level === "critical" ? [glow(10 + 8 * pulse(2.2), lit(role("danger"), 0.65))] : [])],
        clip: "shape",
      }, () => {
        const shown = clamp01(spring("level", frac));
        const deep = mix(ink, { r: 0, g: 0, b: 0, a: 1 }, 0.55);
        const lift = mix(ink, { r: 1, g: 1, b: 1, a: 1 }, 0.25);
        // The liquid: from the bottom, its surface swaying.
        const sway = reduced() ? 0 : Math.sin(phase(0.35) * TAU);
        box("Liquid", { width: pct(1), height: pct(Math.max(0, shown)), part: sel("OrbMeter", "liquid", kind), partClass: "bar" }, {
          fills: [vgrad([[0, lit(lift)], [0.35, lit(ink)], [1, lit(deep)]]), ...(flash > 0 ? [solid(lit({ r: 1, g: 1, b: 1, a: 1 }, 0.5 * flash))] : [])],
        }, () => {
          if (shown > 0.02 && shown < 0.99) {
            leaf("Surface", { width: pct(1.3), height: px(Math.max(4, size * 0.07)), part: sel("OrbMeter", "surface") }, {
              shape: ellipseShape(),
              fills: [solid(lit(lift, 0.9))],
            }, absolute({ start: -size * 0.15, top: -size * 0.035 }, { transform: { x: sway * size * 0.05, rotate: sway * 0.06 } }));
          }
        }, absolute({ start: 0, bottom: 0 }));
        // The glass: a dark rim inside and a highlight at the top left.
        leaf("Glass", { width: pct(1), height: pct(1), part: sel("OrbMeter", "glass") }, {
          shape: ellipseShape(),
          fills: [radial([[0, token("sheen", 0)], [0.72, token("sheen", 0)], [1, token("shadow", 0.55)]])],
          effects: [inner(0, size * 0.08, token("shadow", 0.6))],
        }, absolute({ start: 0, top: 0 }));
        leaf("Highlight", { width: pct(0.42), height: pct(0.26) }, {
          shape: ellipseShape(),
          fills: [vgrad([[0, token("sheen", 0.55)], [1, token("sheen", 0)]])],
        }, absolute({ start: size * 0.2, top: size * 0.09 }, { transform: { rotate: -0.45 } }));
        if (desc.showValue !== false) {
          text("Value", String(Math.round(value)), { step: ts(size >= 96 ? 20 : 15, true), font: "display", color: token("ink"), nowrap: true, outline: 1.2 },
            absolute({ centerX: true, centerY: true }));
        }
      });
      if (desc.label) text("Label", desc.label, { step: ts(12), color: token("ink", 0.9), nowrap: true, outline: 1 });
    });
    apply({
      a11y: a11y("meter", name, { value: `${Math.round(value)} / ${Math.round(max)}`, valueMin: 0, valueMax: max, valueNow: value }),
      ...groups({ level: level === "empty" ? "critical" : level }, { level: ["normal", "low", "critical"] }),
    });
  });
}

export function radialMeter(id: string, desc: Api.RadialMeterDesc = {}): void {
  const wheels = Math.max(1, desc.wheels || 1);
  const value = clamp01((desc.value ?? 0) / wheels) * wheels;
  const full = value >= wheels - 1e-4;
  const exhausted = !!desc.exhausted;
  const lowFrac = value / wheels < 1 / 3;
  const level = exhausted ? "exhausted" : full ? "full" : lowFrac ? "low" : "normal";
  const size = u(desc.size || 56);
  const thick = Math.max(4, Math.round(size * 0.16));
  const gap = Math.max(2, Math.round(thick * 0.35));
  const base = desc.color ? paintRgba(desc.color, role("success")) : role("success");
  const ink = exhausted ? role("danger") : lowFrac ? role("warn") : base;
  // Fades a second after it fills; comes back the tick it is used.
  const t = now();
  const fullAt = kit.memory(`${id}~fullAt`, full ? -1e9 : 1e9);
  if (full && fullAt > t) kit.remember(`${id}~fullAt`, t);
  if (!full && fullAt <= t) kit.remember(`${id}~fullAt`, 1e9);
  const fadeX = full && desc.hideWhenFull !== false ? clamp01((t - Math.min(fullAt, t) - msToTicks(1000)) / msToTicks(400)) : 0;
  const outer = size + (wheels - 1) * (thick + gap) * 2;
  const anchor = anchorOf(desc, null);
  hudPlace(id, anchor, () => {
    box(id, { width: px(outer), height: px(outer) }, undefined, () => {
      for (let i = 0; i < wheels; i++) {
        const d = size + i * (thick + gap) * 2;
        const part = clamp01(value - i);
        const off = (outer - d) / 2;
        box(kit.idi("Wheel", i), { width: px(d), height: px(d), part: sel("RadialMeter", "wheel") }, undefined, () => {
          leaf("Track", { width: pct(1), height: pct(1), part: sel("RadialMeter", "track") }, {
            shape: arc(1, thick),
            fills: [solid(token("shadow", 0.6))],
            strokes: [stroke(1, token("lineStrong", 0.5), "outside")],
          }, absolute({ start: 0, top: 0 }));
          if (part > 0) {
            leaf("Fill", { width: pct(1), height: pct(1), part: sel("RadialMeter", "fill", exhausted ? "exhausted" : lowFrac ? "low" : "normal") }, {
              shape: arc(part, thick - 2, "butt"),
              fills: [solid(lit(ink))],
              effects: exhausted ? [glow(4 + 4 * pulse(2), lit(ink, 0.6))] : [],
            }, absolute({ start: 1, top: 1 }, { opacity: exhausted && !reduced() ? 0.7 + 0.3 * pulse(2) : 1 }));
          }
        }, absolute({ start: off, top: off }));
      }
    }, {
      opacity: 1 - fadeX,
      a11y: a11y("meter", desc.name ?? "Stamina", { value: `${Math.round((value / wheels) * 100)}%`, valueMin: 0, valueMax: wheels, valueNow: value }),
      ...groups({ level }, { level: ["full", "normal", "low", "exhausted"] }),
    });
  });
}

