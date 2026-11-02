// `og ui check` (docs/ui.md §16.5): the rules a UI designer checks by eye,
// as checks with a path and a fix. One checker, called by `og ui check`
// (ui-run.ts, over a screen set × breakpoints × locales), by `og polish`
// (polish.ts, on each playtest frame) and by playtests (`checkUi`, from
// `@origingame/engine/ui-check`, on the frames a `dump_ui:<name>.uicheck.json`
// step saved). It reads one laid-out frame (`og_node_ui_frame`, the native
// host's reading of og-ui's UiFrame: elements, undeclared overlaps, drawn
// text, fills, the accessibility tree, layout counts, viewport and theme)
// and, where there is one, the rendered frame (contrast on pixels, as
// polish measures it).
//
// Every finding names the rule, the element path(s), the measured value,
// the limit, the prop to change and the fix, as JSON and as text (`text`).
// The rules that need data later phases bring are listed in RULES with
// status "not checkable yet" and the reason, never skipped silently.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { decodePng, type Image } from "./png.ts";

// ── what the checker reads ─────────────────────────────────────────────────

/** [x, y, width, height], CSS px. */
export type Rect4 = [number, number, number, number];

/** One element as the probe's `dump_ui` lists it. */
export interface UiElementDump {
  id: string;
  parent: string;
  kind?: string;
  disabled?: boolean;
  clipped?: boolean;
  text: string;
  role?: string;
  name?: string;
  states?: string[];
  x: number;
  y: number;
  width: number;
  height: number;
}

/** What `dump_ui` writes: polish reads its elements. */
export interface UiDump {
  tick: number;
  /** The surface the CSS-pixel bounds are on (dpr: device pixels per CSS pixel). */
  surface?: { width: number; height: number; dpr: number } | null;
  /** The configured UI style's name: "" is the engine's default style. */
  style?: string | null;
  elements: UiElementDump[];
}

/** The native host's reading of a laid-out frame (crates/og-host-node/src/ui_check.rs). */
export interface UiFrameSnapshot {
  viewport: {
    width: number;
    height: number;
    dpr: number;
    /** left, top, right, bottom. */
    safeArea: [number, number, number, number];
    coarsePointer: boolean;
    highContrast: boolean;
    reducedMotion: boolean;
    /** The host's own control (the player shell's Back); zero size: none. */
    covered: Rect4;
    breakpoint: string;
  };
  /** The configured style's name ("": the default style). */
  style: string;
  contrast: "standard" | "high";
  /** The style's event sounds, by event (hover, press, open, close, success, error): whether one is set. */
  sounds: Record<string, boolean>;
  /** The palette's primary, primary hover and primary active, linear RGBA. */
  primary: number[][];
  elements: UiElementDump[];
  overlaps: Array<{ path: string; other: string; rect: Rect4; inFlow: boolean }>;
  /** Paths of the elements whose props say dense (a dense HUD's readouts, §15.2), each covering its subtree. */
  dense?: string[];
  stats: { elements: number; rebuilt: number; laidOut: number; keptCommands: number };
  draws: number;
  /** Every drawn line of text: the string as drawn, its size and line box, and its clip. */
  texts: Array<{ text: string; size: number; bounds: Rect4; clip: Rect4; color: number[] }>;
  /** Solid and image fills of at least 8×8 CSS px, linear RGBA. */
  fills: Array<{ rect: Rect4; color: number[] }>;
  a11y: { focused: string; focusOrder: string[]; nodes: Array<{ path: string; parent: string; role: string; name: string; states: string[]; rect: Rect4 }> };
  /** Elements that declare visual states, with each group's current state. */
  motion: Array<{ path: string; states: Array<[string, string]> }>;
  warnings: string[];
}

/** Where a frame was taken: the screen and the cell of the sheet. */
export interface UiCell {
  screen: string;
  breakpoint: string;
  /** The locale tag, `pseudo` for the pseudo-locale, "" for the game's own. */
  locale: string;
  /** The style the screen wore ("" the game's own). */
  style: string;
  /** Text scale, percent. */
  scale: number;
  /** Laid out in high-contrast mode (the contrast bar is 7:1). */
  highContrast: boolean;
}

export type Severity = "error" | "warning";

export interface UiFinding {
  rule: string;
  severity: Severity;
  /** The element(s), by path (`Hud.top/Timer`). */
  paths: string[];
  /** What was measured, with its unit (`44×44 CSS px overlap`, `3.1:1`). */
  measured: string;
  /** The bar it was held to. */
  limit: string;
  /** The prop (or token) to change. */
  prop: string;
  fix: string;
  /** The cells it was found in (`phonePortrait en`). */
  cells: string[];
}

// ── the rules ──────────────────────────────────────────────────────────────

export type RuleStatus = "checked" | "partial" | "not checkable yet";

export interface UiRule {
  id: string;
  /** docs/ui.md §16.5's words. */
  title: string;
  status: RuleStatus;
  /** For partial and not checkable rules: what is missing and which phase brings it. */
  note?: string;
}

