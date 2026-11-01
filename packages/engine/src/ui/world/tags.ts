// og.ui.nameplate, og.ui.interactionPrompt, og.ui.waypoint, og.ui.lootLabel
// and og.ui.speechBubble: the tags of §14 that sit on a unit or a point,
// each a world anchor (projected with the tick's camera, faded behind the
// scene, decluttered, clamped to the edge where it should be). Read against
// World of Warcraft's and Final Fantasy XIV's nameplates (a reaction's
// colour, a slim bar, the target larger and kept on top, crowds stacked),
// the prompts of The Last of Us and Elden Ring (the glyph of the device in
// hand and the verb, a ring filling for a hold), Apex Legends' pings (a
// diamond, the distance, rings rippling as it lands, held on the edge with
// an arrow), Diablo IV's and Borderlands' loot (a chip edged in its tier,
// a beam rising over the best) and the bubbles of Animal Crossing and Sea
// of Thieves (a tail to the speaker, the words typing out).

import type * as Api from "@origingame/engine";
import { input } from "../../input.ts";
import {
  a11y, box, clamp01, easeBack, easeOut, edge, ellipseShape, fillDraw, glow, groups, icon, inkOn, kit, leaf, lit, mix, msToTicks, now, pct, pop,
  px, rarityColor, rarityPips, reactionColor, reduced, role, sel, solid, sound, stroke, text, token, trackDraw, ts, u, vgrad, waypointColor, absolute,
  tps, arc, paintRgba, type Rgba, apply, type Props,
} from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

/** Opens a world anchor, builds, closes (even when build throws), and answers its info. */
function anchored(id: string, desc: Api.WorldAnchorDesc, build: (info: Api.AnchorInfo) => void, props?: Props): Api.AnchorInfo {
  const info = kit.openWorldAnchor(id, desc);
  try {
    build(info);
  } finally {
    kit.close();
  }
  apply(props);
  return info;
}

const lift = (t: Api.WorldTarget | undefined, h: number): Pick<Api.WorldAnchorDesc, "target" | "offset"> => ({
  target: t ?? { entity: null, point: { x: 0, y: 0, z: 0 } },
  offset: { x: 0, y: h, z: 0 },
});

