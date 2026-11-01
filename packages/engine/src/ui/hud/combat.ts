// og.ui.crosshair, og.ui.ammoCounter, og.ui.killfeed, og.ui.damageDirection
// and og.ui.lowHealthVignette: the shooter rows of Appendix C. Read against
// Valorant's and Counter-Strike's crosshair editors (outlined lines round a
// gap that opens with spread, a T without its top line), Call of Duty's hit
// markers (four diagonal ticks springing out; larger and coloured for a
// headshot or a kill), Apex Legends' and Halo's ammo readouts (the
// magazine in large digits over its reserve, a row of rounds, low in warn
// with the reload hint), the battle royales' killfeeds (killer, weapon,
// victim on a dark chip each, the player's lines highlighted, the newest
// sliding in) and Call of Duty's damage wedges and red edges at low health.

import type * as Api from "@origingame/engine";
import {
  a11y, absolute, anchorOf, arc, box, chip, clamp01, easeOut, edge, ellipseShape, frame, glow, groups, hudPlace, icon, keyId, kit, leaf, lit,
  msToTicks, now, paintRgba, pct, pulse, px, reduced, role, sel, solid, sound, springTo, stroke, teamColor, text, token, ts, u, type Rgba,
} from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

/** A straight line between two points of the element's box, as a path part. */
function line(id: string, box: number, a: [number, number], b: [number, number], width: number, ink: Api.Paint, outline: boolean, part: string): void {
  leaf(id, { width: px(box), height: px(box), part }, {
    shape: { kind: "path", points: [{ x: a[0], y: a[1] }, { x: b[0], y: b[1] }], width, cap: "butt" },
    fills: [solid(ink)],
    strokes: outline ? [stroke(1, token("shadow", 0.85), "outside")] : [],
  }, absolute({ start: 0, top: 0 }));
}

export function crosshair(id: string, desc: Api.CrosshairDesc = {}): void {
  const style: Api.CrosshairStyle = desc.style ?? "cross";
  const len = desc.length || 8;
  const th = desc.thickness || 2;
  const gap0 = desc.gap || 6;
  const outline = desc.outline !== false;
  // The gap follows the spread on a fast spring, so firing blooms it and it settles back.
  const target = gap0 + Math.max(0, desc.spread ?? 0);
  const t = now();
  const hitAt = desc.hitAt ?? null;
  const hitSince = hitAt === null ? 1e9 : t - hitAt;
  const kind: Api.HitKind = desc.hit ?? "hit";
  const hitMs = kind === "kill" ? 520 : kind === "headshot" ? 320 : 220;
  const hitLive = hitSince >= 0 && hitSince < msToTicks(hitMs);
  const base = desc.color ? paintRgba(desc.color, WHITE) : WHITE;
  const ink = desc.onTarget ? role("danger") : base;
  const size = Math.ceil((target + len) * 2 + 24 + (hitLive ? 40 : 0));
  kit.anchor(`${id}Hud`, { anchor: desc.anchor ?? "center", margin: 0 }, () => {
    box(id, { width: px(size), height: px(size), part: sel("Crosshair", "frame", style) }, undefined, () => {
      const gap = reduced() ? target : springTo("gap", target, 140);
      const c = size / 2;
      const paint = lit(ink);
      const lp = sel("Crosshair", "line", desc.onTarget ? "onTarget" : null);
      if (style === "cross" || style === "tShape") {
        if (style === "cross") line("N", size, [c, c - gap - len], [c, c - gap], th, paint, outline, lp);
        line("S", size, [c, c + gap], [c, c + gap + len], th, paint, outline, lp);
        line("W", size, [c - gap - len, c], [c - gap, c], th, paint, outline, lp);
        line("E", size, [c + gap, c], [c + gap + len, c], th, paint, outline, lp);
      } else if (style === "circle") {
        const r = gap + len / 2;
        leaf("Ring", { width: px(r * 2), height: px(r * 2), part: sel("Crosshair", "ring") }, {
          shape: arc(1, th),
          fills: [solid(paint)],
          strokes: outline ? [stroke(1, token("shadow", 0.8), "outside")] : [],
        }, absolute({ start: c - r, top: c - r }));
      } else if (style === "chevron") {
        leaf("Chevron", { width: px(size), height: px(size), part: lp }, {
          shape: { kind: "path", points: [{ x: c - len, y: c + gap + len * 0.8 }, { x: c, y: c + gap * 0.5 }, { x: c + len, y: c + gap + len * 0.8 }], width: th, cap: "round", join: "miter" },
          fills: [solid(paint)],
          strokes: outline ? [stroke(1, token("shadow", 0.85), "outside")] : [],
        }, absolute({ start: 0, top: 0 }));
      }
      if (style === "dot" || desc.dot || style === "circle") {
        const d = Math.max(2, th + (style === "dot" ? 2 : 0));
        leaf("Dot", { width: px(d), height: px(d), part: sel("Crosshair", "dot") }, {
          shape: ellipseShape(),
          fills: [solid(paint)],
          strokes: outline ? [stroke(1, token("shadow", 0.85), "outside")] : [],
        }, absolute({ start: c - d / 2, top: c - d / 2 }));
      }
      if (hitLive) {
        const x = hitSince / msToTicks(hitMs);
        const big = kind === "hit" ? 1 : 1.4;
        const hitInk = kind === "kill" ? role("danger") : kind === "headshot" ? role("gold") : WHITE;
        const r0 = (gap + 4) * (reduced() ? 1 : 0.8 + 0.35 * easeOut(Math.min(1, x * 3)));
        const r1 = r0 + 7 * big;
        const fade = kind === "kill" ? 1 - clamp01((x - 0.6) / 0.4) : 1 - x * x;
        box("Marker", { width: px(size), height: px(size), part: sel("Crosshair", "marker", kind) }, undefined, () => {
          const s = Math.SQRT1_2;
          for (const [i, [dx, dy]] of [[-1, -1], [1, -1], [1, 1], [-1, 1]].entries()) {
            line(kit.idi("Tick", i), size, [c + dx! * r0 * s, c + dy! * r0 * s], [c + dx! * r1 * s, c + dy! * r1 * s], 2 * big, lit(hitInk), true, sel("Crosshair", "tick", kind));
          }
        }, absolute({ start: 0, top: 0 }, { opacity: fade }));
      }
    }, groups({ aim: desc.onTarget ? "onTarget" : "idle", hit: hitLive ? kind : "none" }, { aim: ["idle", "onTarget"], hit: ["none", "hit", "headshot", "kill"] }));
  });
}

