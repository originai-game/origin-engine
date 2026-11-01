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
import { a11y, absolute, anchorOf, box, chip, innerGlow, reduced, clamp01, close, drop, easeOut, edge, fillDraw, frame, glow, groups, hudPlace, icon, inkOn, keyId, kit, leaf, lit, mix, msToTicks, now, open, paintRgba, pct, px, role, sel, solid, sound, stroke, text, token, trackDraw, ts, u, vgrad, ellipseShape, type Rgba, apply } from "./core.ts";
import { statBar } from "./meters.ts";
import { statusGrid } from "./abilities.ts";

const NO_RESPONSE: Api.Response = { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 };

/** Initials for a portrait with no art. */
function initials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "?";
  return (words.length === 1 ? words[0]!.slice(0, 2) : words[0]![0]! + words[1]![0]!).toUpperCase();
}

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

const SIZES: Record<Api.UnitFrameSize, { portrait: number; width: number; bar: number; name: number }> = {
  large: { portrait: 64, width: 232, bar: 16, name: 15 },
  medium: { portrait: 44, width: 176, bar: 12, name: 13 },
  small: { portrait: 30, width: 120, bar: 8, name: 12 },
};

export function unitFrame(id: string, desc: Api.UnitFrameDesc = {}): Api.Response {
  const tk = frame().t;
  const size: Api.UnitFrameSize = desc.size ?? "large";
  const m = SIZES[size];
  const pd = u(m.portrait);
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
  let res: Api.Response = NO_RESPONSE;
  hudPlace(id, anchor, () => {
    // A hit: the health fell this tick. For 450 ms the frame's edge flashes
    // danger and the portrait shakes, read at a glance in the corner of the eye.
    const hp = edge(`${id}:hp`, dead ? 0 : health);
    if (hp.changed && health < hp.prev && !dead) kit.remember(`${id}:hurtAt`, now());
    const hurtAge = now() - kit.memory(`${id}:hurtAt`, -1e9);
    const hurt = hurtAge < msToTicks(450) ? 1 - hurtAge / msToTicks(450) : 0;
    const baseStrokes = desc.targeted ? [stroke(2, lit(role("highlight"))), stroke(1, token("shadow", 0.8), "outside")] : chip().strokes ?? [];
    const baseEffects = desc.targeted ? [drop(2, 8, token("shadow", 0.45)), glow(8, lit(role("highlight"), 0.45))] : chip().effects ?? [];
    open(id, {
      direction: "row",
      gap: tk.sm,
      cross: "center",
      interactive: true,
      padding: { left: u(6), right: tk.md, top: u(6), bottom: u(6) },
      width: px(u(m.width) + pd),
      part: sel("UnitFrame", "frame", size, hostile ? "hostile" : null, desc.targeted ? "targeted" : null, dead ? "dead" : null),
      partClass: "hud",
    }, {
      ...chip(),
      strokes: hurt > 0 ? [stroke(2.5, lit(role("danger"), 0.4 + 0.6 * hurt)), ...baseStrokes.slice(1)] : baseStrokes,
      effects: hurt > 0 ? [...baseEffects, glow(10 * hurt, lit(role("danger"), 0.7 * hurt)), innerGlow(14, lit(role("danger"), 0.45 * hurt))] : baseEffects,
    });
    const portrait = () => {
    // The portrait: round, ringed by its class, the level on a badge at its foot.
    box("Portrait", { width: px(pd), height: px(pd), align: "center", cross: "center", part: sel("UnitFrame", "portrait", cls), partClass: "avatar" }, {
      shape: ellipseShape(),
      fills: [vgrad([[0, lit(mix(nameHue(name), { r: 1, g: 1, b: 1, a: 1 }, 0.15))], [1, lit(mix(nameHue(name), { r: 0, g: 0, b: 0, a: 1 }, 0.35))]])],
      strokes: [stroke(Math.max(2, pd / 18), lit(ringInk)), stroke(1, token("shadow", 0.9), "outside")],
      effects: cls === "boss" || cls === "elite" ? [glow(5, lit(ringInk, 0.5))] : [],
      clip: "shape",
    }, () => {
      if (desc.portrait) {
        kit.image("Art", { texture: desc.portrait, width: px(pd), height: px(pd), label: name });
        apply(absolute({ start: 0, top: 0 }, { opacity: dead ? 0.45 : 1 }));
      } else if (desc.icon && kit.hasIcon(desc.icon)) icon("Art", desc.icon, Math.round(pd * 0.56), token("ink"));
      else text("Initials", initials(name), { step: ts(Math.round(m.portrait * 0.34), true), font: "display", color: lit(inkOn(nameHue(name))), nowrap: true });
      if (hurt > 0 && !reduced()) {
        leaf("Hurt", { width: pct(1), height: pct(1) }, { fills: [solid(lit(role("danger"), 0.5 * hurt))] }, absolute({ start: 0, top: 0 }));
      }
      if (dead) {
        box("Dead", { width: pct(1), height: pct(1), align: "center", cross: "center" }, { fills: [solid(token("shadow", 0.55))] }, () => {
          icon("Skull", "skull", Math.round(pd * 0.5), token("inkMuted"));
        }, absolute({ start: 0, top: 0 }));
      }
    });
    if (hurt > 0 && !reduced()) apply({ transform: { x: Math.sin(hurtAge * 1.9) * u(3) * hurt } });
    const portraitPath = `${kit.openPath()}/Portrait`;
    if (desc.level) {
      box("Level", { padding: { left: u(4), right: u(4), top: 0, bottom: 0 }, height: px(Math.max(14, Math.round(pd * 0.3))), align: "center", cross: "center", part: sel("UnitFrame", "level"), partClass: "badge" }, {
        shape: { kind: "rect", radii: [999, 999, 999, 999] },
        fills: [solid(token("hud", 0.98))],
        strokes: [stroke(1.5, lit(ringInk))],
      }, () => {
        text("Text", desc.level ?? "", { step: ts(11), font: "mono", color: hostile ? lit(role("danger")) : token("ink"), nowrap: true });
      }, absolute(desc.reverse ? { end: u(6) + pd / 2, bottom: u(2) } : { start: u(6) + pd / 2, bottom: u(2) }, { transform: { x: desc.reverse ? pd * 0.25 : -pd * 0.25 }, z: 2, overlaps: [portraitPath] }));
    }
    if (desc.leader) {
      box("Crown", { width: px(u(18)), height: px(u(18)), align: "center", cross: "center", part: sel("UnitFrame", "crown") }, undefined, () => {
        icon("Mark", "crown", u(16), lit(role("gold")));
      }, absolute(desc.reverse ? { end: u(2), top: -u(6) } : { start: u(2), top: -u(6) }, { z: 2, overlaps: [portraitPath] }));
    }
    };
    if (!desc.reverse) portrait();
    kit.column("Bars", { gap: u(3), width: { kind: "grow" } }, () => {
      kit.row("Head", { gap: tk.xs, cross: "center", width: { kind: "grow" } }, () => {
        text("Name", name, { step: ts(m.name), font: "display", color: dead ? token("inkFaint") : hostile ? lit(role("danger")) : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: 1 });
        if (desc.combat) icon("Combat", "crossed-swords", u(14), lit(role("danger")), "In combat");
      });
      if (desc.title && size !== "small") text("Title", desc.title, { step: ts(11), color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
      statBar("Health", {
        value: dead ? 0 : health,
        max: healthMax,
        kind: "health",
        ...(hostile ? { color: token("danger") } : {}),
        shield: desc.shield ?? 0,
        shieldMax: (desc.shield ?? 0) > 0 ? healthMax : 0,
        thickness: m.bar,
        width: { kind: "grow" },
        showValue: size === "large",
        valueText: size === "large" ? `${Math.round(health)} / ${Math.round(healthMax)}` : "",
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
    text("Name", m.name ?? "", { step: ts(11), color: ink, nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: 1 });
    if (m.dead || m.offline) text("Status", m.dead ? "Dead" : "Offline", { step: ts(10), color: token("inkMuted"), nowrap: true });
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
  const h = u(compact ? 10 : 18);
  hudPlace(id, anchorOf(desc, null), () => {
    box(id, {
      direction: "row", gap: tk.xs, cross: "center", width: desc.width ?? px(u(280)),
      part: sel("CastBar", "frame", state, desc.channel ? "channel" : null, desc.uninterruptible ? "uninterruptible" : null),
      partClass: "hud",
    }, undefined, () => {
      if (desc.icon && !compact) {
        box("IconBox", { width: px(h + u(8)), height: px(h + u(8)), align: "center", cross: "center", part: sel("CastBar", "icon"), partClass: "slot" }, chip(3, true), () => {
          icon("Icon", desc.icon ?? "", h, token("ink"));
          if (desc.uninterruptible) box("Shield", { width: px(u(14)), height: px(u(14)), part: sel("CastBar", "shield") }, undefined, () => icon("Mark", "shield", u(14), token("inkMuted")), absolute({ end: -u(4), bottom: -u(4) }));
        });
      }
      const trackPath = open("Track", { width: { kind: "grow" }, height: px(h), cross: "center", part: sel("CastBar", "track"), partClass: "bar" }, {
        ...trackDraw(2),
        strokes: [stroke(desc.uninterruptible ? 2 : 1, desc.uninterruptible ? token("inkMuted") : token("lineStrong", 0.6))],
      });
      const w = kit.bounds(trackPath)?.width ?? u(240);
      const shown = bad ? 1 : state === "success" ? 1 : desc.channel ? 1 - p : p;
      leaf("Fill", { width: pct(shown), height: pct(1), part: sel("CastBar", "fill") }, fillDraw(ink, 2), absolute({ start: 0, top: 0 }));
      if (!ended && shown > 0.01 && shown < 0.995) {
        // The spark at the leading edge.
        leaf("Spark", { width: px(u(10)), height: pct(1.6), part: sel("CastBar", "spark") }, {
          shape: ellipseShape(),
          fills: [vgrad([[0, token("sheen", 0)], [0.5, token("sheen", 0.95)], [1, token("sheen", 0)]], 90)],
          blend: "screen",
        }, absolute({ start: shown * w - u(5), top: -h * 0.3 }));
      }
      if ((desc.latency ?? 0) > 0 && !desc.channel && !ended) {
        leaf("Latency", { width: pct(clamp01(desc.latency ?? 0)), height: pct(1), part: sel("CastBar", "latency") }, { fills: [solid(lit(role("danger"), 0.4))] }, absolute({ end: 0, top: 0 }));
      }
      if (desc.channel && (desc.ticks ?? 0) > 1) {
        for (let i = 1; i < (desc.ticks ?? 0); i++) {
          leaf(kit.idi("Tick", i), { width: px(2), height: pct(1), part: sel("CastBar", "tick") }, { fills: [solid(token("shadow", 0.85))] }, absolute({ start: (i / (desc.ticks ?? 1)) * w - 1, top: 0 }));
        }
      }
      if (state === "success" && since < hold) {
        leaf("Flash", { width: pct(1), height: pct(1) }, { fills: [solid(token("sheen", 0.6 * (1 - since / hold)))] }, absolute({ start: 0, top: 0 }));
      }
      const label = state === "interrupted" ? desc.interruptedLabel ?? "Interrupted" : state === "failed" ? desc.failedLabel ?? "Failed" : desc.name ?? "";
      if (!compact || h >= 10) {
        kit.row("Text", { gap: tk.xs, cross: "center", padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, width: pct(1) }, () => {
          text("Name", label, { step: compact ? ts(10) : ts(13), color: token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, outline: 1 });
          if (!ended && (desc.remaining ?? 0) > 0 && !compact) text("Time", (desc.remaining ?? 0).toFixed(1), { step: ts(12), font: "mono", color: token("ink"), nowrap: true, outline: 1 });
        });
        apply(absolute({ start: 0, top: 0, bottom: 0 }));
      }
      close();
    }, {
      opacity: alpha,
      transform: bad && since < msToTicks(240) ? { x: Math.sin(since * 1.9) * u(4) * (1 - easeOut(since / msToTicks(240))) } : {},
      a11y: a11y("progressbar", desc.name ?? "Cast", { value: bad ? "Interrupted" : `${Math.round(p * 100)}%`, valueMin: 0, valueMax: 100, valueNow: Math.round(p * 100) }),
      ...groups({ cast: state }, { cast: ["casting", "success", "interrupted", "failed", "gone"] }),
    });
  });
}

