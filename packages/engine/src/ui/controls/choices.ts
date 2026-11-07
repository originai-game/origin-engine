// Choices (docs/ui.md §2, Controls): the option cycler of a console
// settings screen and the stepping the choice controls share. The switch,
// the checkbox, the radio group and the segmented control are og-ui's
// native controls (natives.ts).

import type * as Api from "@origingame/engine";
import {
  all, bgValues, byText, chevron, control, drawn, fit, flags, forward, frame, glyphIcon, grow, kit, mix, over, paint, parentOf, keepPath, ownPath, px, role, rtl, spring, still, text, TOUCH, vstate, xy,
} from "./base.ts";
import type { Interaction } from "./base.ts";
import { edge, ring } from "../containers/panels.ts";

const v = (prop: Api.UiProp, value: number): Api.PropValue => ({ prop, value });

/** The ix flag for an inner part dressed by its row's state. */
const ixFlag = (ix: Interaction): string | false => ix !== "normal" && ix;

// ── choice stepping ─────────────────────────────────────────────────────

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

// ── cycler ──────────────────────────────────────────────────────────────

export function cycler(id: string, options: readonly string[], index: number, desc: Api.CyclerDesc = {}): Api.ChoiceIndexResult {
  const { t } = frame();
  const n = options.length;
  const wrap = desc.wrap ?? true;
  const disabled = desc.disabled ?? false;
  const cur = n > 0 ? Math.max(0, Math.min(n - 1, index)) : -1;
  let next = cur;
  let focused = -1;
  const h = Math.max(t.controlMd, frame().touch ? TOUCH : 0);
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
    height: { kind: "fit", min: h },
    padding: xy(t.sm, 0),
    gap: t.sm,
    fill: role("well"),
    washBy: 0.05,
    lift: 0,
    press: 1,
    adjustable: true,
    semantics: { role: "spinbutton", name: desc.label || id, value: cur >= 0 ? options[cur] ?? "" : "", valueMin: 0, valueMax: Math.max(0, n - 1), valueNow: cur },
  }, () => {
    // The label wraps (the row grows) rather than cutting: "Graphics quality" in pixel's wide face.
    if (desc.label) text("Label", desc.label, { step: "body", color: disabled ? "inkFaint" : "ink", width: grow() });
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
    // As wide as the longest option is estimated, and as the widest drawn so far: a face wider than the estimate
    // (pixel's) or a value longer than its option (the pseudo-locale's) widens it once, never cutting the value.
    const middle = `${kit.openPath()}/Middle`;
    const drawn = Math.max(kit.memory(`${id}~vw`, 0), Math.ceil(kit.bounds(middle)?.width ?? 0));
    if (drawn > 0) kit.remember(`${id}~vw`, drawn);
    kit.open("Middle", { direction: "column", width: desc.valueWidth ? px(desc.valueWidth) : { kind: "fit", min: Math.max(fitLongest(options), drawn) }, align: "center", cross: "center", gap: 3 });
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

/** A width that holds the longest option at the body step (so the arrows never move), as estimated from its letters. */
function fitLongest(options: readonly string[]): number {
  const size = frame().theme.typeRamp.body?.size ?? 16;
  // An em for a CJK character, a little over half one for others.
  const ems = (o: string): number => [...o].reduce((w, ch) => w + ((ch.codePointAt(0) ?? 0) >= 0x2e80 ? 1 : 0.6), 0);
  const longest = options.reduce((m, o) => Math.max(m, ems(o)), 1);
  return Math.ceil(longest * size) + 8;
}

