// Ranges (docs/ui.md §11, Controls): the slider, the range slider and the
// number stepper. A slider's track takes a press anywhere and a drag (the
// pointer and a finger), Left and Right while it has the focus (keys, a
// pad), with ticks, marks, a readout and a bubble; a stepper repeats while
// held, faster the longer.

import type * as Api from "@origingame/engine";
import {
  byText, clamp, control, decimalsOf, drawn, flags, formatValue, forward, frame, grow, kit, mix, over, paint, parentOf, keepPath, ownPath, pct, px, role, rtl, snap, still, text, TOUCH, xy,
} from "./base.ts";

/** A number as a slider or stepper shows it: a level's word, a percent, or the number with its unit. */
export function showValue(value: number, min: number, step: number, f: Api.ValueFormat | undefined): string {
  const labels = f?.labels ?? [];
  if (labels.length > 0) {
    const k = Math.round(value - min);
    if (k >= 0 && k < labels.length) return labels[k] ?? "";
  }
  const dec = f?.decimals ?? -1;
  return formatValue(value, { ...(dec >= 0 ? { decimals: dec } : { step }), ...(f?.unit ? { unit: f.unit } : {}), ...(f?.percent ? { percent: true } : {}) });
}

/** A fraction across a track box for the frame's pointer, the knob's half kept at each end; the writing direction's. */
function fractionAt(b: Api.UiRect, knob: number): number {
  const p = kit.framePointer() as Api.UiPointer;
  const x = clamp((p.x - b.x - knob / 2) / Math.max(1, b.width - knob), 0, 1);
  return rtl() ? 1 - x : x;
}

/** The track's geometry: its height (a finger's reach on a coarse pointer), the knob and the rail. */
function geometry(): { h: number; knob: number; rail: number } {
  const touch = frame().touch;
  const knob = byText(touch ? 26 : 20);
  return { h: Math.max(knob + 8, touch ? TOUCH : 28), knob, rail: touch ? 6 : 4 };
}

interface Seg { from: number; to: number }

/** The rail, its fill between two fractions, and its ticks: inside a track of the given geometry. */
function rail(g: { h: number; knob: number; rail: number }, fill: Seg, ticks: number[], part: string, disabled: boolean): void {
  kit.open("Rail", { direction: "row", width: grow(), height: px(g.rail), radius: g.rail / 2, background: paint(mix(role("well"), role("ink"), 0.1)), part: `${part}.rail`, partClass: "bar" });
  kit.open("Fill", { width: pct(clamp(fill.to - fill.from, 0, 1)), height: grow(), radius: g.rail / 2, background: paint(disabled ? role("inkFaint") : role("primary")), part: `${part}.fill`, partClass: "bar" });
  kit.close();
  // From its start fraction (a bipolar slider's origin), along the rail.
  over({ insets: { start: Math.round(clamp(fill.from, 0, 1) * railWidth), top: 0, bottom: 0 }, inert: true });
  ticks.forEach((tk, i) => {
    const passed = tk >= fill.from - 1e-6 && tk <= fill.to + 1e-6;
    kit.open(kit.idi("Tick", i), { width: px(2), height: px(g.rail + 6), radius: 1, background: paint(passed ? role("onPrimary") : role("lineStrong")), part: `${part}.tick${flags(passed && "passed")}`, partClass: "bar" });
    kit.close();
    over({ insets: { start: Math.round(tk * railWidth) - 1, top: -3 }, inert: true, z: 1 });
  });
  kit.close();
  over({ insets: { start: g.knob / 2, end: g.knob / 2, top: (g.h - g.rail) / 2 }, inert: true });
}

/** The rail's width last tick (the track's less a knob), set by each slider before it builds. */
let railWidth = 0;

