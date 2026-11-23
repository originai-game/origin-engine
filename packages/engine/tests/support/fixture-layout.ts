// The layout facts every built-in fixture is held to (docs/ui.md §6.2),
// read from one laid-out frame (fixture-frames.ts). Each is a fact with
// the element's path, never a taste: a control a finger cannot take, a
// control drawn off the surface with nothing to scroll it in, a line of
// text cut by its box or the surface.

import type { FrameElement, Host } from "./fixture-frames.ts";

/** Roles a player presses. */
const PRESSED = new Set(["button", "link", "checkbox", "radio", "switch", "slider", "textbox", "combobox", "option", "tab", "spinbutton", "menuitem", "menuitemcheckbox"]);
/** The touch target a finger needs (Tree::near_target grows a smaller control to it), and the least a control may draw (WCAG 2.5.8). */
const TARGET = 44, LEAST = 24;

type Rect = [number, number, number, number];
const meets = (a: Rect, b: Rect) => Math.min(a[0] + a[2], b[0] + b[2]) - Math.max(a[0], b[0]) > 0.5 && Math.min(a[1] + a[3], b[1] + b[3]) - Math.max(a[1], b[1]) > 0.5;

/** The drawn controls of a frame: enabled, pressed by a player, not inside another one, and what of them their ancestors leave on screen. */
function controls(els: FrameElement[]): Array<{ e: FrameElement; shown: Rect }> {
  const byId = new Map(els.map((e) => [e.id, e]));
  const inControl = (e: FrameElement) => { for (let a = byId.get(e.parent); a; a = byId.get(a.parent)) if (PRESSED.has(a.role)) return true; return false; };
  return els.flatMap((e) => {
    if (!PRESSED.has(e.role) || e.disabled || e.width <= 0 || e.height <= 0 || inControl(e)) return [];
    const c = e.clip ?? [-1e9, -1e9, 2e9, 2e9];
    const x0 = Math.max(e.x, c[0]!), y0 = Math.max(e.y, c[1]!), x1 = Math.min(e.x + e.width, c[0]! + c[2]!), y1 = Math.min(e.y + e.height, c[1]! + c[3]!);
    return x1 - x0 > 0.5 && y1 - y0 > 0.5 ? [{ e, shown: [x0, y0, x1 - x0, y1 - y0] as Rect }] : [];
  });
}

const name = (e: FrameElement) => `${e.role} "${(e.name || e.text).slice(0, 28)}"`;

/** A finding: `key` names it stably (the rule and the element), `detail` says what was measured. */
export interface Finding { key: string; detail: string }
const box = (r: Rect) => r.map((v) => Math.round(v)).join(",");

/**
 * On a touch screen, each control draws at least 24 px each way and, where
 * smaller than 44, its box grown to 44 round its centre meets no other
 * control: the press the tree gives it is never another control's.
 */
export function touchTargets(els: FrameElement[], host: Host): Finding[] {
  if (!host.coarse) return [];
  const list = controls(els);
  const out: Finding[] = [];
  for (const { e, shown } of list) {
    if (Math.min(shown[2], shown[3]) < LEAST - 0.5) { out.push({ key: `touch ${name(e)}`, detail: `draws ${Math.round(shown[2])}x${Math.round(shown[3])}, under ${LEAST} px` }); continue; }
    if (shown[2] >= TARGET - 0.5 && shown[3] >= TARGET - 0.5) continue;
    const gx = Math.max(0, (TARGET - shown[2]) / 2), gy = Math.max(0, (TARGET - shown[3]) / 2);
    const grown: Rect = [shown[0] - gx, shown[1] - gy, shown[2] + 2 * gx, shown[3] + 2 * gy];
    const other = list.find((o) => o.e !== e && meets(grown, o.shown));
    if (other) out.push({ key: `touch ${name(e)}`, detail: `draws ${Math.round(shown[2])}x${Math.round(shown[3])} and its ${TARGET} px target meets ${name(other.e)}` });
  }
  return out;
}

/** A control drawn past the surface where no ancestor cuts it: nothing scrolls it into reach. */
export function offSurface(els: FrameElement[], host: Host): Finding[] {
  return controls(els).flatMap(({ e, shown }) =>
    shown[0] < -0.5 || shown[1] < -0.5 || shown[0] + shown[2] > host.width + 0.5 || shown[1] + shown[3] > host.height + 0.5
      ? [{ key: `off the surface ${name(e)}`, detail: `at ${box(shown)} on ${host.width}x${host.height}` }]
      : []);
}

/**
 * A line of text across the edge of what its ancestors leave on screen, or
 * of the surface: part of it shows and the rest is cut. A line wholly
 * outside (a marquee's or a strip's letters scrolled away) is hidden, not
 * cut; an ellipsis draws inside the box.
 */
export function cutText(els: FrameElement[], host: Host): Finding[] {
  const out: Finding[] = [];
  for (const e of els) {
    const c = e.clip ?? [0, 0, host.width, host.height];
    const x0 = Math.max(0, c[0]!), x1 = Math.min(host.width, c[0]! + c[2]!);
    const line = (e.textLines ?? []).find((l) => l[2]! > 0 && l[3]! > 0 && l[0]! + l[2]! > x0 && l[0]! < x1 && (l[0]! < x0 - 1 || l[0]! + l[2]! > x1 + 1));
    if (line) out.push({ key: `cut text "${(e.text ?? "").slice(0, 32)}"`, detail: `drawn ${Math.round(line[0]!)}..${Math.round(line[0]! + line[2]!)}, shown ${Math.round(x0)}..${Math.round(x1)}` });
  }
  return out;
}

/**
 * A surface's frame that does not show: a tint-mask frame with no fill
 * draws nothing, and one inside another framed surface in the very same
 * fill is lost in it (with no border round it, nothing parts them).
 */
export function lostFrames(els: FrameElement[]): Finding[] {
  const byId = new Map(els.map((e) => [e.id, e]));
  const out: Finding[] = [];
  const what = (e: FrameElement) => `${e.role || "view"} ${Math.round(e.width)}x${Math.round(e.height)}${e.text ? ` "${e.text.slice(0, 20)}"` : ""}`;
  for (const e of els) {
    const f = e.frame;
    if (!f || e.width <= 0 || e.height <= 0) continue;
    if (f[3]! <= 0.001) { out.push({ key: `frame with no fill ${what(e)}`, detail: "a tint-mask frame drawn in a transparent fill draws nothing" }); continue; }
    let a = byId.get(e.parent);
    while (a && !a.frame) a = byId.get(a.parent);
    if (a && a.frame!.every((v, i) => Math.abs(v - f[i]!) < (i < 3 ? 1.5 : 0.01))) {
      out.push({ key: `frame lost in its parent's ${what(e)}`, detail: `drawn in ${f.join(",")}, the fill of the framed ${what(a)} round it` });
    }
  }
  return out;
}
