// What the controls (docs/ui.md §11, U6a) share: one interactive part
// (`control`) that a style dresses by its interaction state, whose motion
// is a §7.3 state group (so a style's springs move it and the probe can
// force each state), with its accessibility and a touch target of at least
// 44 CSS px on a coarse pointer; marks and glyph caps drawn as paths; the
// value formats sliders and steppers share.
//
// Built from public og.ui calls only (open/close with ElementDesc, props,
// setDraw, semantics, animate, memory, visualState, feedback).

import type * as Api from "@origingame/engine";
import {
  all, byText, chevron, clamp, fill, fit, flags, frame, glyphCap, grow, hex, kit, mix, over, paint, parentOf, pct, px, recallPath, rememberPath, role, rtl, spring, still, text, token, vstate, xy,
} from "../containers/base.ts";
import type { Rgba } from "../containers/base.ts";

export { all, byText, chevron, clamp, fill, fit, flags, frame, glyphCap, grow, hex, kit, mix, over, paint, parentOf, pct, px, recallPath, rememberPath, role, rtl, spring, still, text, token, vstate, xy };
export type { Rgba };

/** Response.adjust in the reading direction: it is the screen's (+1 for Right), so a right-to-left screen's Left is forward. */
export function forward(adjust: number): number {
  return rtl() ? -adjust : adjust;
}

/** The smallest touch target on a coarse pointer (docs/ui.md §16.5, XAG). */
export const TOUCH = 44;

/** An element's interaction state, as its `interaction` group says (last tick; a probe-forced state included). */
export type Interaction = "normal" | "hover" | "pressed" | "focused" | "disabled";

const IX = new Set<Interaction>(["normal", "hover", "pressed", "focused", "disabled"]);

/** The interaction state the part remembered under `key` was in last tick. */
export function interactionOf(key: string): Interaction {
  return interactionAt(recallPath(key));
}

/** The interaction state of the element at `path` last tick. */
function interactionAt(p: string): Interaction {
  if (!p) return "normal";
  const s = kit.visualState(p, "interaction") as Interaction;
  return IX.has(s) ? s : "normal";
}

/**
 * Paths by a number a control keeps in UI memory, which is scoped by the container it is built in:
 * so each instance finds its own path (a part id such as `Track` or `Item#0` repeats across sliders
 * and segmented controls, and a key made of the id alone read the previous instance's state).
 */
const pathNumbers = new Map<string, number>();
const numberedPaths: string[] = [];
/** A component's own path last tick, kept under `key` in the memory of the container it is built in (call it before opening its root). */
export function ownPath(key: string): string {
  return numberedPaths[kit.memory(`~p~${key}`, -1)] ?? "";
}

/** Keeps a component's path for next tick's {@link ownPath} (call it after closing its root, in the same container). */
export function keepPath(key: string, path: string): void {
  if (path) kit.remember(`~p~${key}`, pathNumber(path));
}

function pathNumber(p: string): number {
  let n = pathNumbers.get(p);
  if (n === undefined) {
    n = numberedPaths.length;
    numberedPaths.push(p);
    pathNumbers.set(p, n);
  }
  return n;
}

/** A value of an animatable prop. */
const v = (prop: Api.UiProp, value: number): Api.PropValue => ({ prop, value });

/** A colour as the four background values of a visual state. */
export function bgValues(c: Rgba): Api.PropValue[] {
  return [v("backgroundR", c.r), v("backgroundG", c.g), v("backgroundB", c.b), v("backgroundA", c.a)];
}

/**
 * The interaction group a control part moves by: a small lift under the
 * pointer and the focus, a press that sinks it, a disabled part dimmed,
 * and its fill at each (`fills`: rest, hover, pressed). Every move is the
 * style's ui spring unless the style gives the pair; a press lands fast.
 */
