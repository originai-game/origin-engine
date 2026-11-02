// og.ui.unitFrame, og.ui.partyFrames, og.ui.raidFrames and og.ui.castBar:
// Appendix C's "unit, party and raid frames" and "cast bars". Read against
// World of Warcraft's player, target and party frames (a round portrait
// ringed in gold for an elite, the level on its badge, health with its
// absorb, the resource under it) and its compact raid frames (cells of
// class colour in columns of five, the absorb and the incoming heal, a
// dispellable debuff glowing on the border, faded out of range), Final
// Fantasy XIV's party list, and the cast bars of both (a bright spark at
// the fill's edge, the latency zone, a channel draining with its ticks,
// a grey shield on what cannot be interrupted, interrupted in red).

import type * as Api from "@origingame/engine";
import { formatNumber, hudRoom, shrunk, artTile, socketRim, hueOf, BLACK, WHITE, shapeOf, a11y, absolute, anchorOf, box, inner, radial, innerGlow, reduced, clamp01, close, drop, easeOut, edge, fillDraw, frame, glow, groups, hudPlace, icon, keyId, kit, leaf, lit, mix, msToTicks, now, open, paintRgba, pct, px, role, sel, solid, sound, stroke, text, token, trackDraw, ts, u, vgrad, ellipseShape, type Rgba, apply, textScale, stepSize } from "./core.ts";
import { statBar } from "./meters.ts";
import { statusGrid } from "./abilities.ts";

const NO_RESPONSE: Api.Response = { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0 };

/** A colour from a name, so a unit without a class colour keeps one across frames. */
function nameHue(name: string): Rgba {
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  const hue = (h % 360) / 360;
  const f = (n: number) => {
    const k = (n + hue * 12) % 12;
    return 0.45 - 0.3 * Math.max(-1, Math.min(k - 3, 9 - k, 1));
  };
  return { r: f(0), g: f(8), b: f(4), a: 1 };
}

const SLATE: Rgba = { r: 0.36, g: 0.4, b: 0.47, a: 1 };

/** A skull drawn from shapes (cranium, jaw, eyes), the same in every style whatever its icon set has. */
function skull(side: number): void {
  const ink = lit({ r: 0.86, g: 0.87, b: 0.9, a: 1 });
  const hole = lit(BLACK, 0.85);
  box("Skull", { width: px(side), height: px(side) }, undefined, () => {
    leaf("Cranium", { width: px(side), height: px(Math.round(side * 0.78)) }, { shape: ellipseShape(), fills: [solid(ink)], effects: [drop(1, 3, lit(BLACK, 0.8))] }, absolute({ start: 0, top: 0 }));
    leaf("Jaw", { width: px(Math.round(side * 0.5)), height: px(Math.round(side * 0.3)) }, { shape: { kind: "rect", radii: [2, 2, side * 0.12, side * 0.12] }, fills: [solid(ink)] }, absolute({ centerX: true, bottom: 0 }));
    const eye = Math.round(side * 0.24);
    leaf("EyeL", { width: px(eye), height: px(eye) }, { shape: ellipseShape(), fills: [solid(hole)] }, absolute({ start: Math.round(side * 0.2), top: Math.round(side * 0.34) }));
    leaf("EyeR", { width: px(eye), height: px(eye) }, { shape: ellipseShape(), fills: [solid(hole)] }, absolute({ end: Math.round(side * 0.2), top: Math.round(side * 0.34) }));
  });
}

/** Health as a frame writes it: grouped under a hundred thousand, compact above (by the larger of the two, so both halves match). */
function hpText(v: number, max: number): string {
  return formatNumber(Math.round(v), { notation: max >= 100_000 ? "compact" : "grouped" });
}

const SIZES: Record<Api.UnitFrameSize, { portrait: number; width: number; bar: number; name: number }> = {
  large: { portrait: 64, width: 232, bar: 16, name: 15 },
  medium: { portrait: 44, width: 176, bar: 12, name: 13 },
  small: { portrait: 30, width: 120, bar: 8, name: 12 },
};