export const RULES: readonly UiRule[] = [
  { id: "overlap", title: "overlap not declared (§6.2), across breakpoints and locales", status: "checked" },
  { id: "text-cut", title: "text clipped without an ellipsis, or drawn past its box", status: "checked" },
  { id: "text-truncated", title: "text truncated with an ellipsis but without a tooltip", status: "partial", note: "the ellipsis is read from the drawn text; a tooltip exists only while hovered (og-ui builds it then), so its absence is reported as a warning until U3's dump_ui lists an element's tooltip" },
  { id: "offscreen", title: "an interactive element laid out past the screen with nothing to scroll to it", status: "checked" },
  { id: "pseudo-overflow", title: "overflow under the pseudo-locale (+40 % length, accents, brackets)", status: "checked", note: "the pseudo cells run every rule above on text lengthened in process (og.i18n.t and the text arguments of og.ui's widgets)" },
  { id: "text-scale", title: "overflow at 200 % text, and text that does not grow with the player's text size", status: "checked", note: "a scale cell configures UiDesc.textScale (og.ui.configure) and runs every rule above; a text cut there is reported under this rule, and a text drawn at less than the scale times its size at 100 % (a fixed size the scale does not reach) too" },
  { id: "safe-area", title: "elements outside the safe area", status: "checked" },
  { id: "covered", title: "elements under the shell's covered region", status: "checked" },
  { id: "touch-target", title: "touch targets under 44 CSS px on a coarse pointer", status: "checked", note: "an element's hitSlop is not in the frame: a small control with slop enough is still reported; a link inside a rich text's sentence is exempt (WCAG 2.5.8)" },
  { id: "contrast", title: "contrast under 4.5:1 (7:1 in high contrast), on pixels as og polish measures it", status: "checked" },
  { id: "text-size", title: "text under the minimum size", status: "checked", note: "18 px at 1080p; a dense HUD's readouts (ElementProps.dense) 12 px at 1080p times the text scale" },
  { id: "focus-reach", title: "an interactive element unreachable by pad from the default focus", status: "checked" },
  { id: "focus-trap", title: "a focus trap: a control the pad cannot leave back to the default focus", status: "partial", note: "the pad graph is walked both ways, but og-ui's direction rule (nearest, wrapping) leaves no geometric trap (none in 600,000 random layouts); the real trap, a scope Back does not close, needs U3's layer stack" },
  { id: "layer-back", title: "a layer without Back", status: "not checkable yet", note: "the layer stack and its input modes are U3's (§8.1); a frame has no layers yet" },
  { id: "a11y-name", title: "accessibility names missing", status: "checked" },
  { id: "a11y-locale", title: "names and texts in the wrong locale (left untranslated)", status: "checked", note: "a cell of another locale is compared with the game's own, path by path" },
  { id: "primary-actions", title: "more than one primary action in a layer", status: "partial", note: "a button is primary when its fill is the palette's primary; the layers are approximated by the base and each modal until U3's layer stack" },
  { id: "feedback", title: "an interactive element without hover or press states, or without a sound in a styled game", status: "partial", note: "og-ui's widgets carry their hover and press; a custom interactive container is checked for declared visual states (frame.motion), its resolved states arrive with U3's dump_ui" },
  { id: "motion", title: "durations outside the style's bounds, missing reduced-motion fallbacks, input refused longer than the entrance", status: "not checkable yet", note: "the style's motion bounds and per-element fallbacks are U4's style v2 tokens on U2's states; the frame does not carry a transition's duration or its reduced variant yet" },
  { id: "art-slot", title: "art slots still on their placeholder; focal points outside the crop", status: "not checkable yet", note: "slots are specified (og_ui::slot) but not built into a frame or drawn from the style yet (U1, U4); nothing lists a slot's art" },
  { id: "budget", title: "element counts and draws per tick over §15.1", status: "checked", note: "counts per frame (300 elements, the dense-menu budget); per-tick times are the perf lane's" },
];

const ruleOf = new Map(RULES.map((r) => [r.id, r]));

/** A role a player operates (focus, press, drag). */
const INTERACTIVE = new Set(["button", "link", "checkbox", "radio", "switch", "slider", "textbox", "combobox", "option", "tab", "spinbutton", "menuitem"]);
/** og-ui's widgets that draw their own hover and press (and sound them through the style). */
const FEEDBACK_KINDS = new Set(["button", "action", "chip", "toggle", "checkbox", "radio", "slider", "field", "textarea", "select", "option", "tab", "link", "joystick", "dpad"]);
/** Touch controls: the player's thumbs on a HUD, not part of pad navigation. */
const TOUCH_CONTROLS = new Set(["joystick", "dpad", "action"]);
/** Pictures: their `text` is the picture's name, not words. */
const PICTURES = new Set(["icon", "image"]);
/** Containers: they place, they show nothing of their own. */
const CONTAINERS = new Set(["anchor", "row", "column", "panel", "scroll", "spacer", "group", "stack", "grid", "root", ""]);

/** The minimum text, CSS px (§15.2: 18 px at 1080p on desktop; phone-equivalent on touch). */
export function minTextSize(v: UiFrameSnapshot["viewport"]): number {
  if (v.coarsePointer) return 12;
  return Math.max(12, Math.round((18 * Math.min(v.width, v.height * (16 / 9))) / 1920 * 10) / 10);
}

/**
 * The minimum text of a dense HUD's readouts (ElementProps.dense: strategy,
 * MMO, simulation), CSS px: §15.2's 12 px at 1080p on desktop, times the
 * player's text scale (`scale`, percent), so it grows with it. Touch takes
 * no allowance: its 12 px is already a phone's floor.
 */
export function denseTextSize(v: UiFrameSnapshot["viewport"], scale = 100): number {
  if (v.coarsePointer) return minTextSize(v);
  const at1080 = Math.round((12 * Math.min(v.width, v.height * (16 / 9))) / 1920 * 10) / 10;
  return Math.max(8, Math.round(at1080 * (scale / 100) * 10) / 10);
}

const r1 = (x: number) => Math.round(x * 10) / 10;
const r2 = (x: number) => Math.round(x * 100) / 100;
const area = (r: Rect4) => Math.max(0, r[2]) * Math.max(0, r[3]);
const rectOf = (e: { x: number; y: number; width: number; height: number }): Rect4 => [e.x, e.y, e.width, e.height];
function intersect(a: Rect4, b: Rect4): Rect4 {
  const x = Math.max(a[0], b[0]), y = Math.max(a[1], b[1]);
  return [x, y, Math.min(a[0] + a[2], b[0] + b[2]) - x, Math.min(a[1] + a[3], b[1] + b[3]) - y];
}
const sameRect = (a: Rect4, b: Rect4) => a.every((v, i) => Math.abs(v - b[i]!) < 0.6);
/** `path` is `anc` or inside it. */
const within = (path: string, anc: string) => path === anc || path.startsWith(anc + "/");

/**
 * Each element's path: the accessibility tree's where it names one (by role
 * and bounds), else its parent's path and its id (the nearest earlier
 * element with the parent's id: ids repeat, as every toast's `Text` does).
 */
export function elementPaths(f: Pick<UiFrameSnapshot, "elements" | "a11y">): string[] {
  const out: string[] = [];
  for (const [i, e] of f.elements.entries()) {
    const node = e.role ? f.a11y.nodes.find((n) => n.role === e.role && sameRect(n.rect, rectOf(e)) && n.path.endsWith(e.id)) : undefined;
    if (node) {
      out.push(node.path);
      continue;
    }
    let parent = "";
    if (e.parent) for (let j = i - 1; j >= 0; j--) if (f.elements[j]!.id === e.parent) {
      parent = out[j]!;
      break;
    }
    out.push(parent ? `${parent}/${e.id}` : e.id);
  }
  return out;
}

/** The text without a trailing ellipsis, for matching a cut line to its element. */
const stem = (t: string) => t.trim().replace(/(…|\.\.\.)$/, "");

