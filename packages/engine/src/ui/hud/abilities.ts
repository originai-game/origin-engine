// og.ui.abilitySlot, og.ui.abilityBar and og.ui.statusGrid: Appendix C's
// "ability bars with cooldown sweeps, charges and hotkeys" and "buff and
// debuff grids with timers and stacks". Read against World of Warcraft's
// action bars and aura grids (the clockwise sweep with its seconds, a
// buff's time under its icon, debuffs bordered by their dispel type),
// League of Legends' and Dota 2's ability rows (charges at the corner, the
// next one recharging along the bottom, rank pips, the hotkey on a cap,
// the out-of-mana tint, the ultimate's ring) and Overwatch's ultimate
// percent: a cooldown ending is told by a flash and a pop, never only by
// the sweep running out.

import type * as Api from "@origingame/engine";
import { stepSize, hudRoom, shrunk, artTile, socketRim, hueOf, BLACK, WHITE, a11y, arc, inner, type Paint, absolute, anchorOf, box, bp, clamp01, close, drop, edge, frame, glow, groups, hudPlace, icon, innerGlow, keyId, kit, leaf, lit, mix, msToTicks, open, pct, pop, pulse, px, reduced, role, sel, shapeOf, shortTime, solid, sound, stroke, text, token, ts, u, vgrad, type Rgba, apply, textScale, tsFit, minText } from "./core.ts";

const DEBUFF_TYPES: Record<string, string> = {
  magic: "#3d8bff", curse: "#a64dff", poison: "#3ccf4e", disease: "#c89a2c", bleed: "#e03030",
};

function typeColor(type: string | undefined): Rgba {
  const c = type ? DEBUFF_TYPES[type] : undefined;
  if (!c) return role("danger");
  const n = (i: number) => Number.parseInt(c.slice(i, i + 2), 16) / 255;
  return { r: n(1), g: n(3), b: n(5), a: 1 };
}



/** A pie over a square slot: the dark part of a sweep that has `left` of its turn to go, shrinking clockwise. */
function pie(left: number): Api.DrawShape {
  const r = clamp01(left);
  return { kind: "arc", start: -Math.PI / 2 + (1 - r) * Math.PI * 2, sweep: Math.max(0.0001, r) * Math.PI * 2, thickness: 4096, cap: "butt" };
}

/** A slot's side: it grows with the player's text size (half again at 200 %), so its numerals grow and the art stays the larger part. */
function slotSize(size: number | undefined): number {
  return Math.round(u(size || (bp().phone ? 48 : 56)) * (1 + (Math.min(2, textScale()) - 1) / 2));
}


const NOTHING = { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 } as Api.Response;

export function abilitySlot(id: string, desc: Api.AbilitySlotDesc = {}): Api.Response {
  return slot(id, desc);
}

/**
 * One slot, as shipped action bars draw it (Diablo IV, League, Destiny 2,
 * Genshin): the art fills the frame edge to edge and stays the dominant
 * element in every state; one centred numeral per state (the cooldown's
 * seconds over a clockwise darkening, or nothing); the cost and charges
 * small on dark pills; the hotkey a quiet label under the slot. A bar
 * passes its slots' `size` already resolved, so it is never scaled twice;
 * an ultimate is drawn 1.15 times it, its charge a ring round the art.
 */