/** A knob: a round part at a fraction of the rail, dressed by its own state; it swells under a finger. */
function knobAt(id: string, f: number, g: { h: number; knob: number }, part: string, state: { pressed: boolean; focused: boolean; disabled: boolean; extra?: string }, a11y: Api.SemanticsDesc | undefined, adjustable: boolean, focusable: boolean): ReturnType<typeof control> {
  const big = state.pressed && frame().touch && !still();
  // It lies over the rail, in its track: an overlap on purpose.
  const track = parentOf(kit.lastPath());
  // A knob a finger takes (a range's two) reaches 44 px around the disc it draws.
  const reach = focusable && frame().touch ? Math.max(g.knob, TOUCH) : g.knob;
  const ring = reach > g.knob;
  const fill = state.disabled ? role("control") : role("knob");
  const core = (on: boolean): void => {
    // A dot of primary inside the knob while it is held or focused: where the value is, at a glance.
    if (!on) return;
    const c = Math.round(g.knob * 0.42);
    kit.open("Core", { width: px(c), height: px(c), radius: c / 2, background: paint(role("primary")), part: `${part}.core`, partClass: "badge" });
    kit.close();
  };
  return control(id, {
    part: ring ? `${part}.reach` : `${part}.knob`,
    cls: "chip",
    flags: ["round", state.extra, state.pressed && "pressed"],
    disabled: state.disabled,
    focusable,
    adjustable,
    width: px(reach),
    height: px(reach),
    radius: reach / 2,
    fill: ring ? { ...fill, a: 0 } : fill,
    wash: null,
    lift: 0,
    press: 1,
    touch: 0,
    // A slider's one knob is drawn, not pressed: a press on it is the track's (which takes the focus and the drag).
    inert: !focusable && !adjustable,
    props: {
      position: "absolute",
      insets: { start: Math.round(f * railWidth - (reach - g.knob) / 2), top: (g.h - reach) / 2 },
      overlaps: track ? [track] : [],
      transform: { scaleX: big ? 1.15 : 1, scaleY: big ? 1.15 : 1 },
      motion: { kind: "token", token: "ui" },
      z: 2,
    },
    ...(a11y ? { semantics: a11y } : {}),
  }, (ix) => {
    const on = state.pressed || state.focused || ix === "focused";
    if (!ring) {
      core(on);
      return;
    }
    // The disc, dressed as the knob in its state, centred in the reach.
    kit.open("Disc", { width: px(g.knob), height: px(g.knob), radius: g.knob / 2, align: "center", cross: "center", background: paint(fill), part: `${part}.knob${flags("round", state.extra, state.pressed && "pressed", ix !== "normal" && ix)}`, partClass: "chip" });
    core(on);
    kit.close();
  });
}

/** The value bubble over a knob. */
function bubble(id: string, f: number, label: string, g: { h: number; knob: number }): void {
  const { t } = frame();
  kit.open(id, { direction: "row", height: px(byText(26)), padding: xy(t.sm, 0), align: "center", cross: "center", radius: t.control, background: paint(role("ink")), part: "Slider.bubble", partClass: "tooltip" });
  text("Text", label, { step: "sm", color: role("inkInverse"), nowrap: true, font: "mono" });
  kit.close();
  const w = Math.max(32, [...label].length * 8 + t.sm * 2);
  over({
    insets: { start: Math.round(f * railWidth + g.knob / 2 - w / 2), top: -byText(30) },
    inert: true,
    z: 5,
    enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: 4 }], transition: { kind: "token", token: "ui" } },
    exit: { values: [{ prop: "opacity", value: 0 }], transition: { kind: "tween", durationMs: 90, ease: "exit" } },
    reduced: "fade",
  });
}

/** Ticks at every step of a stepped range (at most 20 steps), as fractions. */
function tickFractions(min: number, max: number, step: number, on: boolean): number[] {
  if (!on || !(step > 0) || max <= min) return [];
  const n = Math.round((max - min) / step);
  if (n < 1 || n > 20) return [];
  return Array.from({ length: n + 1 }, (_, i) => i / n);
}