export function nameplate(id: string, desc: Api.NameplateDesc = {}): Api.AnchorInfo {
  const ink = reactionColor(desc.reaction);
  const lods = (desc.lod ?? []).length > 0 ? desc.lod! : [25, 50];
  const targeted = !!desc.targeted;
  const cls: Api.UnitClass = desc.unitClass ?? "normal";
  const big = targeted ? 1.15 : 1;
  const barW = u(cls === "boss" ? 140 : 108 * big);
  const barH = Math.max(5, u(targeted ? 9 : 7));
  let lodName: "full" | "compact" | "bar" = "full";
  const info = anchored(id, {
    ...lift(desc.target, desc.height ?? 2),
    direction: "column",
    cross: "center",
    gap: u(2),
    declutter: "stack",
    priority: (desc.priority ?? 0) + (targeted ? 1000 : 0),
    occlusion: desc.occlusion ?? "fade",
    lod: lods,
    maxDistance: desc.maxDistance || 80,
  }, (inf) => {
    lodName = targeted ? "full" : inf.lod >= 2 ? "bar" : inf.lod >= 1 ? "compact" : "full";
    if (lodName !== "bar") {
      kit.row("Head", { gap: u(4), cross: "center" }, () => {
        if (desc.level) {
          box("Level", { padding: { left: u(3), right: u(3), top: 0, bottom: 0 }, part: sel("Nameplate", "level"), partClass: "badge" }, {
            shape: { kind: "rect", radii: [2, 2, 2, 2] },
            fills: [solid(token("shadow", 0.7))],
            strokes: [stroke(1, lit(cls === "elite" || cls === "boss" ? role("gold") : ink, 0.9))],
          }, () => text("Text", desc.level ?? "", { step: ts(10), font: "mono", color: lit(ink), nowrap: true }));
        }
        if (desc.icon) icon("Icon", desc.icon, u(14), lit(ink));
        text("Name", desc.name ?? "", { step: ts(targeted ? 15 : 13), font: "display", color: lit(mix(ink, WHITE, 0.35)), nowrap: true, outline: 1.1 });
      });
      if (desc.title && lodName === "full") text("Title", desc.title, { step: ts(11), color: lit(mix(ink, WHITE, 0.55), 0.85), nowrap: true, outline: 0.9 });
    }
    if (desc.health !== null && desc.health !== undefined) {
      const hp = clamp01(desc.health);
      box("Bar", { width: px(barW), height: px(barH), part: sel("Nameplate", "bar"), partClass: "bar" }, {
        ...trackDraw(barH / 3),
        strokes: [stroke(targeted ? 1.5 : 1, targeted ? token("highlight") : token("shadow", 0.95), "outside"), ...(cls === "elite" || cls === "boss" ? [stroke(1, lit(role("gold")), "inside")] : [])],
        effects: targeted ? [glow(6, token("highlight", 0.5))] : [],
      }, () => {
        const shown = Math.min(hp, kit.animate("hp", hp, { kind: "token", token: "ui" }));
        // The ghost of a hit holds a moment and drains (the HUD bar's, in short).
        const e = edge("hpv", hp);
        const ghostFrom = kit.memory("ghost", hp);
        if (e.changed && hp < e.prev) kit.remember("ghost", Math.max(ghostFrom, e.prev));
        const g = kit.memory("ghost", hp);
        const gx = clamp01((e.since - msToTicks(350)) / msToTicks(500));
        const ghost = g + (hp - g) * easeOut(gx);
        if (gx >= 1 && g !== hp) kit.remember("ghost", hp);
        if (ghost > shown + 0.001 && !reduced()) leaf("Ghost", { width: pct(ghost), height: pct(1) }, { fills: [solid(token("sheen", 0.7))] }, absolute({ start: 0, top: 0 }));
        leaf("Fill", { width: pct(shown), height: pct(1), part: sel("Nameplate", "fill", desc.reaction ?? "hostile") }, fillDraw(ink, barH / 3), absolute({ start: 0, top: 0 }));
        if ((desc.shield ?? 0) > 0) leaf("Shield", { width: pct(clamp01(desc.shield ?? 0)), height: pct(1), part: sel("Nameplate", "shield") }, { fills: [solid(token("sheen", 0.4))], strokes: [stroke(1, token("sheen", 0.7))] }, absolute({ end: 0, top: 0 }));
      });
    }
    if (desc.castName && lodName !== "bar") {
      box("Cast", { width: px(barW), height: px(Math.max(10, u(12))), cross: "center", align: "center", part: sel("Nameplate", "cast"), partClass: "bar" }, trackDraw(2), () => {
        leaf("Fill", { width: pct(clamp01(desc.castProgress ?? 0)), height: pct(1) }, fillDraw(role("gold"), 2), absolute({ start: 0, top: 0 }));
        text("Name", desc.castName ?? "", { step: ts(9), color: token("ink"), nowrap: true, ellipsis: true, outline: 0.8 });
      });
    }
  });
  apply({
    a11y: a11y("text", desc.name ?? "", { value: [desc.level ? `Level ${desc.level}` : "", desc.health !== null && desc.health !== undefined ? `${Math.round(clamp01(desc.health) * 100)}% health` : ""].filter(Boolean).join(", ") }),
    ...groups({ lod: lodName, selection: targeted ? "targeted" : "none" }, { lod: ["full", "compact", "bar"], selection: ["none", "targeted"] }),
  });
  return info;
}