export function slot(id: string, desc: Api.AbilitySlotDesc, base = slotSize(desc.size), keyCorner = false): Api.Response {
  const size = desc.ultimate != null ? Math.round(base * 1.15) : base;
  const { radius, corner } = shapeOf("slot");
  // The style's slot corner, kept to a frame's few pixels however large the slot grows.
  const r = Math.min(radius, Math.max(4, size / 10));
  const maxCharges = desc.maxCharges ?? 0;
  const charged = maxCharges > 1;
  const charges = Math.min(desc.charges ?? 0, maxCharges);
  const cooldown = Math.max(0, desc.cooldown ?? 0);
  // A charged ability is usable while it has a charge; its cooldown is the next charge's.
  const cooling = charged ? charges === 0 && cooldown > 0 : cooldown > 0;
  const locked = !!desc.locked;
  const disabled = !!desc.disabled && !locked;
  const ready = !cooling && !locked && !disabled;
  const ult = desc.ultimate ?? null;
  const ultReady = ult !== null && ult >= 1;
  const readyState = locked ? "locked" : cooling ? "cooling" : disabled ? "disabled" : "ready";
  const use = desc.active ? "active" : desc.unaffordable ? "unaffordable" : "idle";
  // The frame's line: thin, so the art reaches almost to the slot's edge.
  const line = Math.max(1.5, Math.round(size / 36));
  // The accent is a hue, never the neutral primary of a grey style: active in the info blue, an ultimate ready in gold.
  const accent = desc.active ? role("info") : ultReady ? role("gold") : null;
  let r0: Api.Response | null = null;
  kit.column(id, { gap: Math.max(3, u(3)), cross: "center" }, () => {
    const e = edge("ready", ready && (ult === null || ultReady) ? 1 : 0);
    const cameReady = e.changed && e.prev === 0 && ready;
    sound("focus", cameReady && kit.memory("seen", 0) > 0);
    kit.remember("seen", 1);
    const flashT = ready && e.since < msToTicks(380) && e.prev === 0 ? 1 - e.since / msToTicks(380) : 0;
    const s = ready && e.prev === 0 ? pop(e.since, 0.12, 340) : 1;
    const breath = accent ? pulse(desc.active ? 1.1 : 0.9) : 0;
    const path = open("Slot", {
      width: px(size),
      height: px(size),
      interactive: !locked,
      part: sel("AbilitySlot", "frame", readyState, desc.active ? "active" : null),
      partClass: "slot",
    }, {
      shape: { kind: "rect", radii: [r, r, r, r], corner },
      fills: [solid(lit(BLACK, 0.9))],
      // The style's glass hairline outside; a float shadow under it ties the slots together without a backplate.
      strokes: [stroke(1, token("lineStrong", 0.45), "outside")],
      // Active: a soft glow in its colour, wide enough to hold at phone size (never a white outline).
      effects: [drop(u(3), u(10), lit(BLACK, 0.55)), ...(accent ? [glow(u(7) + u(4) * breath, lit(accent, 0.5 + 0.25 * breath), u(1))] : [])],
      clip: "shape",
    });
    // The art, edge to edge inside the frame's line.
    const artSide = size - line * 2;
    // The art is the slot's ground: the seconds, the charges, the cost and the percent sit on it by design.
    artTile("Icon", desc.image, desc.icon ?? "", artSide, hueOf(desc.icon || desc.name || id), absolute({ start: line, top: line }, { overlaps: [path] }), sel("AbilitySlot", "icon"));
    const full = (insetBy = line) => absolute({ start: insetBy, top: insetBy });
    const cover = { width: px(artSide), height: px(artSide) };
    // What the state does to the art: dark when locked, washed out and slashed when disabled,
    // washed and tinted when it cannot be paid for, reddened when its target is out of range.
    if (locked || disabled || desc.unaffordable || (desc.outOfRange && ready)) {
      // A cool grey over the art takes most of its colour and light: locked leaves it at about a third.
      const wash = locked ? 0.8 : disabled ? 0.55 : desc.unaffordable ? 0.35 : 0;
      if (wash > 0) leaf("Wash", { ...cover, part: sel("AbilitySlot", "tint", locked ? "locked" : disabled ? "disabled" : "unaffordable") }, { fills: [solid(lit({ r: 0.12, g: 0.12, b: 0.14, a: 1 }, wash))] }, full());
      // Too dear: the art reddens, and the cost turns red.
      if (desc.unaffordable && !locked && !disabled) leaf("Unaffordable", cover, { fills: [solid(lit(role("danger"), 0.55))], blend: "multiply" }, full());
      if (desc.outOfRange && ready) leaf("OutOfRange", { ...cover, part: sel("AbilitySlot", "tint", "outOfRange") }, { fills: [solid(lit(role("danger"), 0.55))], blend: "multiply" }, full());
    }
    if (cooling && !locked) {
      const max = desc.cooldownMax ?? 0;
      const left = max > 0 ? clamp01(cooldown / max) : 1;
      // The part still to come darkened by about 60 % as seen, the part done left clean: the fraction left reads without the numeral.
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("AbilitySlot", "sweep") }, {
        shape: pie(left),
        // og-ui composites in linear light: 90 % black there reads as about 60 % darker to the eye (66 % read as a quarter).
        fills: [solid(lit(BLACK, 0.9))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
      // The sweep's leading edge: a faint 1 px line from the rim inward, under the numerals; the dim is what reads.
      const angle = -Math.PI / 2 + (1 - left) * Math.PI * 2;
      const half = 0.5 / Math.max(1, size * 0.5);
      leaf("Hand", { width: px(size * 1.5), height: px(size * 1.5) }, {
        shape: { kind: "arc", start: angle - half, sweep: half * 2, thickness: size * 0.4, cap: "butt" },
        fills: [solid(lit(WHITE, 0.4))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
    }
    // The art set into the frame: a 2 px inner shadow and a soft vignette, and a faint glint along its top.
    leaf("Bevel", cover, {
      fills: [vgrad([[0, lit(WHITE, 0.12)], [0.3, lit(WHITE, 0)]])],
      effects: [inner(1, 2, lit(BLACK, 0.7), 1), innerGlow(size * 0.12, lit(BLACK, 0.45))],
    }, full());
    if (flashT > 0) {
      leaf("Flash", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "flash") }, {
        fills: [solid(token("sheen", 0.6 * flashT))],
        effects: [innerGlow(size * 0.3, token("sheen", flashT))],
      }, absolute({ start: 0, top: 0 }));
    }
    if (locked) {
      // A filled padlock about 40 % of the slot: shackle, body and keyhole.
      const w = Math.round(size * 0.32);
      const bh = Math.round(w * 0.78);
      const sh = Math.round(w * 0.62);
      const t = Math.max(2, Math.round(w / 7));
      const ink = lit({ r: 0.86, g: 0.87, b: 0.9, a: 1 });
      box("Lock", { width: pct(1), height: pct(1), align: "center", cross: "center", part: sel("AbilitySlot", "lock") }, undefined, () => {
        // The shackle's arch, its legs down into the body, then the body over them.
        const leg = Math.round(sh * 0.22);
        const total = Math.round(sh / 2 + leg + bh);
        box("Mark", { width: px(w), height: px(total) }, undefined, () => {
          const x0 = (w - sh) / 2;
          leaf("Shackle", { width: px(sh), height: px(sh) }, { shape: { kind: "arc", start: Math.PI, sweep: Math.PI, thickness: t, cap: "butt" }, fills: [solid(ink)] }, absolute({ start: x0, top: 0 }));
          leaf("Legs", { width: px(sh), height: px(leg + 2) }, {
            shape: { kind: "path", points: [{ x: t / 2, y: 0 }, { x: t / 2, y: leg + 2 }], width: t },
            fills: [solid(ink)],
          }, absolute({ start: x0, top: sh / 2 - 1 }));
          leaf("Leg", { width: px(sh), height: px(leg + 2) }, {
            shape: { kind: "path", points: [{ x: sh - t / 2, y: 0 }, { x: sh - t / 2, y: leg + 2 }], width: t },
            fills: [solid(ink)],
          }, absolute({ start: x0, top: sh / 2 - 1 }));
          box("Body", { width: px(w), height: px(bh), align: "center", cross: "center" }, { shape: { kind: "rect", radii: [t, t, t, t] }, fills: [solid(ink)], effects: [drop(1, 4, lit(BLACK, 0.8))] }, () => {
            leaf("Keyhole", { width: px(Math.max(3, Math.round(w * 0.16))), height: px(Math.max(4, Math.round(bh * 0.4))) }, { shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [solid(lit(BLACK, 0.75))] });
          }, absolute({ start: 0, top: sh / 2 + leg }));
        });
      }, absolute({ start: 0, top: 0 }));
    }
    if (disabled) {
      // Silenced or unusable: the washed art keeps the slot, a small prohibition badge in its top end corner says why
      // (a full-slot slash read as a debug line).
      const b = Math.round(Math.max(14, size * 0.3));
      const bt = Math.max(1.5, b / 9);
      box("Slash", { width: px(b), height: px(b), part: sel("AbilitySlot", "slash") }, {
        shape: { kind: "ellipse" },
        fills: [solid(lit(BLACK, 0.82))],
        strokes: [stroke(bt, lit(role("danger")))],
        effects: [drop(1, 3, lit(BLACK, 0.7))],
      }, () => {
        const q = b * 0.27;
        leaf("Bar", { width: px(b), height: px(b) }, { shape: { kind: "path", points: [{ x: b - q, y: q }, { x: q, y: b - q }], width: bt, cap: "butt" }, fills: [solid(lit(role("danger")))] }, absolute({ start: 0, top: 0 }));
      }, absolute({ end: line + u(2), top: line + u(2) }));
    }
    // A command card's key, printed in the cell's top-left corner (below): a small cap at any text size, about a
    // fifth of the cell. The largest step within a fifth of the cell; at 200 % even the ramp's smallest step is larger
    // than that, so it is drawn down to the fifth, but never under the smallest text the checks allow (§15.2).
    const corner0 = keyCorner ? desc.hotkey || (desc.action ? kit.frameGlyph(desc.action)?.text ?? "" : "") : "";
    const kStep = tsFit(10, size * 0.2, true);
    const kSize = stepSize(kStep).size;
    const kk = Math.min(1, Math.max(minText(), size * 0.2) / kSize);
    const kTop = line + Math.max(1, u(2));
    // Where the key's line ends: the seconds stay below it.
    const keyBottom = corner0 ? kTop + stepSize(kStep).line * kk : 0;
    // One centred numeral: the seconds left while cooling; under a corner key, centred in the room below it, so at
    // 200 % text in a small cell the two never meet.
    if (cooling && !locked) {
      const room = size - line - keyBottom;
      const tStep = tsFit(size >= 52 ? 20 : 17, corner0 ? Math.min(size * 0.4, room * 0.85) : size * 0.4, true);
      const { size: tSize, line: tLine } = stepSize(tStep);
      if ((size - tLine) / 2 >= keyBottom) {
        text("Time", shortTime(cooldown, true), { step: tStep, font: "display", color: token("ink"), nowrap: true, outline: 2.4 }, absolute({ centerX: true, centerY: true }));
      } else {
        // Drawn down to the room's line, never under the smallest text the checks allow (as the key is).
        const tk = Math.min(1, Math.max(minText() / tSize, room / tLine));
        text("Time", shortTime(cooldown, true), { step: tStep, font: "display", color: token("ink"), nowrap: true, outline: 2.4 },
          absolute({ centerX: true, top: keyBottom + Math.max(0, (room - tLine * tk) / 2) }, tk < 1 ? { transform: { scaleX: tk, scaleY: tk, pivotX: 0.5, pivotY: 0 } } : {}));
      }
    }
    // The ultimate's charge: a gold ring filling clockwise round the art, inside the frame so it never
    // crosses into the gap or a neighbour, and its percent small at the bottom on a dark pill.
    if (ult !== null && !locked) {
      const ringT = Math.max(3, Math.round(size / 14));
      const ringD = size - line * 2 - u(4);
      const at = absolute({ start: line + u(2), top: line + u(2) });
      leaf("UltTrack", { width: px(ringD), height: px(ringD) }, { shape: arc(1, ringT, "butt"), fills: [solid(lit(BLACK, 0.55))] }, at);
      leaf("UltRing", { width: px(ringD), height: px(ringD), part: sel("AbilitySlot", "ultimate", ultReady ? "ready" : null) }, {
        shape: arc(clamp01(ult), ringT),
        fills: [solid(lit(mix(role("gold"), WHITE, 0.1), ultReady ? 0.75 + 0.25 * pulse(0.9) : 1))],
        effects: [glow(u(4), lit(role("gold"), 0.6))],
      }, at);
      if (!ultReady) pill("Ultimate", `${Math.floor(clamp01(ult) * 100)}%`, token("ink"), size, line, ringT);
    }
    // The cost: a small figure with a droplet on a dark pill at the bottom, only while the slot could be pressed; red only when it cannot be paid.
    if (desc.cost && ready) {
      pill("Cost", desc.cost, desc.unaffordable ? lit(role("danger")) : token("ink"), size, line, 0, ["drop", "droplet"].find((n) => kit.hasIcon(n)), desc.unaffordable ? lit(role("danger")) : lit(mix(role("mana"), WHITE, 0.35)), sel("AbilitySlot", "cost", desc.unaffordable ? "unaffordable" : null));
    }
    if (charged) {

      // Charges in a small badge at the bottom right corner, sized by its numeral so it stays a badge at any text size.
      // With none left the slot is cooling and its seconds say so: a "0" beside them would be a second numeral.
      if (charges > 0) {
        const badgeStep = tsFit(9, size * 0.2, true);
        // Large enough that its numeral is never under the smallest text the checks allow (§15.2).
        const badge = Math.round(Math.max(16, size * 0.28, Math.ceil(minText() / 0.66)));
        const fit = shrunk(badgeStep, badge * 0.66, stepSize(badgeStep).line);
        box("ChargeBadge", { width: px(badge), height: px(badge), part: sel("AbilitySlot", "charges") }, {
          shape: { kind: "ellipse" },
          fills: [solid(lit(BLACK, 0.78))],
          strokes: [stroke(1, token("lineStrong", 0.9))],
        }, () => {
          text("Charges", String(charges), { step: badgeStep, font: "display", color: token("ink"), nowrap: true }, absolute({ centerX: true, centerY: true }, fit.transform ? { transform: { ...fit.transform, y: 0 } } : {}));
        }, absolute({ end: line + 1, bottom: line + 1 }));
      }
    }
    // A command card's key: printed in the cell's top-left corner, small, over a soft shadow (StarCraft II's grid keys).
    if (corner0) {
      text("Key", corner0, { step: kStep, font: "display", color: token("inkMuted", locked ? 0.6 : 1), nowrap: true, soft: true },
        absolute({ start: line + Math.max(2, u(3)), top: kTop }, kk < 1 ? { transform: { scaleX: kk, scaleY: kk, pivotX: 0, pivotY: 0 } } : {}));
    }
    // Active or an ultimate ready: a thin accent edge inside the frame, over the art.
    if (accent) {
      leaf("Edge", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "edge", desc.active ? "active" : "ultimate") }, {
        shape: { kind: "rect", radii: [r, r, r, r], corner },
        strokes: [stroke(Math.max(1.5, line), lit(accent, 0.8 + 0.2 * breath))],
      }, absolute({ start: 0, top: 0 }));
    } else {
      leaf("Edge", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "edge") }, {
        shape: { kind: "rect", radii: [r, r, r, r], corner },
        // A top-lit rim: lighter along the top, darker along the bottom, so the slot reads as a bevelled frame, not a pasted picture.
        strokes: [{ width: line, align: "inside", paint: vgrad([[0, lit(WHITE, ready ? 0.42 : 0.26)], [0.5, lit(WHITE, 0.08)], [1, lit(BLACK, 0.7)]]) }],
      }, absolute({ start: 0, top: 0 }));
    }
    r0 = close({
      transform: { scaleX: s, scaleY: s },
      a11y: a11y("button", desc.name ?? "", { value: locked ? "Locked" : cooling ? `${Math.ceil(cooldown)}s` : disabled ? "Unavailable" : "Ready" }),
      ...groups({ ready: readyState, use }, { ready: ["ready", "cooling", "disabled", "locked"], use: ["idle", "active", "unaffordable"] }),
    });
    // The ranks: pips right under the slot, centred.
    const maxLevel = desc.maxLevel ?? 0;
    const pipH = Math.max(3, Math.round(size / 20));
    if (maxLevel > 0) {
      kit.row("Pips", { gap: u(2), height: px(pipH) }, () => {
        for (let i = 0; i < maxLevel; i++) {
          const filled = i < (desc.level ?? 0);
          leaf(kit.idi("Pip", i), { width: px(Math.max(6, Math.round(size / 7))), height: px(pipH), part: sel("AbilitySlot", "pip", filled ? "filled" : null) }, {
            shape: { kind: "rect", radii: [1, 1, 1, 1] },
            fills: [solid(filled ? lit(role("gold")) : token("shadow", 0.7))],
            strokes: [stroke(1, filled ? lit(mix(role("gold"), { r: 1, g: 1, b: 1, a: 1 }, 0.4)) : token("line"))],
            effects: filled ? [glow(3, lit(role("gold"), 0.5))] : [],
          });
        }
      });
      // Out of the flow, just under the slot's bottom edge: a slot with ranks keeps the row's bottom line and its hotkey's.
      apply(absolute({ centerX: true, top: size + 1 }, { overlaps: [path.slice(0, path.lastIndexOf("/"))] }));
    }
    // The next charge recharging: a slim bar in the same row under the slot (under the ranks when it has both).
    if (charged && charges < maxCharges) {
      const p = clamp01(desc.chargeProgress ?? 0);
      box("Recharge", { width: px(Math.round(size * 0.56)), height: px(pipH), part: sel("AbilitySlot", "recharge") }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(token("shadow", 0.7))], strokes: [stroke(1, token("line"))] }, () => {
        leaf("Fill", { width: pct(p), height: pct(1) }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(lit(role("gold")))], effects: [glow(3, lit(role("gold"), 0.45))] });
      }, absolute({ centerX: true, top: keyCorner ? size - pipH - line - Math.max(2, u(3)) : size + 1 + (maxLevel > 0 ? pipH + u(2) : 0) }, { overlaps: [path.slice(0, path.lastIndexOf("/"))] }));
    }
    // The hotkey: a small, quiet label under the slot, else the action's glyph on the device used last.
    const glyph = !keyCorner && !desc.hotkey && desc.action ? kit.frameGlyph(desc.action) : null;
    const keyText = keyCorner ? "" : desc.hotkey || glyph?.text || "";
    const keyIcon = glyph && glyph.device !== "keyboardMouse" && kit.hasIcon(glyph.icon) ? glyph.icon : "";
    if (keyText || keyIcon) {
      const keyStep = ts(10);
      // One neutral ink at full strength: quiet means small, not faint (it reads at 4.5:1 on a light scene with its shadow);
      // state lives on the slot (the tints, the glow), never on the key's label.
      const capInk = token("ink");
      // At most about a quarter of the slot's height, so at 200 % text the key stays a quiet label and the art leads.
      const keyPx = Math.min(stepSize(keyStep).size, size * 0.26);
      const fit = shrunk(keyStep, keyPx, stepSize(keyStep).line);
      box("Hotkey", { height: px(Math.ceil(keyPx * 1.05)), width: px(Math.ceil(Math.max(keyPx, keyText.length * keyPx * 0.72) + 4)), cross: "center", align: "center", part: sel("AbilitySlot", "hotkey") }, undefined, () => {
        if (keyIcon) icon("Glyph", keyIcon, Math.round(keyPx * 1.1), capInk);
        else {
          // No outline (at 200 % one makes the letters look hollow): a soft shadow holds them on a bright scene.
          text("Key", keyText, { step: keyStep, font: "display", color: capInk, nowrap: true, soft: true }, absolute({ centerX: true, centerY: true }, fit.transform ? { transform: { scaleX: fit.k, scaleY: fit.k } } : {}));
        }
      }, { overlaps: [path] });
    }
  });
  return r0 ?? NOTHING;
}

