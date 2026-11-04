// rpg.unitFrame, rpg.partyFrames, rpg.raidFrames and rpg.castBar:
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
import { formatNumber, ut, hudRoom, shrunk, artTile, socketRim, hueOf, BLACK, WHITE, shapeOf, a11y, absolute, anchorOf, box, inner, radial, innerGlow, reduced, clamp01, close, drop, easeOut, edge, fillDraw, frame, glow, groups, hudPlace, icon, keyId, kit, leaf, lit, mix, msToTicks, now, open, paintRgba, pct, px, role, sel, solid, sound, stroke, text, token, trackDraw, ts, u, vgrad, ellipseShape, type Rgba, apply, textScale, stepSize, minText, tsFit, statBar, statusGrid } from "@origingame/engine/ui-parts/hud";
import type { CastBarDesc, CastState, FramesResult, PartyFramesDesc, RaidFramesDesc, RaidMember, RaidRole, UnitFrameDesc, UnitFrameSize } from "../types.ts";

/** How much of the level badge's height its figure takes. */
const LEVEL_FILL = 0.6;

const NO_RESPONSE: Api.Response = { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0, adjust: 0, adjustY: 0, ring: false, tapped: false, doubleTapped: false, longPressed: false, longPress: 0 };

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
function skull(side: number, alpha = 1): void {
  const ink = lit({ r: 0.86, g: 0.87, b: 0.9, a: 1 }, alpha);
  const hole = lit(BLACK, 0.85);
  box("Skull", { width: px(side), height: px(side) }, undefined, () => {
    leaf("Cranium", { width: px(side), height: px(Math.round(side * 0.78)) }, { shape: ellipseShape(), fills: [solid(ink)], effects: [drop(1, 3, lit(BLACK, 0.8))] }, absolute({ start: 0, top: 0 }));
    leaf("Jaw", { width: px(Math.round(side * 0.5)), height: px(Math.round(side * 0.3)) }, { shape: { kind: "rect", radii: [2, 2, side * 0.12, side * 0.12] }, fills: [solid(ink)] }, absolute({ centerX: true, bottom: 0 }));
    const eye = Math.round(side * 0.24);
    leaf("EyeL", { width: px(eye), height: px(eye) }, { shape: ellipseShape(), fills: [solid(hole)] }, absolute({ start: Math.round(side * 0.2), top: Math.round(side * 0.34) }));
    leaf("EyeR", { width: px(eye), height: px(eye) }, { shape: ellipseShape(), fills: [solid(hole)] }, absolute({ end: Math.round(side * 0.2), top: Math.round(side * 0.34) }));
  });
}

/** A plug drawn from shapes (two prongs over a body and its cord): offline, the same in every style. */
function plug(side: number, ink: Api.Paint): void {
  box("Plug", { width: px(side), height: px(side) }, undefined, () => {
    const pw = Math.max(1, Math.round(side * 0.12));
    const ph = Math.round(side * 0.26);
    leaf("ProngL", { width: px(pw), height: px(ph) }, { fills: [solid(ink)] }, absolute({ start: Math.round(side * 0.3), top: 0 }));
    leaf("ProngR", { width: px(pw), height: px(ph) }, { fills: [solid(ink)] }, absolute({ end: Math.round(side * 0.3), top: 0 }));
    leaf("Body", { width: px(Math.round(side * 0.66)), height: px(Math.round(side * 0.4)) }, { shape: { kind: "rect", radii: [1, 1, side * 0.2, side * 0.2] }, fills: [solid(ink)], effects: [drop(1, 2, lit(BLACK, 0.6))] }, absolute({ centerX: true, top: ph }));
    leaf("Cord", { width: px(Math.max(1, Math.round(side * 0.14))), height: px(side - ph - Math.round(side * 0.4)) }, { fills: [solid(ink)] }, absolute({ centerX: true, bottom: 0 }));
  });
}

/** Health as a frame writes it: grouped under a hundred thousand, compact above (by the larger of the two, so both halves match). */
function hpText(v: number, max: number): string {
  return formatNumber(Math.round(v), { notation: max >= 100_000 ? "compact" : "grouped" });
}