/**
 * The element that drew a line of text: among the elements with text whose
 * box touches the line's box, the one showing the same text (or the text
 * the line is a piece of: a wrapped line, an ellipsis), else the smallest.
 * An element's box is its visible part (a scroll container cuts it), so a
 * point of the line (its centre) may lie outside it while the line is its.
 */
function owner(f: UiFrameSnapshot, line: { text: string; bounds: Rect4 }): number {
  const [x, y, w, h] = line.bounds;
  const want = stem(line.text);
  let best = -1, bestScore = Infinity;
  for (const [i, e] of f.elements.entries()) {
    if (!e.text) continue;
    if (e.x > x + w + 0.5 || e.y > y + h + 0.5 || e.x + e.width < x - 0.5 || e.y + e.height < y - 0.5) continue;
    const same = e.text === line.text || (want !== "" && e.text.includes(want));
    // A matching text first, then the smallest box.
    const score = (same ? 0 : 1e12) + e.width * e.height;
    if (score < bestScore) {
      best = i;
      bestScore = score;
    }
  }
  return best;
}

/** og-ui's own neighbour rule (crates/og-ui/src/nav.rs `neighbour`): the control a pad direction lands on, wrapping. */
export function neighbour(from: Rect4, cands: Array<[number, Rect4]>, dir: "up" | "down" | "left" | "right"): number | null {
  const fx = from[0] + from[2] / 2, fy = from[1] + from[3] / 2;
  const gap = (a0: number, a1: number, b0: number, b1: number) => Math.max(b0 - a1, a0 - b1, 0);
  const axes = (r: Rect4): [number, number] => {
    const cx = r[0] + r[2] / 2, cy = r[1] + r[3] / 2;
    if (dir === "down") return [cy - fy, gap(from[0], from[0] + from[2], r[0], r[0] + r[2])];
    if (dir === "up") return [fy - cy, gap(from[0], from[0] + from[2], r[0], r[0] + r[2])];
    if (dir === "right") return [cx - fx, gap(from[1], from[1] + from[3], r[1], r[1] + r[3])];
    return [fx - cx, gap(from[1], from[1] + from[3], r[1], r[1] + r[3])];
  };
  let best: [number, number] | null = null;
  for (const [k, r] of cands) {
    const [along, across] = axes(r);
    if (along > 0.5) {
      const s = along + 3 * across;
      if (!best || s < best[0]) best = [s, k];
    }
  }
  if (best) return best[1];
  for (const [k, r] of cands) {
    const [along, across] = axes(r);
    if (along < -0.5) {
      const s = along + 3 * across;
      if (!best || s < best[0]) best = [s, k];
    }
  }
  return best ? best[1] : null;
}

/** The focusable controls each pad direction leads to, from each: the graph pads walk. */
export function padGraph(rects: Rect4[]): number[][] {
  return rects.map((r, i) => {
    const others = rects.map((o, j) => [j, o] as [number, Rect4]).filter(([j]) => j !== i);
    const out = new Set<number>();
    for (const d of ["up", "down", "left", "right"] as const) {
      const n = neighbour(r, others, d);
      if (n !== null) out.add(n);
    }
    return [...out];
  });
}

/**
 * Over a pad graph (each control's neighbours by direction) from the
 * default focus: the controls no path reaches, and those reached that no
 * path leads back from (a trap).
 */
export function padFindings(graph: number[][], start: number): Array<{ rule: "focus-reach" | "focus-trap"; index: number }> {
  const out: Array<{ rule: "focus-reach" | "focus-trap"; index: number }> = [];
  const seen = reachable(graph, start);
  for (let k = 0; k < graph.length; k++) {
    if (!seen.has(k)) out.push({ rule: "focus-reach", index: k });
    else if (!reachable(graph, k).has(start)) out.push({ rule: "focus-trap", index: k });
  }
  return out;
}

function reachable(graph: number[][], from: number): Set<number> {
  const seen = new Set([from]);
  const queue = [from];
  while (queue.length) for (const n of graph[queue.shift()!]!) if (!seen.has(n)) {
    seen.add(n);
    queue.push(n);
  }
  return seen;
}

/** Linear to sRGB, 0 to 1. */
const srgb = (v: number) => (v <= 0.0031308 ? v * 12.92 : 1.055 * v ** (1 / 2.4) - 0.055);
/** The same opaque colour (sRGB within about 2/255 a channel; a translucent fill is never the primary). */
const near = (a: number[], b: number[]) => (a[3] ?? 1) >= 0.95 && [0, 1, 2].reduce((n, k) => n + Math.abs(srgb(a[k]!) - srgb(b[k]!)), 0) < 0.025;

export interface CheckOptions {
  /** The rendered frame (device pixels), for contrast; without one contrast is not measured for this frame. */
  image?: Image | null;
  /** Rules to leave out (polish measures contrast and text size its own way). */
  skip?: readonly string[];
}