/**
 * A small figure on a dark pill at the bottom centre of a slot (a cost, an
 * ultimate's percent): about a fifth of the slot whatever the text size, so
 * it never competes with the cooldown's numeral, and readable over any art.
 */
function pill(id: string, value: string, ink: Paint, size: number, line: number, lift: number, glyphName?: string, glyphInk?: Paint, part?: string): void {
  const step = tsFit(10, size * 0.2, true);
  const { line: lh, size: fs } = stepSize(step);
  const fit = shrunk(step, Math.max(12, size * 0.19), lh);
  box(id, { direction: "row", gap: 1, cross: "center", height: px(Math.ceil(lh)), padding: { left: u(3), right: u(4), top: 0, bottom: 0 }, ...(part ? { part } : {}) }, {
    shape: { kind: "rect", radii: [lh / 2, lh / 2, lh / 2, lh / 2] },
    fills: [solid(lit(BLACK, 0.68))],
  }, () => {
    if (glyphName) icon("Drop", glyphName, Math.round(fs * 0.9), glyphInk ?? ink);
    text("Value", value, { step, font: "display", color: ink, nowrap: true, outline: 1.2 });
  }, absolute({ centerX: true, bottom: line + Math.max(1, Math.round(size * 0.03)) + lift }, fit.transform ? { transform: fit.transform } : {}));
}