/** A dimmed name's ink: a solid muted grey (#A9AEB8), never an alpha, so it steps down a tone and keeps its letter shape. */
const MUTED: Rgba = { r: 0.663, g: 0.682, b: 0.722, a: 1 };

const SIZES: Record<UnitFrameSize, { portrait: number; width: number; bar: number; name: number }> = {
  large: { portrait: 64, width: 232, bar: 16, name: 15 },
  medium: { portrait: 44, width: 176, bar: 12, name: 13 },
  small: { portrait: 30, width: 120, bar: 8, name: 12 },
};

export function unitFrame(id: string, desc: UnitFrameDesc = {}): Api.Response {
  const tk = frame().t;
  const size: UnitFrameSize = desc.size ?? "large";
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
  // Chrome is neutral; hue only where it means something: a hostile unit's ring in danger, an elite's or a friendly boss's in gold.
  const ringInk = hostile ? role("danger") : cls === "elite" || cls === "boss" ? role("gold") : cls === "rare" ? { r: 0.82, g: 0.86, b: 0.92, a: 1 } : role("lineStrong");
  const anchor = anchorOf(desc, null);
  // A frame at a corner shares its edge with the opposite corner's stack (a player's frame and a target's):
  // it takes at most half the band while that corner holds something (last tick), else the frame's own width.
  // Where half the band leaves its bars too narrow (a phone at 200 % text), the frame built after the other
  // steps under it onto a row of its own at its corner, and both take the band's whole width: they never share space.
  const facing: Partial<Record<Api.Anchor, Api.Anchor>> = { topLeft: "topRight", topRight: "topLeft", bottomLeft: "bottomRight", bottomRight: "bottomLeft" };
  const opposite = anchor ? facing[anchor] : undefined;
  const tick = kit.tick();
  const tk0 = frame().t;
  const oppTick = opposite ? kit.memory(`~hudStack/${opposite}/tick`, -10) : -10;
  const narrow = (hudRoom() - tk0.sm) / 2 - pd - u(6) - tk0.md - tk0.sm < ut(96);
  const below = !!opposite && narrow && oppTick === tick;
  if (below) kit.remember(`~unitFrameBelow/${anchor}`, tick);
  const oppBelow = !!opposite && kit.memory(`~unitFrameBelow/${opposite}`, -10) >= tick - 1;
  const shared = oppTick >= tick - 1 && !below && !oppBelow;
  const stepDown = below ? kit.memory(`~hudStack/${opposite}/reach`, 0) + tk0.sm : 0;
  let res: Api.Response = NO_RESPONSE;
  hudPlace(id, anchor, () => {
    // A hit: the health fell this tick. For 450 ms the frame's edge flashes
    // danger and the portrait shakes, read at a glance in the corner of the eye.
    const hp = edge(`${id}:hp`, dead ? 0 : health);
    if (hp.changed && health < hp.prev && !dead) kit.remember(`${id}:hurtAt`, now());
    const hurtAge = now() - kit.memory(`${id}:hurtAt`, -1e9);
    const hurt = hurtAge < msToTicks(450) ? 1 - hurtAge / msToTicks(450) : 0;
    // No plate behind the frame (§15.5.6): the portrait, the name and the bars float on their own shadows.
    // Targeted keeps the ring's own (semantic) colour, a step thicker, with a neutral light glow; a hit flashes it danger.
    const ring = ringInk;
    // The level badge: a circle about 40 % of the portrait, seated on its rim at the bottom inner side (45°), its
    // figure shrunk to fit at any text size; never so small its figure falls under the smallest text the checks allow
    // (§15.2: a small party frame's 12 px badge drew its level at 6.7 px).
    const chars = Math.max(2, (desc.level ?? "").length);
    // Sized as at 100 % text (the portrait grows only half as fast as the text), then grown with the text: its figure
    // doubles at 200 % rather than staying at the smallest size.
    const pd0 = pd / (1 + (Math.min(2, textScale()) - 1) / 2);
    const badge0 = Math.max(Math.round(Math.min(pd0 * 0.4, Math.max(16, pd0 * 0.32))), Math.ceil(Math.max(minText() / LEVEL_FILL, (minText() * chars * 0.6) / 0.78)));
    const badgeH = Math.round(badge0 * textScale());
    const badgeHang = Math.max(0, Math.round(badgeH / 2 - pd * 0.146));
    const frameW = Math.min(u(m.width) + pd, shared ? (hudRoom() - tk.sm) / 2 : hudRoom());
    // The buffs under the bars in as many columns as fit the bars' width, so they never widen the frame.
    const auraSize = size === "large" ? 28 : 18;
    const barsW = frameW - pd - u(6) - tk.md - tk.sm;
    const auraColumns = Math.max(1, Math.min(8, Math.floor((barsW + u(4)) / (u(auraSize) + u(4)))));
    open(id, {
      direction: "row",
      gap: tk.sm,
      cross: "start",
      interactive: true,
      padding: { left: u(6), right: tk.md, top: u(6), bottom: u(6) + (desc.level ? badgeHang : 0) },
      // At a top or bottom corner a frame takes at most half the HUD band, so a player's and a target's frames sit side by side.
      width: px(frameW),
      part: sel("UnitFrame", "frame", size, hostile ? "hostile" : null, desc.targeted ? "targeted" : null, dead ? "dead" : null),
      partClass: "hud",
    });
    // The placeholder's ground: the unit's hue, or plain slate when it is dead or offline (never a tinted state).
    const ground = dead || offline ? SLATE : nameHue(name);
    const portrait = () => {
    // The portrait: round, ringed by its class, the level on a badge centred at its foot and the crown on its
    // shoulder, all held by the avatar so they stay with it at any text size or frame height.
    open("Avatar", { width: px(pd), height: px(pd) });
    box("Portrait", { width: px(pd), height: px(pd), align: "center", cross: "center", part: sel("UnitFrame", "portrait", cls), partClass: "avatar" }, {
      shape: ellipseShape(),
      // Behind the art (and the placeholder's ground): the unit's hue, muted toward slate so a bust never reads as a tinted state.
      fills: [radial([[0, lit(mix(mix(ground, SLATE, 0.6), WHITE, 0.1))], [0.7, lit(mix(mix(ground, SLATE, 0.6), BLACK, 0.4))], [1, lit(mix(SLATE, BLACK, 0.65))]], 0.5, 0.3, 0.8, 0.8)],
      strokes: [stroke(Math.max(2, pd / 18) + (desc.targeted ? 1 : 0), lit(hurt > 0 ? mix(ring, role("danger"), hurt) : ring)), stroke(1, lit(BLACK, 0.8), "outside")],
      effects: [
        drop(u(2), u(8), lit(BLACK, 0.55)),
        ...(cls === "boss" || cls === "elite" ? [glow(u(5), lit(ringInk, 0.45))] : []),
        ...(desc.targeted ? [glow(u(7), lit(WHITE, 0.45))] : []),
        ...(hurt > 0 ? [glow(u(10) * hurt, lit(role("danger"), 0.75 * hurt))] : []),
      ],
      clip: "shape",
    }, () => {
      if (desc.portrait && (dead || offline)) {
        // Dead or offline: the portrait itself in greyscale at about 45 % light (a colorAdjust layer), never a tinted disc.
        box("Grey", { width: px(pd), height: px(pd), part: sel("UnitFrame", "dead") }, { layer: { material: "colorAdjust", params: [0, 0, dead ? 0.72 : 0.85, 1] } }, () => {
          kit.image("Art", { texture: desc.portrait!, width: px(pd), height: px(pd), label: name });
        }, absolute({ start: 0, top: 0 }));
      } else if (desc.portrait) {
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
      if (dead || offline) {
        // A small skull on the greyed portrait (a placeholder bust is greyed by its slate ground already).
        if (dead) box("Dead", { width: pct(1), height: pct(1), align: "center", cross: "center" }, undefined, () => {
          skull(Math.round(pd * 0.32));
        }, absolute({ start: 0, top: 0 }));
      }
    });
    const portraitPath = `${kit.openPath()}/Portrait`;
    if (desc.level) {
      // One ink on a dark disc: the figure in danger for a hostile unit, else the HUD's; a neutral hairline, never a second hue.
      const at = Math.round(pd * 0.146 - badgeH / 2);
      box("Level", { width: px(badgeH), height: px(badgeH), align: "center", cross: "center", part: sel("UnitFrame", "level"), partClass: "badge" }, {
        shape: ellipseShape(),
        fills: [solid(lit({ r: 0.06, g: 0.065, b: 0.08, a: 1 }, 0.96))],
        strokes: [stroke(1, lit(WHITE, 0.22))],
        effects: [drop(1, 3, lit(BLACK, 0.6))],
      }, () => {
        // A step at least the smallest text, drawn down to the badge's room.
        const room = Math.max(minText(), Math.min(badge0 * LEVEL_FILL, (badge0 * 0.78) / (chars * 0.6))) * textScale();
        const step = tsFit(11, room, true);
        const fit = shrunk(step, room, stepSize(step).line);
        text("Text", desc.level ?? "", { step, font: "display", color: hostile ? lit(mix(role("danger"), WHITE, 0.15)) : token("ink"), nowrap: true }, absolute({ centerX: true, centerY: true }, fit.transform ? { transform: { ...fit.transform, y: 0 } } : {}));
      }, absolute(desc.reverse ? { start: at, bottom: at } : { end: at, bottom: at }, { z: 2, overlaps: [portraitPath] }));
    }
    if (desc.leader) {
      // Seated on the rim at its upper outer corner (45°), sized by the portrait.
      const cs = Math.round(pd * 0.34);
      const at = Math.round(pd * 0.146 - cs / 2);
      // A filled crown drawn from its outline (three points on a band), gold with a dark edge, on its own shadow.
      const pts = [[0.06, 0.84], [0.06, 0.34], [0.3, 0.58], [0.5, 0.16], [0.7, 0.58], [0.94, 0.34], [0.94, 0.84]].map(([x, y]) => ({ x: x! * cs, y: y! * cs }));
      leaf("Crown", { width: px(cs), height: px(cs), part: sel("UnitFrame", "crown") }, {
        shape: { kind: "path", points: pts, closed: true, filled: true, join: "round" },
        fills: [vgrad([[0, lit(mix(role("gold"), WHITE, 0.35))], [1, lit(mix(role("gold"), BLACK, 0.2))]])],
        strokes: [stroke(Math.max(1, cs / 14), lit(mix(role("gold"), BLACK, 0.6)))],
        effects: [drop(1, 3, lit(BLACK, 0.7))],
      }, absolute(desc.reverse ? { end: at, top: at } : { start: at, top: at }, { z: 2, overlaps: [portraitPath] }));
    }
    close(hurt > 0 && !reduced() ? { transform: { x: Math.sin(hurtAge * 1.9) * u(3) * hurt } } : undefined);
    };
    if (!desc.reverse) portrait();
    kit.column("Bars", { gap: u(3), width: { kind: "grow" } }, () => {
      kit.row("Head", { gap: tk.xs, cross: "center", width: { kind: "grow" } }, () => {
        const nameInk = dead || offline ? lit(MUTED) : hostile ? lit(mix(role("danger"), WHITE, 0.45)) : token("ink");
        // A long name (a boss's) first shrinks, to four fifths of its step at most, to keep its whole identity; past that
        // it is cut. The display face runs about 0.56 em a glyph.
        const nStep = ts(m.name);
        const nSize = stepSize(nStep).size;
        const room = barsW - (desc.combat ? u(14) + tk.xs : 0);
        const k = Math.max(0.8, Math.min(1, room / Math.max(1, name.length * nSize * 0.56)));
        if (k < 1) {
          const fit = shrunk(nStep, nSize * k, stepSize(nStep).line);
          const tw = Math.max(0, room / fit.k);
          const tl = stepSize(nStep).line;
          box("NameFit", { height: px(Math.ceil(tl * fit.k)), width: { kind: "grow" }, cross: "start" }, undefined, () => {
            text("Name", name, { step: nStep, font: "display", color: nameInk, nowrap: true, ellipsis: true, width: px(tw), name: true },
              absolute(desc.reverse ? { end: 0, top: 0 } : { start: 0, top: 0 }, fit.transform ? { transform: { scaleX: fit.k, scaleY: fit.k, x: (desc.reverse ? 1 : -1) * ((1 - fit.k) * tw) / 2, y: -((1 - fit.k) * tl) / 2 } } : {}));
          });
        } else {
          text("Name", name, { step: nStep, font: "display", color: nameInk, nowrap: true, ellipsis: true, width: { kind: "grow" }, name: true });
        }
        if (desc.combat) icon("Combat", "crossed-swords", u(14), lit(role("danger")), "In combat");
      });
      if (desc.title && size !== "small") {
        // A step under the name at any text size: the ramp's floor gives both the same step on a phone at 200 %, so the
        // class line is drawn at about three quarters of the name's size, in muted ink.
        const tStep = ts(11);
        const want = Math.min(stepSize(tStep).size, Math.max(stepSize(ts(m.name)).size * 0.74, minText() * textScale()));
        const fit = shrunk(tStep, want, stepSize(tStep).line);
        const tw = Math.max(0, barsW / Math.max(0.01, fit.k));
        const tl = stepSize(tStep).line;
        box("TitleRow", { height: px(Math.ceil(want * 1.25)), width: { kind: "grow" }, cross: "start" }, undefined, () => {
          // Scaled about its own centre, then moved back so it keeps the name's start and sits at the row's top.
          text("Title", desc.title ?? "", { step: tStep, color: lit(MUTED), nowrap: true, ellipsis: true, width: px(tw), name: true },
            absolute(desc.reverse ? { end: 0, top: 0 } : { start: 0, top: 0 }, fit.transform ? { transform: { scaleX: fit.k, scaleY: fit.k, x: (desc.reverse ? 1 : -1) * ((1 - fit.k) * tw) / 2, y: -((1 - fit.k) * tl) / 2 } } : {}));
        });
      }
      statBar("Health", {
        value: dead ? 0 : health,
        max: healthMax,
        kind: "health",
        // Offline: the bar greyed (the portrait is too), the name kept at a readable ink rather than the whole frame faded.
        ...(offline ? { color: { color: { r: 0.42, g: 0.44, b: 0.47 } } } : hostile ? { color: token("danger") } : {}),
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
        // The frame is the plate.
        bare: true,
      });
      if (desc.resource !== null && desc.resource !== undefined && !dead) {
        statBar("Resource", { value: desc.resource, max: desc.resourceMax ?? 100, kind: desc.resourceKind ?? "mana", color: desc.resourceColor ?? null, thickness: Math.max(5, Math.round(m.bar * 0.55)), width: { kind: "grow" }, ghost: false, name: `${name} resource`, anchor: null, bare: true });
      }
      if (desc.castName && size !== "small") {
        cast("Cast", { name: desc.castName, progress: desc.castProgress ?? 0, width: { kind: "grow" } }, true, size === "large" ? Math.max(u(m.bar), Math.ceil(stepSize(ts(11)).size * 1.3)) : u(m.bar));
      }
      if ((desc.effects ?? []).length > 0 && size !== "small") statusGrid("Effects", { effects: desc.effects ?? [], size: auraSize, columns: auraColumns, timers: false, split: false, anchor: null }, true);
    });
    if (desc.reverse) portrait();
    res = close({
      opacity: 1,
      a11y: a11y("button", name, { value: dead ? "Dead" : offline ? "Offline" : `${desc.level ? `Level ${desc.level}, ` : ""}${Math.round((health / Math.max(1, healthMax)) * 100)}% health`, selected: !!desc.targeted }),
      ...groups({ life, selection: desc.targeted ? "targeted" : "none", hurt: hurt > 0 ? "hit" : "none" }, { life: ["alive", "dead", "offline"], selection: ["none", "targeted"], hurt: ["none", "hit"] }),
    });
  }, stepDown ? { offsetY: anchor?.startsWith("bottom") ? -stepDown : stepDown } : {});
  return res;
}

export function partyFrames(id: string, desc: PartyFramesDesc = {}): FramesResult {
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

const ROLE_ICON: Record<RaidRole, string> = { none: "", tank: "shield", healer: "plus", damage: "sword" };

export function raidFrames(id: string, desc: RaidFramesDesc = {}): FramesResult {
  const per = Math.max(1, desc.columns || 5);
  const cw = u(desc.cellWidth || 96);
  // Tall enough for the name's line over the health bar at any text size.
  const ch = Math.max(u(desc.cellHeight || 36), Math.ceil(stepSize(ts(11)).line) + raidBar() + raidInset() * 2);
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

/** A raid cell's health bar: its height, and its inset from the cell's edges. */
const raidBar = (): number => Math.max(4, Math.round(u(9)));
const raidInset = (): number => Math.max(2, u(3));

function cell(id: string, m: RaidMember, w: number, h: number): Api.Response {
  const max = Math.max(1, m.healthMax ?? 100);
  const out = !!(m.dead || m.offline);
  const hp = out ? 0 : clamp01((m.health ?? 0) / max);
  const absorb = out ? 0 : clamp01((m.absorb ?? 0) / max);
  const heal = out ? 0 : clamp01((m.incomingHeal ?? 0) / max);
  const cls = paintRgba(m.classColor, role("health"));
  const low = hp < 0.35 && hp > 0;
  const state = m.dead ? "dead" : m.offline ? "offline" : m.outOfRange ? "outOfRange" : null;
  const dispel = m.dispel ? DISPEL[m.dispel] ?? role("danger") : null;
  // One rim, by priority: selection a neutral light rim, aggro danger, a dispellable debuff its type's colour (pulsing).
  const border = m.selected ? WHITE : m.aggro ? role("danger") : dispel;
  // A dark cell (World of Warcraft's and FFXIV's compact frames): health one muted tint, the class a stripe at its
  // start, the missing health the dark track. Falling health ramps one hue, green to amber to red, all muted alike.
  // Muted toward a neutral grey and only a little darker, so amber stays amber (darkening it read as brown, as skin).
  // Three stops of one saturation and lightness: a muted green, an amber, a red (an amber mixed toward grey read as skin).
  const GREEN: Rgba = { r: 0.27, g: 0.56, b: 0.33, a: 1 };
  const AMBER: Rgba = { r: 0.78, g: 0.56, b: 0.16, a: 1 };
  const RED: Rgba = { r: 0.74, g: 0.24, b: 0.2, a: 1 };
  const ramp = clamp01((0.6 - hp) / 0.45);
  const fillInk = ramp <= 0 ? GREEN : ramp < 0.5 ? mix(GREEN, AMBER, ramp * 2) : mix(AMBER, RED, (ramp - 0.5) * 2);
  const stripe = Math.max(3, u(3));
  // Out of range dims what the cell shows (its bar and stripe) and mutes its name, never the whole element: an element
  // opacity would fade the name's outline with its fill, and a dimmed name keeps its outline at full strength.
  const far = m.outOfRange && !out ? 0.45 : 1;
  open(id, {
    width: px(w), height: px(h), interactive: true, cross: "center",
    part: sel("RaidFrames", "cell", state, m.aggro ? "aggro" : null, m.dispel ? ["dispel", m.dispel] : null, m.selected ? "selected" : null),
    partClass: "bar",
  }, {
    ...trackDraw(3),
    // A neutral ground (the HUD's dark neutral) with the health a bar along its foot, the cell's one hue; dead and
    // offline keep the near-black well. Near-opaque either way: a scene behind a raid cell never tints it.
    fills: [solid(out ? lit({ r: 0.05, g: 0.055, b: 0.065, a: 1 }) : token("hud", 0.97)), vgrad([[0, lit(WHITE, out ? 0 : 0.06)], [0.5, lit(BLACK, 0)], [1, lit(BLACK, 0.2)]])],
    strokes: border ? [stroke(m.selected ? 2 : 1.5, lit(border, dispel && !m.selected && !m.aggro ? 0.6 + 0.4 * (0.5 + 0.5 * Math.cos(kit.seconds() * 6)) : m.selected ? 0.9 : 1))] : [stroke(1, token("shadow", 0.9), "outside")],
  });
  // The bar: inset along the cell's foot from the stripe to the end.
  const inset = raidInset();
  const barH = raidBar();
  const room = Math.max(0, w - stripe - inset * 2);
  if (!out) {
    box("Bar", { width: px(room), height: px(barH), part: sel("RaidFrames", "bar") }, { shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [solid(lit(BLACK, 0.7))], strokes: [stroke(1, token("line", 0.6), "outside")], clip: "shape" }, () => {
      if (hp > 0) leaf("Fill", { width: px(hp * room), height: pct(1), part: sel("RaidFrames", "fill", low ? "low" : null) }, fillDraw(fillInk, 2, far), absolute({ start: 0, top: 0 }));
      // The incoming heal: the fill's own hue a third lighter, opaque and at full saturation, at the fill's end.
      const hw = Math.min(heal, 1 - hp);
      const healInk: Rgba = { r: Math.min(1, fillInk.r * 1.35), g: Math.min(1, fillInk.g * 1.35), b: Math.min(1, fillInk.b * 1.35), a: 1 };
      if (hw > 0) leaf("Heal", { width: px(hw * room), height: pct(1), part: sel("RaidFrames", "heal") }, fillDraw(healInk, 0, far), absolute({ start: hp * room, top: 0 }));
      if (absorb > 0) {
        // The absorb: the player frame's white-blue overlay past the health (and heal); what overflows lies back over the fill's end.
        const past = Math.min(absorb, 1 - hp - hw);
        const over = Math.min(hp, absorb - past);
        const paint = (a: number): Api.Fill[] => [solid(lit({ r: 0.82, g: 0.93, b: 1, a: 1 }, a * far)), vgrad([[0, lit(WHITE, 0.45 * far)], [0.5, lit(WHITE, 0.1 * far)]])];
        if (past > 0) leaf("Absorb", { width: px(past * room), height: pct(1), part: sel("RaidFrames", "absorb") }, { fills: paint(0.62) }, absolute({ start: (hp + hw) * room, top: 0 }));
        if (over > 0.001) leaf("Overabsorb", { width: px(over * room), height: pct(1), part: sel("RaidFrames", "absorb", "over") }, { fills: paint(0.45) }, absolute({ start: (hp - over) * room, top: 0 }));
      }
    }, absolute({ start: stripe + inset, bottom: inset }));
  }
  leaf("Class", { width: px(stripe), height: pct(1), part: sel("RaidFrames", "class") }, {
    shape: { kind: "rect", radii: [3, 0, 0, 3] },
    fills: [solid(lit(cls, out ? 0.4 : far))],
  }, absolute({ start: 0, top: 0 }));
  // The name keeps its line and place; dead or offline says so at the cell's end on that line, a step smaller and muted.
  const ink = out || far < 1 ? lit(MUTED) : token("ink");
  kit.row("Label", { gap: u(3), cross: "center", padding: { left: stripe + u(4), right: u(4), top: 0, bottom: 0 }, width: pct(1) }, () => {
    const roleName = ROLE_ICON[m.role ?? "none"];
    if (roleName) icon("Role", roleName, Math.max(10, Math.round(h * 0.3)), token("ink", out || far < 1 ? 0.62 : 0.85), m.role ?? "");
    // The name keeps priority and the status is never cut: when both words do not fit, the status becomes a small
    // muted glyph at the cell's end (the skull for dead, a plug for offline), its word kept for the tooltip and the
    // screen reader.
    text("Name", m.name ?? "", { step: ts(11), color: ink, nowrap: true, ellipsis: true, width: { kind: "grow" }, name: true });
    if (out) {
      const word = m.dead ? "Dead" : "Offline";
      const roleW = roleName ? Math.max(10, Math.round(h * 0.3)) + u(3) : 0;
      const nameW = (m.name ?? "").length * stepSize(ts(11)).size * 0.6;
      const wordW = word.length * stepSize(ts(9)).size * 0.6;
      if (u(8) + roleW + nameW + u(3) + wordW <= room) {
        text("Status", word, { step: ts(9), color: token("ink", 0.62), nowrap: true, soft: true });
      } else {
        const g = Math.max(10, Math.round(stepSize(ts(9)).size * 1.05));
        box("Status", { width: px(g), height: px(g), align: "center", cross: "center", part: sel("RaidFrames", "status", state) }, undefined, () => {
          if (m.dead) skull(g, 0.62);
          else plug(g, token("ink", 0.62));
        }, { a11y: a11y("image", word) });
        kit.tooltip(word);
      }
    }
  });
  // The name's line over the bar (the whole cell when there is no bar).
  apply(absolute({ start: 0, top: 0, bottom: out ? 0 : barH + inset }));
  return close({
    a11y: a11y("button", m.name ?? "", { value: m.dead ? "Dead" : m.offline ? "Offline" : `${Math.round(hp * 100)}%`, selected: !!m.selected }),
  });
}

const DISPEL: Record<string, Rgba> = {
  magic: { r: 0.24, g: 0.55, b: 1, a: 1 },
  curse: { r: 0.65, g: 0.3, b: 1, a: 1 },
  poison: { r: 0.24, g: 0.81, b: 0.31, a: 1 },
  disease: { r: 0.78, g: 0.6, b: 0.17, a: 1 },
};

export function castBar(id: string, desc: CastBarDesc = {}): void {
  cast(id, desc, false);
}

/** A cast bar; compact is the thin one under a unit frame's bars. */
function cast(id: string, desc: CastBarDesc, compact: boolean, healthH = 0): void {
  const tk = frame().t;
  const state: CastState = desc.state ?? "casting";
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
  const gold = compact ? mix(role("gold"), BLACK, 0.3) : role("gold");
  const ink = bad ? role("danger") : state === "success" ? role("success") : desc.uninterruptible ? { r: 0.62, g: 0.64, b: 0.68, a: 1 } : desc.channel ? role("info") : gold;
  // Compact (under a unit frame): a slim bar about 60 % of the health bar's height in a darker amber, its name a step
  // smaller on a line over it, so the health stays the frame's dominant bar. Full: the name and seconds on a line
  // of their own over a slim bar, the spell's art beside both (Diablo IV, Destiny 2, WoW's modern cast bar).
  const nameStep = compact ? ts(9) : ts(14);
  const h = compact ? Math.max(3, Math.round((healthH || u(12)) * 0.6)) : Math.max(u(8), Math.round(stepSize(nameStep).size * 0.55));
  const label = state === "interrupted" ? desc.interruptedLabel ?? "Interrupted" : state === "failed" ? desc.failedLabel ?? "Failed" : desc.name ?? "";
  const shown = bad ? 1 : state === "success" ? 1 : desc.channel ? 1 - p : p;
  const bar = () => {
    const trackPath = open("Track", { width: { kind: "grow" }, height: px(h), cross: "center", part: sel("CastBar", "track"), partClass: "bar" }, {
      ...trackDraw(Math.min(h / 2, u(3))),
      ...(desc.uninterruptible ? { strokes: [stroke(1.5, token("inkMuted"), "outside")] } : {}),
    });
    const w = kit.bounds(trackPath)?.width ?? u(240);
    leaf("Fill", { width: pct(shown), height: pct(1), part: sel("CastBar", "fill") }, fillDraw(ink, Math.min(h / 2, u(3))), absolute({ start: 0, top: 0 }));
    if (!ended && shown > 0.01 && shown < 0.995) {
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
    close();
  };
  hudPlace(id, anchorOf(desc, null), () => {
    box(id, {
      direction: "row", gap: tk.sm, cross: "end", width: desc.width ?? px(u(280)),
      part: sel("CastBar", "frame", state, desc.channel ? "channel" : null, desc.uninterruptible ? "uninterruptible" : null),
      partClass: "hud",
    }, undefined, () => {
      if (compact) {
        kit.column("Body", { gap: u(2), width: { kind: "grow" } }, () => {
          // On a narrow HUD (a phone) the label goes under the bar, as FFXIV's enemy casts do, so the boss's health and cast
          // bars part its name from the cast's label rather than stacking text on text.
          const under = hudRoom() < u(600);
          if (under) bar();
          kit.row("Head", { width: { kind: "grow" } }, () => {
            text("Name", label, { step: nameStep, font: "display", color: bad ? lit(mix(role("danger"), WHITE, 0.25)) : token("ink"), nowrap: true, ellipsis: true, width: { kind: "grow" }, name: true });
          });
          if (!under) bar();
        });
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
        bar();
      });
    }, {
      opacity: alpha,
      transform: bad && since < msToTicks(240) ? { x: Math.sin(since * 1.9) * u(4) * (1 - easeOut(since / msToTicks(240))) } : {},
      a11y: a11y("progressbar", desc.name ?? "Cast", { value: bad ? "Interrupted" : `${Math.round(p * 100)}%`, valueMin: 0, valueMax: 100, valueNow: Math.round(p * 100) }),
      ...groups({ cast: state }, { cast: ["casting", "success", "interrupted", "failed", "gone"] }),
    });
  });
}

