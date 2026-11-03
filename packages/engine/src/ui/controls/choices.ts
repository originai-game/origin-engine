// Choices (docs/ui.md §11, Controls): the switch, the checkbox, the radio
// group, the segmented control and the option cycler of a console settings
// screen. Each moves between its states as a §7.3 group (selection on and
// off, the knob's slide, the tick's pop, the thumb's glide) on the style's
// motion, and is a row the whole of which takes the press.

import type * as Api from "@origingame/engine";
import {
  all, bgValues, byText, chevron, control, drawn, fit, flags, forward, frame, glyphIcon, grow, kit, mix, over, paint, parentOf, keepPath, ownPath, px, role, rtl, spring, still, text, TOUCH, vstate, xy,
} from "./base.ts";
import type { Interaction } from "./base.ts";
import { edge, ring } from "../containers/panels.ts";

const v = (prop: Api.UiProp, value: number): Api.PropValue => ({ prop, value });

/** A label and its description, a column that grows: the text beside a switch, a check, a radio. */
function labelled(label: string, description: string, disabled: boolean): void {
  const { t } = frame();
  if (!label && !description) return;
  kit.open("Text", { direction: "column", width: grow(), gap: t.xxs });
  if (label) text("Label", label, { step: "body", color: disabled ? "inkFaint" : "ink", width: grow() });
  if (description) text("Description", description, { step: "sm", color: disabled ? "inkFaint" : "inkMuted", width: grow() });
  kit.close();
}

/** The ix flag for an inner part dressed by its row's state. */
const ixFlag = (ix: Interaction): string | false => ix !== "normal" && ix;

// ── switch ──────────────────────────────────────────────────────────────

export function toggleSwitch(id: string, on: boolean, desc: Api.ToggleSwitchDesc = {}): Api.SwitchResult {
  const { t } = frame();
  const disabled = desc.disabled ?? false;
  const small = desc.size === "small";
  const large = desc.size === "large";
  const th = byText(small ? 22 : large ? 32 : 28);
  const tw = Math.round(th * 1.72);
  const inset = Math.max(2, Math.round(th * 0.11));
  const knob = th - inset * 2;
  const travel = (tw - knob - inset * 2) * (rtl() ? -1 : 1);
  const side = desc.side ?? "start";
  const offFill = mix(role("controlStrong"), role("ink"), 0.12);
  const onFill = role("primary");
  const built = control(id, {
    part: "Toggle.row",
    cls: "button",
    disabled,
    flags: [on ? "on" : "off"],
    direction: "row",
    width: desc.width ?? null,
    padding: xy(t.xs, t.xxs),
    gap: t.md,
    cross: "center",
    fill: null,
    wash: role("ink"),
    washBy: 0.06,
    lift: 0,
    press: 1,
    semantics: { role: "switch", name: desc.label || id, checked: on, ...(desc.onLabel || desc.offLabel ? { value: on ? desc.onLabel ?? "" : desc.offLabel ?? "" } : {}) },
  }, (ix) => {
    if (side === "start") labelled(desc.label ?? "", desc.description ?? "", disabled);
    kit.open("Track", {
      direction: "row",
      width: px(tw),
      height: px(th),
      cross: "center",
      radius: th / 2,
      background: paint(on ? onFill : offFill),
      part: `Toggle.track${flags(on ? "on" : "off", "round", ixFlag(ix))}`,
      partClass: "chip",
    });
    kit.open("Knob", { width: px(knob), height: px(knob), radius: knob / 2, align: "center", cross: "center", background: paint(role("knob")), part: `Toggle.knob${flags(on && "on", "round")}`, partClass: "chip" });
    // A tick in the knob while on: the state is never colour alone (XAG 103).
    kit.open("Mark", { width: px(knob), height: px(knob), align: "center", cross: "center" });
    drawn("Tick", "tick", Math.round(knob * 0.62), onFill, Math.max(1.5, knob / 9));
    kit.close();
    kit.props({ states: [{ name: "selection", states: [vstate("off", [v("opacity", 0), v("scale", 0.5)]), vstate("on", [v("opacity", 1), v("scale", 1)])], transitions: [] }], state: [{ group: "selection", state: on ? "on" : "off" }] });
    kit.close();
    // The knob slides along the track (start to end as it turns on), and stretches a little while pressed.
    over({
      insets: { start: inset, top: inset },
      states: [{ name: "selection", states: [vstate("off", [v("x", 0)]), vstate("on", [v("x", travel)])], transitions: [] }],
      state: [{ group: "selection", state: on ? "on" : "off" }],
      transform: { scaleX: ix === "pressed" && !still() ? 1.12 : 1 },
      motion: { kind: "token", token: "ui" },
    });
    kit.close();
    // The track's fill moves with it.
    kit.props({ states: [{ name: "selection", states: [vstate("off", bgValues(offFill)), vstate("on", bgValues(onFill))], transitions: [] }], state: [{ group: "selection", state: on ? "on" : "off" }] });
    if (side === "end") labelled(desc.label ?? "", desc.description ?? "", disabled);
  });
  let next = on;
  if (built.r.clicked && !disabled) {
    next = !on;
    kit.feedback(next ? "toggleOn" : "toggleOff", -1);
  }
  return { response: built.r, on: next, changed: next !== on };
}