/** Every rule that one frame shows, for the cell it was taken in. */
export function checkFrame(f: UiFrameSnapshot, cell: UiCell, opts: CheckOptions = {}): UiFinding[] {
  const out: UiFinding[] = [];
  const skip = new Set(opts.skip ?? []);
  const where = cellName(cell);
  const add = (rule: string, x: Omit<UiFinding, "rule" | "severity" | "cells"> & { severity?: Severity }) => {
    if (skip.has(rule)) return;
    out.push({ rule, severity: x.severity ?? "error", paths: x.paths, measured: x.measured, limit: x.limit, prop: x.prop, fix: x.fix, cells: [where] });
  };
  const v = f.viewport;
  const paths = elementPaths(f);
  const byPath = new Map(paths.map((p, i) => [p, i]));
  const kindAt = (p: string) => f.elements[byPath.get(p) ?? -1]?.kind ?? "";
  const underTouch = (i: number) => {
    for (const [j, e] of f.elements.entries()) if (TOUCH_CONTROLS.has(e.kind ?? "") && within(paths[i]!, paths[j]!)) return true;
    return false;
  };
  const pseudo = cell.locale === "pseudo" ? " under the pseudo-locale" : "";

  // Overlap: the outermost pairs (a pair whose elements are both inside an
  // already listed pair's says nothing new).
  const pairs = [...f.overlaps].sort((a, b) => a.path.split("/").length + a.other.split("/").length - (b.path.split("/").length + b.other.split("/").length));
  const listed: Array<{ path: string; other: string }> = [];
  for (const o of pairs) {
    if (listed.some((l) => (within(o.path, l.path) && within(o.other, l.other)) || (within(o.path, l.other) && within(o.other, l.path)))) continue;
    listed.push(o);
    add("overlap", {
      paths: [o.path, o.other],
      measured: `${r1(o.rect[2])}×${r1(o.rect[3])} CSS px overlap at ${r1(o.rect[0])},${r1(o.rect[1])}${o.inFlow ? " (siblings in the flow)" : ""}${pseudo}`,
      limit: "no overlap unless declared",
      prop: o.inFlow ? `${o.path}: size, gap or wrap of its row` : `${o.path}: anchor, offset or insets (or \`overlaps: ["${o.other}"]\` in og.ui.props when it is meant)`,
      fix: o.inFlow
        ? `siblings in the flow never overlap: give their row room (wrap, a smaller step, grow) at ${v.breakpoint}`
        : `move ${o.path} off ${o.other} at ${v.breakpoint} (a breakpoint variant of its anchor or offset, or one group for both), or declare the overlap when it is the design`,
    });
  }

  // Text: cut by its clip, drawn past its box, truncated.
  const minSize = minTextSize(v);
  const denseSize = denseTextSize(v, cell.scale);
  const dense = f.dense ?? [];
  const isDense = (path: string) => dense.some((d) => path === d || path.startsWith(`${d}/`));
  for (const t of f.texts) {
    if (!/[\p{L}\p{N}]/u.test(t.text)) continue;
    const [, , w, h] = t.bounds;
    const i = owner(f, t);
    const path = i >= 0 ? paths[i]! : `text "${t.text.slice(0, 24)}"`;
    const e = i >= 0 ? f.elements[i]! : undefined;
    const shown = intersect(t.bounds, t.clip);
    const cutX = w - Math.max(0, shown[2]);
    const cutY = h - Math.max(0, shown[3]);
    // A scroll container's edge (the probe marks the element clipped) cuts a
    // row at its top or bottom: the rest scrolls into view. A cut across the
    // line is not scrolling in a list (a right-to-left line past its box's
    // start, the kit's gallery in Arabic), so it stays, as a warning: the
    // frame does not say whether that scroller runs sideways.
    if (e?.clipped && cutX <= 1) continue;
    // What the cut took, along the axis it cut: never "0 px" for a line cut top or bottom.
    const cut = [cutX > 1 ? `${r1(cutX)} CSS px of its width` : "", cutY > h * 0.3 ? `${r1(cutY)} of its ${r1(h)} CSS px height` : ""].filter(Boolean).join(" and ");
    const ellipsis = /(…|\.\.\.)$/.test(t.text.trim());
    if (cutX > 1 || cutY > h * 0.3) {
      add(cell.scale !== 100 ? "text-scale" : "text-cut", {
        ...(e?.clipped ? { severity: "warning" as const } : {}),
        paths: [path],
        measured: `"${t.text.slice(0, 32)}" cut by its clip: ${cut}${pseudo}`,
        limit: "text shows whole, or ends in an ellipsis with a tooltip",
        prop: `${path}: ellipsis and maxLines (og.ui.label), or its box's width`,
        fix: "let the box grow with its text, wrap it, or cut it with an ellipsis and give it a tooltip",
      });
    } else if (e && !PICTURES.has(e.kind ?? "") && w > e.width + 1.5 && (e.kind === "label" || e.kind === "button")) {
      add(cell.scale !== 100 ? "text-scale" : "text-cut", {
        paths: [path],
        measured: `"${t.text.slice(0, 32)}" is ${r1(w)} CSS px in a ${r1(e.width)} px box${pseudo}`,
        limit: "text inside its box",
        prop: `${path}: width (fit or grow), or ellipsis`,
        fix: "no box is sized to its English text (§15.3): let it fit its text, or wrap",
      });
    }
    if (ellipsis && e && !/(…|\.\.\.)$/.test(e.text.trim())) {
      add("text-truncated", {
        severity: "warning",
        paths: [path],
        measured: `"${e.text.slice(0, 40)}" drawn as "${t.text.slice(0, 40)}"${pseudo}`,
        limit: "truncated text has a tooltip with the whole of it",
        prop: `${path}: og.ui.tooltip after it, or its width`,
        fix: "give it a tooltip (og.ui.tooltip right after the control) or room for the whole text; a tooltip cannot be seen in the frame data yet, so check that one exists",
      });
    }
    const inDense = i >= 0 && isDense(path);
    if (t.size + 0.01 < (inDense ? denseSize : minSize)) {
      add("text-size", {
        paths: [path],
        measured: `${r1(t.size)} CSS px "${t.text.slice(0, 32)}"${inDense ? " (dense)" : ""}`,
        limit: inDense
          ? `${denseSize} CSS px at ${v.width}×${v.height} at ${cell.scale} % text (a dense HUD: 12 px at 1080p, growing with the text scale)`
          : `${minSize} CSS px at ${v.width}×${v.height}${v.coarsePointer ? " on touch" : " (18 px at 1080p)"}`,
        prop: `${path}: step (the type ramp's step, \`xs\` and \`meta\` are the smallest)`,
        fix: inDense
          ? "a dense readout may go to 12 px at 1080p, never below, and grows with UiDesc.textScale (§15.2)"
          : "use the ramp's body step or larger for anything read in play (§15.2); a dense HUD's readouts (strategy, MMO, simulation) may declare `dense: true` in og.ui.props and go to 12 px at 1080p",
      });
    }
  }

  // Laid out past the screen with nothing to scroll to it (og-ui's own warning).
  for (const w of f.warnings) {
    const m = /^'([^']+)' is laid out (\d+) px past the (\d+x\d+) screen/.exec(w);
    if (m) add("offscreen", {
      paths: [m[1]!],
      measured: `${m[2]} CSS px past the ${m[3]} screen${pseudo}`,
      limit: "every control on screen or in a scroll container",
      prop: `${m[1]}: its container (a scroll container), step or direction`,
      fix: "fit the screen (a smaller step, fewer rows, a row instead of a column) or put it in a scroll container",
    });
  }

  // Safe area and the shell's covered region: what shows something.
  const [sl, st, sr, sb] = v.safeArea;
  const safe: Rect4 = [sl, st, v.width - sl - sr, v.height - st - sb];
  const surface = v.width * v.height;
  const placed = new Set<string>();
  for (const [i, e] of f.elements.entries()) {
    const shows = !CONTAINERS.has(e.kind ?? "") || INTERACTIVE.has(e.role ?? "") || /[\p{L}\p{N}]/u.test(e.text);
    if (!shows || e.width * e.height <= 0 || e.width * e.height >= 0.9 * surface || e.clipped) continue;
    const r = rectOf(e);
    // One finding per placed group: the outermost element out of bounds.
    if ([...placed].some((p) => within(paths[i]!, p))) continue;
    const inside = intersect(r, safe);
    const outside = area(r) - Math.max(0, inside[2]) * Math.max(0, inside[3]);
    if (outside > 1) {
      placed.add(paths[i]!);
      add("safe-area", {
        paths: [paths[i]!],
        measured: `${r1(outside)} CSS px² outside the safe area (${safe.map(r1).join(", ")})`,
        limit: "inside the safe area",
        prop: `${paths[i]!}: its anchor or insets (anchored groups keep to the safe area by themselves)`,
        fix: "place it with og.ui.anchor (inside the safe area) or inset it by og.ui.frameViewport().safeArea",
      });
    }
    const c = v.covered;
    if (c[2] > 0 && c[3] > 0) {
      const under = intersect(r, c);
      if (under[2] > 0.5 && under[3] > 0.5) add("covered", {
        paths: [paths[i]!],
        measured: `${r1(under[2])}×${r1(under[3])} CSS px under the shell's control at ${c.map(r1).join(", ")}`,
        limit: "nothing under the host's control",
        prop: `${paths[i]!}: its placement (anchored groups move off the covered region; read og.ui.frameViewport().covered for a corner of your own)`,
        fix: "keep the top-left corner free of the player shell's Back",
      });
    }
  }

  // Touch targets.
  if (v.coarsePointer) for (const [i, e] of f.elements.entries()) {
    if (!INTERACTIVE.has(e.role ?? "") || e.disabled || e.clipped) continue;
    if (e.width + 0.5 >= 44 && e.height + 0.5 >= 44) continue;
    // A link inside a sentence (a rich text's span) is exempt (WCAG 2.5.8's inline exception).
    if (e.kind === "link" && kindAt(paths[i]!.slice(0, paths[i]!.lastIndexOf("/"))) === "richtext") continue;
    add("touch-target", {
      paths: [paths[i]!],
      measured: `${r1(e.width)}×${r1(e.height)} CSS px`,
      limit: "44×44 CSS px on a coarse pointer",
      prop: `${paths[i]!}: size (min 44) or hitSlop`,
      fix: "raise it to 44 CSS px (a control's `size`, a row's min height) or give it hit slop",
    });
  }

  // Contrast, on pixels (polish's measure).
  if (opts.image && !skip.has("contrast")) {
    const scale = opts.image.width / v.width;
    const scaled: UiDump = { tick: 0, elements: f.elements.map((e) => ({ ...e, x: e.x * scale, y: e.y * scale, width: e.width * scale, height: e.height * scale })) };
    const bar = cell.highContrast || v.highContrast || f.contrast === "high" ? 7 : 4.5;
    for (const b of textBoxes(opts.image, scaled)) {
      if (b.contrast >= bar) continue;
      const i = scaled.elements.findIndex((e) => e.id === b.id && e.text === b.text);
      const path = i >= 0 ? paths[i]! : b.id;
      add("contrast", {
        paths: [path],
        measured: `${b.contrast}:1 "${b.text.slice(0, 32)}"`,
        limit: `${bar}:1${bar === 7 ? " (high contrast)" : ""}`,
        prop: "the style's ink and ground tokens (palette), never per call",
        fix: "og-taste §3 (value before hue): darken the ground or lighten the ink in the style's palette",
      });
    }
  }

  // Focus: from the default focus, by pad, every control keys reach; and back.
  const order = f.a11y.focusOrder;
  const nodes = order.map((p) => f.a11y.nodes.find((n) => n.path === p)).filter((n): n is NonNullable<typeof n> => !!n && !n.states.includes("offscreen"));
  if (nodes.length > 0) {
    const start = Math.max(0, nodes.findIndex((n) => n.path === f.a11y.focused));
    for (const p of padFindings(padGraph(nodes.map((n) => n.rect)), start)) {
      const n = nodes[p.index]!, from = nodes[start]!.path;
      if (p.rule === "focus-reach") add("focus-reach", {
        paths: [n.path, from],
        measured: `no pad direction leads from ${from} to it`,
        limit: "every control reachable by d-pad from the default focus",
        prop: `${n.path}: its place in the layout (pads go to the nearest control in a direction; two controls in one place hide the second)`,
        fix: "line it up with its neighbours (a row or column the d-pad walks), or move the default focus (autofocus)",
      });
      else add("focus-trap", {
        paths: [n.path],
        measured: `from ${n.path} no pad path leads back to ${from}`,
        limit: "no focus trap",
        prop: `${n.path}: its place in the layout`,
        fix: "every control reaches every other by d-pad: line the controls up in rows and columns",
      });
    }
  }
  const reach = new Set(order);
  // A dialog takes the pad and the pointer: what is behind it is meant to be out of reach.
  const topModal = f.elements.map((e, i) => (e.kind === "modal" ? paths[i]! : null)).filter((p): p is string => !!p).at(-1);
  for (const n of f.a11y.nodes) {
    if (!INTERACTIVE.has(n.role) || n.states.includes("disabled") || n.states.includes("offscreen") || reach.has(n.path)) continue;
    if (topModal && !within(n.path, topModal)) continue;
    const i = byPath.get(n.path);
    if (i !== undefined && underTouch(i)) continue;
    if (TOUCH_CONTROLS.has(kindAt(n.path))) continue;
    add("focus-reach", {
      paths: [n.path],
      measured: "not focusable: keys and pads cannot reach it",
      limit: "every control a pointer can press, a pad can reach",
      prop: `${n.path}: focusable (an interactive container takes \`nav\`), or og.ui.setNavigation on this screen`,
      fix: "make it focusable, or give the action a button or glyph a pad has",
    });
  }

  // Names.
  for (const n of f.a11y.nodes) {
    if (!INTERACTIVE.has(n.role) || n.name.trim()) continue;
    add("a11y-name", {
      paths: [n.path],
      measured: `a ${n.role} with no accessible name`,
      limit: "every control named",
      prop: `${n.path}: its text, or its desc's \`label\` where the widget has one (an icon button: the name of what it does)`,
      fix: "name it in the player's language (an icon button's name is what it does: \"Pause\")",
    });
  }

  // Hierarchy: buttons filled with the primary colour, per layer (the base and each modal).
  const modals = f.elements.map((e, i) => (e.kind === "modal" ? paths[i]! : null)).filter((p): p is string => !!p);
  const layerOf = (p: string) => modals.filter((m) => within(p, m)).at(-1) ?? "base";
  const primaries = new Map<string, string[]>();
  for (const [i, e] of f.elements.entries()) {
    if (e.role !== "button" || e.disabled || underTouch(i) || TOUCH_CONTROLS.has(e.kind ?? "")) continue;
    const filled = f.fills.some((fl) => sameRect(fl.rect, rectOf(e)) && f.primary.some((p) => near(fl.color, p)));
    if (!filled) continue;
    const l = layerOf(paths[i]!);
    primaries.set(l, [...(primaries.get(l) ?? []), paths[i]!]);
  }
  for (const [layer, ps] of primaries) if (ps.length > 1) add("primary-actions", {
    paths: ps,
    measured: `${ps.length} primary buttons in ${layer === "base" ? "the screen" : layer}`,
    limit: "one primary action per screen or layer",
    prop: `variant of all but the one action the screen is for (\`secondary\` or \`quiet\`)`,
    fix: "one primary action per layer: the rest secondary",
  });

  // Feedback: sounds in a styled game; states on custom interactive containers.
  if (f.style !== "") for (const ev of ["hover", "press"]) if (!f.sounds[ev]) add("feedback", {
    paths: ["(style)"],
    measured: `the style ${f.style} has no ${ev} sound`,
    limit: "a styled game sounds its controls' hover and press",
    prop: `the style's \`sounds ${ev}\` (or og.ui.configure({ sounds: { ${ev} } }))`,
    fix: "og-taste §6 (feedback) and og-audio §3: give the style its hover and press sounds",
  });
  const stated = new Set(f.motion.filter((m) => m.states.length > 0).map((m) => m.path));
  for (const [i, e] of f.elements.entries()) {
    if (!INTERACTIVE.has(e.role ?? "") || FEEDBACK_KINDS.has(e.kind ?? "") || e.disabled) continue;
    if ([...stated].some((p) => within(paths[i]!, p))) continue;
    add("feedback", {
      severity: "warning",
      paths: [paths[i]!],
      measured: `a custom ${e.role} (${e.kind || "container"}) with no visual states`,
      limit: "hover and press states on every interactive element",
      prop: `${paths[i]!}: states (a \`press\` group: og.ui.props({ states }))`,
      fix: "give it hover and press states (§7.3), or build it from og.ui.button",
    });
  }

  // Counts.
  if (f.stats.elements > 300) add("budget", {
    paths: ["(frame)"],
    measured: `${f.stats.elements} elements, ${f.draws} draw commands`,
    limit: "300 elements (the dense-menu budget of §15.1)",
    prop: "the screen's structure: virtualise long lists (og.ui.list), fewer wrappers",
    fix: "fewer elements per tick: a virtual list draws only the rows in view",
  });
  return out;
}