export function abilityBar(id: string, desc: Api.AbilityBarDesc = {}): Api.AbilityBarResult {
  const f = frame();
  const tk = f.t;
  const anchor = anchorOf(desc, "bottom");
  let clicked: string | null = null;
  const pressed: string[] = [];
  const size = slotSize(desc.size);
  const gap = desc.gap || Math.max(tk.sm, Math.round(size * 0.18));
  const row = (desc.direction ?? "row") === "row";
  const slots = desc.slots ?? [];
  // More slots than the HUD band holds wrap into even rows (a phone's bar), never off the screen; each row's columns line up with the first's.
  const room = hudRoom() - tk.sm * 2;
  // An ultimate is 1.15 times a slot: a wrapped row holds its extra width.
  const extra = slots.some((x) => x.ultimate != null) ? Math.round(size * 0.15) : 0;
  const perRow = Math.max(1, Math.floor((room - extra + gap) / (size + gap)));
  const count = slots.length;
  const wraps = row && count > perRow;
  const cols = wraps ? Math.ceil(count / Math.ceil(count / perRow)) : count;
  hudPlace(id, anchor, () => {
    // No box behind the slots: they sit on the scene, or on what a style dresses AbilityBar.frame with.
    box(id, {
      direction: desc.direction ?? "row",
      gap,
      wrap: wraps,
      ...(wraps ? { width: px(cols * size + (cols - 1) * gap + tk.xs * 2 + extra) } : {}),
      align: wraps ? "start" : "center",
      padding: { left: tk.xs, right: tk.xs, top: tk.xs, bottom: 0 },
      // Bottom-aligned: a larger ultimate rises above the row and every hotkey shares a line.
      cross: "end",
      part: sel("AbilityBar", "frame"),
      partClass: "hud",
    }, undefined, () => {
      for (const s of slots) {
        const key = s.key ?? "";
        const res = slot(keyId(key), s, size);
        if (res.clicked) clicked = key;
        if (res.pressed) pressed.push(key);
      }
    }, { a11y: a11y("group", "Abilities") });
  }, { margin: tk.gutter });
  return { clicked, pressed };
}

