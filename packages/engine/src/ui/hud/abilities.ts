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
import { stepSize, a11y, absolute, anchorOf, box, bp, chip, clamp01, close, drop, edge, frame, glow, groups, hudPlace, icon, innerGlow, keyId, kit, leaf, lit, mix, msToTicks, open, pct, pop, pulse, px, radial, reduced, role, sel, shapeOf, shortTime, solid, sound, stroke, teamColor, text, token, ts, u, vgrad, type Rgba, apply, textScale, tsFit } from "./core.ts";

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

/** A stable hue for an ability without art, from its icon or name, so a placeholder bar is still told apart by colour as well as glyph. */
function hueOf(key: string): Rgba {
  let h = 0;
  for (const ch of key) h = (h * 31 + ch.charCodeAt(0)) | 0;
  return teamColor(Math.abs(h));
}

const NOTHING = { hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 } as Api.Response;

export function abilitySlot(id: string, desc: Api.AbilitySlotDesc = {}): Api.Response {
  return slot(id, desc, (desc.maxLevel ?? 0) > 0);
}

/**
 * One slot, as shipped action bars draw it (Diablo IV, League, Destiny 2,
 * Genshin): the art fills the frame edge to edge and stays the dominant
 * element in every state; one centred numeral per state (the cooldown's
 * seconds over a clockwise darkening, or nothing); the cost and charges
 * small in the corners; the hotkey a quiet label under the slot.
 * `pipRow` keeps the rank pips' line under every slot of a bar, so slots
 * with and without ranks line up; a bar passes its slots' `size` already
 * resolved, so it is never scaled twice.
 */