export function interactionGroup(d: { lift?: number; press?: number; dim?: number; fills?: [Rgba, Rgba, Rgba] | null } = {}): Api.StateGroup {
  const lift = still() ? 0 : (d.lift ?? -1);
  const press = still() ? 1 : (d.press ?? 0.97);
  const f = d.fills;
  const bg = (k: 0 | 1 | 2): Api.PropValue[] => (f ? bgValues(f[k]) : []);
  return {
    name: "interaction",
    states: [
      { name: "normal", values: [v("y", 0), v("scale", 1), v("opacity", 1), ...bg(0)], onEnter: "", onExit: "" },
      { name: "hover", values: [v("y", lift), v("scale", 1), v("opacity", 1), ...bg(1)], onEnter: "", onExit: "" },
      { name: "pressed", values: [v("y", 0), v("scale", press), v("opacity", 1), ...bg(2)], onEnter: "", onExit: "" },
      { name: "focused", values: [v("y", lift), v("scale", 1), v("opacity", 1), ...bg(1)], onEnter: "", onExit: "" },
      { name: "disabled", values: [v("y", 0), v("scale", 1), v("opacity", d.dim ?? 0.5), ...bg(0)], onEnter: "", onExit: "" },
    ],
    transitions: [
      // A press lands at once and lets go on the spring: a control answers the finger in the tick it is touched.
      { from: "*", to: "pressed", transition: { kind: "tween", durationMs: still() ? 0 : 60, ease: "out" } },
    ],
  };
}

export interface ControlDesc {
  /** The part a style dresses (`Button.base`); the interaction state and `flags` are added as qualifiers. */
  part: string;
  /** The element class an undressed part takes its shape and elevation from. */
  cls?: string;
  /** Other qualifiers: a variant, `selected`, `checked`, `invalid`, `tone=danger`. */
  flags?: Array<string | false | null | undefined>;
  disabled?: boolean;
  /** Keys and pads reach it (true by default). */
  focusable?: boolean;
  /** With focusable: Left and Right are its own while it has the focus (Response.adjust). */
  adjustable?: boolean;
  /** Its fill at rest (moving towards `wash` as it is hovered and pressed), null for none. */
  fill?: Rgba | null;
  /** What hover (half) and press (whole) move the fill towards, `washBy` of the way; null keeps the fill. */
  wash?: Rgba | null;
  washBy?: number;
  /** The fills at rest, hovered and pressed, in place of fill and wash (a button variant's own colours). */
  fills?: [Rgba, Rgba, Rgba];
  direction?: Api.Direction;
  width?: Api.Size | null;
  height?: Api.Size | null;
  padding?: Api.Insets;
  gap?: number;
  align?: Api.Align;
  cross?: Api.Cross;
  radius?: number;
  semantics?: Api.SemanticsDesc;
  /** The interaction group's lift and press (interactionGroup); the disabled part's opacity. */
  lift?: number;
  press?: number;
  dim?: number;
  /** More state groups (selection, validity) and their current states. */
  states?: Api.StateGroup[];
  state?: Api.StateValue[];
  /** Further element props (a transform, z, a drag). */
  props?: Api.ElementProps;
  /** The smallest side a coarse pointer gets: the control grows its reach (hit slop) to it. 44 by default. */
  touch?: number;
  /** Disabled, it still takes the pointer and the focus (inert) so its tooltip or reason can be read. */
  hoverable?: boolean;
  /** A part drawn in its state that takes no input: presses fall through to what holds it (a slider's knob). */
  inert?: boolean;
}

export interface Built {
  /** Its response (clicked, pressed, hovered, focused, adjust; zero while disabled). */
  r: Api.Response;
  /** Its full path. */
  path: string;
  /** Its interaction state last tick. */
  ix: Interaction;
}

/**
 * A control's interactive part: a container keys, pads, the pointer and a
 * finger reach, dressed as `part` with its interaction state
 * (`[hover]`, `[pressed]`, `[focused]`, `[disabled]`, as its interaction
 * group was last tick, so a probe-forced state is dressed too) and the
 * given flags, moving between those states as a §7.3 group: its lift,
 * press and fill on the style's motion. On a coarse pointer its reach
 * grows to 44 CSS px.
 */