/**
 * Names and texts left in the game's own language in another locale's
 * frame: a path whose name (or text) with letters is the same in both.
 */
export function checkLocales(base: UiFrameSnapshot, other: UiFrameSnapshot, cell: UiCell): UiFinding[] {
  if (cell.locale === "" || cell.locale === "pseudo") return [];
  const out: UiFinding[] = [];
  const named = new Map(base.a11y.nodes.map((n) => [n.path, n.name]));
  for (const n of other.a11y.nodes) {
    const was = named.get(n.path);
    // A picture's name is its own (an icon names itself); a name with no word is data.
    if (!was || was !== n.name || n.role === "image" || !/\p{L}{2,}/u.test(n.name)) continue;
    out.push({
      rule: "a11y-locale",
      severity: "warning",
      paths: [n.path],
      measured: `"${n.name.slice(0, 40)}" is the same in ${cell.locale} as in the game's own locale`,
      limit: "names and texts in the player's locale",
      prop: `${n.path}: its text through og.i18n.t (a key in every string table)`,
      fix: "og-i18n §2: every word the player reads comes from the string table",
      cells: [cellName(cell)],
    });
  }
  return out;
}

/**
 * The text-scale rule's comparison: each text the 100 % frame draws, found
 * again in the scaled frame (by its string, in drawing order), drawn at
 * the scale times its size. A text that stays smaller is a size the
 * player's setting does not reach (a fixed pixel size, a text drawn by the
 * game from the style's authored tokens).
 */