export function valueSlider(id: string, value: number, desc: Api.ValueSliderDesc = {}): Api.ValueSliderResult {
  const { t } = frame();
  const min = desc.min ?? 0;
  const max = desc.max ?? 1;
  const step = desc.step ?? 0;
  const span = max - min || 1;
  const disabled = desc.disabled ?? false;
  const g = geometry();
  const unit = step > 0 ? step : span / 20;
  const key = `Slider~${id}`;
  // Last tick's track, for the pointer and the rail's width.
  const trackPath = ownPath(key);
  const tb = trackPath ? kit.bounds(trackPath) : null;
  railWidth = tb ? Math.max(1, tb.width - g.knob) : 0;
  const ptr = kit.framePointer() as Api.UiPointer;
  const held = kit.memory(`${id}~held`, 0) > 0 && ptr.down && !disabled;
  let v = clamp(Number.isFinite(value) ? value : min, Math.min(min, max), Math.max(min, max));
  if (held && tb) v = snap(min + fractionAt(tb, g.knob) * span, min, max, step);
  const shown = showValue(v, min, step, desc.format);
  const flash = kit.memory(`${id}~flash`, 0);
  let committed = false;
  let response: Api.Response | null = null;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: t.xs, cross: "stretch" });
  try {
    if (desc.label || (desc.showValue ?? true)) {
      kit.open("Head", { direction: "row", width: grow(), gap: t.sm, cross: "baseline" });
      // A long label wraps (at 200 % text, in a narrow column) rather than losing its end.
      if (desc.label) text("Label", desc.label, { step: "sm", color: disabled ? "inkFaint" : "inkMuted", width: grow() });
      else kit.spacer("Lead", grow());
      if (desc.showValue ?? true) text("Value", shown, { step: "sm", color: disabled ? "inkFaint" : "ink", nowrap: true, font: "mono" });
      kit.close();
    }
    const f = (v - min) / span;
    const origin = desc.origin === undefined || desc.origin === null ? min : clamp(desc.origin, Math.min(min, max), Math.max(min, max));
    const of = (origin - min) / span;
    const built = control("Track", {
      part: "Slider.track",
      cls: "bar",
      disabled,
      adjustable: true,
      width: grow(),
      height: px(g.h),
      fill: null,
      wash: null,
      lift: 0,
      press: 1,
      semantics: { role: "slider", name: desc.label || id, value: shown, valueMin: min, valueMax: max, valueNow: v },
    }, (ix) => {
      rail(g, { from: Math.min(f, of), to: Math.max(f, of) }, tickFractions(min, max, step, desc.ticks ?? false), "Slider", disabled);
      const pressed = held || ix === "pressed";
      knobAt("Knob", f, g, "Slider", { pressed, focused: ix === "focused", disabled }, undefined, false, false);
      if ((desc.bubble ?? true) && !disabled && (pressed || flash > kit.seconds())) bubble("Bubble", f, shown, g);
    });
    response = built.r;
    if (built.r.pressed && tb && !held) v = snap(min + fractionAt(tb, g.knob) * span, min, max, step);
    if (built.r.adjust && !disabled) {
      v = snap(v + unit * forward(built.r.adjust), min, max, step > 0 ? step : 0);
      committed = true;
      kit.remember(`${id}~flash`, kit.seconds() + 0.9);
    } else kit.remember(`${id}~flash`, flash);
    const wasHeld = kit.memory(`${id}~held`, 0) > 0;
    if (wasHeld && !built.r.pressed && !ptr.down) committed = true;
    kit.remember(`${id}~held`, built.r.pressed || held ? 1 : 0);
    if (desc.marks && desc.marks.length > 0) marks("Marks", desc.marks, min, span, g);
  } finally {
    kit.close();
  }
  // The track's path, for next tick's pointer.
  keepPath(key, `${kit.lastPath()}/Track`);
  const changed = v !== value;
  if (changed) {
    const before = Math.round((value - min) / unit);
    const after = Math.round((v - min) / unit);
    if (before !== after) kit.feedback("sliderTick", clamp((v - min) / span, 0, 1));
  }
  return { response: response ?? ({} as Api.Response), value: v, changed, committed };
}


/** Labelled points under a track. */
function marks(id: string, list: readonly Api.SliderMark[], min: number, span: number, g: { knob: number }): void {
  kit.open(id, { direction: "row", width: grow(), height: px(byText(18)) });
  list.forEach((m, i) => {
    const f = clamp((m.value - min) / span, 0, 1);
    const w = Math.max(24, [...(m.label ?? "")].length * 8);
    kit.open(kit.idi("Mark", i), { direction: "row", width: px(w), align: "center" });
    if (m.label) text("Text", m.label, { step: "caption", color: "inkMuted", nowrap: true, align: "center" });
    kit.close();
    // At its point, centred under it; the end ones kept inside.
    const at = clamp(f * railWidth + g.knob / 2 - w / 2, 0, Math.max(0, railWidth + g.knob - w));
    over({ insets: { start: Math.round(at), top: 0 } });
  });
  kit.close();
}

