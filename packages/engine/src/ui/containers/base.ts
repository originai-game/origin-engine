// What the containers and data components (docs/ui.md §2, U6b) share:
// the frame's tokens, colours mixed from the style's palette, interactive
// surfaces a style dresses by part and state, chevrons and glyph caps
// drawn as shapes (so they mirror under a right-to-left locale), and the
// placement of floating panels (popovers, menus, sheets) beside what they
// belong to, flipped and kept on screen.
//
// Every component is built from public og.ui calls only: open/close with
// ElementDesc (interactive, focusable, part), props, setDraw, semantics,
// animate, memory, feedback, the modal layer, and the widgets.

import type * as Api from "@origingame/engine";
import { kit } from "../../ui-kit/bind.ts";
import { all, fit, frame, grow, pct, px, token, xy } from "../../ui-kit/util.ts";

export { all, fit, frame, grow, kit, pct, px, token, xy };

/** A colour, sRGB 0–1 with alpha. */
export interface Rgba { r: number; g: number; b: number; a: number }

const cache = new Map<string, Rgba>();
/** `#rrggbb` or `#rrggbbaa` as a colour. */
export function hex(s: string): Rgba {
  const hit = cache.get(s);
  if (hit) return hit;
  const h = s.replace("#", "");
  const n = (k: number) => parseInt(h.slice(k, k + 2), 16) / 255;
  const c = { r: n(0), g: n(2), b: n(4), a: h.length >= 8 ? n(6) : 1 };
  cache.set(s, c);
  return c;
}

/** The style's colour for a role of the frame being built. */
export function role(name: Api.ThemeColor): Rgba {
  return hex(frame().theme.palette[name] ?? "#ff00ff");
}

/** a towards b by t (0–1), in sRGB, alpha too. */
export function mix(a: Rgba, b: Rgba, t: number): Rgba {
  const k = Math.max(0, Math.min(1, t));
  return { r: a.r + (b.r - a.r) * k, g: a.g + (b.g - a.g) * k, b: a.b + (b.b - a.b) * k, a: a.a + (b.a - a.a) * k };
}

/** A colour as a Paint. */
export function paint(c: Rgba, alpha = 1): Api.Paint {
  return { color: { r: c.r, g: c.g, b: c.b }, alpha: Math.max(0, Math.min(1, c.a * alpha)) };
}

/** A data series' colour from the style's chart series (docs/ui.md §3.1), cycling. */
export function series(k: number): Rgba {
  const list = (frame().theme.tokens as { series?: string[] } | null)?.series ?? [];
  return list.length > 0 ? hex(list[((k % list.length) + list.length) % list.length]!) : role(k % 2 === 0 ? "primary" : "info");
}

/** A spring of the style's motion personality (ui, panel, emphasis, playful). */
export function spring(name: Api.MotionToken = "ui"): Api.Transition {
  return { kind: "token", token: name };
}

/** Right to left (og.i18n's locale): code-drawn arrows and paths mirror. */
export function rtl(): boolean {
  return kit.frameDirection() === "rtl";
}

/**
 * A fixed height that holds text, at the player's text scale
 * (UiDesc.textScale): a virtual list's row height grows with its text, so
 * the rows stay whole at 200 %.
 */
export function byText(h: number): number {
  return Math.round(h * Math.max(1, frame().theme.textScale ?? 1));
}

/**
 * A virtual row's height that holds two lines of small text once the
 * player's text size grows (a label that wraps rather than cuts), and `h`
 * scaled with the text before that.
 */
export function wrapRow(h: number): number {
  const k = Math.max(1, frame().theme.textScale ?? 1);
  return k > 1 ? Math.max(byText(h), Math.round(2 * 20 * k + 12)) : h;
}

/**
 * A pointer target's side: at least 44 CSS px under a finger (a coarse
 * pointer) whatever its visual (a dot, a crumb), `fine` under a mouse.
 */
export function target(fine: number): number {
  return frame().touch ? Math.max(44, fine) : fine;
}

/** Reduced motion is asked for: movement jumps. */
export function still(): boolean {
  return frame().theme.reducedMotion;
}

/**
 * The full paths of a component's own parts, from the last tick, by a
 * key the component makes (its id and the part): a component reads a
 * part's gesture before it builds that part again. A cache of names only
 * (each tick rewrites it from the tree), so it holds no state a snapshot
 * would miss.
 */