export function control(id: string, d: ControlDesc, build: (ix: Interaction) => void): Built {
  const key = `${d.part}~${id}`;
  const disabled = d.disabled ?? false;
  // A disabled control that explains itself (a tooltip, a reason) still takes the pointer and the focus, inert.
  const reach = !d.inert && (!disabled || (d.hoverable ?? false));
  const mine = `~ix~${key}`;
  const own = numberedPaths[kit.memory(mine, -1)] ?? "";
  const ix: Interaction = disabled ? "disabled" : d.inert ? "normal" : interactionAt(own);
  const wash = d.wash === undefined ? role("ink") : d.wash;
  const by = d.washBy ?? 0.1;
  const rest: Rgba = d.fills?.[0] ?? d.fill ?? { ...(wash ?? role("ink")), a: 0 };
  const tint = (k: number): Rgba => (!wash ? rest : rest.a > 0 ? mix(rest, { ...wash, a: Math.max(rest.a, 1) }, k * by) : { ...wash, a: k * by * 1.2 });
  const fills: [Rgba, Rgba, Rgba] = d.fills ?? [rest, tint(0.5), tint(1)];
  const part = `${d.part}${flags(...(d.flags ?? []), ix !== "normal" && ix)}`;
  kit.open(id, {
    direction: d.direction ?? "row",
    width: d.width ?? null,
    // A row that fits its content is at least a finger's height on a coarse pointer.
    height: d.height ?? (frame().touch ? { kind: "fit", min: d.touch ?? TOUCH } : null),
    padding: d.padding ?? all(0),
    gap: d.gap ?? 0,
    align: d.align ?? "center",
    cross: d.cross ?? "center",
    interactive: reach,
    focusable: reach && (d.focusable ?? true),
    adjustable: !disabled && (d.adjustable ?? false),
    // A transparent rest still has a fill for the states to move (hover washes it in).
    background: paint(rest.a > 0 ? rest : { ...rest, a: 0 }),
    radius: d.radius ?? frame().t.control,
    part,
    partClass: d.cls ?? "button",
  });
  let r: Api.Response;
  try {
    build(ix);
  } finally {
    r = kit.close() as Api.Response;
  }
  const path = kit.lastPath();
  rememberPath(key, path);
  kit.remember(mine, pathNumber(path));
  const b = kit.bounds(path);
  const want = frame().touch ? (d.touch ?? TOUCH) : 0;
  const slop = b && want > 0 ? Math.max(0, Math.ceil((want - Math.min(b.width, b.height)) / 2)) : 0;
  const extra = d.props ?? {};
  kit.props({
    ...extra,
    ...(slop > 0 ? { hitSlop: Math.max(slop, extra.hitSlop ?? 0) } : {}),
    states: [
      interactionGroup({ fills, ...(d.lift === undefined ? {} : { lift: d.lift }), ...(d.press === undefined ? {} : { press: d.press }), ...(d.dim === undefined ? {} : { dim: d.dim }) }),
      ...(d.states ?? []),
      ...(extra.states ?? []),
    ],
    state: [...(d.state ?? []), ...(extra.state ?? [])],
  });
  if (d.semantics || disabled) kit.semantics({ ...(d.semantics ?? {}), disabled });
  return { r: disabled ? { ...r, clicked: false, pressed: false, adjust: 0 } : r, path, ix };
}

/** A two-state group (`selection`: off, on) whose values a style's motion moves between. */
export function twoState(name: string, on: boolean, off: Api.PropValue[], onValues: Api.PropValue[]): { group: Api.StateGroup; value: Api.StateValue } {
  return {
    group: { name, states: [{ name: "off", values: off, onEnter: "", onExit: "" }, { name: "on", values: onValues, onEnter: "", onExit: "" }], transitions: [] },
    value: { group: name, state: on ? "on" : "off" },
  };
}

/** The `validity` group (valid, invalid): an invalid field shakes once as it turns invalid. */
export function validity(invalid: boolean): { group: Api.StateGroup; value: Api.StateValue } {
  return {
    group: { name: "validity", states: [{ name: "valid", values: [], onEnter: "", onExit: "" }, { name: "invalid", values: [], onEnter: still() ? "" : "og.controls.shake", onExit: "" }], transitions: [] },
    value: { group: "validity", state: invalid ? "invalid" : "valid" },
  };
}

let defined = false;
/** The timelines the controls play (defined once, as configuration). */
export function defineTimelines(): void {
  if (defined) return;
  defined = true;
  const key = (atMs: number, value: number): Api.TimelineKey => ({ atMs, value, ease: { kind: "ease", ease: "out" } });
  kit.defineTimeline("og.controls.shake", { tracks: [{ target: "", prop: "x", keys: [key(0, 0), key(45, -6), key(95, 5), key(145, -3), key(195, 2), key(260, 0)] }], events: [], labels: [], durationMs: 260 });
  kit.defineTimeline("og.controls.bump", { tracks: [{ target: "", prop: "scale", keys: [key(0, 1), key(90, 1.06), { atMs: 320, value: 1, ease: { kind: "spring", durationMs: 230, bounce: 0.3 } }] }], events: [], labels: [], durationMs: 320 });
  kit.defineTimeline("og.controls.pop", { tracks: [{ target: "", prop: "scale", keys: [key(0, 0.6), { atMs: 220, value: 1, ease: { kind: "spring", durationMs: 220, bounce: 0.45 } }] }], events: [], labels: [], durationMs: 220 });
}