export function checkScale(base: UiFrameSnapshot, scaled: UiFrameSnapshot, cell: UiCell): UiFinding[] {
  if (cell.scale === 100) return [];
  const k = cell.scale / 100;
  const out: UiFinding[] = [];
  const left = new Map<string, number[]>();
  for (const t of scaled.texts) left.set(t.text, [...(left.get(t.text) ?? []), t.size]);
  const paths = scaled.elements.map((e) => e.id);
  const seen = new Set<string>();
  for (const t of base.texts) {
    if (!/[\p{L}\p{N}]/u.test(t.text)) continue;
    const sizes = left.get(t.text);
    const size = sizes?.shift();
    if (size === undefined || size + 0.05 >= t.size * k * 0.95 || seen.has(t.text)) continue;
    seen.add(t.text);
    const i = owner(scaled, scaled.texts.find((u) => u.text === t.text && u.size === size)!);
    const path = i >= 0 ? paths[i]! : `text "${t.text.slice(0, 24)}"`;
    out.push({
      rule: "text-scale",
      severity: "error",
      paths: [path],
      measured: `"${t.text.slice(0, 32)}" ${r1(size)} CSS px at ${cell.scale} %, ${r1(t.size)} at 100 %`,
      limit: `${r1(t.size * k)} CSS px: every text grows with UiDesc.textScale`,
      prop: `${path}: step (a type ramp step, which the scale reaches), not a fixed size`,
      fix: "size text from the ramp (og.ui's step) or multiply a size of your own by og.ui.theme().textScale (§15.2)",
      cells: [cellName(cell)],
    });
  }
  return out;
}

export function cellName(c: UiCell): string {
  return [c.screen, c.breakpoint, c.locale || "game locale", c.style || null, c.scale !== 100 ? `${c.scale} %` : null, c.highContrast ? "high contrast" : null].filter(Boolean).join(" ");
}

/** The same finding in several cells, once: by rule, paths and prop. */
export function mergeFindings(all: readonly UiFinding[]): UiFinding[] {
  const by = new Map<string, UiFinding>();
  for (const f of all) {
    const k = `${f.rule}\u0000${f.paths.join("\u0000")}`;
    const had = by.get(k);
    if (had) for (const c of f.cells) {
      if (!had.cells.includes(c)) had.cells.push(c);
    }
    else by.set(k, { ...f, cells: [...f.cells] });
  }
  return [...by.values()];
}