const paths = new Map<string, string>();
export function rememberPath(key: string, path: string): void {
  if (path) paths.set(key, path);
}
export function recallPath(key: string): string {
  return paths.get(key) ?? "";
}

/**
 * The frame's pointer from the centre of a part as it was laid out last
 * tick (by the key its path was remembered under): for a component that
 * hit-tests a round shape by angle and radius (a pie, a wheel). Null when
 * the pointer is off the surface or the part was not there.
 */
export function pointerFrom(key: string): { dx: number; dy: number; d: number; x: number; y: number; down: boolean } | null {
  const p = recallPath(key);
  const b = p ? kit.bounds(p) : null;
  const at = kit.framePointer() as Api.UiPointer;
  if (!b || !at.inside) return null;
  const dx = at.x - (b.x + b.width / 2);
  const dy = at.y - (b.y + b.height / 2);
  return { dx, dy, d: Math.hypot(dx, dy), x: at.x, y: at.y, down: at.down };
}

/** The gesture on a part last tick, by the key its path was remembered under. */
export function lastGesture(key: string): Api.GestureInfo | null {
  const p = recallPath(key);
  return p ? (kit.gesture(p) as Api.GestureInfo) : null;
}

/** A path's parent path (empty at the top). */
export function parentOf(path: string): string {
  const k = path.lastIndexOf("/");
  return k < 0 ? "" : path.slice(0, k);
}

/** Props for the element just built as a layer over its parent's content (a badge, a ring, a fade): absolute, declared to overlap the parent's subtree on purpose (docs/ui.md §2). */
export function over(p: Api.ElementProps): void {
  kit.props(overProps(p));
}

/** over()'s props, for an element whose props are set together with others (a surface's `layer`). */
export function overProps(p: Api.ElementProps): Api.ElementProps {
  const parent = parentOf(kit.lastPath());
  return { ...p, position: "absolute", overlaps: [...(p.overlaps ?? []), ...(parent ? [parent] : [])] };
}

export const clamp = (v: number, lo: number, hi: number): number => Math.max(lo, Math.min(hi, v));

/** The last tick's interaction of an element under the open element: what it was told by close(). */
export interface Interaction { hovered: boolean; pressed: boolean; focused: boolean }

/** What close() said about `id` last tick (kept in the UI's memory, so a snapshot carries it). */
export function lastInteraction(id: string): Interaction {
  const bits = kit.memory(`${id}~ix`, 0);
  return { hovered: (bits & 1) !== 0, pressed: (bits & 2) !== 0, focused: (bits & 4) !== 0 };
}

function keep(id: string, r: Api.Response): void {
  kit.remember(`${id}~ix`, (r.hovered ? 1 : 0) | (r.pressed ? 2 : 0) | (r.focused ? 4 : 0));
}

/** A selector's flags as `[a][b]`. */
export function flags(...f: Array<string | false | null | undefined>): string {
  return f.filter((x): x is string => typeof x === "string" && x.length > 0).map((x) => `[${x}]`).join("");
}

export interface SurfaceDesc {
  /** The part a style dresses: `Menu.item`, `Tabs.tab`. Flags for its state are added. */
  part: string;
  /** The element class whose shape language and elevation an undressed part takes. */
  cls?: string;
  /** Other selector flags (selected, danger, current) and attributes (`tone=warn`). */
  flags?: string[];
  /** Its fill at rest; null is transparent (a quiet row). */
  fill?: Rgba | null;
  /** What the fill moves towards under hover and press. */
  wash?: Rgba | undefined;
  /** How far hover (×0.5) and press (×1) move it towards the wash (0.10 by default). */
  washBy?: number;
  disabled?: boolean;
  /** Keys and pads reach it (true by default). */
  focusable?: boolean;
  direction?: Api.Direction;
  width?: Api.Size | null;
  height?: Api.Size | null;
  padding?: Api.Insets;
  gap?: number;
  align?: Api.Align;
  cross?: Api.Cross;
  radius?: number;
  wrap?: boolean;
  /** Its accessibility: a role other than button, a name, states. */
  semantics?: Api.SemanticsDesc;
  /** Selectable (a tab, a page, a day, a card): its `selection` state group, off or on. */
  selected?: boolean;
  /** Its fill while selected (null: transparent); `fill` is its fill while not. */
  selectedFill?: Rgba | null;
  /** Expandable (an accordion header, a submenu's row): its `expansion` state group, collapsed or expanded. */
  expanded?: boolean;
  /** With focusable: Left and Right are its own while it has the focus (Response.adjust). */
  adjustable?: boolean;
  /** How far it rises (negative y, CSS px) under the pointer and the focus: a card's lift. */
  lift?: number;
  /** Its scale while pressed (a card, a hint, a round button sinks to 0.96): 1 is none. */
  pressScale?: number;
  /** Its other props (a drag, a placement): set with its states, which a later og.ui.props would replace. */
  props?: Api.ElementProps;
  /** Placed as a layer over its parent's content (over()), with these props. */
  layer?: Api.ElementProps;
}