export function unitFrame(id: string, desc: Api.UnitFrameDesc = {}): Api.Response {
  const tk = frame().t;
  const size: Api.UnitFrameSize = desc.size ?? "large";
  const m = SIZES[size];
  // The portrait grows with the player's text size (half again at 200 %), so its badge and crown keep their room on it.
  const pd = Math.round(u(m.portrait) * (1 + (Math.min(2, textScale()) - 1) / 2));
  const dead = !!desc.dead;
  const offline = !!desc.offline;
  const hostile = !!desc.hostile;
  const cls: Api.UnitClass = desc.unitClass ?? "normal";
  const life = dead ? "dead" : offline ? "offline" : "alive";
  const name = desc.name ?? "";
  const health = desc.health ?? 0;
  const healthMax = desc.healthMax ?? 100;
  const ringInk = cls === "elite" || cls === "boss" ? role("gold") : cls === "rare" ? { r: 0.82, g: 0.86, b: 0.92, a: 1 } : hostile ? role("danger") : role("lineStrong");
  const anchor = anchorOf(desc, null);
  const corner = anchor === "topLeft" || anchor === "topRight" || anchor === "bottomLeft" || anchor === "bottomRight";
  let res: Api.Response = NO_RESPONSE;
  hudPlace(id, anchor, () => {
    // A hit: the health fell this tick. For 450 ms the frame's edge flashes
    // danger and the portrait shakes, read at a glance in the corner of the eye.
    const hp = edge(`${id}:hp`, dead ? 0 : health);
    if (hp.changed && health < hp.prev && !dead) kit.remember(`${id}:hurtAt`, now());
    const hurtAge = now() - kit.memory(`${id}:hurtAt`, -1e9);
    const hurt = hurtAge < msToTicks(450) ? 1 - hurtAge / msToTicks(450) : 0;
    // No plate behind the frame (§15.5.6): the portrait, the name and the bars float on their own shadows.
    // Targeted lights the portrait's ring with the highlight and a glow; a hit flashes it danger.
    const ring = desc.targeted ? role("highlight") : ringInk;
    // The level badge at most 40 % of the portrait, seated on its rim; its figure shrunk to fit at any text size.
    const badgeH = Math.round(Math.min(pd * 0.4, Math.max(14, pd * 0.3) * textScale()));
    open(id, {
      direction: "row",
      gap: tk.sm,
      cross: "start",
      interactive: true,
      padding: { left: u(6), right: tk.md, top: u(6), bottom: u(6) + (desc.level ? Math.round(badgeH * 0.5) : 0) },
      // At a top or bottom corner a frame takes at most half the HUD band, so a player's and a target's frames sit side by side.
      width: px(corner ? Math.min(u(m.width) + pd, (hudRoom() - tk.sm) / 2) : u(m.width) + pd),
      part: sel("UnitFrame", "frame", size, hostile ? "hostile" : null, desc.targeted ? "targeted" : null, dead ? "dead" : null),
      partClass: "hud",
    });
    const portrait = () => {
    // The portrait: round, ringed by its class, the level on a badge centred at its foot and the crown on its
    // shoulder, all held by the avatar so they stay with it at any text size or frame height.
    open("Avatar", { width: px(pd), height: px(pd) });
    box("Portrait", { width: px(pd), height: px(pd), align: "center", cross: "center", part: sel("UnitFrame", "portrait", cls), partClass: "avatar" }, {
      shape: ellipseShape(),
      // Behind the art (and the placeholder's ground): the unit's hue, muted toward slate so a bust never reads as a tinted state.
      fills: [radial([[0, lit(mix(mix(nameHue(name), SLATE, 0.6), WHITE, 0.1))], [0.7, lit(mix(mix(nameHue(name), SLATE, 0.6), BLACK, 0.4))], [1, lit(mix(SLATE, BLACK, 0.65))]], 0.5, 0.3, 0.8, 0.8)],
      strokes: [stroke(Math.max(2, pd / 18) + (desc.targeted ? 1 : 0), lit(hurt > 0 ? mix(ring, role("danger"), hurt) : ring)), stroke(1, lit(BLACK, 0.8), "outside")],
      effects: [
        drop(u(2), u(8), lit(BLACK, 0.55)),
        ...(cls === "boss" || cls === "elite" ? [glow(u(5), lit(ringInk, 0.5))] : []),
        ...(desc.targeted ? [glow(u(7), lit(role("highlight"), 0.55))] : []),
        ...(hurt > 0 ? [glow(u(10) * hurt, lit(role("danger"), 0.75 * hurt))] : []),
      ],
      clip: "shape",
    }, () => {
      if (desc.portrait) {
        kit.image("Art", { texture: desc.portrait, width: px(pd), height: px(pd), label: name });
        apply(absolute({ start: 0, top: 0 }, {}));
      } else if (desc.icon && kit.hasIcon(desc.icon)) icon("Art", desc.icon, Math.round(pd * 0.56), lit(WHITE, 0.95));
      else {
        // No art: a bust's silhouette on the unit's colour (§15.5.3: never initials in an empty slot).
        const ink = lit(mix(SLATE, WHITE, 0.45), 0.85);
        leaf("Head", { width: px(Math.round(pd * 0.34)), height: px(Math.round(pd * 0.38)) }, { shape: ellipseShape(), fills: [solid(ink)] }, absolute({ centerX: true, top: Math.round(pd * 0.2) }));
        leaf("Shoulders", { width: px(Math.round(pd * 0.7)), height: px(Math.round(pd * 0.5)) }, { shape: ellipseShape(), fills: [solid(ink)] }, absolute({ centerX: true, top: Math.round(pd * 0.62) }));
      }
      // Set into its ring: an inner shadow and a soft vignette over the art.
      leaf("Inset", { width: pct(1), height: pct(1) }, { shape: ellipseShape(), effects: [inner(1, 2, lit(BLACK, 0.7), 1), innerGlow(pd * 0.12, lit(BLACK, 0.35))] }, absolute({ start: 0, top: 0 }));
      if (hurt > 0 && !reduced()) {
        leaf("Hurt", { width: pct(1), height: pct(1) }, { fills: [solid(lit(role("danger"), 0.5 * hurt))] }, absolute({ start: 0, top: 0 }));
      }
      if (dead) {
        // Dead: the portrait itself, its colour taken by a neutral grey and its light to about 40 %, a small skull on it.
        leaf("Grey", { width: pct(1), height: pct(1), part: sel("UnitFrame", "dead") }, { fills: [solid(lit({ r: 0.2, g: 0.2, b: 0.21, a: 1 }, 0.72))] }, absolute({ start: 0, top: 0 }));
        leaf("Dim", { width: pct(1), height: pct(1) }, { fills: [solid(lit({ r: 0.55, g: 0.55, b: 0.56, a: 1 }))], blend: "multiply" }, absolute({ start: 0, top: 0 }));
        box("Dead", { width: pct(1), height: pct(1), align: "center", cross: "center" }, undefined, () => {
          skull(Math.round(pd * 0.32));
        }, absolute({ start: 0, top: 0 }));
      }
    });
    const portraitPath = `${kit.openPath()}/Portrait`;
    if (desc.level) {
      box("Level", { padding: { left: Math.round(badgeH * 0.25), right: Math.round(badgeH * 0.25), top: 0, bottom: 0 }, height: px(badgeH), align: "center", cross: "center", part: sel("UnitFrame", "level"), partClass: "badge" }, {
        shape: { kind: "rect", radii: [999, 999, 999, 999] },
        fills: [solid(token("hud", 0.98))],
        strokes: [stroke(1.5, lit(ringInk))],
      }, () => {
        const step = ts(11);
        const fit = shrunk(step, badgeH * 0.62, stepSize(step).line);
        text("Text", desc.level ?? "", { step, font: "display", color: hostile ? lit(role("danger")) : token("ink"), nowrap: true }, fit.transform ? { transform: { ...fit.transform, y: 0 } } : undefined);
      }, absolute({ centerX: true, bottom: -Math.round(badgeH / 2) }, { z: 2, overlaps: [portraitPath] }));
    }
    if (desc.leader) {
      // Seated on the rim at its upper outer corner (45°), sized by the portrait.
      const cs = Math.round(pd * 0.34);
      const at = Math.round(pd * 0.146 - cs / 2);
      box("Crown", { width: px(cs), height: px(cs), align: "center", cross: "center", part: sel("UnitFrame", "crown") }, undefined, () => {
        icon("Mark", "crown", Math.round(cs * 0.9), lit(role("gold")));
      }, absolute(desc.reverse ? { end: at, top: at } : { start: at, top: at }, { z: 2, overlaps: [portraitPath] }));
    }
    close(hurt > 0 && !reduced() ? { transform: { x: Math.sin(hurtAge * 1.9) * u(3) * hurt } } : undefined);
    };
    if (!desc.reverse) portrait();
    kit.column("Bars", { gap: u(3), width: { kind: "grow" } }, () => {
      kit.row("Head", { gap: tk.xs, cross: "center", width: { kind: "grow" } }, () => {
        text("Name", name, { step: ts(m.name), font: "display", color: dead || offline ? token("inkMuted") : hostile ? lit(mix(role("danger"), WHITE, 0.15)) : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, soft: true });
        if (desc.combat) icon("Combat", "crossed-swords", u(14), lit(role("danger")), "In combat");
      });
      if (desc.title && size !== "small") text("Title", desc.title, { step: ts(11), color: token("ink", 0.9), nowrap: true, ellipsis: true, width: { kind: "grow" }, soft: true });
      statBar("Health", {
        value: dead ? 0 : health,
        max: healthMax,
        kind: "health",
        ...(hostile ? { color: token("danger") } : {}),
        // The absorb lies on the health as a white-blue overlay (World of Warcraft, Diablo IV), never a bar of its own.
        absorb: dead ? 0 : desc.shield ?? 0,
        // Health is the frame's dominant bar: full length, the tallest, its numerals inside its end.
        thickness: m.bar,
        width: { kind: "grow" },
        showValue: size === "large",
        valueInside: true,
        // A boss's millions shortened as shipped frames do (800K / 1.25M); a hero's thousands grouped (34,960 / 38,000).
        valueText: size === "large" ? `${hpText(health, healthMax)} / ${hpText(healthMax, healthMax)}` : "",
        name: `${name} health`,
        anchor: null,
      });
      if (desc.resource !== null && desc.resource !== undefined && !dead) {
        statBar("Resource", { value: desc.resource, max: desc.resourceMax ?? 100, kind: desc.resourceKind ?? "mana", thickness: Math.max(5, Math.round(m.bar * 0.55)), width: { kind: "grow" }, ghost: false, name: `${name} resource`, anchor: null });
      }
      if (desc.castName && size !== "small") {
        cast("Cast", { name: desc.castName, progress: desc.castProgress ?? 0, width: { kind: "grow" } }, true);
      }
      if ((desc.effects ?? []).length > 0 && size !== "small") statusGrid("Effects", { effects: desc.effects ?? [], size: size === "large" ? 22 : 18, columns: 8, timers: false, split: false, anchor: null });
    });
    if (desc.reverse) portrait();
    res = close({
      opacity: offline ? 0.5 : 1,
      a11y: a11y("button", name, { value: dead ? "Dead" : offline ? "Offline" : `${desc.level ? `Level ${desc.level}, ` : ""}${Math.round((health / Math.max(1, healthMax)) * 100)}% health`, selected: !!desc.targeted }),
      ...groups({ life, selection: desc.targeted ? "targeted" : "none", hurt: hurt > 0 ? "hit" : "none" }, { life: ["alive", "dead", "offline"], selection: ["none", "targeted"], hurt: ["none", "hit"] }),
    });
  });
  return res;
}