export function interactionPrompt(world: Api.World, id: string, desc: Api.InteractionPromptDesc = {}): Api.AnchorInfo {
  // A gameplay action (og.input) first, then a UI action (og.ui.defineActions).
  const glyph = desc.action ? ((input.glyph(world as unknown as { readonly id: number }, desc.action) as Api.InputGlyph | null) ?? kit.glyph(world, desc.action)) : null;
  const disabled = !!desc.disabled;
  const hold = desc.hold ?? null;
  const holding = hold !== null && hold > 0 && hold < 1;
  const state = disabled ? "disabled" : holding ? "holding" : "ready";
  const t = now();
  const info = anchored(id, {
    ...lift(desc.target, desc.height ?? 1.2),
    direction: "column",
    cross: "center",
    gap: u(4),
    maxDistance: desc.maxDistance || 6,
    occlusion: "none",
    declutter: "shift",
    priority: 500,
  }, () => {
    // Springs in the first tick it shows (again after it was gone a while).
    const last = kit.memory("lastSeen", -1e9);
    const since0 = kit.memory("since", t);
    const shownAt = t - last > 2 ? t : since0;
    kit.remember("since", shownAt);
    kit.remember("lastSeen", t);
    const enter = reduced() ? 1 : easeBack(clamp01((t - shownAt) / msToTicks(260)));
    const done = edge("done", hold !== null && hold >= 1 ? 1 : 0);
    const s = hold !== null && hold >= 1 ? pop(done.since, 0.2, 300) : 1;
    if (desc.name) text("Name", desc.name, { step: ts(12), color: token("inkMuted"), nowrap: true, outline: 1 });
    box("Frame", {
      direction: "row", gap: u(8), cross: "center", padding: { left: u(6), right: u(12), top: u(5), bottom: u(5) },
      part: sel("InteractionPrompt", "frame", disabled ? "disabled" : null), partClass: "hud",
    }, {
      shape: { kind: "rect", radii: [999, 999, 999, 999] },
      fills: [solid(token("hud", 0.88))],
      strokes: [stroke(1, token("lineStrong", 0.6))],
      effects: [{ kind: "dropShadow", offset: { x: 0, y: 2 }, blur: 8, spread: 0, color: token("shadow", 0.5) }],
    }, () => {
      const gs = u(26);
      box("Glyph", { width: px(gs), height: px(gs), align: "center", cross: "center", part: sel("InteractionPrompt", "glyph") }, undefined, () => {
        if (glyph && kit.hasIcon(glyph.icon)) icon("Icon", glyph.icon, gs, disabled ? token("inkFaint") : token("ink"));
        else {
          box("Cap", { width: px(gs - 2), height: px(gs - 2), align: "center", cross: "center", partClass: "chip" }, {
            shape: { kind: "rect", radii: [4, 4, 4, 4] },
            fills: [vgrad([[0, token("ink")], [1, lit(mix(role("ink"), { r: 0, g: 0, b: 0, a: 1 }, 0.2))]])],
            effects: [{ kind: "dropShadow", offset: { x: 0, y: 2 }, blur: 0, spread: 0, color: token("shadow", 0.6) }],
          }, () => text("Key", glyph?.text || "E", { step: ts(12), font: "display", color: lit(inkOn(role("ink"))), nowrap: true }));
        }
        if (hold !== null) {
          leaf("Track", { width: px(gs + u(8)), height: px(gs + u(8)) }, { shape: arc(1, u(3)), fills: [solid(token("shadow", 0.6))] }, absolute({ start: -u(4), top: -u(4) }));
          if (hold > 0) {
            leaf("Hold", { width: px(gs + u(8)), height: px(gs + u(8)), part: sel("InteractionPrompt", "hold") }, {
              shape: arc(clamp01(hold), u(3)),
              fills: [solid(lit(hold >= 1 ? role("success") : role("highlight")))],
              effects: hold >= 1 ? [glow(6, lit(role("success"), 0.7))] : [],
            }, absolute({ start: -u(4), top: -u(4) }));
          }
        }
      }, { transform: { scaleX: s, scaleY: s } });
      kit.column("Words", { gap: 0 }, () => {
        text("Verb", desc.verb ?? "", { step: ts(14), font: "display", color: disabled ? token("inkFaint") : token("ink"), nowrap: true });
        if (disabled && desc.reason) text("Reason", desc.reason, { step: ts(11), color: lit(role("danger")), nowrap: true });
      });
    }, { opacity: (disabled ? 0.7 : 1) * Math.min(1, enter + 0.2), transform: { scaleX: 0.85 + 0.15 * enter, scaleY: 0.85 + 0.15 * enter, pivotY: 1 } });
  });
  apply({
    a11y: a11y("button", [desc.verb, desc.name].filter(Boolean).join(" "), { value: glyph?.text ?? "", description: disabled ? desc.reason ?? "" : "" }),
    ...groups({ use: state }, { use: ["ready", "holding", "disabled"] }),
  });
  return info;
}