// ── checkbox ────────────────────────────────────────────────────────────

export function checkField(id: string, checked: boolean, desc: Api.CheckFieldDesc = {}): Api.SwitchResult {
  const { t } = frame();
  const disabled = desc.disabled ?? false;
  const mixed = desc.mixed ?? false;
  const invalid = desc.invalid ?? false;
  const on = checked || mixed;
  const box = byText(22);
  const side = desc.side ?? "end";
  const offFill = role("well");
  const onFill = role("primary");
  const built = control(id, {
    part: "Checkbox.row",
    cls: "button",
    disabled,
    flags: [checked && "checked", mixed && "mixed", invalid && "invalid"],
    direction: "row",
    width: desc.width ?? null,
    padding: xy(t.xs, t.xxs),
    gap: t.md,
    cross: desc.description ? "start" : "center",
    fill: null,
    washBy: 0.06,
    lift: 0,
    press: 1,
    semantics: { role: "checkbox", name: desc.label || id, checked, ...(mixed ? { value: "mixed" } : {}) },
  }, (ix) => {
    if (side === "start") labelled(desc.label ?? "", desc.description ?? "", disabled);
    kit.open("Box", {
      direction: "row",
      width: px(box),
      height: px(box),
      align: "center",
      cross: "center",
      radius: Math.max(3, Math.round(box * 0.24)),
      background: paint(on ? onFill : offFill),
      part: `Checkbox.box${flags(checked && "checked", mixed && "mixed", invalid && "invalid", ixFlag(ix))}`,
      partClass: "badge",
    });
    // Its edge: strong while empty (so the box shows on any panel), danger when invalid, none once filled.
    if (!on || invalid) kit.setDraw({ strokes: [{ width: invalid ? 2 : 1.5, align: "inside", paint: { kind: "solid", color: { token: invalid ? "danger" : "lineStrong" } } }] });
    kit.open("Mark", { width: px(box), height: px(box), align: "center", cross: "center", part: "Checkbox.mark", partClass: "badge" });
    drawn("Glyph", mixed ? "minus" : "tick", Math.round(box * 0.78), role("onPrimary"), Math.max(2, box / 9));
    kit.close();
    // The mark pops in as it turns on (selection group), and leaves at once.
    kit.props({
      states: [{
        name: "selection",
        states: [vstate("off", [v("opacity", 0), v("scale", 0.4)]), vstate("on", [v("opacity", 1), v("scale", 1)])],
        transitions: [{ from: "off", to: "on", transition: still() ? { kind: "tween", durationMs: 0 } : { kind: "spring", durationMs: 260, bounce: 0.45 } }, { from: "on", to: "off", transition: { kind: "tween", durationMs: still() ? 0 : 90, ease: "exit" } }],
      }],
      state: [{ group: "selection", state: on ? "on" : "off" }],
    });
    kit.close();
    kit.props({ states: [{ name: "selection", states: [vstate("off", bgValues(offFill)), vstate("on", bgValues(onFill))], transitions: [] }], state: [{ group: "selection", state: on ? "on" : "off" }] });
    if (side === "end") labelled(desc.label ?? "", desc.description ?? "", disabled);
  });
  let next = checked;
  if (built.r.clicked && !disabled) {
    // A mixed box turns on, as a group's parent does.
    next = mixed ? true : !checked;
    kit.feedback(next ? "toggleOn" : "toggleOff", -1);
  }
  return { response: built.r, on: next, changed: next !== checked || (mixed && built.r.clicked && !disabled) };
}