/** A colour as the four background values of a visual state. */
function bgValues(c: Rgba): Api.PropValue[] {
  return [
    { prop: "backgroundR", value: c.r },
    { prop: "backgroundG", value: c.g },
    { prop: "backgroundB", value: c.b },
    { prop: "backgroundA", value: c.a },
  ];
}

/** A visual state: its name and values, no timelines. */
export function vstate(name: string, values: Api.PropValue[] = []): Api.VisualState {
  return { name, values, onEnter: "", onExit: "" };
}

/**
 * An interactive surface: a container keys, pads and the pointer reach,
 * drawn as its part in its interaction state (`Menu.item[hover]`,
 * `[pressed]`, `[focused]`, `[disabled]` and the given flags), its fill
 * washing towards ink as it is hovered and pressed on the style's ui
 * spring. Answers close()'s response (zero while disabled).
 */
export function surface(id: string, d: SurfaceDesc, build: () => void): Api.Response {
  const was = lastInteraction(id);
  const on = !d.disabled;
  const wash = d.wash ?? role("ink");
  const by = d.washBy ?? 0.1;
  const sel = d.selected ?? false;
  // Its fill at rest, selected or not; transparent is the wash's colour at no alpha (a fade from
  // nothing passes through no grey).
  const rest = (s: boolean): Rgba => (s && d.selectedFill !== undefined ? d.selectedFill : d.fill) ?? { ...wash, a: 0 };
  // Hover and focus move it half-way to the wash, a press 2.4 times as far: a press reads at a
  // glance, not only beside the idle one.
  const tint = (base: Rgba, k: number): Rgba => (base.a > 0 ? mix(base, { ...wash, a: Math.max(base.a, 1) }, k * by) : { ...wash, a: k * by * 1.4 });
  // The motion values of each interaction state: the lift and the press's sink.
  const lift = still() ? 0 : d.lift ?? 0;
  const sink = still() ? 1 : d.pressScale ?? 1;
  const moved = (y: number, k: number): Api.PropValue[] => (lift !== 0 || sink !== 1 ? [{ prop: "y", value: y }, { prop: "scale", value: k }] : []);
  const base = rest(sel);
  const part = `${d.part}${flags(...(d.flags ?? []), !on && "disabled", on && was.pressed && "pressed", on && was.hovered && !was.pressed && "hover", on && was.focused && "focused")}`;
  kit.open(id, {
    direction: d.direction ?? "row",
    width: d.width ?? null,
    height: d.height ?? null,
    padding: d.padding ?? all(0),
    gap: d.gap ?? 0,
    align: d.align ?? "start",
    cross: d.cross ?? "center",
    wrap: d.wrap ?? false,
    interactive: on,
    focusable: on && (d.focusable ?? true),
    adjustable: on && (d.adjustable ?? false),
    background: base.a > 0 ? paint(base) : null,
    radius: d.radius ?? frame().t.control,
    part,
    partClass: d.cls ?? "button",
  });
  let r: Api.Response;
  try {
    build();
  } finally {
    r = kit.close() as Api.Response;
  }
  keep(id, r);
  // Its states (docs/ui.md §3.1): og-ui derives interaction from input, the game gives selection
  // and expansion; every change moves on the style's motion (its reduced fallback under reduced
  // motion). The part's selector flags above stay for a style's dressings.
  const states: Api.StateGroup[] = [];
  const state: Api.StateValue[] = [];
  if (d.selected !== undefined) {
    states.push({ name: "selection", states: [vstate("off", bgValues(rest(false))), vstate("on", bgValues(rest(true)))], transitions: [] });
    state.push({ group: "selection", state: sel ? "on" : "off" });
  }
  states.push({
    name: "interaction",
    states: [
      vstate("normal", [...bgValues(base), ...moved(0, 1)]),
      vstate("hover", [...bgValues(tint(base, 0.5)), ...moved(lift, 1)]),
      // Pressed: down past rest (a lifted part drops below its place) and darker by twice hover's.
      vstate("pressed", [...bgValues(tint(base, 2.4)), ...moved(lift !== 0 ? 1 : 0, sink)]),
      vstate("focused", [...bgValues(tint(base, 0.5)), ...moved(lift, 1)]),
      vstate("disabled", [...bgValues(base), ...moved(0, 1)]),
    ],
    transitions: [],
  });
  if (d.expanded !== undefined) {
    states.push({ name: "expansion", states: [vstate("collapsed"), vstate("expanded")], transitions: [] });
    state.push({ group: "expansion", state: d.expanded ? "expanded" : "collapsed" });
  }
  const extra = d.layer ? overProps(d.layer) : d.props ?? {};
  kit.props({ ...extra, states: [...(extra.states ?? []), ...states], state: [...(extra.state ?? []), ...state] });
  if (d.semantics || d.disabled) kit.semantics({ ...(d.semantics ?? {}), disabled: d.disabled ?? false });
  return on ? r : { ...r, clicked: false, pressed: false, adjust: 0 };
}

