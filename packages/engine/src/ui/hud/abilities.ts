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
import { stepSize, hudRoom, a11y, arc, inner, type Paint, absolute, anchorOf, box, bp, chip, clamp01, close, drop, edge, frame, glow, groups, hudPlace, icon, innerGlow, keyId, kit, leaf, lit, mix, msToTicks, open, pct, pop, pulse, px, radial, reduced, role, sel, shapeOf, shortTime, solid, sound, stroke, teamColor, text, token, ts, u, vgrad, type Rgba, apply, textScale, tsFit } from "./core.ts";

const DEBUFF_TYPES: Record<string, string> = {
  magic: "#3d8bff", curse: "#a64dff", poison: "#3ccf4e", disease: "#c89a2c", bleed: "#e03030",
};

function typeColor(type: string | undefined): Rgba {
  const c = type ? DEBUFF_TYPES[type] : undefined;
  if (!c) return role("danger");
  const n = (i: number) => Number.parseInt(c.slice(i, i + 2), 16) / 255;
  return { r: n(1), g: n(3), b: n(5), a: 1 };
}

const BLACK = { r: 0, g: 0, b: 0, a: 1 };
const WHITE = { r: 1, g: 1, b: 1, a: 1 };

/**
 * Text shrunk below the type ramp's smallest step: at 200 % even the ramp's
 * least step outgrows a corner figure, so it is drawn at a step and scaled
 * to `target` px about its centre; `h` is its box's height, and the move
 * keeps its bottom where layout put it.
 */