export function ammoCounter(id: string, desc: Api.AmmoDesc = {}): void {
  const tk = frame().t;
  const mag = desc.magazine ?? 0;
  const magMax = Math.max(1, desc.magazineMax ?? (mag || 1));
  const reserve = desc.reserve ?? null;
  const reloading = desc.reloading ?? null;
  const frac = mag / magMax;
  const empty = mag === 0 && reloading === null;
  const low = !empty && reloading === null && frac <= (desc.low ?? 0.25);
  const state = reloading !== null ? "reloading" : empty ? "empty" : low ? "low" : "full";
  const ink = empty ? role("danger") : low ? role("warn") : WHITE;
  hudPlace(id, anchorOf(desc, "bottomRight"), () => {
    box(id, { direction: "column", gap: u(4), cross: "end", padding: { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm }, part: sel("Ammo", "frame", state === "full" ? null : state), partClass: "hud" }, chip(), () => {
      const e = edge("mag", mag);
      sound("warning", e.changed && mag === 0);
      if (desc.weapon || desc.icon || desc.fireMode) {
        kit.row("Weapon", { gap: tk.sm, cross: "center" }, () => {
          if (desc.icon) icon("Icon", desc.icon, u(22), token("ink"));
          if (desc.weapon) text("Name", desc.weapon, { step: ts(13), font: "display", color: token("inkMuted"), nowrap: true, outline: 0.8 });
          if (desc.fireMode) {
            box("Mode", { padding: { left: u(5), right: u(5), top: 0, bottom: 0 }, part: sel("Ammo", "mode"), partClass: "chip" }, {
              shape: { kind: "rect", radii: [3, 3, 3, 3] },
              strokes: [stroke(1, token("lineStrong"))],
            }, () => text("Text", desc.fireMode ?? "", { step: ts(10), font: "mono", color: token("inkMuted"), nowrap: true }));
          }
        });
      }
      kit.row("Count", { gap: tk.sm, cross: "end" }, () => {
        const shake = e.changed || e.since < msToTicks(90) ? (e.prev > mag ? 1 - e.since / msToTicks(90) : 0) : 0;
        if (reloading !== null) {
          box("Reload", { width: px(u(40)), height: px(u(40)), align: "center", cross: "center", part: sel("Ammo", "reload") }, undefined, () => {
            leaf("Track", { width: pct(1), height: pct(1) }, { shape: arc(1, u(4)), fills: [solid(token("shadow", 0.6))] }, absolute({ start: 0, top: 0 }));
            leaf("Fill", { width: pct(1), height: pct(1) }, { shape: arc(clamp01(reloading), u(4)), fills: [solid(lit(role("primary")))] }, absolute({ start: 0, top: 0 }));
          });
          text("ReloadLabel", desc.reloadLabel ?? "Reloading", { step: ts(16), font: "display", color: token("ink"), nowrap: true, outline: 1 });
        } else {
          text("Magazine", String(mag), { step: ts(40, true), font: "display", color: lit(ink), nowrap: true, outline: 1.4 }, {
            transform: { y: reduced() ? 0 : -2 * shake, scaleX: 1 + 0.06 * shake, scaleY: 1 + 0.06 * shake, pivotX: 1, pivotY: 1 },
            ...(empty ? { opacity: 0.65 + 0.35 * pulse(2) } : {}),
          });
        }
        text("Slash", "/", { step: ts(18), font: "display", color: token("inkFaint"), nowrap: true, outline: 1 });
        text("Reserve", reserve === null ? "∞" : String(reserve), { step: ts(18), font: "display", color: reserve === 0 ? lit(role("danger")) : token("inkMuted"), nowrap: true, outline: 1 });
      });
      if (desc.pips !== false && magMax <= 40 && reloading === null) {
        const w = Math.max(3, Math.min(u(6), Math.floor(u(220) / magMax) - 2));
        kit.row("Rounds", { gap: u(2), cross: "end" }, () => {
          for (let i = 0; i < magMax; i++) {
            // Rounds empty from the left, as a magazine strip reads.
            const spent = i < magMax - mag;
            const leaving = !spent ? 0 : i === magMax - mag - 1 && e.since < msToTicks(220) && e.prev > mag ? 1 - e.since / msToTicks(220) : 0;
            leaf(kit.idi("Round", i), { width: px(w), height: px(u(14)), part: sel("Ammo", "round", spent ? "spent" : null) }, {
              shape: { kind: "rect", radii: [w / 2, w / 2, 1, 1] },
              fills: [solid(spent ? token("inkFaint", 0.35) : lit(ink))],
              strokes: [stroke(1, token("shadow", 0.7), "outside")],
            }, leaving > 0 && !reduced() ? { transform: { y: -u(10) * (1 - leaving), rotate: 0.6 * (1 - leaving) }, opacity: leaving } : undefined);
          }
        });
      }
      if ((low || empty) && desc.reloadHint) {
        text("Hint", desc.reloadHint, { step: ts(13), font: "display", color: lit(ink), nowrap: true, outline: 1 }, { opacity: 0.6 + 0.4 * pulse(1.4) });
      }
    }, {
      a11y: a11y("status", "Ammunition", { value: `${mag} / ${reserve === null ? "infinite" : reserve}` }),
      ...groups({ ammo: state }, { ammo: ["full", "low", "empty", "reloading"] }),
    });
  });
}