/** Which way a chevron points: forward (the reading direction), back, up, down. */
export type Way = "forward" | "back" | "up" | "down";

/** A part's turn by state: a group whose states each turn it by quarter turns (an accordion's chevron: collapsed 0, expanded 2), moved on the style's motion. */
export interface Turns {
  group: string;
  current: string;
  turns: Record<string, number>;
}

/**
 * A chevron drawn as a path (no icon atlas needed), `size` square, in
 * `color`, turned by `turn` quarter turns on top of its way, or by state
 * (`Turns`: an accordion's opening, a sort's direction). Forward and back
 * follow the writing direction.
 */
export function chevron(id: string, way: Way, size: number, color: Rgba, turn: number | Turns = 0, weight = 2): void {
  const flip = rtl();
  const base = way === "down" ? 90 : way === "up" ? -90 : (way === "back") !== flip ? 180 : 0;
  kit.open(id, { width: px(size), height: px(size) });
  const s = size;
  const pts = [{ x: s * 0.38, y: s * 0.22 }, { x: s * 0.66, y: s * 0.5 }, { x: s * 0.38, y: s * 0.78 }];
  kit.setDraw({ shape: { kind: "path", points: pts, width: weight, cap: "round", join: "round" }, fills: [{ kind: "solid", color: paint(color) }] });
  kit.close();
  const rad = (q: number) => ((base + 90 * q) * Math.PI) / 180;
  if (typeof turn === "number") {
    kit.props({ transform: { rotate: rad(turn) } });
    return;
  }
  kit.props({
    transform: { rotate: rad(turn.turns[turn.current] ?? 0) },
    states: [{ name: turn.group, states: Object.entries(turn.turns).map(([name, q]) => vstate(name, [{ prop: "rotate", value: rad(q) }])), transitions: [] }],
    state: [{ group: turn.group, state: turn.current }],
  });
}