function shrunk(step: Api.TypeStepName, target: number, h: number): { k: number; transform?: Api.UiTransform } {
  const k = Math.min(1, target / stepSize(step).size);
  return k >= 1 ? { k } : { k, transform: { x: 0, y: ((1 - k) * h) / 2, scaleX: k, scaleY: k } };
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

/** A stable hue for an ability without art, from its icon or name, so a placeholder bar is still told apart by colour as well as glyph. */
function hueOf(key: string): Rgba {
  let h = 0;
  for (const ch of key) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return teamColor(Math.abs(h));
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
function slot(id: string, desc: Api.AbilitySlotDesc, base = slotSize(desc.size)): Api.Response {
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
    if (desc.image) {
      kit.image("Icon", { texture: desc.image, width: px(artSide), height: px(artSide), label: "" });
      apply(absolute({ start: line, top: line }));
    } else {
      // No texture: a tinted gradient with the glyph large, never a dark square with a thin line icon.
      const hue = hueOf(desc.icon || desc.name || id);
      box("IconBox", { width: px(artSide), height: px(artSide), align: "center", cross: "center", part: sel("AbilitySlot", "icon") }, {
        fills: [
          radial([[0, lit(mix(hue, { r: 1, g: 1, b: 1, a: 1 }, 0.25))], [0.65, lit(mix(hue, { r: 0, g: 0, b: 0, a: 1 }, 0.35))], [1, lit(mix(hue, { r: 0, g: 0, b: 0, a: 1 }, 0.7))]], 0.5, 0.35, 0.75, 0.75),
        ],
      }, () => {
        icon("Glyph", desc.icon ?? "", Math.round(artSide * 0.66), lit({ r: 1, g: 1, b: 1, a: 1 }, 0.95));
      }, absolute({ start: line, top: line }));
    }
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
      // The part still to come under 66 % black, the part done left clean: the fraction left reads without the numeral.
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("AbilitySlot", "sweep") }, {
        shape: pie(left),
        fills: [solid(lit(BLACK, 0.66))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
      // The sweep's leading edge: a bright line about 1.5 px wide from the rim inward, under the numerals.
      const angle = -Math.PI / 2 + (1 - left) * Math.PI * 2;
      const half = 0.75 / Math.max(1, size * 0.5);
      leaf("Hand", { width: px(size * 1.5), height: px(size * 1.5) }, {
        shape: { kind: "arc", start: angle - half, sweep: half * 2, thickness: size * 0.4, cap: "butt" },
        fills: [solid(lit(WHITE, 0.9))],
        effects: [glow(3, lit(WHITE, 0.5))],
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
      // A red slash corner to corner over the washed art.
      const m = artSide * 0.16;
      leaf("Slash", { ...cover, part: sel("AbilitySlot", "slash") }, {
        shape: { kind: "path", points: [{ x: artSide - m, y: m }, { x: m, y: artSide - m }], width: Math.max(2.5, size / 16), cap: "round" },
        fills: [solid(lit(role("danger")))],
        effects: [glow(5, lit(role("danger"), 0.75))],
      }, full());
    }
    // One centred numeral: the seconds left while cooling.
    if (cooling && !locked) {
      text("Time", shortTime(cooldown, true), { step: tsFit(size >= 52 ? 20 : 17, size * 0.4, true), font: "display", color: token("ink"), nowrap: true, outline: 2.4 },
        absolute({ centerX: true, centerY: true }));
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
      if (charges < maxCharges) {
        const p = clamp01(desc.chargeProgress ?? 0);
        box("Recharge", { width: px(artSide), height: px(Math.max(2, Math.round(size / 20))), part: sel("AbilitySlot", "recharge") }, { fills: [solid(token("shadow", 0.7))] }, () => {
          leaf("Fill", { width: pct(p), height: pct(1) }, { fills: [solid(lit(role("gold")))] });
        }, absolute({ start: line, bottom: line }));
      }
      // Charges in a small badge at the bottom right corner, sized by its numeral so it stays a badge at any text size.
      // With none left the slot is cooling and its seconds say so: a "0" beside them would be a second numeral.
      if (charges > 0) {
        const badgeStep = tsFit(9, size * 0.2, true);
        const badge = Math.round(Math.max(16, size * 0.28));
        const fit = shrunk(badgeStep, badge * 0.66, stepSize(badgeStep).line);
        box("ChargeBadge", { width: px(badge), height: px(badge), part: sel("AbilitySlot", "charges") }, {
          shape: { kind: "ellipse" },
          fills: [solid(lit(BLACK, 0.78))],
          strokes: [stroke(1, token("lineStrong", 0.9))],
        }, () => {
          text("Charges", String(charges), { step: badgeStep, font: "display", color: token("ink"), nowrap: true }, absolute({ centerX: true, centerY: true }, fit.transform ? { transform: { ...fit.transform, y: 0 } } : {}));
        }, absolute({ end: line + 1, bottom: line + 1 + (charges < maxCharges ? Math.max(2, Math.round(size / 20)) : 0) }));
      }
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
    // The ranks: pips right under the slot, centred (a bar bottom-aligns its slots, so every hotkey still shares a line).
    const maxLevel = desc.maxLevel ?? 0;
    const pipH = Math.max(3, Math.round(size / 16));
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
    }
    // The hotkey: a small, quiet label under the slot, else the action's glyph on the device used last.
    const glyph = !desc.hotkey && desc.action ? kit.frameGlyph(desc.action) : null;
    const keyText = desc.hotkey || glyph?.text || "";
    const keyIcon = glyph && glyph.device !== "keyboardMouse" && kit.hasIcon(glyph.icon) ? glyph.icon : "";
    if (keyText || keyIcon) {
      const keyStep = ts(10);
      // One quiet ink: state lives on the slot (the tints, the glow), never on the key's label.
      const capInk = token("inkMuted");
      box("Hotkey", { height: px(Math.ceil(stepSize(keyStep).size)), cross: "center", align: "center", part: sel("AbilitySlot", "hotkey") }, undefined, () => {
        if (keyIcon) icon("Glyph", keyIcon, Math.round(stepSize(keyStep).size * 1.1), capInk);
        else text("Key", keyText, { step: keyStep, font: "display", color: capInk, nowrap: true, outline: 1.4 });
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

export function statusGrid(id: string, desc: Api.StatusGridDesc = {}): Api.StatusGridResult {
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
    box(rowId, { direction: "row", wrap: true, gap: u(4), width: px(columns * (size + u(4))) }, undefined, () => {
      for (const fx of shown) {
        const res = tile(keyId(fx.key ?? ""), fx, size, desc.timers !== false);
        if (res.hovered || res.focused) hovered = fx.key ?? "";
        if (res.clicked) clicked = fx.key ?? "";
      }
      if (more > 0) {
        box("More", { width: px(size), height: px(size), align: "center", cross: "center", part: sel("StatusGrid", "more"), partClass: "slot" }, chip(shapeOf("slot").radius), () => {
          text("Count", `+${more}`, { step: tsFit(13, size * 0.42), font: "mono", color: token("ink"), nowrap: true });
        }, { a11y: a11y("text", `${more} more`) });
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

function tile(id: string, fx: Api.StatusEffect, base: number, timers: boolean): Api.Response {
  const size = fx.mine ? base : Math.round(base * 0.85);
  const remaining = fx.remaining ?? 0;
  const lasting = remaining <= 0;
  const expiring = !lasting && remaining <= 5;
  const ink = fx.debuff ? typeColor(fx.type) : role("success");
  const time = lasting ? "lasting" : expiring ? "expiring" : "running";
  // Blinks in its last five seconds, faster in its last two (WoW's auras).
  const blink = expiring && !reduced() ? 0.45 + 0.55 * (0.5 + 0.5 * Math.cos(kit.seconds() * Math.PI * 2 * (remaining <= 2 ? 3 : 1.5))) : 1;
  const r = Math.min(shapeOf("slot").radius, size / 5);
  let res: Api.Response | null = null;
  // The timer under the tile can be wider than the tile (a long time, 200 % text): the column grows to hold it.
  const timeStep = tsFit(11, size * 0.3);
  const timeText = timers && !lasting ? shortTime(remaining, false) : "";
  // With a gutter, so neighbouring timers read as numbers apart ("5m 6m"), never one run ("5m6m").
  const timeWidth = timeText ? Math.ceil(timeText.length * stepSize(timeStep).size * 0.62) + frame().t.sm + 4 : 0;
  kit.column(id, { gap: 0, cross: "center", width: px(Math.max(size, base, timeWidth)) }, () => {
    open("Tile", {
      width: px(size), height: px(size), interactive: true,
      part: sel("StatusGrid", "tile", fx.debuff ? "debuff" : "buff", fx.type ? ["type", fx.type] : null, expiring ? "expiring" : null),
      partClass: "slot",
    }, {
      // A debuff's notch at its top right corner, so its kind is not told by colour alone.
      shape: { kind: "rect", radii: fx.debuff ? [r, size * 0.3, r, r] : [r, r, r, r], corner: fx.debuff ? "bevel" : "round" },
      fills: [solid(token("shadow", 0.85))],
      strokes: [stroke(Math.max(1.5, size / 16), lit(ink, fx.debuff ? 1 : 0.6))],
      effects: [drop(1, 3, token("shadow", 0.5))],
      clip: "shape",
    });
    if (fx.image) {
      kit.image("Icon", { texture: fx.image, width: px(size - 4), height: px(size - 4), label: "" });
      apply(absolute({ start: 2, top: 2 }));
    } else {
      // No texture: a gradient in the effect's colour with the glyph large, not a line icon on a dark square.
      const hue = fx.debuff ? ink : hueOf(fx.icon || fx.name || id);
      box("IconBox", { width: px(size - 4), height: px(size - 4), align: "center", cross: "center", part: sel("StatusGrid", "icon") }, {
        fills: [radial([[0, lit(mix(hue, { r: 1, g: 1, b: 1, a: 1 }, 0.2))], [0.7, lit(mix(hue, { r: 0, g: 0, b: 0, a: 1 }, 0.4))], [1, lit(mix(hue, { r: 0, g: 0, b: 0, a: 1 }, 0.7))]], 0.5, 0.35, 0.75, 0.75)],
      }, () => {
        icon("Icon", fx.icon ?? fx.name ?? "", Math.round(size * 0.66), lit({ r: 1, g: 1, b: 1, a: 1 }, 0.95));
      }, absolute({ start: 2, top: 2 }));
    }
    if (!lasting && (fx.duration ?? 0) > 0) {
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("StatusGrid", "sweep") }, {
        shape: pie(1 - clamp01(remaining / (fx.duration ?? 1))),
        fills: [solid(token("shadow", 0.55))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
    }
    if ((fx.stacks ?? 0) >= 2) {
      text("Stacks", String(fx.stacks), { step: tsFit(12, size * 0.5, true), font: "display", color: token("ink"), nowrap: true, outline: 1.2 }, absolute({ end: 1, bottom: 0 }));
    }
    res = close({
      opacity: blink,
      a11y: a11y("image", [fx.name ?? "", (fx.stacks ?? 0) >= 2 ? `${fx.stacks} stacks` : "", lasting ? "" : `${Math.ceil(remaining)} seconds`].filter(Boolean).join(", ")),
      ...groups({ time }, { time: ["lasting", "running", "expiring"] }),
    });
    if (timers && !lasting) {
      text("Time", timeText, { step: timeStep, font: "mono", color: expiring ? lit(role("warn")) : token("ink"), nowrap: true, outline: 1 });
    }
  });
  return res ?? ({ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 } as Api.Response);
}