export function killfeed(id: string, desc: Api.KillfeedDesc = {}): void {
  const tk = frame().t;
  const phone = frame().phone;
  const t = now();
  const life = msToTicks(desc.durationMs || 6000);
  const max = desc.max || (phone ? 3 : 5);
  const live = (desc.entries ?? []).filter((x) => t - (x.at ?? 0) < life && t >= (x.at ?? 0));
  const shown = live.slice(-max).reverse();
  // A phone's top edge is the compass's and the timer's: in portrait the feed
  // stacks under them, in landscape it sits mid-right, clear of the crosshair.
  const place: Api.Anchor = !phone ? "topRight" : frame().viewport.breakpoint === "phonePortrait" ? "top" : "right";
  hudPlace(id, anchorOf(desc, place), () => {
    box(id, { direction: "column", gap: u(4), cross: "end", part: sel("Killfeed", "frame") }, undefined, () => {
      for (const x of shown) {
        const age = t - (x.at ?? 0);
        const enter = reduced() ? 1 : easeOut(age / msToTicks(220));
        const leave = clamp01((life - age) / msToTicks(400));
        box(keyId(x.key ?? String(x.at)), {
          direction: "row", gap: tk.sm, cross: "center", padding: { left: tk.sm, right: tk.sm, top: u(3), bottom: u(3) },
          part: sel("Killfeed", "line", x.local ? "local" : null), partClass: "chip",
        }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] },
          fills: [solid(x.local ? token("highlight", 0.28) : token("shadow", 0.62))],
          strokes: x.local ? [stroke(1.5, token("highlight"))] : [],
        }, () => {
          if (x.killer) {
            text("Killer", x.killer, { step: ts(13), font: "display", color: lit(teamColor(x.killerTeam ?? 0)), nowrap: true, outline: 0.8 });
            if (x.assist) text("Assist", `+ ${x.assist}`, { step: ts(11), color: token("inkMuted"), nowrap: true });
          }
          if (x.weapon && kit.hasIcon(x.weapon)) icon("Weapon", x.weapon, u(20), token("ink"), x.weaponText || x.weapon);
          else text("Weapon", x.weaponText || x.weapon || "›", { step: ts(12), font: "mono", color: token("inkMuted"), nowrap: true });
          if (x.headshot) icon("Headshot", "target", u(14), lit(role("gold")), "Headshot");
          text("Victim", x.victim ?? "", { step: ts(13), font: "display", color: lit(teamColor(x.victimTeam ?? 1)), nowrap: true, outline: 0.8 });
        }, {
          transform: { x: (1 - enter) * u(60) },
          opacity: Math.min(enter, leave),
          a11y: a11y("text", [x.killer, x.weaponText || x.weapon, x.victim].filter(Boolean).join(" ")),
        });
      }
    }, { a11y: a11y("log", "Killfeed") });
  });
}