/** A small mark drawn as a path: a tick, a cross, a plus, a minus, a dot or a ring. */
export function mark(id: string, kind: "tick" | "cross" | "plus" | "minus" | "dot" | "ring" | "more", size: number, color: Rgba, weight = 2): void {
  const s = size;
  kit.open(id, { width: px(s), height: px(s) });
  const fill: Api.Fill[] = [{ kind: "solid", color: paint(color) }];
  if (kind === "dot") kit.setDraw({ shape: { kind: "ellipse" }, fills: fill });
  else if (kind === "ring") kit.setDraw({ shape: { kind: "arc", thickness: weight }, fills: fill });
  else if (kind === "more") kit.setDraw({ shape: { kind: "path", points: [{ x: s * 0.2, y: s / 2 }, { x: s * 0.2 + 0.01, y: s / 2 }, { x: s / 2, y: s / 2 }, { x: s / 2 + 0.01, y: s / 2 }, { x: s * 0.8, y: s / 2 }, { x: s * 0.8 + 0.01, y: s / 2 }], width: weight * 1.4, cap: "round" }, fills: fill });
  else {
    const p = (x: number, y: number) => ({ x: x * s, y: y * s });
    const pts = kind === "tick" ? [p(0.2, 0.52), p(0.42, 0.74), p(0.8, 0.28)]
      : kind === "cross" ? [p(0.25, 0.25), p(0.75, 0.75), p(0.5, 0.5), p(0.75, 0.25), p(0.25, 0.75)]
      : kind === "plus" ? [p(0.5, 0.2), p(0.5, 0.8), p(0.5, 0.5), p(0.2, 0.5), p(0.8, 0.5)]
      : [p(0.22, 0.5), p(0.78, 0.5)];
    kit.setDraw({ shape: { kind: "path", points: pts, width: weight, cap: "round", join: "round" }, fills: fill });
  }
  kit.close();
}

/** The ink roles, which text over a scene (onScene) takes in the scene's ink instead. */
const INKS: ReadonlyMap<string, number> = new Map([["ink", 1], ["inkMuted", 0.82], ["inkFaint", 0.68]]);

/**
 * A label in a role colour: the common call, shorter. `onScene` is text laid over a scene or key art rather than a
 * panel (LabelDesc.onScene): the ink roles become the style's sceneInk, at their emphasis, with its sceneShadow
 * separation, so a light style's dark ink never lands on dark art.
 */
export function text(id: string, value: string, d: { step?: Api.TypeStepName; color?: Api.ThemeColor | Rgba; nowrap?: boolean; ellipsis?: boolean; width?: Api.Size; align?: Api.TextAlign; font?: Api.FontRole; heading?: boolean; maxLines?: number; fitMin?: Api.TypeStepName; onScene?: boolean; uppercase?: boolean } = {}): void {
  const scene = d.onScene ? INKS.get(typeof d.color === "string" ? d.color : d.color === undefined ? "ink" : "") : undefined;
  const color = scene !== undefined ? token("sceneInk", scene) : d.color === undefined ? token("ink") : typeof d.color === "string" ? token(d.color) : paint(d.color);
  kit.label(id, value, {
    step: d.step ?? "body",
    color,
    ...(d.onScene ? { onScene: true } : {}),
    ...(d.uppercase ? { uppercase: true } : {}),
    nowrap: d.nowrap ?? false,
    ellipsis: d.ellipsis ?? false,
    align: d.align ?? "start",
    font: d.font ?? "body",
    heading: d.heading ?? false,
    ...(d.width ? { width: d.width } : {}),
    ...(d.maxLines ? { maxLines: d.maxLines } : {}),
    ...(d.fitMin ? { fitMin: d.fitMin } : {}),
  });
}

/**
 * Text that fits a `width` × `height` box whole, its words never split (a
 * hub's name, a narrow cell): the largest of `steps` at which every word
 * fits the width and the words, packed greedily, take at most `maxLines`
 * lines within the height; the smallest step otherwise. Each word is its
 * own unwrapped label in a wrapping row (`Word#i`), so a line breaks only
 * between words. Word widths are last tick's per pixel of type size (a
 * word not yet drawn guesses 0.6 em a character).
 */