// ── radio group ─────────────────────────────────────────────────────────

/** The next enabled index from `from` by `dir` (wrapping), or `from` when none is. */
export function stepEnabled(count: number, from: number, dir: number, disabled: (i: number) => boolean, wrap = true): number {
  if (count <= 0) return -1;
  let i = from;
  for (let n = 0; n < count; n++) {
    i += dir;
    if (i < 0 || i >= count) {
      if (!wrap) return from;
      i = (i + count) % count;
    }
    if (!disabled(i)) return i;
  }
  return from;
}

function radioRing(selected: boolean, ix: Interaction, size: number): void {
  const dot = Math.round(size * 0.46);
  kit.open("Ring", { width: px(size), height: px(size), align: "center", cross: "center", radius: size / 2, background: paint(selected ? role("primary") : role("well")), part: `Radio.ring${flags(selected && "selected", "round", ixFlag(ix))}`, partClass: "badge" });
  if (!selected) kit.setDraw({ strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: { token: "lineStrong" } } }] });
  kit.open("Dot", { width: px(dot), height: px(dot), radius: dot / 2, background: paint(role("onPrimary")), part: "Radio.dot[round]", partClass: "badge" });
  kit.close();
  kit.props({
    states: [{ name: "selection", states: [vstate("off", [v("scale", 0), v("opacity", 0)]), vstate("on", [v("scale", 1), v("opacity", 1)])], transitions: [{ from: "off", to: "on", transition: still() ? { kind: "tween", durationMs: 0 } : { kind: "spring", durationMs: 240, bounce: 0.4 } }] }],
    state: [{ group: "selection", state: selected ? "on" : "off" }],
  });
  kit.close();
  const p = role("primary");
  const w = role("well");
  kit.props({ states: [{ name: "selection", states: [vstate("off", bgValues(w)), vstate("on", bgValues(p))], transitions: [] }], state: [{ group: "selection", state: selected ? "on" : "off" }] });
}

export function radioGroup(id: string, desc: Api.RadioGroupDesc = {}): Api.ChoiceIndexResult {
  const { t } = frame();
  const options = desc.options ?? [];
  const all0 = desc.disabled ?? false;
  const cards = desc.variant === "cards";
  let selected = desc.selected ?? -1;
  let focused = -1;
  let moved = 0;
  let moveFrom = -1;
  const size = byText(20);
  kit.open(id, { direction: "column", width: desc.width ?? null, gap: t.sm, cross: "stretch" });
  try {
    if (desc.label) text("Legend", desc.label, { step: "sm", color: "inkMuted", nowrap: true });
    kit.open("Options", { direction: desc.row ? "row" : "column", width: desc.row ? null : grow(), gap: cards ? t.sm : t.xxs, wrap: desc.row ?? false, cross: "stretch" });
    options.forEach((o, i) => {
      const on = i === selected;
      const off = all0 || (o.disabled ?? false);
      const built = control(kit.idi("Option", i), {
        part: cards ? "Radio.card" : "Radio.option",
        cls: cards ? "card" : "button",
        disabled: off,
        flags: [on && "selected"],
        direction: "row",
        width: desc.row ? null : grow(),
        padding: cards ? all(t.md) : xy(t.xs, t.xxs),
        gap: t.md,
        cross: o.description ? "start" : "center",
        fill: cards ? (on ? mix(role("panelRaised"), role("primary"), 0.1) : role("panelRaised")) : null,
        washBy: cards ? 0.05 : 0.06,
        lift: cards ? -2 : 0,
        press: cards ? 0.985 : 1,
        adjustable: true,
        radius: cards ? t.panel : t.control,
        semantics: { role: "radio", name: o.label ?? "", checked: on },
      }, (ix) => {
        if (cards && o.icon) glyphIcon("Icon", o.icon, byText(24), on ? role("primary") : role("ink"));
        if (!cards) radioRing(on, ix, size);
        labelled(o.label ?? "", o.description ?? "", off);
        if (cards) radioRing(on, ix, size);
        // The chosen card is ringed in primary.
        if (cards && on) ring("Chosen", 2, "primary", "Radio.chosen");
      });
      if (built.r.focused) focused = i;
      if (built.r.clicked && !off && i !== selected) {
        selected = i;
        kit.feedback("select", -1);
      }
      if (built.r.adjust) {
        moved = forward(built.r.adjust);
        moveFrom = i;
      }
    });
    kit.close();
  } finally {
    kit.close();
  }
  kit.semantics({ role: "radiogroup", ...(desc.label ? { name: desc.label } : {}) });
  const before = desc.selected ?? -1;
  if (moved && moveFrom >= 0) {
    // Arrows move the choice and the focus with it, skipping disabled ones (a native radio group).
    const next = stepEnabled(options.length, moveFrom, moved, (i) => all0 || (options[i]?.disabled ?? false));
    if (next !== moveFrom) {
      selected = next;
      kit.feedback("select", -1);
      kit.setFocus(`${kit.lastPath()}/Options/${kit.idi("Option", next)}`);
    }
  }
  return { selected, changed: selected !== before, focused };
}