export function damageDirection(id: string, desc: Api.DamageDirectionDesc = {}): void {
  const f = frame();
  const t = now();
  const life = msToTicks(desc.durationMs || 1400);
  const radius = desc.radius || Math.round(Math.min(f.viewport.width, f.viewport.height) / 5);
  const ink = desc.color ? paintRgba(desc.color, role("danger")) : role("danger");
  const hits = (desc.hits ?? []).filter((h) => t - (h.at ?? 0) < life && t >= (h.at ?? 0)).slice(-8);
  const size = radius * 2 + 40;
  kit.anchor(`${id}Hud`, { anchor: "center", margin: 0 }, () => {
    box(id, { width: px(size), height: px(size), part: sel("DamageDirection", "frame") }, undefined, () => {
      hits.forEach((h, i) => {
        const age = (t - (h.at ?? 0)) / life;
        const s = clamp01(h.strength ?? 1);
        const half = (0.22 + 0.22 * s);
        // In fast, holding, then fading out over the last half.
        const flare = reduced() ? 1 : Math.min(1, age * 10);
        const alpha = flare * (age < 0.5 ? 1 : 1 - (age - 0.5) / 0.5) * (0.55 + 0.45 * s);
        const start = (h.angle ?? 0) - Math.PI / 2 - half;
        const th = Math.round(10 + 8 * s);
        leaf(kit.idi("Hit", i), { width: px(radius * 2), height: px(radius * 2), part: sel("DamageDirection", "wedge") }, {
          shape: { kind: "arc", start, sweep: half * 2, thickness: th, cap: "butt" },
          fills: [solid(lit(ink, alpha))],
          effects: [glow(10, lit(ink, alpha * 0.7))],
        }, absolute({ start: 20, top: 20 }, { transform: { scaleX: 1 + 0.06 * (1 - flare), scaleY: 1 + 0.06 * (1 - flare) } }));
      });
    });
  });
}

export function lowHealthVignette(id: string, desc: Api.LowHealthDesc = {}): void {
  const hp = clamp01(desc.health ?? 1);
  const threshold = desc.threshold ?? 0.35;
  const color = desc.color ?? { color: { r: 0.62, g: 0.02, b: 0.04 } };
  const depth = hp < threshold ? 1 - hp / Math.max(1e-6, threshold) : 0;
  if (depth > 0) {
    // Deeper as health falls; the heartbeat quickens toward death.
    kit.meta(id, {
      kind: "vignette",
      color,
      inner: 0.62 - 0.32 * depth,
      outer: 1,
      intensity: 0.35 + 0.6 * depth,
      pulseHz: reduced() ? 0 : 0.9 + 1.4 * depth,
      pulseDepth: 0.35,
    });
  }
  const hitAt = desc.hitAt ?? null;
  if (hitAt !== null) {
    const since = now() - hitAt;
    const dur = msToTicks(320);
    if (since >= 0 && since < dur) {
      kit.meta(`${id}Hit`, { kind: "vignette", color, inner: 0.5, outer: 1, intensity: 0.7 * (1 - since / dur), pulseHz: 0 });
    }
  }
}