export function wholeWords(id: string, value: string, d: { width: number; height: number; maxLines: number; steps: Api.TypeStepName[]; color?: Api.ThemeColor; heading?: boolean; align?: "start" | "center" }): void {
  const ramp = frame().theme.typeRamp as Record<string, Api.TypeStep>;
  const words = value.split(/\s+/).filter((w) => w.length > 0);
  const perPx = words.map((w) => kit.memory(`~ww:${w}`, 0) || w.length * 0.6);
  const fits = (s: Api.TypeStepName): boolean => {
    const step = ramp[s];
    if (!step) return false;
    const space = step.size * 0.3;
    let lines = 1;
    let line = 0;
    for (const k of perPx) {
      const w = k * step.size;
      if (w > d.width) return false;
      if (line > 0 && line + space + w > d.width) {
        lines++;
        line = w;
      } else line += (line > 0 ? space : 0) + w;
    }
    return lines <= d.maxLines && lines * step.lineHeight <= d.height + 0.5;
  };
  const chosen = d.steps.find(fits) ?? d.steps[d.steps.length - 1]!;
  const size = ramp[chosen]?.size ?? 16;
  kit.open(id, { direction: "row", wrap: true, width: px(d.width), align: d.align ?? "center", cross: "center", gap: Math.round(size * 0.3) });
  words.forEach((w, i) => text(kit.idi("Word", i), w, { step: chosen, nowrap: true, ...(d.color ? { color: d.color } : {}), ...(d.heading ? { heading: true } : {}) }));
  kit.close();
  const row = kit.lastPath();
  kit.semantics({ role: "text", name: value });
  words.forEach((w, i) => {
    const b = kit.bounds(`${row}/${kit.idi("Word", i)}`);
    if (b && b.width > 0) kit.remember(`~ww:${w}`, b.width / size);
  });
}

/**
 * A key cap or pad button glyph for an action hint (docs/ui.md §2.1),
 * drawn from the glyph's text: a keyboard key as a rounded cap, a pad face
 * button as a disc, a bumper or trigger as a pill.
 */
export function glyphCap(id: string, glyph: { text: string; icon: string }, size0: number, progress = 0): void {
  const t = frame().t;
  // A cap's name is text: it grows with the player's text size.
  const size = byText(size0);
  const pad = /^(xbox|ps|switch|pad)\./.test(glyph.icon);
  const shoulder = /bumper|trigger|\.(lb|rb|lt|rt|l1|r1|l2|r2|l|r|zl|zr)$/i.test(glyph.icon) || /^(LB|RB|LT|RT|L1|R1|L2|R2|ZL|ZR)$/.test(glyph.text);
  // A face button prints one letter or symbol in its disc; a longer name (L Stick, View, Options) is a pill that fits it.
  const face = pad && !shoulder && (PS_FACE[glyph.icon] !== undefined || [...glyph.text].length <= 1);
  const pill = pad && !face;
  const ink = role("ink");
  kit.open(id, {
    width: face ? px(size) : fit(size),
    height: px(size),
    padding: xy(face ? 0 : Math.round(size * 0.28), 0),
    align: "center",
    cross: "center",
    radius: face || pill ? size / 2 : Math.max(3, t.small * 2),
    background: paint(mix(role("panelRaised"), ink, 0.12)),
    part: `ActionBar.glyph${flags(face ? "face" : pill ? "shoulder" : "key", (face || pill) && "round")}`,
    partClass: "chip",
  });
  const symbol = PS_FACE[glyph.icon];
  if (symbol) faceSymbol("Symbol", symbol, Math.round(size * 0.5), ink);
  // A key's name at the body's 16 px (at 1080p), as the hint beside it.
  else text("Text", glyph.text, { step: "base", nowrap: true, font: "mono" });
  // A hold's progress: a ring around the cap, centred on it and scaled with it (the cap's width
  // as laid out last tick, so a key cap wider than tall is ringed round its middle).
  if (progress > 0) {
    const cap = kit.bounds(parentOf(kit.lastPath()));
    const gap = Math.max(2, Math.round(size * 0.1));
    const w = Math.max(2, Math.round(size * 0.1));
    const d = size + 2 * gap;
    const cw = cap?.width ?? size;
    kit.open("Ring", { width: px(d), height: px(d) });
    kit.setDraw({ shape: { kind: "arc", thickness: w }, fills: [{ kind: "solid", color: token("lineStrong", 0.6) }] });
    kit.close();
    over({ insets: { start: (cw - d) / 2, top: -gap }, inert: true, z: 1 });
    kit.open("Progress", { width: px(d), height: px(d) });
    kit.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.PI * 2 * clamp(progress, 0, 1), thickness: w, cap: "round" }, fills: [{ kind: "solid", color: token("primary") }] });
    kit.close();
    over({ insets: { start: (cw - d) / 2, top: -gap }, inert: true, z: 2 });
  }
  kit.close();
  // A drawn symbol has no text: the cap says what it is.
  if (symbol) kit.semantics({ role: "image", name: glyph.text });
}

