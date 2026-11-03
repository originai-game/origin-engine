// Buttons (docs/ui.md §11, Controls): every variant with icons, a badge, a
// loading state and a disabled reason; the icon button; hold-to-confirm
// with its toggle alternative (XAG 107). Each is a `Button.base` part a
// style dresses by variant and state (a version 1 style's button art too),
// lifting under the pointer and the focus and sinking as it is pressed.

import type * as Api from "@origingame/engine";
import { cornerBadge } from "../foundations/media.ts";
import {
  byText, clamp, contrast, control, defineTimelines, frame, glyphCap, glyphIcon, kit, mix, over, paint, keepPath, ownPath, pct, px, role, rtl, still, text, TOUCH, xy,
} from "./base.ts";
import type { Rgba } from "./base.ts";

/**
 * A fill the ink reads on at 4.5:1: moved away from the ink (towards black or white), as little as it
 * takes, so a style whose hover or press colour is close to its label's (a dark primary hover under a
 * dark label) still shows the label in every state. A translucent fill is left to what is under it.
 */
export function readableFill(fill: Rgba, ink: Rgba): Rgba {
  if (fill.a < 0.99 || contrast(ink, fill) >= 4.5) return fill;
  const lighter = contrast(ink, { r: 0, g: 0, b: 0, a: 1 }) >= contrast(ink, { r: 1, g: 1, b: 1, a: 1 });
  const away: Rgba = lighter ? { r: 0, g: 0, b: 0, a: 1 } : { r: 1, g: 1, b: 1, a: 1 };
  for (let n = 1; n <= 20; n++) {
    const c = mix(fill, away, n * 0.05);
    if (contrast(ink, c) >= 4.5) return c;
  }
  return away;
}

/** A variant's fills at rest, hovered and pressed, and its ink: the ink reads at 4.5:1 on each. */
export function variantColors(v: Api.ButtonVariant | undefined, disabled = false): { fills: [Rgba, Rgba, Rgba]; ink: Rgba; line: boolean } {
  const c0 = variantRaw(v, disabled);
  if (disabled) return c0;
  const [a, b, c] = c0.fills;
  return { ...c0, fills: [readableFill(a, c0.ink), readableFill(b, c0.ink), readableFill(c, c0.ink)] };
}

function variantRaw(v: Api.ButtonVariant | undefined, disabled: boolean): { fills: [Rgba, Rgba, Rgba]; ink: Rgba; line: boolean } {
  const c = (n: Api.ThemeColor) => role(n);
  if (disabled) {
    const f = c("control");
    return { fills: [f, f, f], ink: c("inkFaint"), line: v === "ghost" };
  }
  const clear = (x: Rgba): Rgba => ({ ...x, a: 0 });
  switch (v ?? "secondary") {
    case "primary": return { fills: [c("primary"), c("primaryHover"), c("primaryActive")], ink: c("onPrimary"), line: false };
    case "danger": {
      const d = c("danger");
      const ground = c("panel");
      return { fills: [mix(ground, d, 0.14), mix(ground, d, 0.22), mix(ground, d, 0.3)], ink: d, line: false };
    }
    case "ghost": return { fills: [clear(c("control")), c("control"), c("controlStrong")], ink: c("ink"), line: true };
    case "quiet": return { fills: [clear(c("control")), c("control"), c("controlStrong")], ink: c("ink"), line: false };
    default: return { fills: [c("control"), c("controlStrong"), mix(c("controlStrong"), c("ink"), 0.08)], ink: c("ink"), line: false };
  }
}

/** A size's height (at least 44 on a coarse pointer for medium and large) and its label's type step. */
export function buttonMetrics(size: Api.ControlSize | undefined): { h: number; step: Api.TypeStepName; pad: number; icon: number } {
  const { t, touch } = frame();
  switch (size ?? "medium") {
    case "small": return { h: t.controlSm, step: "sm", pad: t.md, icon: 16 };
    case "large": return { h: Math.max(t.controlLg, touch ? TOUCH : 0), step: "base", pad: t.xl, icon: 22 };
    default: return { h: Math.max(t.controlMd, touch ? TOUCH : 0), step: "body", pad: t.lg, icon: 18 };
  }
}