const KIND_ICON: Record<Api.WaypointKind, string> = { objective: "star", marker: "pin", ping: "warning-triangle", enemy: "warning-circle", ally: "user", loot: "gift" };

export function waypoint(id: string, desc: Api.WaypointDesc = {}): Api.AnchorInfo {
  const kind: Api.WaypointKind = desc.kind ?? "marker";
  const col = paintRgba(desc.color, waypointColor(kind, desc.rarity));
  const t = now();
  const at = desc.at ?? 0;
  const age = t - at;
  const life = kind === "ping" ? msToTicks(desc.durationMs || 8000) : desc.durationMs ? msToTicks(desc.durationMs) : Infinity;
  if (age >= life || age < 0) return { inView: false, onScreen: false, clamped: false, lod: 0, distance: 0, x: 0, y: 0, angle: 0, scale: 1, alpha: 0 };
  const landed = edge(`${id}~at`, at);
  sound("notify", landed.changed && kind === "ping" && age < 2);
  let clampedNow = false;
  const info = anchored(id, {
    ...lift(desc.target, desc.height ?? 0.5),
    direction: "column",
    cross: "center",
    gap: u(2),
    pivot: "center",
    occlusion: "none",
    declutter: "none",
    clamp: { margin: desc.clampMargin || 32, arrow: lit(col), arrowSize: u(12) },
  }, (inf) => {
    clampedNow = inf.clamped;
    const drop = reduced() || kind !== "ping" ? 1 : easeBack(clamp01(age / msToTicks(320)), 2.2);
    const fade = Number.isFinite(life) ? clamp01((life - age) / msToTicks(500)) : 1;
    const ms = u(28);
    box("Marker", { width: px(ms), height: px(ms), align: "center", cross: "center", part: sel("Waypoint", "marker", kind, inf.clamped ? "clamped" : null) }, {
      shape: { kind: "polygon", sides: 4, rounding: u(3) },
      fills: [solid(token("shadow", 0.72)), solid(lit(col, 0.22))],
      strokes: [stroke(2, lit(col)), stroke(1, token("shadow", 0.85), "outside")],
      effects: [glow(6, lit(col, 0.45))],
    }, () => {
      icon("Icon", desc.icon || KIND_ICON[kind], Math.round(ms * 0.5), lit(mix(col, WHITE, 0.3)));
      if (kind === "ping" && !reduced()) {
        for (let k = 0; k < 2; k++) {
          const p = ((age / msToTicks(1100)) + k * 0.5) % 1;
          const r = ms * (0.6 + p * 1.1);
          leaf(kit.idi("Ripple", k), { width: px(r * 2), height: px(r * 2), part: sel("Waypoint", "ripple") }, {
            shape: arc(1, 2),
            fills: [solid(lit(col, 0.8 * (1 - p)))],
          }, absolute({ start: ms / 2 - r, top: ms / 2 - r }, { z: -1 }));
        }
      }
    }, { transform: { y: (1 - drop) * -u(30), scaleX: drop, scaleY: drop }, opacity: fade });
    text("Distance", `${Math.round(inf.distance)}${desc.units ?? "m"}`, { step: ts(12), font: "mono", color: token("ink"), nowrap: true, outline: 1 }, { opacity: fade });
    if ((desc.label || desc.by) && !inf.clamped) {
      text("Label", [desc.label, desc.by ? `· ${desc.by}` : ""].filter(Boolean).join(" "), { step: ts(12), font: "display", color: lit(mix(col, WHITE, 0.4)), nowrap: true, outline: 1 }, { opacity: fade });
    }
  });
  apply({
    a11y: a11y("image", desc.label || kind, { value: `${Math.round(info.distance)} ${desc.units ?? "m"}` }),
    ...groups({ place: clampedNow ? "clamped" : "onScreen" }, { place: ["onScreen", "clamped"] }),
  });
  return info;
}