export function statusGrid(id: string, desc: Api.StatusGridDesc = {}, target = false): Api.StatusGridResult {
  const tk = frame().t;
  const size = u(desc.size || 32);
  const columns = Math.max(1, desc.columns || 8);
  const effects = desc.effects ?? [];
  const split = desc.split !== false;
  const limit = desc.max ?? 0;
  let hovered: string | null = null;
  let clicked: string | null = null;
  const anchor = anchorOf(desc, null);
  const rowOf = (rowId: string, list: readonly Api.StatusEffect[], label: string) => {
    if (list.length === 0) return;
    const shown = limit > 0 && list.length > limit ? list.slice(0, limit - 1) : list;
    const more = list.length - shown.length;
    // Every column as wide as the row's widest timer needs (WoW's auras): the tiles stay evenly spaced, each duration
    // centred under its tile and overhanging it, never pushing its own column wider than its neighbours'.
    const timers = desc.timers !== false;
    const col = Math.max(size, ...shown.map((fx) => timeOf(fx, size, timers, target).width));
    // Anchored, a row never runs past the HUD band: wider columns (200 % text) wrap sooner.
    const fit = anchor ? Math.max(1, Math.min(columns, Math.floor((hudRoom() + u(4)) / (col + u(4))))) : columns;
    box(rowId, { direction: "row", wrap: true, gap: u(4), width: px(fit * (col + u(4))) }, undefined, () => {
      for (const fx of shown) {
        const res = tile(keyId(fx.key ?? ""), fx, size, timers, col, target);
        if (res.hovered || res.focused) hovered = fx.key ?? "";
        if (res.clicked) clicked = fx.key ?? "";
      }
      if (more > 0) {
        // The overflow in the tiles' own glass: the dark socket, a hairline, the top light, a small count in the display face.
        const r = Math.min(shapeOf("slot").radius, size / 5);
        kit.column("More", { width: px(col), cross: "center" }, () => box("Tile", { width: px(size), height: px(size), align: "center", cross: "center", part: sel("StatusGrid", "more"), partClass: "slot" }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [solid(lit(BLACK, 0.9)), vgrad([[0, lit(WHITE, 0.1)], [0.55, lit(WHITE, 0.02)]])],
          strokes: [stroke(1, lit(BLACK, 0.85)), stroke(1, lit(BLACK, 0.6), "outside")],
          effects: [drop(u(2), u(6), lit(BLACK, 0.5))],
        }, () => {
          socketRim("Rim", size - 2, Math.max(0, r - 1), undefined, 1, absolute({ start: 1, top: 1 }), 0.35);
          text("Count", `+${more}`, { step: tsFit(11, size * 0.34), font: "display", color: token("ink"), nowrap: true, soft: true });
        }, { a11y: a11y("text", `${more} more`) }));
      }
    }, { a11y: a11y("list", label) });
  };
  hudPlace(id, anchor, () => {
    kit.column(id, { gap: tk.xs }, () => {
      if (split) {
        rowOf("Buffs", effects.filter((x) => !x.debuff), "Buffs");
        rowOf("Debuffs", effects.filter((x) => x.debuff), "Debuffs");
      } else rowOf("Effects", effects, "Effects");
    });
  });
  return { hovered, clicked };
}