/** The report as text: one block per finding, then the rules not checkable yet. */
export function formatText(findings: readonly UiFinding[], header = ""): string {
  const L: string[] = [];
  if (header) L.push(header, "");
  const errors = findings.filter((f) => f.severity === "error").length;
  L.push(`${errors} error(s), ${findings.length - errors} warning(s)`, "");
  for (const f of findings) {
    L.push(`${f.severity === "error" ? "✗" : "!"} ${f.rule}: ${f.paths.join(" × ")}`);
    L.push(`    measured ${f.measured}; limit ${f.limit}`);
    L.push(`    change   ${f.prop}`);
    L.push(`    fix      ${f.fix}`);
    L.push(`    in       ${f.cells.slice(0, 6).join("; ")}${f.cells.length > 6 ? `; and ${f.cells.length - 6} more` : ""}`);
  }
  const later = RULES.filter((r) => r.status !== "checked");
  if (later.length) {
    L.push("", "Partial or not checkable yet:");
    for (const r of later) L.push(`  ${r.status === "partial" ? "~" : "-"} ${r.id} (${r.status}): ${r.note}`);
  }
  return L.join("\n") + "\n";
}

// ── for playtests ──────────────────────────────────────────────────────────

/** What a playtest's run offers (PlaytestRun's members this reads). */
interface RunFiles {
  readonly files: readonly string[];
  json<T = unknown>(file: string): T;
}

/**
 * The checker over a playtest's frame: the script took `dump_ui:<name>.uicheck.json`
 * (the run saved `<name>.uiframe.json` beside it, og run's native host) and,
 * for contrast, `screenshot:<name>.png` at the same time. Rules in `skip`
 * are left out. `dir` is the run's output directory (the playtest's own:
 * PlaytestRun has no path, so pass the out dir when contrast is wanted).
 */
export function checkUi(run: RunFiles, name: string, opts: { cell?: Partial<UiCell>; skip?: readonly string[]; dir?: string } = {}): UiFinding[] {
  const frame = run.json<UiFrameSnapshot>(`${name}.uiframe.json`);
  const png = opts.dir ? join(opts.dir, `${name}.png`) : "";
  const image = png && existsSync(png) ? decodePng(new Uint8Array(readFileSync(png))) : null;
  const cell: UiCell = { screen: name, breakpoint: frame.viewport.breakpoint, locale: "", style: frame.style, scale: 100, highContrast: frame.viewport.highContrast, ...opts.cell };
  return checkFrame(frame, cell, { image, skip: opts.skip ?? [] });
}

// ── pixels (shared with og polish) ─────────────────────────────────────────

/** Relative luminance (WCAG) of an sRGB byte triple. */
export function luminance(r: number, g: number, b: number): number {
  const c = (v: number) => {
    const s = v / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * c(r) + 0.7152 * c(g) + 0.0722 * c(b);
}

export const median = (xs: number[]) => {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.floor(s.length / 2)]!;
};
export const pct = (xs: number[], p: number) => {
  if (xs.length === 0) return 0;
  const s = [...xs].sort((a, b) => a - b);
  return s[Math.min(s.length - 1, Math.floor(p * s.length))]!;
};

export interface TextBox {
  id: string;
  text: string;
  contrast: number;
  /** The tallest run of ink rows, px: a line's ascenders to its descenders. */
  ink: number;
}

/**
 * Whether an element's text is read as text: not a picture (og-ui reports
 * an icon's name as its text), not cut by a scroll container (a sliver of a
 * line is no measure of it) and not a disabled control or inside one (WCAG
 * exempts inactive controls from contrast). The dump names each element's
 * `kind`, `clipped` and `disabled`.
 */
function readsAsText(e: UiElementDump, byId: Map<string, UiElementDump>): boolean {
  if (PICTURES.has(e.kind ?? "") || e.clipped) return false;
  for (let at: UiElementDump | undefined = e, n = 0; at && n < 64; at = at.parent ? byId.get(at.parent) : undefined, n++) {
    if (at.disabled) return false;
  }
  return true;
}

/**
 * What a dialog's or a tutorial's scrim covers: the elements built before the
 * topmost `modal` or `spotlight` layer and outside it, dimmed on purpose
 * (what shows through a spotlight's `hole` is not).
 */
function behindScrim(ui: UiDump): Set<number> {
  const out = new Set<number>();
  const els = ui.elements;
  let d = -1;
  for (let i = 0; i < els.length; i++) if (els[i]!.kind === "modal" || els[i]!.kind === "spotlight") d = i;
  if (d < 0) return out;
  const byId = new Map(els.map((e) => [e.id, e]));
  const inside = (e: UiElementDump) => {
    for (let at: UiElementDump | undefined = e, n = 0; at && n < 64; at = at.parent ? byId.get(at.parent) : undefined, n++) if (at === els[d]) return true;
    return false;
  };
  const holes = els.filter((e) => e.kind === "hole" && inside(e));
  for (let i = 0; i < d; i++) {
    const e = els[i]!;
    const cx = e.x + e.width / 2, cy = e.y + e.height / 2;
    if (!holes.some((h) => cx >= h.x && cx <= h.x + h.width && cy >= h.y && cy <= h.y + h.height)) out.add(i);
  }
  return out;
}

/**
 * The text elements of a frame: the contrast between ink and ground, and the
 * height of the ink's lines. Inside the element's rectangle (inset from its
 * rounded ends), the ground is the median luminance. The ink is measured
 * over the text's own glyphs, not the rectangle: the glyph pixels are those
 * at least half-way from the ground to the strongest ink (nearer the text's
 * colour than the ground's), and the ink is the mean of the farthest
 * twentieth of them, the glyph cores the eye reads (for a line of text
 * that fills its rectangle, what its 1 % farthest pixels read); so a short word in a wide label
 * (kestrel-stair FINDINGS #20: "You" on a 356 px row) reads what its
 * letters show, not its antialiased edges diluted by the empty rest. A
 * rectangle with no ink is not drawn (covered by a transition, faded out)
 * and is left out. `moving` (indices) are mid-transition and left out too.
 */