function slot(id: string, desc: Api.AbilitySlotDesc, pipRow: boolean, size = slotSize(desc.size)): Api.Response {
  const { radius, corner } = shapeOf("slot");
  const r = Math.min(radius, size / 5);
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
  kit.column(id, { gap: u(3), cross: "center" }, () => {
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
      fills: [solid(token("shadow", 0.92))],
      strokes: [stroke(1, token("shadow", 0.9), "outside")],
      // Selected or active: a soft glow in its colour (never a white outline); otherwise a plain drop onto the scene.
      effects: [drop(2, 6, token("shadow", 0.5)), ...(accent ? [glow(8 + 8 * breath, lit(accent, 0.45 + 0.25 * breath), 1)] : [])],
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
      const wash = locked ? 0.72 : disabled ? 0.5 : desc.unaffordable ? 0.5 : 0;
      if (wash > 0) leaf("Wash", { ...cover, part: sel("AbilitySlot", "tint", locked ? "locked" : disabled ? "disabled" : "unaffordable") }, { fills: [solid(token("shadow", wash))] }, full());
      if (desc.unaffordable && !locked && !disabled) leaf("Unaffordable", cover, { fills: [solid(lit(role("mana"), 0.5))], blend: "multiply" }, full());
      if (desc.outOfRange && ready) leaf("OutOfRange", { ...cover, part: sel("AbilitySlot", "tint", "outOfRange") }, { fills: [solid(lit(role("danger"), 0.55))], blend: "multiply" }, full());
    }
    // An ultimate charging: a gold fill rising from the bottom of the art, with a bright line on its top.
    if (ult !== null && !ultReady && !locked) {
      const h = Math.round(artSide * clamp01(ult));
      if (h > 0) {
        box("UltFill", { width: px(artSide), height: px(h), part: sel("AbilitySlot", "ultimate") }, {
          fills: [vgrad([[0, lit(role("gold"), 0.12)], [1, lit(role("gold"), 0.38)]])],
          blend: "screen",
        }, () => {
          leaf("Crest", { width: pct(1), height: px(Math.max(1.5, line)) }, { fills: [solid(lit(mix(role("gold"), { r: 1, g: 1, b: 1, a: 1 }, 0.3), 0.95))], effects: [glow(4, lit(role("gold"), 0.8))] });
        }, absolute({ start: line, bottom: line }));
      }
    }
    if (cooling && !locked) {
      const max = desc.cooldownMax ?? 0;
      const left = max > 0 ? clamp01(cooldown / max) : 1;
      // The art dims under the sweep (the part still to come darkens hardest), so the turning reads at a glance.
      leaf("Dim", cover, { fills: [solid(token("shadow", 0.18))] }, full());
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("AbilitySlot", "sweep") }, {
        shape: pie(left),
        fills: [solid(token("shadow", 0.72))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
      // The sweep's edge: a bright hand from the rim inward, drawn under the numerals so it never strikes through them.
      const angle = -Math.PI / 2 + (1 - left) * Math.PI * 2;
      leaf("Hand", { width: px(size * 1.5), height: px(size * 1.5) }, {
        shape: { kind: "arc", start: angle - 0.018, sweep: 0.036, thickness: size * 0.36, cap: "butt" },
        fills: [solid(token("sheen", 0.55))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
    }
    // A soft vignette and a glint along the top, so the art sits in the slot as a lit, inset picture.
    leaf("Bevel", cover, {
      fills: [vgrad([[0, token("sheen", 0.16)], [0.35, token("sheen", 0)]])],
      effects: [innerGlow(size * 0.14, token("shadow", 0.55))],
    }, full());
    if (flashT > 0) {
      leaf("Flash", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "flash") }, {
        fills: [solid(token("sheen", 0.6 * flashT))],
        effects: [innerGlow(size * 0.3, token("sheen", flashT))],
      }, absolute({ start: 0, top: 0 }));
    }
    if (locked) {
      box("Lock", { width: pct(1), height: pct(1), align: "center", cross: "center", part: sel("AbilitySlot", "lock") }, undefined, () => {
        icon("Mark", "lock", Math.round(size * 0.4), lit({ r: 1, g: 1, b: 1, a: 1 }, 0.82));
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
      text("Time", shortTime(cooldown, true), { step: tsFit(size >= 52 ? 20 : 17, size * 0.4, true), font: "display", color: token("ink"), nowrap: true, outline: 2 },
        absolute({ centerX: true, centerY: true }));
    }
    // The ultimate's percent: small, near the bottom, under the art's subject.
    if (ult !== null && !ultReady && !locked) {
      text("Ultimate", `${Math.floor(clamp01(ult) * 100)}%`, { step: tsFit(11, size * 0.19, true), font: "display", color: token("ink"), nowrap: true, outline: 2 },
        absolute({ centerX: true, bottom: line + Math.round(size * 0.05) }));
    }
    // The cost: a small figure with a droplet at the bottom left, only while the slot could be pressed.
    if (desc.cost && ready) {
      const costStep = tsFit(10, size * 0.19);
      box("Cost", { direction: "row", gap: 1, cross: "center", part: sel("AbilitySlot", "cost", desc.unaffordable ? "unaffordable" : null) }, undefined, () => {
        const glyphName = ["drop", "droplet"].find((n) => kit.hasIcon(n));
        if (glyphName) icon("Drop", glyphName, Math.round(stepSize(costStep).size * 0.95), lit(desc.unaffordable ? role("danger") : mix(role("mana"), { r: 1, g: 1, b: 1, a: 1 }, 0.25)));
        text("Value", desc.cost ?? "", { step: costStep, font: "display", color: desc.unaffordable ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1.6 });
      }, absolute({ start: line + 2, bottom: line + 1 }));
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
        const badge = Math.max(Math.round(size * 0.3), Math.ceil(stepSize(badgeStep).size * 1.35));
        box("ChargeBadge", { width: px(badge), height: px(badge), part: sel("AbilitySlot", "charges") }, {
          shape: { kind: "ellipse" },
          fills: [solid(token("shadow", 0.85))],
          strokes: [stroke(1, token("lineStrong", 0.9))],
        }, () => {
          text("Charges", String(charges), { step: badgeStep, font: "display", color: token("ink"), nowrap: true }, absolute({ centerX: true, centerY: true }));
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
        // A dark rim with a faint light line outside it: the slot reads as an inset frame, not an outlined box.
        strokes: [stroke(line, token("shadow", 0.85)), stroke(1, token("lineStrong", ready ? 0.4 : 0.25), "outside")],
      }, absolute({ start: 0, top: 0 }));
    }
    r0 = close({
      transform: { scaleX: s, scaleY: s },
      a11y: a11y("button", desc.name ?? "", { value: locked ? "Locked" : cooling ? `${Math.ceil(cooldown)}s` : disabled ? "Unavailable" : "Ready" }),
      ...groups({ ready: readyState, use }, { ready: ["ready", "cooling", "disabled", "locked"], use: ["idle", "active", "unaffordable"] }),
    });
    // The ranks: pips right under the slot, centred, on a line every slot of the bar keeps.
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
    } else if (pipRow) {
      leaf("NoPips", { width: px(1), height: px(pipH) });
    }
    // The hotkey: a small, quiet label under the slot, else the action's glyph on the device used last.
    const glyph = !desc.hotkey && desc.action ? kit.frameGlyph(desc.action) : null;
    const keyText = desc.hotkey || glyph?.text || "";
    const keyIcon = glyph && glyph.device !== "keyboardMouse" && kit.hasIcon(glyph.icon) ? glyph.icon : "";
    if (keyText || keyIcon) {
      const keyStep = ts(10);
      const capInk = desc.outOfRange ? lit(role("danger")) : accent ? lit(mix(accent, { r: 1, g: 1, b: 1, a: 1 }, 0.35)) : token("ink");
      box("Hotkey", { height: px(Math.ceil(stepSize(keyStep).line)), cross: "center", align: "center", part: sel("AbilitySlot", "hotkey", desc.outOfRange ? "outOfRange" : null) }, undefined, () => {
        if (keyIcon) icon("Glyph", keyIcon, Math.round(stepSize(keyStep).size * 1.1), capInk);
        else text("Key", keyText, { step: keyStep, font: "display", color: capInk, nowrap: true, outline: 1.6 });
      }, { overlaps: [path] });
    }
  });
  return r0 ?? NOTHING;
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
  // More slots than the safe area holds wrap into even rows (a phone's bar), never off the screen; each row's columns line up with the first's.
  const vp = frame().viewport;
  const room = vp.width - vp.safeArea.left - vp.safeArea.right - tk.gutter * 2 - tk.sm * 2;
  const perRow = Math.max(1, Math.floor((room + gap) / (size + gap)));
  const count = slots.length;
  const wraps = row && count > perRow;
  const cols = wraps ? Math.ceil(count / Math.ceil(count / perRow)) : count;
  const pipRow = slots.some((x) => (x.maxLevel ?? 0) > 0);
  hudPlace(id, anchor, () => {
    // No box behind the slots: they sit on the scene, or on what a style dresses AbilityBar.frame with.
    box(id, {
      direction: desc.direction ?? "row",
      gap,
      wrap: wraps,
      ...(wraps ? { width: px(cols * size + (cols - 1) * gap + tk.xs * 2) } : {}),
      align: wraps ? "start" : "center",
      padding: { left: tk.xs, right: tk.xs, top: tk.xs, bottom: 0 },
      cross: "start",
      part: sel("AbilityBar", "frame"),
      partClass: "hud",
    }, undefined, () => {
      for (const s of slots) {
        const key = s.key ?? "";
        const res = slot(keyId(key), s, pipRow, size);
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