/** A tile's timer text and the width it takes with a gutter, so neighbouring timers read apart ("5m 6m", never "5m6m"). */
/** A tile's side: one size across a grid; in a target's frame the player's own effects larger than the rest. */
function tileSide(fx: Api.StatusEffect, base: number, target: boolean): number {
  return target && !fx.mine ? Math.round(base * 0.85) : base;
}

function timeOf(fx: Api.StatusEffect, base: number, timers: boolean, target = false): { text: string; step: Api.TypeStepName; px: number; width: number } {
  const size = tileSide(fx, base, target);
  const remaining = fx.remaining ?? 0;
  const step = tsFit(11, size * 0.3);
  // About half the tile's height at most, whatever the text size: a secondary figure under the art, never its rival.
  const px0 = Math.min(stepSize(step).size, size * 0.46);
  const text = timers && remaining > 0 ? shortTime(remaining, false) : "";
  return { text, step, px: px0, width: text ? Math.ceil(text.length * px0 * 0.62) + frame().t.sm + 4 : 0 };
}

function tile(id: string, fx: Api.StatusEffect, base: number, timers: boolean, col = base, target = false): Api.Response {
  const size = tileSide(fx, base, target);
  const remaining = fx.remaining ?? 0;
  const lasting = remaining <= 0;
  const expiring = !lasting && remaining <= 5;
  const ink = fx.debuff ? typeColor(fx.type) : role("success");
  const time = lasting ? "lasting" : expiring ? "expiring" : "running";
  // Blinks in its last five seconds, faster in its last two (WoW's auras).
  const blink = expiring && !reduced() ? 0.45 + 0.55 * (0.5 + 0.5 * Math.cos(kit.seconds() * Math.PI * 2 * (remaining <= 2 ? 3 : 1.5))) : 1;
  const r = Math.min(shapeOf("slot").radius, size / 5);
  let res: Api.Response | null = null;
  // The column is the row's (as wide as its widest timer): every tile in it centred, its timer centred under it.
  const { text: timeText, step: timeStep, px: timePx } = timeOf(fx, base, timers, target);
  const timeFit = shrunk(timeStep, timePx, stepSize(timeStep).line);
  const timeH = timeText ? Math.ceil(timePx * 1.25) : 0;
  kit.column(id, { gap: 0, cross: "center", width: px(Math.max(size, base, col)), height: px(size + timeH) }, () => {
    // A debuff's rim carries its dispel type; a buff's is a dark hairline under the socket's top light,
    // so type colour belongs to debuffs alone (WoW's and Diablo IV's auras).
    const rim = fx.debuff ? Math.max(1.5, Math.round(size / 18)) : 1;
    const art = size - rim * 2;
    open("Tile", {
      width: px(size), height: px(size), interactive: true,
      part: sel("StatusGrid", "tile", fx.debuff ? "debuff" : "buff", fx.type ? ["type", fx.type] : null, expiring ? "expiring" : null),
      partClass: "slot",
    }, {
      // A debuff's notch at its top right corner, so its kind is not told by colour alone.
      shape: { kind: "rect", radii: fx.debuff ? [r, size * 0.3, r, r] : [r, r, r, r], corner: fx.debuff ? "bevel" : "round" },
      fills: [solid(lit(BLACK, 0.9))],
      // The rim in the effect's colour (a debuff's dispel type), on the tile's own float shadow.
      strokes: [stroke(rim, fx.debuff ? lit(ink) : lit(BLACK, 0.85)), stroke(1, lit(BLACK, 0.6), "outside")],
      effects: [drop(u(2), u(6), lit(BLACK, 0.5))],
      clip: "shape",
    });
    artTile("Icon", fx.image, fx.icon ?? fx.name ?? "", art, fx.debuff ? ink : hueOf(fx.icon || fx.name || id), absolute({ start: rim, top: rim }), sel("StatusGrid", "icon"));
    if (!lasting && (fx.duration ?? 0) > 0) {
      // The time gone darkens clockwise (linear-light blending: 75 % reads as about half).
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("StatusGrid", "sweep") }, {
        shape: pie(1 - clamp01(remaining / (fx.duration ?? 1))),
        fills: [solid(lit(BLACK, 0.75))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
    }
    socketRim("Rim", art, Math.max(0, r - rim), undefined, 1, absolute({ start: rim, top: rim }), 0.35);
    if ((fx.stacks ?? 0) >= 2) {
      // Small in the bottom end corner (as a charge badge is): the art stays the tile's one large thing.
      const sStep = tsFit(10, size * 0.4, true);
      const sFit = shrunk(sStep, size * 0.36, stepSize(sStep).line);
      text("Stacks", String(fx.stacks), { step: sStep, font: "display", color: token("ink"), nowrap: true, outline: 1.2 }, absolute({ end: rim + 1, bottom: 0 }, sFit.transform ? { transform: { ...sFit.transform, x: ((1 - sFit.k) * String(fx.stacks).length * stepSize(sStep).size * 0.3) } } : {}));
    }
    res = close({
      opacity: blink,
      a11y: a11y("image", [fx.name ?? "", (fx.stacks ?? 0) >= 2 ? `${fx.stacks} stacks` : "", lasting ? "" : `${Math.ceil(remaining)} seconds`].filter(Boolean).join(", ")),
      ...groups({ time }, { time: ["lasting", "running", "expiring"] }),
    });
    if (timers && !lasting) {
      // One ink: expiring is said by the tile's blink, not by a second colour.
      text("Time", timeText, { step: timeStep, font: "display", color: token("ink"), nowrap: true, soft: true }, absolute({ centerX: true, top: size + (timeH - stepSize(timeStep).line) / 2 }, timeFit.transform ? { transform: { scaleX: timeFit.k, scaleY: timeFit.k } } : {}));
    }
  });
  return res ?? ({ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 } as Api.Response);
}