// ── segmented ───────────────────────────────────────────────────────────

export function segmented(id: string, desc: Api.SegmentedDesc = {}): Api.ChoiceIndexResult {
  const { t } = frame();
  const items = desc.items ?? [];
  const before = desc.selected ?? 0;
  let selected = before;
  let focused = -1;
  const small = desc.size === "small";
  // A segment is at least 44 tall under a finger: the track's padding is around it.
  const inner = byText(Math.max((small ? t.controlSm : t.controlMd) - 4, frame().touch ? TOUCH : 0));
  const pad = Math.max(2, Math.round(inner * 0.08));
  const h = inner + 2 * pad;
  const offAll = desc.disabled ?? false;
  const equal = desc.equal ?? true;
  let adjust = 0;
  // Each segment's own width (its parts', remembered from last tick): segments share the row
  // but never shrink below it; when they do not fit, the track scrolls sideways under an edge fade.
  const nat = items.map((_, i) => kit.memory(`${id}~nat${i}`, 0));
  const widest = Math.max(0, ...nat);
  const natSum = equal ? widest * items.length : nat.reduce((a, b) => a + b, 0);
  const measured: number[] = [];
  // The track as laid out last tick: where the thumb goes.
  const self = ownPath(`Segmented~${id}`);
  kit.open(id, { direction: "row", width: desc.width ?? grow(), height: px(h) });
  let trackPath = "";
  try {
    kit.openScroll("Strip", { width: grow(), height: px(h), axes: "horizontal", padding: all(0), gap: 0 });
    try {
      kit.open("Track", {
        direction: "row",
        width: grow(natSum + 2 * pad),
        height: px(h),
        padding: all(pad),
        gap: 0,
        cross: "stretch",
        radius: t.control + pad,
        background: paint(role("well")),
        part: `Segmented.track${flags(offAll && "disabled")}`,
        partClass: "tab",
      });
      try {
        // The thumb under the chosen segment: laid where that segment was last tick, gliding to a new one (layout animation).
        const sel = self ? kit.bounds(`${self}/${kit.idi("Item", selected)}`) : null;
        const me = self ? kit.bounds(self) : null;
        if (sel && me) {
          kit.open("Thumb", { width: px(sel.width), height: px(sel.height), radius: t.control, background: paint(role("panelRaised")), part: "Segmented.thumb", partClass: "tab" });
          kit.close();
          kit.props({
            position: "absolute",
            // Insets are inside the track's padding, from its start (the writing direction's).
            insets: { start: rtl() ? me.x + me.width - pad - (sel.x + sel.width) : sel.x - me.x - pad, top: sel.y - me.y - pad },
            overlaps: [self],
            inert: true,
            layoutMotion: still() ? null : { kind: "token", token: "ui" },
            z: 0,
          });
        }
        items.forEach((it, i) => {
          const on = i === selected;
          const off = offAll || (it.disabled ?? false);
          const padX = byText(small ? t.sm : t.md);
          const built = control(kit.idi("Item", i), {
            part: "Segmented.item",
            cls: "tab",
            disabled: off,
            flags: [on && "selected"],
            direction: "row",
            width: equal ? grow(Math.max(widest, frame().touch ? TOUCH : 0)) : fit(frame().touch ? TOUCH : 0),
            height: grow(),
            padding: xy(padX, 0),
            gap: t.xs,
            align: "center",
            fill: null,
            washBy: on ? 0 : 0.05,
            lift: 0,
            press: 0.97,
            adjustable: true,
            props: { z: 1 },
            semantics: { role: "radio", name: it.label || it.ariaLabel || "", checked: on },
          }, () => {
            const ink = on ? role("ink") : role("inkMuted");
            if (it.icon) glyphIcon("Icon", it.icon, byText(small ? 16 : 18), ink);
            if (it.label) text("Label", it.label, { step: small ? "sm" : "body", color: ink, nowrap: true, align: "center" });
          });
          // Its own width: its parts' as laid out, never what a share squeezed it to.
          const ip = kit.lastPath();
          const parts = ["Icon", "Label"].map((c) => kit.bounds(`${ip}/${c}`)?.width ?? -1).filter((w) => w >= 0);
          if (parts.length > 0) measured[i] = Math.ceil(parts.reduce((a, b) => a + b, 0) + (parts.length - 1) * t.xs + 2 * padX);
          if (built.r.focused) focused = i;
          if (built.r.clicked && !off) selected = i;
          if (built.r.adjust) adjust = forward(built.r.adjust);
        });
      } finally {
        kit.close();
        trackPath = kit.lastPath();
      }
    } finally {
      kit.close();
    }
    // The fades: where the track runs past the strip's edge (last tick's).
    const sb = kit.bounds(parentOf(trackPath));
    const tb = kit.bounds(trackPath);
    const leftCut = sb !== null && tb !== null && tb.x < sb.x - 0.5;
    const rightCut = sb !== null && tb !== null && tb.x + tb.width > sb.x + sb.width + 0.5;
    edge("FadeStart", true, kit.animate(`${id}~fs`, (rtl() ? rightCut : leftCut) ? 1 : 0, spring("ui")), true, 24, "panel");
    edge("FadeEnd", false, kit.animate(`${id}~fe`, (rtl() ? leftCut : rightCut) ? 1 : 0, spring("ui")), true, 24, "panel");
  } finally {
    kit.close();
  }
  keepPath(`Segmented~${id}`, trackPath);
  measured.forEach((w, i) => kit.remember(`${id}~nat${i}`, w));
  kit.semantics({ role: "radiogroup", ...(desc.label ? { name: desc.label } : {}) });
  if (adjust) {
    const next = stepEnabled(items.length, selected, adjust, (i) => offAll || (items[i]?.disabled ?? false), false);
    if (next !== selected) {
      selected = next;
      kit.setFocus(`${trackPath}/${kit.idi("Item", next)}`);
    }
  }
  if (selected !== before) kit.feedback("tab", -1);
  return { selected, changed: selected !== before, focused };
}