export function rangeSlider(id: string, low: number, high: number, desc: Api.RangeSliderDesc = {}): Api.RangeResult {
  const { t } = frame();
  const min = desc.min ?? 0;
  const max = desc.max ?? 1;
  const step = desc.step ?? 0;
  const span = max - min || 1;
  const gap = Math.max(0, desc.minGap ?? 0);
  const disabled = desc.disabled ?? false;
  const g = geometry();
  const unit = step > 0 ? step : span / 20;
  const key = `RangeSlider~${id}`;
  const trackPath = ownPath(key);
  const tb = trackPath ? kit.bounds(trackPath) : null;
  railWidth = tb ? Math.max(1, tb.width - g.knob) : 0;
  const ptr = kit.framePointer() as Api.UiPointer;
  // Which knob a held press moves: 1 low, 2 high (UI memory).
  const active = kit.memory(`${id}~knob`, 0);
  let lo = clamp(Math.min(low, high), min, max);
  let hi = clamp(Math.max(low, high), min, max);
  const place = (k: number, at: number): void => {
    if (k === 1) lo = clamp(snap(at, min, max, step), min, hi - gap);
    else hi = clamp(snap(at, min, max, step), lo + gap, max);
  };
  if (active && ptr.down && tb && !disabled) place(active, min + fractionAt(tb, g.knob) * span);
  const fmt = (x: number) => showValue(x, min, step, desc.format);
  let committed = false;
  let press = 0;
  let adjusted = false;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: t.xs, cross: "stretch" });
  try {
    if (desc.label || (desc.showValue ?? true)) {
      kit.open("Head", { direction: "row", width: grow(), gap: t.sm, cross: "baseline" });
      // A long label wraps (at 200 % text, in a narrow column) rather than losing its end.
      if (desc.label) text("Label", desc.label, { step: "sm", color: disabled ? "inkFaint" : "inkMuted", width: grow() });
      else kit.spacer("Lead", grow());
      if (desc.showValue ?? true) text("Value", `${fmt(lo)} – ${fmt(hi)}`, { step: "sm", color: disabled ? "inkFaint" : "ink", nowrap: true, font: "mono" });
      kit.close();
    }
    const fl = (lo - min) / span;
    const fh = (hi - min) / span;
    const built = control("Track", {
      part: "RangeSlider.track",
      cls: "bar",
      disabled,
      focusable: false,
      width: grow(),
      height: px(g.h),
      fill: null,
      wash: null,
      lift: 0,
      press: 1,
    }, () => {
      rail(g, { from: fl, to: fh }, [], "RangeSlider", disabled);
      const k1 = knobAt("Low", fl, g, "RangeSlider", { pressed: active === 1 && ptr.down, focused: false, disabled, extra: "low" }, { role: "slider", name: desc.lowLabel || "Minimum", value: fmt(lo), valueMin: min, valueMax: max, valueNow: lo }, true, true);
      const k2 = knobAt("High", fh, g, "RangeSlider", { pressed: active === 2 && ptr.down, focused: false, disabled, extra: "high" }, { role: "slider", name: desc.highLabel || "Maximum", value: fmt(hi), valueMin: min, valueMax: max, valueNow: hi }, true, true);
      if (k1.r.pressed) press = 1;
      if (k2.r.pressed) press = 2;
      if (k1.r.adjust) { place(1, lo + unit * forward(k1.r.adjust)); adjusted = true; }
      if (k2.r.adjust) { place(2, hi + unit * forward(k2.r.adjust)); adjusted = true; }
    });
    if (built.r.pressed && tb && press === 0) {
      // A press on the track takes the nearer knob there.
      const at = min + fractionAt(tb, g.knob) * span;
      press = Math.abs(at - lo) <= Math.abs(at - hi) ? 1 : 2;
      place(press, at);
    }
  } finally {
    kit.close();
  }
  keepPath(key, `${kit.lastPath()}/Track`);
  const down = press !== 0 || (active !== 0 && ptr.down);
  if (active && !down) committed = true;
  if (adjusted) committed = true;
  kit.remember(`${id}~knob`, press || (down ? active : 0));
  const changed = lo !== Math.min(low, high) || hi !== Math.max(low, high);
  if (changed) kit.feedback("sliderTick", -1);
  return { low: lo, high: hi, changed, committed };
}