export function lootLabel(id: string, desc: Api.LootLabelDesc = {}): Api.LootLabelResult {
  const rarity: Api.Rarity = desc.rarity ?? "common";
  const col = rarityColor(rarity) ?? role("ink");
  const pips = Math.max(0, rarityPips(rarity));
  const beam = desc.beam !== false && pips >= 3;
  let clicked = false;
  let hovered = false;
  const info = anchored(id, {
    ...lift(desc.target, desc.height ?? 0.4),
    direction: "column",
    cross: "center",
    gap: 0,
    declutter: "stack",
    priority: pips * 10,
    occlusion: "fade",
    maxDistance: desc.maxDistance || 40,
  }, () => {
    const r = box("Chip", {
      direction: "row", gap: u(5), cross: "center", interactive: true,
      padding: { left: u(7), right: u(8), top: u(3), bottom: u(3) },
      part: sel("LootLabel", "chip", ["rarity", rarity]), partClass: "chip",
    }, {
      shape: { kind: "rect", radii: [3, 3, 3, 3] },
      fills: [solid(token("shadow", 0.86)), vgrad([[0, lit(col, 0.22)], [1, lit(col, 0.05)]])],
      strokes: [stroke(1.5, lit(col))],
      effects: pips >= 5 ? [glow(6 + 4 * (0.5 + 0.5 * Math.cos(kit.seconds() * 3)), lit(col, 0.5))] : [],
    }, () => {
      if (desc.icon) icon("Icon", desc.icon, u(14), lit(col));
      text("Name", desc.name ?? "", { step: ts(12), font: "display", color: lit(mix(col, WHITE, 0.25)), nowrap: true });
      if ((desc.count ?? 0) > 1) text("Count", `×${desc.count}`, { step: ts(11), font: "mono", color: token("inkMuted"), nowrap: true });
      if (pips > 0) {
        kit.column("Pips", { gap: 1 }, () => {
          for (let i = 0; i < Math.min(pips, 6); i++) leaf(kit.idi("Pip", i), { width: px(u(3)), height: px(u(2)), part: sel("LootLabel", "pip") }, { fills: [solid(lit(col))] });
        });
      }
    }, { a11y: a11y("button", desc.name ?? "", { value: rarity }) });
    clicked = r.clicked;
    hovered = r.hovered;
    if (beam) {
      // The beam rises from the item to its label, brightest at the ground.
      leaf("Beam", { width: px(u(6)), height: px(u(64)), part: sel("LootLabel", "beam") }, {
        fills: [vgrad([[0, lit(col, 0)], [0.7, lit(col, 0.5)], [1, lit(mix(col, WHITE, 0.5), 0.9)]])],
        effects: [glow(8, lit(col, 0.35))],
        blend: "screen",
      }, { opacity: reduced() ? 0.85 : 0.75 + 0.25 * (0.5 + 0.5 * Math.cos(kit.seconds() * 2)) });
    }
  });
  return { clicked, hovered, info };
}