// ── cycler ──────────────────────────────────────────────────────────────

export function cycler(id: string, options: readonly string[], index: number, desc: Api.CyclerDesc = {}): Api.ChoiceIndexResult {
  const { t } = frame();
  const n = options.length;
  const wrap = desc.wrap ?? true;
  const disabled = desc.disabled ?? false;
  const cur = n > 0 ? Math.max(0, Math.min(n - 1, index)) : -1;
  let next = cur;
  let focused = -1;
  const h = byText(Math.max(t.controlMd, frame().touch ? TOUCH : 0));
  const step = (d: number): void => {
    if (n <= 0) return;
    const k = next + d;
    next = wrap ? (k + n) % n : Math.max(0, Math.min(n - 1, k));
  };
  // Which way the value last moved: it slides in from that side.
  const dir = kit.memory(`${id}~dir`, 1);
  let moved = 0;
  const built = control(id, {
    part: "Cycler.frame",
    cls: "field",
    disabled,
    direction: "row",
    width: desc.width ?? grow(),
    height: px(h),
    padding: xy(t.sm, 0),
    gap: t.sm,
    fill: role("well"),
    washBy: 0.05,
    lift: 0,
    press: 1,
    adjustable: true,
    semantics: { role: "spinbutton", name: desc.label || id, value: cur >= 0 ? options[cur] ?? "" : "", valueMin: 0, valueMax: Math.max(0, n - 1), valueNow: cur },
  }, () => {
    if (desc.label) text("Label", desc.label, { step: "body", color: disabled ? "inkFaint" : "ink", nowrap: true, ellipsis: true, width: grow() });
    else kit.spacer("Lead", grow());
    const arrow = (aid: string, way: "back" | "forward", can: boolean): boolean => {
      const a = control(aid, {
        part: `Cycler.arrow${flags(way === "back" ? "prev" : "next")}`,
        cls: "button",
        disabled: disabled || !can,
        focusable: false,
        // A finger's arrow is the row's whole height (44 on a coarse pointer).
        width: px(frame().touch ? h : Math.round(h * 0.8)),
        height: px(frame().touch ? h : Math.round(h * 0.8)),
        fill: null,
        washBy: 0.1,
        lift: 0,
        press: 0.9,
        radius: h / 2,
        // The pointer's shortcut for what keys and pads do on the focused control (Left and Right; a
        // field's own editing): a button under a finger, a named image to the pad graph.
        semantics: { role: frame().touch ? "button" : "image", name: way === "back" ? "Previous" : "Next" },
      }, () => chevron("Chevron", way, Math.round(h * 0.44), can && !disabled ? role("ink") : role("inkFaint"), 0, 2.2));
      return a.r.clicked;
    };
    const canBack = wrap ? n > 1 : cur > 0;
    const canFwd = wrap ? n > 1 : cur < n - 1;
    if (arrow("Prev", "back", canBack)) moved = -1;
    kit.open("Middle", { direction: "column", width: desc.valueWidth ? px(desc.valueWidth) : fitLongest(options), align: "center", cross: "center", gap: 3 });
    kit.open("Clip", { direction: "row", width: grow(), align: "center" });
    // Keyed by the option, so a new one enters (and the old one is gone).
    text(kit.idi("Value", Math.max(0, cur)), cur >= 0 ? options[cur] ?? "" : "—", { step: "body", color: disabled ? "inkFaint" : "ink", nowrap: true, ellipsis: true, align: "center", width: grow() });
    // The value slides in from the side it came from (a short move on the ui spring), still under reduced motion.
    kit.props({ enter: { values: [{ prop: "x", value: still() ? 0 : 14 * dir * (rtl() ? -1 : 1) }, { prop: "opacity", value: 0 }], transition: { kind: "token", token: "ui" } }, reduced: "fade" });
    kit.close();
    if ((desc.pips ?? true) && n > 1 && n <= 12) {
      kit.open("Pips", { direction: "row", gap: 4, align: "center" });
      for (let i = 0; i < n; i++) {
        const lit = i === cur;
        kit.open(kit.idi("Pip", i), { width: px(lit ? 14 : 6), height: px(4), radius: 2, background: paint(lit ? role("primary") : role("lineStrong")), part: `Cycler.pip${flags(lit && "current")}`, partClass: "bar" });
        kit.close();
        kit.props({ layoutMotion: still() ? null : { kind: "token", token: "ui" } });
      }
      kit.close();
    }
    kit.close();
    if (arrow("Next", "forward", canFwd)) moved = 1;
  });
  if (built.r.adjust) moved = forward(built.r.adjust);
  if (built.r.clicked && !moved) moved = 1;
  if (built.r.focused) focused = cur;
  if (moved && !disabled) step(moved);
  if (next !== cur) {
    kit.feedback("select", -1);
    kit.remember(`${id}~dir`, moved);
  } else kit.remember(`${id}~dir`, dir);
  return { selected: next, changed: next !== cur, focused };
}

/** A width that holds the longest option at the body step (so the arrows never move). */
function fitLongest(options: readonly string[]): Api.Size {
  const size = frame().theme.typeRamp.body?.size ?? 16;
  // An em for a CJK character, a little over half one for others.
  const ems = (o: string): number => [...o].reduce((w, ch) => w + ((ch.codePointAt(0) ?? 0) >= 0x2e80 ? 1 : 0.6), 0);
  const longest = options.reduce((m, o) => Math.max(m, ems(o)), 1);
  return px(Math.ceil(longest * size) + 8);
}