export function partyFrames(id: string, desc: Api.PartyFramesDesc = {}): Api.FramesResult {
  const tk = frame().t;
  let clicked: string | null = null;
  hudPlace(id, anchorOf(desc, "left"), () => {
    box(id, { direction: "column", gap: desc.gap || tk.sm, part: sel("PartyFrames", "frame") }, undefined, () => {
      for (const m of desc.members ?? []) {
        const r = unitFrame(keyId(m.key ?? m.name ?? ""), { ...m, size: m.size ?? "medium", anchor: null });
        if (r.clicked) clicked = m.key ?? "";
      }
    }, { a11y: a11y("list", "Party") });
  });
  return { clicked };
}

const ROLE_ICON: Record<Api.RaidRole, string> = { none: "", tank: "shield", healer: "plus", damage: "sword" };

export function raidFrames(id: string, desc: Api.RaidFramesDesc = {}): Api.FramesResult {
  const per = Math.max(1, desc.columns || 5);
  const cw = u(desc.cellWidth || 96);
  const ch = u(desc.cellHeight || 36);
  const members = (desc.members ?? []).slice(0, 40);
  let clicked: string | null = null;
  hudPlace(id, anchorOf(desc, "left"), () => {
    box(id, { direction: "row", gap: u(4), cross: "start", part: sel("RaidFrames", "frame") }, undefined, () => {
      for (let g = 0; g * per < members.length; g++) {
        kit.column(kit.idi("Group", g), { gap: u(2) }, () => {
          for (const m of members.slice(g * per, g * per + per)) {
            if (cell(keyId(m.key ?? m.name ?? ""), m, cw, ch).clicked) clicked = m.key ?? "";
          }
        });
      }
    }, { a11y: a11y("table", "Raid") });
  });
  return { clicked };
}