/** PlayStation's face buttons: drawn, never set as text, so no style's font can show tofu for × ○ □ △. */
const PS_FACE: Record<string, "cross" | "circle" | "square" | "triangle"> = { "ps.cross": "cross", "ps.circle": "circle", "ps.square": "square", "ps.triangle": "triangle" };

/**
 * A PlayStation face symbol in a `size` box, stroked in `color`. The four
 * are matched optically, as the pad prints them: the same stroke, the ring
 * a little inside the box, the square smaller again (its corners carry
 * more ink), the cross's arms reaching the ring's edge, the triangle as
 * wide as the ring.
 */
function faceSymbol(id: string, kind: "cross" | "circle" | "square" | "triangle", size: number, color: Rgba): void {
  const w = Math.max(1.5, size * 0.12);
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center" });
  const fill: Api.Fill[] = [{ kind: "solid", color: paint(color) }];
  const p = (x: number, y: number) => ({ x: x * size, y: y * size });
  if (kind === "circle") {
    const d = Math.round(size * 0.9);
    kit.open("Shape", { width: px(d), height: px(d) });
    kit.setDraw({ shape: { kind: "arc", thickness: w }, fills: fill });
    kit.close();
  } else {
    // A closed outline repeats its first two points, so its first corner joins too.
    const pts = kind === "cross" ? [p(0.16, 0.16), p(0.84, 0.84), p(0.5, 0.5), p(0.84, 0.16), p(0.16, 0.84)]
      : kind === "square" ? [p(0.14, 0.14), p(0.86, 0.14), p(0.86, 0.86), p(0.14, 0.86), p(0.14, 0.14), p(0.86, 0.14)]
      : [p(0.5, 0.1), p(0.97, 0.9), p(0.03, 0.9), p(0.5, 0.1), p(0.97, 0.9)];
    kit.setDraw({ shape: { kind: "path", points: pts, width: kind === "cross" ? w * 1.1 : w, cap: "round", join: "round" }, fills: fill });
  }
  kit.close();
}

/** Where a floating surface's content goes this tick. */
export interface Placed {
  x: number;
  y: number;
  /** Its own size (last tick's); 0 before it was measured. */
  width: number;
  height: number;
  /** Measured at least once: it shows. */
  measured: boolean;
  /** 0 closed to 1 open, on the style's spring (1 at once under reduced motion). */
  openness: number;
}

/**
 * The modal layer used as a floating surface (a popover, a menu, a sheet):
 * a focus scope, Back and a press outside dismiss it, focus returns where
 * it was when it closes, and it plays the style's open and close sounds.
 * The backdrop is clear unless `scrim`. `place` answers where the content
 * goes from its own laid-out size (last tick's), and the content (built
 * by `build`, rooted at `Content` with floatingContent) is moved there by
 * a transform: beside an anchor, on an edge. Nothing shows until it has
 * been measured once, so it never flashes in the middle.
 */
export function floating(
  id: string,
  open: boolean,
  d: { scrim?: boolean; locked?: boolean; transition?: Api.TransitionKind; width?: Api.Size; spring?: Api.MotionToken },
  place: (size: { width: number; height: number }) => { x: number; y: number },
  build: (at: Placed) => void,
  props?: (at: Placed) => Api.ElementProps,
): { visible: boolean; dismissed: boolean; path: string } {
  const m = kit.openModal(id, open, {
    bare: true,
    width: d.width ?? fit(),
    locked: d.locked ?? false,
    backdrop: d.scrim ? token("scrim") : token("scrim", 0),
    transition: d.transition ?? "fade",
    title: "",
  }) as Api.ModalResult;
  const path = `${id}/Panel/Content`;
  try {
    if (m.visible) {
      // Where the content is laid out: the bare panel around it, which never moves (the content's
      // own bounds include the move it takes, and are cut to the screen).
      const b = kit.bounds(`${id}/Panel`);
      const open1 = b !== null && open;
      // A menu or a popover is there to be used at once: a short ease out (a sheet passes its spring).
      const k = kit.animate("~open", open1 ? 1 : 0, d.spring ? spring(d.spring) : { kind: "tween", durationMs: open ? 140 : 100, bezier: open ? [0.23, 1, 0.32, 1] : [0.4, 0, 1, 1] });
      const want = b ? place({ width: b.width, height: b.height }) : { x: 0, y: 0 };
      const at: Placed = { x: want.x, y: want.y, width: b?.width ?? 0, height: b?.height ?? 0, measured: b !== null, openness: still() ? (open ? 1 : 0) : k };
      build(at);
      const extra = props?.(at) ?? {};
      const tr = extra.transform ?? {};
      const to = b ? { x: Math.round(want.x - b.x), y: Math.round(want.y - b.y) } : { x: 0, y: 0 };
      kit.props({
        ...extra,
        transform: { ...tr, x: to.x + (tr.x ?? 0), y: to.y + (tr.y ?? 0) },
        opacity: b ? (extra.opacity ?? 1) : 0,
      });
    }
  } finally {
    kit.closeModal();
  }
  return { visible: m.visible, dismissed: m.dismissed, path };
}