export function numberStepper(id: string, value: number, desc: Api.NumberStepperDesc = {}): Api.NumberResult {
  const { t } = frame();
  const min = desc.min ?? null;
  const max = desc.max ?? null;
  const step = desc.step && desc.step > 0 ? desc.step : 1;
  const large = desc.largeStep && desc.largeStep > 0 ? desc.largeStep : step * 10;
  const disabled = desc.disabled ?? false;
  const wrap = (desc.wrap ?? false) && min !== null && max !== null;
  const h = Math.max(t.controlMd, frame().touch ? TOUCH : 0);
  const now = kit.seconds();
  // A held button repeats: after 0.4 s, every 0.12 s, then 0.05 s; past a second by the large step.
  const since = kit.memory(`${id}~since`, -1);
  const lastRep = kit.memory(`${id}~rep`, -1);
  const fit = (x: number): number => {
    let y = Number(x.toFixed(Math.min(10, decimalsOf(step) + 2)));
    if (wrap && min !== null && max !== null) {
      if (y > max) y = min;
      else if (y < min) y = max;
    }
    if (min !== null) y = Math.max(min, y);
    if (max !== null) y = Math.min(max, y);
    return y;
  };
  let v = Number.isFinite(value) ? value : (min ?? 0);
  const shown = showValue(v, min ?? 0, step, desc.format);
  let held = 0;
  let pressedNow = 0;
  // The label and the buttons on one line while they fit; the buttons on a line of their own when they do not
  // (larger text, a narrow column), never the label cut to nothing.
  kit.open(id, { direction: "row", width: desc.width ?? null, gap: t.md, cross: "center", wrap: true });
  try {
    if (desc.label) {
      text("Label", desc.label, { step: "body", color: disabled ? "inkFaint" : "ink" });
      kit.spacer("Gap", grow());
    }
    kit.open("Frame", { direction: "row", height: px(h), cross: "stretch", radius: t.control, background: paint(role("well")), part: `Stepper.frame${flags(disabled && "disabled")}`, partClass: "field" });
    const btn = (bid: string, sign: -1 | 1): void => {
      const atBound = !wrap && (sign < 0 ? min !== null && v <= min : max !== null && v >= max);
      const b = control(bid, {
        part: `Stepper.button${flags(sign < 0 ? "dec" : "inc")}`,
        cls: "button",
        disabled: disabled || atBound,
        // A button under a finger is a stop keys and pads reach too; a fine pointer's named image is not.
        focusable: frame().touch,
        width: px(h),
        height: grow(),
        fill: null,
        washBy: 0.1,
        lift: 0,
        press: 0.92,
        radius: t.control,
        // The pointer's shortcut for what keys and pads do on the focused control (Left and Right; a
        // field's own editing): a button under a finger, a named image to the pad graph.
        semantics: { role: frame().touch ? "button" : "image", name: sign < 0 ? "Decrease" : "Increase" },
      }, () => drawn("Glyph", sign < 0 ? "minus" : "plus", byText(18), disabled || atBound ? role("inkFaint") : role("ink")));
      if (b.r.pressed) {
        held = sign;
        if (since < 0) pressedNow = sign;
      }
    };
    btn("Dec", -1);
    const val = control("Value", {
      part: "Stepper.value",
      cls: "field",
      disabled,
      adjustable: true,
      width: { kind: "fit", min: byText(56) },
      height: grow(),
      padding: xy(t.sm, 0),
      fill: null,
      wash: null,
      lift: 0,
      press: 1,
      semantics: { role: "spinbutton", name: desc.label || id, value: shown, ...(min !== null ? { valueMin: min } : {}), ...(max !== null ? { valueMax: max } : {}), valueNow: v },
    }, () => text("Text", shown, { step: "body", color: disabled ? "inkFaint" : "ink", nowrap: true, font: "mono", align: "center" }));
    btn("Inc", 1);
    kit.close();
    if (val.r.adjust && !disabled) v = fit(v + step * forward(val.r.adjust));
  } finally {
    kit.close();
  }
  if (!disabled) {
    if (pressedNow) {
      v = fit(v + step * pressedNow);
      kit.remember(`${id}~since`, now);
      kit.remember(`${id}~rep`, now);
    } else if (held && since >= 0) {
      const age = now - since;
      const every = age > 1.6 ? 0.05 : 0.12;
      if (age > 0.4 && now - lastRep >= every) {
        v = fit(v + (age > 1 ? large : step) * held);
        kit.remember(`${id}~rep`, now);
      } else kit.remember(`${id}~rep`, lastRep);
      kit.remember(`${id}~since`, since);
    } else {
      kit.remember(`${id}~since`, -1);
      kit.remember(`${id}~rep`, -1);
    }
  }
  if (v !== value) kit.feedback("sliderTick", -1);
  return { value: v, changed: v !== value };
}