function cell(id: string, m: Api.RaidMember, w: number, h: number): Api.Response {
  const max = Math.max(1, m.healthMax ?? 100);
  const hp = m.dead || m.offline ? 0 : clamp01((m.health ?? 0) / max);
  const absorb = clamp01((m.absorb ?? 0) / max);
  const heal = clamp01((m.incomingHeal ?? 0) / max);
  const cls = paintRgba(m.classColor, role("health"));
  const low = hp < 0.35 && hp > 0;
  const state = m.dead ? "dead" : m.offline ? "offline" : m.outOfRange ? "outOfRange" : null;
  const dispel = m.dispel ? DISPEL[m.dispel] ?? role("danger") : null;
  const border = m.selected ? role("highlight") : m.aggro ? role("danger") : dispel;
  open(id, {
    width: px(w), height: px(h), interactive: true, cross: "center",
    part: sel("RaidFrames", "cell", state, m.aggro ? "aggro" : null, m.dispel ? ["dispel", m.dispel] : null, m.selected ? "selected" : null),
    partClass: "bar",
  }, {
    ...trackDraw(3),
    strokes: border ? [stroke(m.selected ? 2 : 1.5, lit(border, dispel && !m.selected && !m.aggro ? 0.6 + 0.4 * (0.5 + 0.5 * Math.cos(kit.seconds() * 6)) : 1))] : [stroke(1, token("shadow", 0.9), "outside")],
    effects: dispel ? [glow(4, lit(dispel, 0.6))] : [],
  });
  if (hp > 0) leaf("Fill", { width: pct(hp), height: pct(1), part: sel("RaidFrames", "fill", low ? "low" : null) }, fillDraw(cls, 2), absolute({ start: 0, top: 0 }));
  if (heal > 0 && hp < 1) leaf("Heal", { width: pct(Math.min(heal, 1 - hp)), height: pct(1), part: sel("RaidFrames", "heal") }, { fills: [solid(lit(role("success"), 0.42))] }, absolute({ start: hp * w, top: 0 }));
  if (absorb > 0) {
    // The absorb: a white sheen over the end of the health it covers (WoW's), striped so it reads without colour.
    leaf("Absorb", { width: pct(Math.min(absorb, 1)), height: pct(1), part: sel("RaidFrames", "absorb") }, {
      fills: [solid(token("sheen", 0.35))],
      strokes: [stroke(1, token("sheen", 0.65))],
    }, absolute({ start: Math.max(0, Math.min(hp + heal, 1) - absorb) * w, top: 0 }));
  }
  const ink = m.dead || m.offline ? token("inkFaint") : token("ink");
  kit.row("Label", { gap: u(3), cross: "center", padding: { left: u(5), right: u(4), top: 0, bottom: 0 }, width: pct(1) }, () => {
    const roleName = ROLE_ICON[m.role ?? "none"];
    if (roleName) icon("Role", roleName, Math.max(10, Math.round(h * 0.34)), token("ink"), m.role ?? "");
    // At a large text size the status takes its own line under the name, so neither is cut to nothing.
    const stacked = textScale() >= 1.5 && !!(m.dead || m.offline);
    if (stacked) open("Names", { direction: "column", gap: 0, width: { kind: "grow" } });
    text("Name", m.name ?? "", { step: ts(11), color: ink, nowrap: true, ellipsis: true, width: { kind: "grow" }, soft: true });
    if (m.dead || m.offline) text("Status", m.dead ? "Dead" : "Offline", { step: ts(10), color: token("inkMuted"), nowrap: true, ellipsis: stacked });
    if (stacked) close();
  });
  apply(absolute({ start: 0, top: 0, bottom: 0 }));
  return close({
    opacity: m.outOfRange ? 0.42 : 1,
    a11y: a11y("button", m.name ?? "", { value: m.dead ? "Dead" : m.offline ? "Offline" : `${Math.round(hp * 100)}%`, selected: !!m.selected }),
  });
}