/** The open floating content's root: an interactive column (a press on it is not a press outside). */
export function floatingContent(d: Api.ElementDesc, build: () => void): void {
  kit.open("Content", { ...d, interactive: true });
  try {
    build();
  } finally {
    kit.close();
  }
}

export type Side = "below" | "above" | "start" | "end";

/**
 * Where a floating box of `size` sits beside `anchor` on `side`, `gap`
 * away, aligned to its start: flipped to the other side when it does not
 * fit there and the other side has more room, then kept inside the safe
 * area. Start and end follow the writing direction.
 */
export function beside(anchor: Api.UiRect, size: { width: number; height: number }, side: Side, gap: number, align: "start" | "center" | "end" = "start"): { x: number; y: number; side: Side } {
  const v = frame().viewport;
  const m = frame().t.sm;
  const area = { x: v.safeArea.left + m, y: v.safeArea.top + m, r: v.width - v.safeArea.right - m, b: v.height - v.safeArea.bottom - m };
  const flip = rtl();
  let s: Side = side;
  if (s === "start" || s === "end") {
    const physicalLeft = (s === "start") !== flip;
    const roomL = anchor.x - area.x;
    const roomR = area.r - (anchor.x + anchor.width);
    let left = physicalLeft;
    if (left && roomL < size.width + gap && roomR > roomL) left = false;
    else if (!left && roomR < size.width + gap && roomL > roomR) left = true;
    s = left !== flip ? "start" : "end";
    const x = left ? anchor.x - gap - size.width : anchor.x + anchor.width + gap;
    const y = align === "center" ? anchor.y + (anchor.height - size.height) / 2 : align === "end" ? anchor.y + anchor.height - size.height : anchor.y;
    return { x: clamp(x, area.x, Math.max(area.x, area.r - size.width)), y: clamp(y, area.y, Math.max(area.y, area.b - size.height)), side: s };
  }
  const roomB = area.b - (anchor.y + anchor.height);
  const roomA = anchor.y - area.y;
  let below = s === "below";
  if (below && roomB < size.height + gap && roomA > roomB) below = false;
  else if (!below && roomA < size.height + gap && roomB > roomA) below = true;
  const y = below ? anchor.y + anchor.height + gap : anchor.y - gap - size.height;
  let x: number;
  if (align === "center") x = anchor.x + (anchor.width - size.width) / 2;
  else if ((align === "start") !== flip) x = anchor.x;
  else x = anchor.x + anchor.width - size.width;
  return { x: clamp(x, area.x, Math.max(area.x, area.r - size.width)), y: clamp(y, area.y, Math.max(area.y, area.b - size.height)), side: below ? "below" : "above" };
}

/** A count of a thing in a label: `{n}` filled. */
export function fill(template: string, values: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (all, k: string) => (k in values ? String(values[k]) : all));
}

/** The elevation shadows of a level as effects, from the style's tokens. */
export function elevation(level: "raised" | "floating" | "overlay"): Api.Effect[] {
  const e = (frame().theme.tokens as { elevation?: Record<string, Array<{ dx: number; dy: number; blur: number; spread: number; color: string }>> } | null)?.elevation?.[level] ?? [];
  return e.map((s) => ({ kind: "dropShadow", offset: { x: s.dx, y: s.dy }, blur: s.blur, spread: s.spread, color: paint(hex(s.color)) }) as Api.Effect);
}