export function speechBubble(id: string, desc: Api.SpeechBubbleDesc = {}): Api.AnchorInfo {
  const t = now();
  const full = Array.from(desc.text ?? "");
  const cps = desc.charsPerSecond ?? 40;
  const age = t - (desc.at ?? 0);
  const typed = cps <= 0 || reduced() ? full.length : Math.min(full.length, Math.floor((age / tps()) * cps));
  const typeDone = cps > 0 ? msToTicks((full.length / cps) * 1000) : 0;
  const stay = msToTicks(desc.durationMs || 1500 + 60 * full.length);
  const leaveAt = typeDone + stay;
  const leaving = age > leaveAt;
  const gone = age > leaveAt + msToTicks(300) || age < 0;
  if (gone) return { inView: false, onScreen: false, clamped: false, lod: 0, distance: 0, x: 0, y: 0, angle: 0, scale: 1, alpha: 0 };
  const state = typed < full.length ? "typing" : leaving ? "leaving" : "shown";
  const ticked = edge(`${id}~typed`, Math.floor(typed / 3));
  sound("counterTick", ticked.changed && typed < full.length && typed > 0, 2);
  const shout = !!desc.shout;
  const thought = !!desc.thought;
  const info = anchored(id, {
    ...lift(desc.target, desc.height ?? 2.2),
    direction: "column",
    cross: "center",
    gap: 0,
    declutter: "shift",
    priority: 200,
    occlusion: "none",
    ...(desc.clamp ? { clamp: { margin: 24, arrow: token("panelRaised"), arrowSize: u(10) } } : {}),
  }, () => {
    const enter = reduced() ? 1 : easeBack(clamp01(age / msToTicks(240)));
    const out = leaving ? 1 - clamp01((age - leaveAt) / msToTicks(300)) : 1;
    const r = thought ? u(22) : shout ? u(4) : u(12);
    box("Bubble", {
      direction: "column", gap: u(2), padding: { left: u(12), right: u(12), top: u(8), bottom: u(9) },
      width: { kind: "fit", max: u(desc.maxWidth || 260) },
      part: sel("SpeechBubble", "bubble", shout ? "shout" : thought ? "thought" : null), partClass: "tooltip",
    }, {
      shape: { kind: "rect", radii: [r, r, r, r], corner: shout ? "bevel" : "round" },
      fills: [solid(token("panelRaised", 0.97))],
      strokes: [stroke(shout ? 2.5 : 1.5, shout ? lit(role("danger")) : token("lineStrong"))],
      effects: [{ kind: "dropShadow", offset: { x: 0, y: 3 }, blur: 10, spread: 0, color: token("shadow", 0.45) }],
    }, () => {
      if (desc.speaker) text("Speaker", desc.speaker, { step: ts(11), font: "display", color: token("highlight"), nowrap: true });
      // The whole text laid out from the first tick (its box does not grow as it types); the untyped rest is clear.
      text("Text", full.slice(0, typed).join(""), { step: ts(shout ? 15 : 14), font: shout ? "display" : "body", color: token("ink"), width: { kind: "fit", max: u(desc.maxWidth || 260) - u(24) } });
    }, { transform: { scaleX: 0.7 + 0.3 * enter, scaleY: 0.7 + 0.3 * enter, pivotY: 1 }, opacity: Math.min(1, enter * 1.5) * out });
    if (thought) {
      box("Tail", { direction: "column", gap: u(2), cross: "center", part: sel("SpeechBubble", "tail") }, undefined, () => {
        leaf("Dot0", { width: px(u(10)), height: px(u(10)) }, { shape: ellipseShape(), fills: [solid(token("panelRaised", 0.97))], strokes: [stroke(1.5, token("lineStrong"))] });
        leaf("Dot1", { width: px(u(6)), height: px(u(6)) }, { shape: ellipseShape(), fills: [solid(token("panelRaised", 0.97))], strokes: [stroke(1.5, token("lineStrong"))] });
      }, { opacity: out });
    } else {
      // The tail: a triangle pointing down to the speaker, tucked under the bubble's edge.
      leaf("Tail", { width: px(u(16)), height: px(u(11)), part: sel("SpeechBubble", "tail") }, {
        shape: { kind: "polygon", sides: 3, rotation: Math.PI, rounding: 1 },
        fills: [solid(token("panelRaised", 0.97))],
        strokes: [stroke(1.5, shout ? lit(role("danger")) : token("lineStrong"))],
      }, { transform: { y: -u(3) }, opacity: out });
    }
  });
  apply({
    a11y: a11y("text", desc.speaker ?? "", { value: desc.text ?? "" }),
    ...groups({ speech: state }, { speech: ["typing", "shown", "leaving"] }),
  });
  return info;
}