const DISPEL: Record<string, Rgba> = {
  magic: { r: 0.24, g: 0.55, b: 1, a: 1 },
  curse: { r: 0.65, g: 0.3, b: 1, a: 1 },
  poison: { r: 0.24, g: 0.81, b: 0.31, a: 1 },
  disease: { r: 0.78, g: 0.6, b: 0.17, a: 1 },
};

export function castBar(id: string, desc: Api.CastBarDesc = {}): void {
  cast(id, desc, false);
}

/** A cast bar; compact is the thin one under a unit frame's bars. */
function cast(id: string, desc: Api.CastBarDesc, compact: boolean): void {
  const tk = frame().t;
  const state: Api.CastState = desc.state ?? "casting";
  const t = now();
  const ended = state !== "casting";
  const since = ended ? t - (desc.stateAt ?? t) : 0;
  const hold = msToTicks(state === "success" ? 250 : 700);
  const fade = msToTicks(300);
  const gone = ended && since >= hold + fade;
  const e = edge(`${id}~state`, ["casting", "success", "interrupted", "failed"].indexOf(state));
  sound("error", e.changed && (state === "interrupted" || state === "failed"));
  if (gone) return;
  const alpha = ended ? 1 - clamp01((since - hold) / fade) : 1;
  const p = clamp01(desc.progress ?? 0);
  const bad = state === "interrupted" || state === "failed";
  const ink = bad ? role("danger") : state === "success" ? role("success") : desc.uninterruptible ? { r: 0.62, g: 0.64, b: 0.68, a: 1 } : desc.channel ? role("info") : role("gold");
  // Compact (under a unit frame): the name inside a bar as tall as its line. Full: the name and seconds on a line
  // of their own over a slim bar, the spell's art beside both (Diablo IV, Destiny 2, WoW's modern cast bar).
  const nameStep = compact ? ts(10) : ts(14);
  const h = compact ? Math.max(u(10), stepSize(nameStep).line) : Math.max(u(8), Math.round(stepSize(nameStep).size * 0.55));
  const label = state === "interrupted" ? desc.interruptedLabel ?? "Interrupted" : state === "failed" ? desc.failedLabel ?? "Failed" : desc.name ?? "";
  const shown = bad ? 1 : state === "success" ? 1 : desc.channel ? 1 - p : p;
  const bar = (withText: boolean) => {
    const trackPath = open("Track", { width: { kind: "grow" }, height: px(h), cross: "center", part: sel("CastBar", "track"), partClass: "bar" }, {
      ...trackDraw(Math.min(h / 2, u(3))),
      ...(desc.uninterruptible ? { strokes: [stroke(1.5, token("inkMuted"), "outside")] } : {}),
      // Compact, the name sits inside the bar: the bar clips it (an ellipsis first), so no text leaves its box.
      ...(withText ? { clip: "shape" as const } : {}),
    });
    const w = kit.bounds(trackPath)?.width ?? u(240);
    leaf("Fill", { width: pct(shown), height: pct(1), part: sel("CastBar", "fill") }, fillDraw(ink, Math.min(h / 2, u(3))), absolute({ start: 0, top: 0 }));
    if (!ended && shown > 0.01 && shown < 0.995 && withText) {
      // Under its name: a faint edge, never a bright tick across the letters.
      leaf("Spark", { width: px(2), height: pct(1), part: sel("CastBar", "spark") }, { fills: [solid(lit(WHITE, 0.45))] }, absolute({ start: shown * w - 1, top: 0 }));
    } else if (!ended && shown > 0.01 && shown < 0.995) {
      // The spark at the leading edge.
      leaf("Spark", { width: px(u(10)), height: pct(1.8), part: sel("CastBar", "spark") }, {
        shape: ellipseShape(),
        fills: [vgrad([[0, lit(WHITE, 0)], [0.5, lit(WHITE, 0.95)], [1, lit(WHITE, 0)]], 90)],
        blend: "screen",
      }, absolute({ start: shown * w - u(5), top: -h * 0.4 }));
    }
    if ((desc.latency ?? 0) > 0 && !desc.channel && !ended) {
      leaf("Latency", { width: pct(clamp01(desc.latency ?? 0)), height: pct(1), part: sel("CastBar", "latency") }, { fills: [solid(lit(mix(ink, WHITE, 0.6), 0.5))] }, absolute({ end: 0, top: 0 }));
    }
    if (desc.channel && (desc.ticks ?? 0) > 1) {
      for (let i = 1; i < (desc.ticks ?? 0); i++) {
        leaf(kit.idi("Tick", i), { width: px(2), height: pct(1), part: sel("CastBar", "tick") }, { fills: [solid(lit(BLACK, 0.85))] }, absolute({ start: (i / (desc.ticks ?? 1)) * w - 1, top: 0 }));
      }
    }
    if (state === "success" && since < hold) {
      leaf("Flash", { width: pct(1), height: pct(1) }, { fills: [solid(lit(WHITE, 0.6 * (1 - since / hold)))] }, absolute({ start: 0, top: 0 }));
    }
    if (withText) {
      kit.row("Text", { gap: tk.xs, cross: "center", padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, width: pct(1) }, () => {
        text("Name", label, { step: nameStep, font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: 1.2 });
      });
      apply(absolute({ start: 0, top: 0, bottom: 0 }));
    }
    close();
  };
  hudPlace(id, anchorOf(desc, null), () => {
    box(id, {
      direction: "row", gap: tk.sm, cross: "end", width: desc.width ?? px(u(280)),
      part: sel("CastBar", "frame", state, desc.channel ? "channel" : null, desc.uninterruptible ? "uninterruptible" : null),
      partClass: "hud",
    }, undefined, () => {
      if (compact) {
        bar(true);
        return;
      }
      if (desc.icon || desc.image) {
        // The spell's art in a small framed socket as tall as the name and bar together.
        const side = Math.round(stepSize(nameStep).line + u(3) + h);
        const rim = Math.max(1.5, Math.round(side / 20));
        const r = Math.min(shapeOf("slot").radius, Math.max(3, side / 8));
        box("IconBox", { width: px(side), height: px(side), part: sel("CastBar", "icon"), partClass: "slot" }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [solid(lit(BLACK, 0.9))],
          strokes: [stroke(1, token("lineStrong", 0.45), "outside")],
          effects: [drop(u(2), u(6), lit(BLACK, 0.5))],
          clip: "shape",
        }, () => {
          artTile("Icon", desc.image, desc.icon ?? "", side - rim * 2, hueOf(desc.icon || desc.name || id), absolute({ start: rim, top: rim }));
          if (bad) leaf("Wash", { width: pct(1), height: pct(1) }, { fills: [solid(lit(role("danger"), 0.55))], blend: "multiply" }, absolute({ start: 0, top: 0 }));
          socketRim("Rim", side - rim * 2, Math.max(0, r - rim), undefined, 1, absolute({ start: rim, top: rim }), 0.35);
        });
        if (desc.uninterruptible) {
          box("Shield", { width: px(u(16)), height: px(u(16)), align: "center", cross: "center", part: sel("CastBar", "shield") }, { shape: ellipseShape(), fills: [solid(lit(BLACK, 0.85))], strokes: [stroke(1, token("inkMuted"))] }, () => icon("Mark", "shield", u(11), token("ink")), absolute({ start: side - u(11), bottom: -u(4) }, { z: 2 }));
        }
      }
      kit.column("Body", { gap: u(3), width: { kind: "grow" } }, () => {
        kit.row("Head", { gap: tk.sm, cross: "end", width: { kind: "grow" } }, () => {
          // The name in the style's display face, the line's dominant text; the seconds a quieter tabular figure at its end.
          text("Name", label, { step: nameStep, font: "display", color: bad ? lit(mix(role("danger"), WHITE, 0.25)) : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: 1.4 });
          if (!ended && (desc.remaining ?? 0) > 0) text("Time", (desc.remaining ?? 0).toFixed(1), { step: ts(12), font: "display", color: token("ink"), nowrap: true, outline: 1.4 });
        });
        bar(false);
      });
    }, {
      opacity: alpha,
      transform: bad && since < msToTicks(240) ? { x: Math.sin(since * 1.9) * u(4) * (1 - easeOut(since / msToTicks(240))) } : {},
      a11y: a11y("progressbar", desc.name ?? "Cast", { value: bad ? "Interrupted" : `${Math.round(p * 100)}%`, valueMin: 0, valueMax: 100, valueNow: Math.round(p * 100) }),
      ...groups({ cast: state }, { cast: ["casting", "success", "interrupted", "failed", "gone"] }),
    });
  });
}

