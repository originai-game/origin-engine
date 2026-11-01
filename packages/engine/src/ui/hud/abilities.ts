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
import { stepSize, a11y, absolute, anchorOf, box, bp, chip, clamp01, close, drop, edge, frame, glow, groups, hudPlace, icon, innerGlow, keyId, kit, leaf, lit, mix, msToTicks, open, pct, pop, pulse, px, reduced, role, sel, shapeOf, shortTime, solid, sound, stroke, text, token, ts, u, vgrad, arc, type Rgba, apply, textScale, tsFit } from "./core.ts";

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

export function abilitySlot(id: string, desc: Api.AbilitySlotDesc = {}): Api.Response {
  const size = u(desc.size || (bp().phone ? 48 : 56));
  const { radius, corner } = shapeOf("slot");
  const r = Math.min(radius, size / 4);
  const maxCharges = desc.maxCharges ?? 0;
  const charged = maxCharges > 1;
  const charges = Math.min(desc.charges ?? 0, maxCharges);
  const cooldown = Math.max(0, desc.cooldown ?? 0);
  // A charged ability is usable while it has a charge; its cooldown is the next charge's.
  const cooling = charged ? charges === 0 && cooldown > 0 : cooldown > 0;
  const locked = !!desc.locked;
  const ready = !cooling && !locked;
  const ult = desc.ultimate ?? null;
  const ultReady = ult !== null && ult >= 1;
  const readyState = locked ? "locked" : cooling ? "cooling" : "ready";
  const use = desc.active ? "active" : desc.unaffordable ? "unaffordable" : "idle";
  let r0: Api.Response | null = null;
  kit.column(id, { gap: u(3), cross: "center" }, () => {
    const e = edge("ready", ready && (ult === null || ultReady) ? 1 : 0);
    const cameReady = e.changed && e.prev === 0 && ready;
    sound("focus", cameReady && kit.memory("seen", 0) > 0);
    kit.remember("seen", 1);
    const flashT = ready && e.since < msToTicks(380) && e.prev === 0 ? 1 - e.since / msToTicks(380) : 0;
    const s = ready && e.prev === 0 ? pop(e.since, 0.12, 340) : 1;
    const ringInk = desc.active ? role("primary") : ultReady ? role("gold") : ready ? role("lineStrong") : role("line");
    const breath = desc.active ? pulse(1.1) : ultReady ? pulse(0.9) : 0;
    const path = open("Slot", {
      width: px(size),
      height: px(size),
      interactive: !locked,
      part: sel("AbilitySlot", "frame", readyState, desc.active ? "active" : null),
      partClass: "slot",
    }, {
      shape: { kind: "rect", radii: [r, r, r, r], corner },
      fills: [solid(token("shadow", 0.85)), vgrad([[0, token("panelRaised", 0.9)], [1, token("well", 0.95)]])],
      strokes: [stroke(Math.max(2, size / 28), lit(ringInk, desc.active || ultReady ? 0.75 + 0.25 * breath : 1)), stroke(1, token("shadow", 0.9), "outside")],
      effects: [drop(2, 6, token("shadow", 0.55)), ...(desc.active || ultReady ? [glow(6 + 6 * breath, lit(ringInk, 0.55))] : [])],
      clip: "shape",
    });
    // The art, inset by the frame's ring.
    const inset = Math.max(2, Math.round(size / 18));
    if (desc.image) {
      kit.image("Icon", { texture: desc.image, width: px(size - inset * 2), height: px(size - inset * 2), label: "" });
      apply(absolute({ start: inset, top: inset }, { opacity: locked ? 0.35 : 1 }));
    } else {
      box("IconBox", { width: pct(1), height: pct(1), align: "center", cross: "center", part: sel("AbilitySlot", "icon") }, undefined, () => {
        icon("Icon", desc.icon ?? "", Math.round(size * 0.58), locked ? token("inkFaint") : token("ink"));
      }, absolute({ start: 0, top: 0 }, { opacity: locked ? 0.45 : 1 }));
    }
    if (desc.unaffordable && !locked) {
      leaf("Unaffordable", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "tint", "unaffordable") }, {
        fills: [solid(lit(role("mana"), 0.42))],
        blend: "multiply",
      }, absolute({ start: 0, top: 0 }));
    }
    if (cooling && !locked) {
      const max = desc.cooldownMax ?? 0;
      const left = max > 0 ? clamp01(cooldown / max) : 1;
      leaf("Sweep", { width: px(size * 1.5), height: px(size * 1.5), part: sel("AbilitySlot", "sweep") }, {
        shape: pie(left),
        fills: [solid(token("shadow", 0.68))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
      // The sweep's edge: a bright hand at the boundary, so the turning reads at a glance.
      const angle = -Math.PI / 2 + (1 - left) * Math.PI * 2;
      leaf("Hand", { width: px(size * 1.5), height: px(size * 1.5) }, {
        shape: { kind: "arc", start: angle - 0.02, sweep: 0.04, thickness: size * 0.75, cap: "butt" },
        fills: [solid(token("sheen", 0.45))],
      }, absolute({ start: -size * 0.25, top: -size * 0.25 }));
      text("Time", shortTime(cooldown, true), { step: tsFit(size >= 52 ? 18 : 15, size * 0.42, true), font: "display", color: token("ink"), nowrap: true, outline: 1.2 }, absolute({ centerX: true, centerY: true }));
    }
    if (flashT > 0) {
      leaf("Flash", { width: pct(1), height: pct(1), part: sel("AbilitySlot", "flash") }, {
        fills: [solid(token("sheen", 0.75 * flashT))],
        effects: [innerGlow(size * 0.3, token("sheen", flashT))],
      }, absolute({ start: 0, top: 0 }));
    }
    if (ult !== null && !ultReady) {
      text("Ultimate", `${Math.floor(clamp01(ult) * 100)}%`, { step: tsFit(16, size * 0.36, true), font: "display", color: lit(role("gold")), nowrap: true, outline: 1.2 }, absolute({ centerX: true, centerY: true }));
    }
    if (locked) {
      box("Lock", { width: pct(1), height: pct(1), align: "center", cross: "center", part: sel("AbilitySlot", "lock") }, { fills: [solid(token("shadow", 0.5))] }, () => {
        icon("Mark", "lock", Math.round(size * 0.36), token("inkMuted"));
      }, absolute({ start: 0, top: 0 }));
    }
    if (charged) {
      if (charges < maxCharges) {
        const p = clamp01(desc.chargeProgress ?? 0);
        box("Recharge", { width: pct(1), height: px(Math.max(3, Math.round(size / 14))), part: sel("AbilitySlot", "recharge") }, { fills: [solid(token("shadow", 0.7))] }, () => {
          leaf("Fill", { width: pct(p), height: pct(1) }, { fills: [solid(lit(role("gold")))] });
        }, absolute({ start: 0, bottom: 0 }));
      }
      text("Charges", String(charges), { step: tsFit(14, size * 0.36, true), font: "display", color: charges === 0 ? lit(role("danger")) : token("ink"), nowrap: true, outline: 1.2 },
        absolute({ end: inset + 1, bottom: inset }));
    }
    if (desc.cost) {
      text("Cost", desc.cost, { step: ts(11), font: "mono", color: desc.unaffordable ? lit(role("danger")) : lit(role("mana")), nowrap: true, outline: 1 },
        absolute(charged ? { start: inset + 1, bottom: inset } : { centerX: true, bottom: inset }));
    }
    r0 = close({
      transform: { scaleX: s, scaleY: s },
      a11y: a11y("button", desc.name ?? "", { value: locked ? "Locked" : cooling ? `${Math.ceil(cooldown)}s` : "Ready" }),
      ...groups({ ready: readyState, use }, { ready: ["ready", "cooling", "locked"], use: ["idle", "active", "unaffordable"] }),
    });
    // The ultimate's ring, round the outside.
    if (ult !== null) {
      leaf("UltRing", { width: px(size + u(8)), height: px(size + u(8)), part: sel("AbilitySlot", "ultimate", ultReady ? "ready" : null) }, {
        shape: arc(clamp01(ult), u(3)),
        fills: [solid(lit(role("gold"), ultReady ? 0.7 + 0.3 * pulse(0.9) : 0.95))],
      }, absolute({ start: -u(4), top: -u(4) }, { overlaps: [path] }));
    }
    // The hotkey's cap, over the slot's top left corner: the label, else the action's glyph on the device used last.
    const glyph = !desc.hotkey && desc.action ? kit.frameGlyph(desc.action) : null;
    const keyText = desc.hotkey || glyph?.text || "";
    const keyIcon = glyph && glyph.device !== "keyboardMouse" && kit.hasIcon(glyph.icon) ? glyph.icon : "";
    if (keyText || keyIcon) {
      const capInk = desc.outOfRange ? role("danger") : role("ink");
      box("Hotkey", { padding: { left: u(4), right: u(4), top: 0, bottom: 0 }, height: px(Math.max(14, Math.round(size * 0.3)) * textScale()), cross: "center", align: "center", part: sel("AbilitySlot", "hotkey", desc.outOfRange ? "outOfRange" : null), partClass: "chip" }, {
        shape: { kind: "rect", radii: [3, 3, 3, 3] },
        fills: [solid(token("hud", 0.95))],
        strokes: [stroke(1, desc.outOfRange ? lit(role("danger")) : token("lineStrong"))],
        effects: [drop(1, 2, token("shadow", 0.6))],
      }, () => {
        if (keyIcon) icon("Glyph", keyIcon, Math.max(12, Math.round(size * 0.26)), lit(capInk));
        else text("Key", keyText, { step: ts(11), font: "mono", color: lit(capInk), nowrap: true });
      }, absolute({ start: -u(3), top: -u(4) }, { z: 2, overlaps: [path] }));
    }
    const maxLevel = desc.maxLevel ?? 0;
    if (maxLevel > 0) {
      kit.row("Pips", { gap: u(2) }, () => {
        for (let i = 0; i < maxLevel; i++) {
          const filled = i < (desc.level ?? 0);
          leaf(kit.idi("Pip", i), { width: px(Math.max(4, Math.round(size / 9))), height: px(Math.max(3, Math.round(size / 16))), part: sel("AbilitySlot", "pip", filled ? "filled" : null) }, {
            shape: { kind: "rect", radii: [1, 1, 1, 1] },
            fills: [solid(filled ? lit(role("gold")) : token("shadow", 0.7))],
            strokes: [stroke(1, filled ? lit(mix(role("gold"), { r: 1, g: 1, b: 1, a: 1 }, 0.4)) : token("line"))],
          });
        }
      });
    }
  });
  return r0 ?? ({ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 } as Api.Response);
}

export function abilityBar(id: string, desc: Api.AbilityBarDesc = {}): Api.AbilityBarResult {
  const f = frame();
  const tk = f.t;
  const anchor = anchorOf(desc, "bottom");
  let clicked: string | null = null;
  const pressed: string[] = [];
  const size = u(desc.size || (bp().phone ? 48 : 56));
  const gap = desc.gap || tk.sm;
  const row = (desc.direction ?? "row") === "row";
  // More slots than the safe area holds wrap into rows (a phone's bar), never off the screen.
  const vp = frame().viewport;
  const room = vp.width - vp.safeArea.left - vp.safeArea.right - tk.gutter * 2 - tk.sm * 2;
  const perRow = Math.max(1, Math.floor((room + gap) / (size + gap)));
  const count = (desc.slots ?? []).length;
  const wraps = row && count > perRow;
  const cols = wraps ? Math.ceil(count / Math.ceil(count / perRow)) : count;
  hudPlace(id, anchor, () => {
    box(id, {
      direction: desc.direction ?? "row",
      gap,
      wrap: wraps,
      ...(wraps ? { width: px(cols * size + (cols - 1) * gap + tk.sm * 2) } : {}),
      align: "center",
      padding: { left: tk.sm, right: tk.sm, top: u(10), bottom: tk.sm },
      cross: "end",
      part: sel("AbilityBar", "frame"),
      partClass: "hud",
    }, chip(), () => {
      for (const slot of desc.slots ?? []) {
        const key = slot.key ?? "";
        const res = abilitySlot(keyId(key), { ...slot, size });
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
  const timeWidth = Math.ceil(timeText.length * stepSize(timeStep).size * 0.62) + 4;
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
      box("IconBox", { width: pct(1), height: pct(1), align: "center", cross: "center", part: sel("StatusGrid", "icon") }, undefined, () => {
        icon("Icon", fx.icon ?? fx.name ?? "", Math.round(size * 0.62), lit(mix(ink, { r: 1, g: 1, b: 1, a: 1 }, 0.35)));
      }, absolute({ start: 0, top: 0 }));
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