/** A button's interactive part: a `Button.base` control of a variant, its fill and lift on the style's motion. */
function buttonBase(
  id: string,
  d: { variant?: Api.ButtonVariant; size?: Api.ControlSize; disabled?: boolean; hoverable?: boolean; selected?: boolean; width?: Api.Size | null; square?: boolean; round?: boolean; flags?: string[]; part?: string; semantics?: Api.SemanticsDesc },
  build: (ink: Rgba, m: ReturnType<typeof buttonMetrics>) => void,
) {
  const { t } = frame();
  const m = buttonMetrics(d.size);
  const disabled = d.disabled ?? false;
  const col = variantColors(d.variant, disabled);
  const selected = d.selected ?? false;
  // At least the control's height; taller only as far as what it holds needs at a large text size (the label's
  // line, or a square's icon, with room round it), so a 200 % button grows with its text rather than doubling.
  const content = d.square ? byText(m.icon + 2) + 2 * t.md : (frame().theme.typeRamp[m.step]?.lineHeight ?? 22) + 2 * t.sm;
  const h = Math.max(m.h, Math.ceil(content));
  const fills: [Rgba, Rgba, Rgba] = selected ? [col.fills[2], col.fills[2], col.fills[2]] : col.fills;
  return control(id, {
    part: d.part ?? "Button.base",
    cls: "button",
    flags: [d.variant ?? "secondary", ...(d.flags ?? []), d.size === "small" && "small", d.size === "large" && "large", d.square && "square", selected && "selected", d.round && "round"],
    disabled,
    ...(d.hoverable ? { hoverable: true } : {}),
    // The states move between the variant's own colours, not a wash.
    fill: fills[0],
    width: d.square ? px(h) : d.width ?? null,
    height: px(h),
    padding: d.square ? xy(0, 0) : xy(byText(m.pad), 0),
    gap: t.sm,
    align: "center",
    radius: d.round ? h / 2 : t.control,
    dim: 1,
    fills,
    semantics: { role: "button", ...(d.selected !== undefined ? { selected } : {}), ...(d.semantics ?? {}) },
  }, () => build(col.ink, m));
}

export function pushButton(id: string, label: string, desc: Api.PushButtonDesc = {}): Api.Response {
  const loading = desc.loading ?? false;
  const disabled = (desc.disabled ?? false) || loading;
  const reason = desc.disabled ? desc.disabledReason ?? "" : "";
  const built = buttonBase(id, {
    ...(desc.variant ? { variant: desc.variant } : {}),
    ...(desc.size ? { size: desc.size } : {}),
    disabled,
    hoverable: reason.length > 0,
    ...(desc.selected !== undefined && desc.selected ? { selected: true } : {}),
    width: desc.width ?? null,
    flags: loading ? ["loading"] : [],
    semantics: { name: label, busy: loading, ...(reason ? { description: reason } : {}) },
  }, (ink, m) => {
    const size = byText(m.icon);
    if (loading) kit.spinner("Spinner", size);
    else if (desc.icon) glyphIcon("Icon", desc.icon, size, ink);
    text("Label", label, { step: m.step, color: ink, nowrap: true, ellipsis: true, align: "center" });
    if (desc.iconEnd) glyphIcon("IconEnd", desc.iconEnd, size, ink);
    if (desc.badge) cornerBadge("Badge", desc.badge, false, size);
  });
  const tip = reason || desc.tooltip || "";
  if (tip) kit.tooltip(tip);
  autofocus(id, built.path, desc.autofocus ?? false);
  if (built.r.clicked && desc.variant === "primary") kit.feedback("confirm", -1);
  return built.r;
}

/** Take the focus the first tick it appears (what opens takes the focus), once. */
function autofocus(id: string, path: string, on: boolean): void {
  if (!on) return;
  const seen = kit.memory(`${id}~seen`, 0);
  if (seen === 0 && path) kit.setFocus(path);
  kit.remember(`${id}~seen`, 1);
}

export function iconButton(id: string, icon: string, desc: Api.IconButtonDesc = {}): Api.Response {
  const label = desc.label ?? "";
  const built = buttonBase(id, {
    variant: desc.variant ?? "quiet",
    ...(desc.size ? { size: desc.size } : {}),
    disabled: desc.disabled ?? false,
    ...(desc.selected ? { selected: true } : {}),
    square: true,
    round: desc.round ?? false,
    flags: ["icon"],
    semantics: { name: label || icon },
  }, (ink, m) => {
    glyphIcon("Icon", icon, byText(m.icon + 2), desc.selected && (desc.variant ?? "quiet") === "quiet" ? role("primary") : ink);
    cornerBadge("Badge", desc.badge ?? "", desc.dot ?? false, m.icon);
  });
  if (label) kit.tooltip(label);
  return built.r;
}

/** A width that holds the longest of `labels` at a type step: an em a CJK character, 0.6 of one others (generous for proportional faces). */
function labelWidth(step: Api.TypeStepName, labels: readonly string[]): number {
  const size = frame().theme.typeRamp[step]?.size ?? 16;
  const ems = (o: string): number => [...o].reduce((w, ch) => w + ((ch.codePointAt(0) ?? 0) >= 0x2e80 ? 1 : 0.6), 0);
  return Math.ceil(Math.max(...labels.map(ems)) * size);
}

/** A pad family's glyph (a face button disc) for the confirm action, when the device used last is a pad. */
function padConfirm(): { text: string; icon: string } | null {
  const g = kit.frameGlyph("confirm") as { text: string; icon: string } | null;
  return g && /^(xbox|ps|switch|pad)\./.test(g.icon) ? g : null;
}