export function textBoxes(img: Image, ui: UiDump, moving: Set<number> = new Set()): TextBox[] {
  const out: TextBox[] = [];
  const byId = new Map(ui.elements.map((e) => [e.id, e]));
  const dimmed = behindScrim(ui);
  for (const [n, e] of ui.elements.entries()) {
    if (dimmed.has(n) || moving.has(n) || !/[\p{L}\p{N}]/u.test(e.text) || !readsAsText(e, byId) || e.width < 6 || e.height < 6) continue;
    // A pill's rounded ends show what is behind it: inset a wide element's ends.
    const ix = e.width > 3 * e.height ? Math.min(e.height / 2, 12) : 0, iy = e.height * 0.12;
    const x0 = Math.max(0, Math.floor(e.x + ix)), y0 = Math.max(0, Math.floor(e.y + iy));
    const x1 = Math.min(img.width, Math.ceil(e.x + e.width - ix)), y1 = Math.min(img.height, Math.ceil(e.y + e.height - iy));
    if (x1 - x0 < 3 || y1 - y0 < 3) continue;
    const w = x1 - x0;
    const lum: number[] = [];
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) {
      const i = (y * img.width + x) * 4;
      lum.push(luminance(img.rgba[i]!, img.rgba[i + 1]!, img.rgba[i + 2]!));
    }
    const ground = median(lum);
    const dev = lum.map((v) => Math.abs(v - ground));
    // The strongest ink (a few pixels short of the very farthest, so one
    // stray pixel does not set it), then the glyphs: every pixel at least
    // half-way to it, on the side most of them are.
    const peak = pct(dev, 0.998);
    if (peak < 0.01) continue;
    let up = 0, down = 0;
    for (let k = 0; k < lum.length; k++) {
      if (dev[k]! < peak / 2) continue;
      if (lum[k]! > ground) up++;
      else down++;
    }
    const light = up >= down;
    const glyphs: number[] = [];
    for (let k = 0; k < lum.length; k++) if (dev[k]! >= peak / 2 && (lum[k]! > ground) === light) glyphs.push(dev[k]!);
    // Their cores: the farthest twentieth (what the 1 % farthest of a
    // rectangle a line of text fills reads, the measure before glyphs).
    const core = pct(glyphs, 0.95);
    let sum = 0, cores = 0;
    for (let k = 0; k < lum.length; k++) {
      if (dev[k]! >= core && dev[k]! >= peak / 2 && (lum[k]! > ground) === light) {
        sum += lum[k]!;
        cores++;
      }
    }
    const ink = sum / Math.max(1, cores);
    // The ink's rows, as before for a text that fills its rectangle; a
    // sparse one's from its glyphs.
    const far = Math.max(pct(dev, 0.99), peak / 2);
    const contrast = (Math.max(ink, ground) + 0.05) / (Math.min(ink, ground) + 0.05);
    const rows: boolean[] = [];
    for (let y = 0; y < y1 - y0; y++) {
      let any = false;
      for (let x = 0; x < w && !any; x++) any = dev[y * w + x]! >= far / 4 && (lum[y * w + x]! > ground) === light;
      rows.push(any);
    }
    const runs: number[] = [];
    let run = 0;
    // A run of ink rows, bridging one-row gaps (thin strokes between a glyph's bars).
    for (let y = 0; y <= rows.length; y++) {
      if (rows[y] || (rows[y - 1] && rows[y + 1])) run++;
      else if (run > 0) {
        runs.push(run);
        run = 0;
      }
    }
    out.push({ id: e.id, text: e.text, contrast: r2(contrast), ink: Math.max(0, ...runs.filter((h) => h >= 2)) });
  }
  return out;
}

// ── the default screen set ─────────────────────────────────────────────────

/**
 * The screens a game's show action lists in its usage, as the probe's
 * `help` prints it (`show:<title|play|shop>`): each with the show template
 * (`show:{screen}`) filled in. None when the usage names no list.
 */
export function screensFromHelp(help: string, show: string): Array<{ name: string; commands: string }> {
  const action = show.split(":")[0]!;
  const line = help.split("\n").find((l) => l.startsWith(`${action}:<`));
  const values = /^[^:]+:<([^>]*)>/.exec(line ?? "")?.[1]?.split("|").map((v) => v.trim()).filter(Boolean) ?? [];
  return values.length > 1 ? values.map((v) => ({ name: v, commands: show.replaceAll("{screen}", v) })) : [];
}

/** Actions that only observe or end a run: not part of reaching a screen. */
const OBSERVE = /^(screenshot|dump_[a-z_]+|expect|quit|help|listen|wait)$/;

/**
 * The screens a playtest visits: one per screenshot, reached by the steps
 * before it that act (input, the game's commands; not dumps, checks or
 * other screenshots), captured at the screenshot's time.
 */
export function screensFromPlaytest(playtest: string, commands: string): Array<{ name: string; commands: string; at: number }> {
  const steps = commands.split(";").map((x) => x.trim()).filter(Boolean);
  const out: Array<{ name: string; commands: string; at: number }> = [];
  for (const [k, st] of steps.entries()) {
    const [t, act = "", arg = ""] = st.split(":");
    if (act !== "screenshot") continue;
    const shot = arg && !/^(full|\d+)$/.test(arg) ? arg.replace(/\.png$/, "") : `shot-${k}`;
    out.push({ name: `${playtest}-${shot}`, commands: steps.slice(0, k).filter((x) => !OBSERVE.test(x.split(":")[1] ?? "")).join(";"), at: Number(t) || 0 });
  }
  return out;
}

// ── the pseudo-locale ──────────────────────────────────────────────────────

const ACCENTS: Record<string, string> = {
  a: "á", e: "é", i: "í", o: "ö", u: "ü", c: "ç", n: "ñ", y: "ý", A: "Á", E: "É", I: "Í", O: "Ö", U: "Ü", C: "Ç", N: "Ñ", Y: "Ý",
};

/**
 * The pseudo-locale (§15.3): accents on the Latin letters, 40 % longer
 * (padded with the text's own letters), then in brackets.
 * Text without letters (a score, a time) is data and stays as it is.
 */
export function pseudo(text: string): string {
  if (!/\p{L}/u.test(text) || /^\[.*\]$/su.test(text)) return text;
  const accented = [...text].map((c) => ACCENTS[c] ?? c).join("");
  const letters = [...text].filter((c) => /\p{L}/u.test(c));
  const extra = Math.max(1, Math.round([...text].length * 0.4));
  let pad = "";
  for (let k = 0; k < extra; k++) pad += (ACCENTS[letters[k % letters.length]!] ?? letters[k % letters.length]!).toLowerCase();
  return `[${accented} ${pad}]`;
}