/** The colour on a fill people can read at 4.5:1: ink or inkInverse, whichever reads better. */
export function inkOn(fill: Rgba): Rgba {
  const ink = role("ink");
  const inv = role("inkInverse");
  return contrast(ink, fill) >= contrast(inv, fill) ? ink : inv;
}

function lum(c: Rgba): number {
  const f = (x: number) => (x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4);
  return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b);
}

/** WCAG contrast of two opaque colours. */
export function contrast(a: Rgba, b: Rgba): number {
  const x = lum(a);
  const y = lum(b);
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

/** An icon of the style's set when it has it, else a mark drawn as a path (never a missing glyph). */
export function glyphIcon(id: string, name: string, size: number, color: Rgba): void {
  if (kit.hasIcon(name)) {
    kit.icon(id, name, { size, tint: paint(color), label: "" });
    return;
  }
  // A name neither the icon set nor the drawn marks have: og-ui's empty slot and its warning,
  // never a stand-in disc that reads as a picture.
  if (hasMark(name)) drawn(id, name, size, color);
  else kit.icon(id, name, { size, tint: paint(color), label: "" });
}

/** The marks the controls draw themselves: the shapes no icon set is needed for. */
export type Drawn = "tick" | "cross" | "plus" | "minus" | "dot" | "search" | "chevronDown" | "chevronUp" | "chevronStart" | "chevronEnd" | "eye" | "reset" | "dash" | "backspace" | "shift" | "enter" | "globe" | "lock" | "star" | "warning" | "info";

const MARKS = new Set(["tick", "cross", "plus", "minus", "dot", "search", "chevronDown", "chevronUp", "chevronStart", "chevronEnd", "eye", "reset", "dash", "backspace", "shift", "enter", "globe", "lock", "star", "warning", "info"]);
/** Whether the controls draw a mark of this name. */
export function hasMark(name: string): boolean {
  return MARKS.has(name);
}

/** A mark drawn as a path in `size` square; start and end follow the writing direction. Unknown names draw a dot. */
export function drawn(id: string, name: string, size: number, color: Rgba, weight = Math.max(1.5, size / 11)): void {
  const s = size;
  const p = (x: number, y: number) => ({ x: (rtl() ? 1 - x : x) * s, y: y * s });
  const q = (x: number, y: number) => ({ x: x * s, y: y * s });
  const fill: Api.Fill[] = [{ kind: "solid", color: paint(color) }];
  const line = (pts: Api.Vector2[], closed = false): Api.DrawShape => ({ kind: "path", points: pts, width: weight, cap: "round", join: "round", closed } as Api.DrawShape);
  kit.open(id, { width: px(s), height: px(s) });
  switch (name) {
    case "tick": kit.setDraw({ shape: line([q(0.2, 0.53), q(0.42, 0.74), q(0.8, 0.28)]), fills: fill }); break;
    case "cross": kit.setDraw({ shape: line([q(0.27, 0.27), q(0.73, 0.73), q(0.5, 0.5), q(0.73, 0.27), q(0.27, 0.73)]), fills: fill }); break;
    case "plus": kit.setDraw({ shape: line([q(0.5, 0.22), q(0.5, 0.78), q(0.5, 0.5), q(0.22, 0.5), q(0.78, 0.5)]), fills: fill }); break;
    case "minus": case "dash": kit.setDraw({ shape: line([q(0.24, 0.5), q(0.76, 0.5)]), fills: fill }); break;
    case "chevronDown": kit.setDraw({ shape: line([q(0.26, 0.38), q(0.5, 0.62), q(0.74, 0.38)]), fills: fill }); break;
    case "chevronUp": kit.setDraw({ shape: line([q(0.26, 0.62), q(0.5, 0.38), q(0.74, 0.62)]), fills: fill }); break;
    case "chevronStart": kit.setDraw({ shape: line([p(0.62, 0.26), p(0.38, 0.5), p(0.62, 0.74)]), fills: fill }); break;
    case "chevronEnd": kit.setDraw({ shape: line([p(0.38, 0.26), p(0.62, 0.5), p(0.38, 0.74)]), fills: fill }); break;
    case "search": {
      // A lens (an arc) and its handle.
      kit.open("Lens", { width: px(s * 0.62), height: px(s * 0.62) });
      kit.setDraw({ shape: { kind: "arc", thickness: weight }, fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: s * 0.1, top: s * 0.1 } });
      kit.open("Handle", { width: px(s), height: px(s) });
      kit.setDraw({ shape: line([q(0.64, 0.64), q(0.86, 0.86)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    }
    case "eye": {
      kit.setDraw({ shape: line([q(0.1, 0.5), q(0.3, 0.3), q(0.5, 0.24), q(0.7, 0.3), q(0.9, 0.5), q(0.7, 0.7), q(0.5, 0.76), q(0.3, 0.7)], true), fills: fill });
      kit.open("Pupil", { width: px(s * 0.26), height: px(s * 0.26) });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: s * 0.37, top: s * 0.37 } });
      break;
    }
    case "reset": kit.setDraw({ shape: line([q(0.3, 0.2), q(0.3, 0.42), q(0.52, 0.42), q(0.3, 0.42), q(0.26, 0.5), q(0.3, 0.66), q(0.44, 0.78), q(0.62, 0.8), q(0.76, 0.7), q(0.82, 0.52), q(0.76, 0.34), q(0.6, 0.22), q(0.44, 0.22), q(0.32, 0.3)]), fills: fill }); break;
    case "backspace": kit.setDraw({ shape: line([p(0.08, 0.5), p(0.3, 0.24), p(0.92, 0.24), p(0.92, 0.76), p(0.3, 0.76)], true), fills: fill });
      kit.open("X", { width: px(s), height: px(s) });
      kit.setDraw({ shape: line([p(0.48, 0.38), p(0.72, 0.62), p(0.6, 0.5), p(0.72, 0.38), p(0.48, 0.62)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "shift": kit.setDraw({ shape: line([q(0.5, 0.14), q(0.86, 0.52), q(0.66, 0.52), q(0.66, 0.84), q(0.34, 0.84), q(0.34, 0.52), q(0.14, 0.52)], true), fills: fill }); break;
    case "enter": kit.setDraw({ shape: line([p(0.82, 0.22), p(0.82, 0.6), p(0.22, 0.6), p(0.4, 0.42), p(0.22, 0.6), p(0.4, 0.78)]), fills: fill }); break;
    case "globe": {
      kit.setDraw({ shape: { kind: "arc", thickness: weight }, fills: fill });
      kit.open("Lines", { width: px(s), height: px(s) });
      kit.setDraw({ shape: line([q(0.08, 0.5), q(0.92, 0.5), q(0.5, 0.5), q(0.5, 0.08), q(0.34, 0.3), q(0.34, 0.7), q(0.5, 0.92), q(0.66, 0.7), q(0.66, 0.3), q(0.5, 0.08)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    }
    case "lock": kit.setDraw({ shape: line([q(0.22, 0.48), q(0.78, 0.48), q(0.78, 0.86), q(0.22, 0.86), q(0.22, 0.48), q(0.32, 0.48), q(0.32, 0.32), q(0.4, 0.18), q(0.5, 0.14), q(0.6, 0.18), q(0.68, 0.32), q(0.68, 0.48)]), fills: fill }); break;
    case "star": kit.setDraw({ shape: { kind: "polygon", sides: 5, inner: 0.45, rounding: 0.1 } as Api.DrawShape, fills: fill }); break;
    case "warning": kit.setDraw({ shape: line([q(0.5, 0.12), q(0.9, 0.84), q(0.1, 0.84)], true), fills: fill });
      kit.open("Bang", { width: px(s), height: px(s) });
      kit.setDraw({ shape: line([q(0.5, 0.4), q(0.5, 0.58), q(0.5, 0.58), q(0.5, 0.72), q(0.5, 0.7201)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    case "info": kit.setDraw({ shape: { kind: "arc", thickness: weight }, fills: fill });
      kit.open("I", { width: px(s), height: px(s) });
      kit.setDraw({ shape: line([q(0.5, 0.3), q(0.5, 0.3001), q(0.5, 0.46), q(0.5, 0.72)]), fills: fill });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: 0 } });
      break;
    default: kit.setDraw({ shape: { kind: "ellipse" }, fills: fill });
  }
  kit.close();
}

/** A number as a control shows it: its decimals by the step (0.25 → 2), grouped, a true minus, a unit after. */
export function formatValue(value: number, d: { step?: number; decimals?: number; unit?: string; percent?: boolean } = {}): string {
  if (!Number.isFinite(value)) return "—";
  if (d.percent) return `${Math.round(value * 100)}%`;
  const dec = d.decimals ?? decimalsOf(d.step ?? 0);
  const abs = Math.abs(value);
  const fixed = abs.toFixed(dec);
  const [int, frac] = fixed.split(".");
  const grouped = (int ?? "0").replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const neg = value < 0 && Number(fixed) !== 0;
  return `${neg ? "\u2212" : ""}${grouped}${frac ? `.${frac}` : ""}${d.unit ? d.unit : ""}`;
}

/** How many decimals a step needs: 1 → 0, 0.5 → 1, 0.05 → 2; continuous (0) shows 2. */
export function decimalsOf(step: number): number {
  if (!(step > 0)) return 2;
  for (let k = 0; k <= 6; k++) if (Math.abs(Math.round(step * 10 ** k) - step * 10 ** k) < 1e-6) return k;
  return 6;
}

/** `value` snapped to `step` from `min` and kept in range. */
export function snap(value: number, min: number, max: number, step: number): number {
  const lo = Math.min(min, max);
  const hi = Math.max(min, max);
  let v = clamp(value, lo, hi);
  if (step > 0) v = clamp(lo + Math.round((v - lo) / step) * step, lo, hi);
  // A step of 0.1 lands on 0.30000000000000004 otherwise.
  return Number(v.toFixed(Math.min(10, decimalsOf(step) + 2)));
}

/** The gesture on an element by path, or null before it was built. */
export function gestureAt(path: string): Api.GestureInfo | null {
  return path ? (kit.gesture(path) as Api.GestureInfo) : null;
}

/**
 * Where the pointer is across an element's box last tick, 0 to 1 from its
 * start (the writing direction's), or null when it is not over it and no
 * press it began is held.
 */
export function pointerAcross(path: string, held: boolean): { x: number; y: number } | null {
  const b = path ? kit.bounds(path) : null;
  const p = kit.framePointer() as Api.UiPointer;
  if (!b) return null;
  const inside = p.x >= b.x && p.x <= b.x + b.width && p.y >= b.y && p.y <= b.y + b.height;
  if (!inside && !held) return null;
  const x = clamp((p.x - b.x) / Math.max(1, b.width), 0, 1);
  const y = clamp((p.y - b.y) / Math.max(1, b.height), 0, 1);
  return { x: rtl() ? 1 - x : x, y };
}

/** The label above a control and its helper or error line below: a field's frame, shared. */
export function fieldChrome(id: string, d: { label?: string; helper?: string; error?: string; required?: boolean; width?: Api.Size | null; counter?: string }, build: () => void): void {
  const { t } = frame();
  kit.open(id, { direction: "column", width: d.width ?? grow(), gap: t.xs, cross: "stretch" });
  try {
    if (d.label) {
      kit.open("Head", { direction: "row", width: grow(), gap: t.xs, cross: "baseline" });
      text("Label", d.label, { step: "sm", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
      if (d.required) text("Required", "*", { step: "sm", color: "danger" });
      kit.close();
    }
    build();
    if (d.error || d.helper || d.counter) {
      kit.open("Foot", { direction: "row", width: grow(), gap: t.sm, cross: "start" });
      if (d.error) {
        drawn("ErrorIcon", "warning", 14, role("danger"));
        text("Error", d.error, { step: "caption", color: "danger", width: grow() });
        kit.semantics({ role: "alert", name: d.error });
      } else if (d.helper) text("Helper", d.helper, { step: "caption", color: "inkMuted", width: grow() });
      else kit.spacer("Fill", grow());
      if (d.counter) text("Counter", d.counter, { step: "caption", color: "inkMuted", nowrap: true, font: "mono" });
      kit.close();
    }
  } finally {
    kit.close();
  }
}

/** The style's hairline stroke width. */
export function hairline(): number {
  const st = (frame().theme.tokens as { stroke?: Record<string, number> } | null)?.stroke;
  return st?.hairline ?? 1;
}