export function holdButton(id: string, label: string, desc: Api.HoldButtonDesc = {}): Api.HoldResult {
  defineTimelines();
  const { t } = frame();
  const holdMs = Math.max(100, desc.holdMs ?? 1200);
  const disabled = desc.disabled ?? false;
  // The hold's state is UI memory, so a snapshot carries it: how full, armed (the toggle alternative), and whether
  // the press last tick was the pointer's (its release's click is not a key's activation).
  let p = kit.memory(`${id}~p`, 0);
  let armed = kit.memory(`${id}~arm`, 0) > 0;
  const pointer = kit.memory(`${id}~ptr`, 0) > 0;
  const last = kit.memory(`${id}~t`, -1);
  const now = kit.seconds();
  const dt = last < 0 ? 0 : clamp(now - last, 0, 0.1);
  const variant = desc.variant ?? "danger";
  const col = variantColors(variant, disabled);
  let holding = false;
  let confirmed = false;
  const wasHolding = kit.memory(`${id}~h`, 0) > 0;
  // The button's box last tick: the inverse label in the fill is laid out across all of it.
  const prev = kit.bounds(ownPath(`Hold~${id}`));
  kit.open(id, { direction: "column", width: desc.width ?? null, gap: t.xs, cross: "center" });
  let built: ReturnType<typeof buttonBase>;
  try {
    built = buttonBase("Button", {
      variant,
      ...(desc.size ? { size: desc.size } : {}),
      disabled,
      width: desc.width ? pct(1) : null,
      part: "HoldButton.base",
      flags: p > 0.001 ? ["holding"] : [],
      semantics: { name: label, value: `${Math.round(p * 100)}%`, valueMin: 0, valueMax: 100, valueNow: Math.round(p * 100) },
    }, (ink, m) => {
      const g = padConfirm();
      // Only while it is held does it say so; draining after a release, it shows its own label.
      const showing = wasHolding && desc.holdingLabel ? desc.holdingLabel : label;
      const content = (color: Rgba): void => {
        if (g && !disabled) glyphCap("Glyph", g, Math.round(m.h * 0.5));
        else if (desc.icon) glyphIcon("Icon", desc.icon, byText(m.icon), color);
        // As wide as the longer of the two labels, so the button keeps its width as it says Keep holding.
        text("Label", showing, { step: m.step, color, nowrap: true, ellipsis: true, align: "center", ...(desc.holdingLabel ? { width: { kind: "fit", min: labelWidth(m.step, [label, desc.holdingLabel]) } } : {}) });
      };
      content(ink);
      // The fill: the label's ink, solid, from the start edge (the writing direction's), over the label;
      // in it the same label in the button's own colour, cut by the fill's edge (an inverse clip), so
      // the label reads at the variant's contrast on the track and on the fill alike.
      if (p > 0.001 && prev && !disabled) {
        kit.open("Fill", { direction: "row", width: pct(clamp(p, 0, 1)), height: { kind: "grow" }, background: paint({ ...ink, a: 1 }), part: "HoldButton.fill", partClass: "bar" });
        // Rounded with the button at its start; its leading edge square until it reaches the end.
        const rr = t.control;
        const end = p > 0.97 ? rr : 0;
        kit.setDraw({ shape: { kind: "rect", radii: rtl() ? [end, rr, rr, end] : [rr, end, end, rr] }, clip: "shape" });
        kit.open("Inverse", { direction: "row", width: px(prev.width), height: px(prev.height), padding: xy(byText(m.pad), 0), gap: t.sm, align: "center", cross: "center" });
        content({ ...col.fills[0], a: 1 });
        kit.close();
        kit.close();
        over({ insets: { start: 0, top: 0, bottom: 0 }, inert: true, z: 1 });
      }
    });
    if (desc.hint) text("Hint", desc.hint, { step: "caption", color: "inkMuted", nowrap: true, align: "center" });
  } finally {
    kit.close();
  }
  keepPath(`Hold~${id}`, built.path);
  const r = built.r;
  if (!disabled) {
    const toggleMode = desc.toggle ?? false;
    // A click the pointer did not start is a key's or a pad's activation: the toggle alternative.
    if (r.clicked && (!pointer || toggleMode)) {
      armed = !armed;
      if (!armed) kit.feedback("cancel", -1);
    }
    if (armed && !r.focused && !pointer && !toggleMode && !r.hovered) armed = false;
    holding = armed || (!toggleMode && r.pressed);
  } else armed = false;
  const before = p;
  if (holding) p = clamp(p + (dt * 1000) / holdMs, 0, 1);
  else p = clamp(p - (dt * 1000) / (holdMs * 0.35), 0, 1);
  // A tick every fifth of the way, rising in pitch.
  if (holding && Math.floor(p * 5) > Math.floor(before * 5) && p < 1) kit.feedback("sliderTick", p);
  if (p >= 1 && holding) {
    confirmed = true;
    p = 0;
    armed = false;
    kit.feedback("confirm", -1);
    if (!still()) kit.playTimeline(built.path, "og.controls.bump", {});
  }
  kit.remember(`${id}~p`, p);
  kit.remember(`${id}~h`, holding ? 1 : 0);
  kit.remember(`${id}~arm`, armed ? 1 : 0);
  kit.remember(`${id}~ptr`, r.pressed ? 1 : 0);
  kit.remember(`${id}~t`, now);
  return { response: r, confirmed, holding, progress: p };
}

