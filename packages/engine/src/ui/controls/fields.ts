// Text fields (docs/ui.md §2, Controls): the field and the area framed
// with a label, an icon, a suffix, a clear button and a helper, error or
// counter line. The editing is
// og.ui.textField's (IME, selection, clipboard, undo); the frame is this
// component's `Input.frame` part, dressed by focus, validity and hover.

import type * as Api from "@origingame/engine";
import {
  bgValues, byText, control, drawn, fieldChrome, flags, frame, glyphIcon, grow, kit, paint, px, role, text, TOUCH, validity, vstate, xy, defineTimelines,
} from "./base.ts";

/** A field's frame and the field in it, with an icon, a suffix and a clear button. */
function framed(
  id: string,
  value: string,
  d: { placeholder?: string; icon?: string; suffix?: string; clearable?: boolean; maxChars?: number; disabled?: boolean; invalid?: boolean; area?: boolean; rows?: number; part?: string; name?: string },
): Api.InputResult {
  defineTimelines();
  const { t } = frame();
  const disabled = d.disabled ?? false;
  const invalid = d.invalid ?? false;
  const focusedLast = kit.memory(`${id}~focus`, 0) > 0;
  const rest = role("well");
  const lit = role("panel");
  let r: Api.FieldResult | null = null;
  let cleared = false;
  const h = Math.max(t.controlMd, frame().touch ? TOUCH : 0);
  const vd = validity(invalid);
  kit.open(id, {
    direction: "row",
    width: grow(),
    height: d.area ? null : px(h),
    padding: xy(t.sm, d.area ? t.xs : 0),
    gap: t.xs,
    cross: d.area ? "start" : "center",
    radius: t.control,
    // Disabled: the well at half strength, so it reads as a value shown, not one to edit.
    background: paint(disabled ? { ...rest, a: rest.a * 0.5 } : focusedLast ? lit : rest),
    part: `${d.part ?? "Input.frame"}${flags(d.area && "area", focusedLast && "focused", invalid && "invalid", disabled && "disabled")}`,
    partClass: "field",
  });
  try {
    // Its edge: the focus ink while edited, danger when wrong, the line otherwise.
    const edge = invalid ? "danger" : focusedLast ? "focus" : disabled ? "line" : "lineStrong";
    kit.setDraw({ strokes: [{ width: focusedLast || invalid ? 2 : 1, align: "inside", paint: { kind: "solid", color: { token: edge } } }] });
    if (d.icon) glyphIcon("Icon", d.icon, byText(18), focusedLast ? role("ink") : role("inkMuted"));
    const desc = { placeholder: d.placeholder ?? "", width: grow(), maxChars: d.maxChars ?? 0, invalid, bare: true };
    if (disabled) {
      // Shown but inert: its text in muted ink where the field would be, out of the focus order, a textbox the tree calls disabled.
      text(d.area ? "Area" : "Field", value || (d.placeholder ?? ""), { step: "body", color: "inkMuted", width: grow(), nowrap: !d.area, ellipsis: !d.area });
      kit.semantics({ role: "textbox", ...(d.name ? { name: d.name } : {}), value, disabled: true });
    } else {
      r = (d.area
        ? kit.textArea("Area", value, { ...desc, rows: d.rows ?? 4 })
        : kit.textField("Field", value, desc)) as Api.FieldResult;
      if (d.name) kit.semantics({ name: d.name, ...(invalid ? { description: "invalid" } : {}) });
    }
    if (d.suffix) text("Suffix", d.suffix, { step: "body", color: "inkMuted", nowrap: true });
    if ((d.clearable ?? true) && value.length > 0 && !disabled) {
      const clear = frame().touch ? TOUCH : byText(28);
      const c = control("Clear", {
        part: "Input.clear",
        cls: "button",
        // A button under a finger is a stop keys and pads reach too; a fine pointer's named image is not.
        focusable: frame().touch,
        // A finger's whole target; a pointer's small round one.
        width: px(clear),
        height: px(clear),
        radius: clear / 2,
        fill: null,
        washBy: 0.14,
        lift: 0,
        press: 0.9,
        // The pointer's shortcut for what keys and pads do on the focused control (Left and Right; a
        // field's own editing): a button under a finger, a named image to the pad graph.
        semantics: { role: frame().touch ? "button" : "image", name: "Clear" },
      }, () => drawn("Glyph", "cross", byText(16), role("inkMuted")));
      cleared = c.r.clicked;
    }
  } finally {
    kit.close();
  }
  const focused = r?.response.focused ?? false;
  kit.props({
    states: [
      { name: "focus", states: [vstate("off", bgValues(rest)), vstate("on", bgValues(lit))], transitions: [] },
      vd.group,
    ],
    state: [{ group: "focus", state: focused ? "on" : "off" }, vd.value],
  });
  kit.remember(`${id}~focus`, focused ? 1 : 0);
  const out = cleared ? "" : r?.value ?? value;
  return {
    response: r?.response ?? ({} as Api.Response),
    value: out,
    changed: out !== value,
    submitted: r?.response.submitted ?? false,
    cleared,
  };
}

/** `12/20` or `12`. */
function counterText(value: string, max: number): string {
  const n = [...value].length;
  return max > 0 ? `${n}/${max}` : `${n}`;
}

export function inputField(id: string, value: string, desc: Api.InputFieldDesc = {}): Api.InputResult {
  let out: Api.InputResult | null = null;
  fieldChrome(id, {
    ...(desc.label ? { label: desc.label } : {}),
    ...(desc.helper ? { helper: desc.helper } : {}),
    ...(desc.error ? { error: desc.error } : {}),
    ...(desc.required ? { required: true } : {}),
    width: desc.width ?? grow(),
    ...(desc.counter ? { counter: counterText(value, desc.maxChars ?? 0) } : {}),
  }, () => {
    out = framed("Frame", value, {
      ...(desc.placeholder ? { placeholder: desc.placeholder } : {}),
      ...(desc.icon ? { icon: desc.icon } : {}),
      ...(desc.suffix ? { suffix: desc.suffix } : {}),
      clearable: desc.clearable ?? true,
      maxChars: desc.maxChars ?? 0,
      disabled: desc.disabled ?? false,
      invalid: !!desc.error,
      ...(desc.label ? { name: desc.label } : {}),
    });
  });
  return out ?? { response: {} as Api.Response, value, changed: false, submitted: false, cleared: false };
}

export function inputArea(id: string, value: string, desc: Api.InputAreaDesc = {}): Api.InputResult {
  let out: Api.InputResult | null = null;
  fieldChrome(id, {
    ...(desc.label ? { label: desc.label } : {}),
    ...(desc.helper ? { helper: desc.helper } : {}),
    ...(desc.error ? { error: desc.error } : {}),
    width: desc.width ?? grow(),
    ...(desc.counter ? { counter: counterText(value, desc.maxChars ?? 0) } : {}),
  }, () => {
    out = framed("Frame", value, {
      ...(desc.placeholder ? { placeholder: desc.placeholder } : {}),
      clearable: false,
      maxChars: desc.maxChars ?? 0,
      disabled: desc.disabled ?? false,
      invalid: !!desc.error,
      area: true,
      rows: desc.rows ?? 4,
      ...(desc.label ? { name: desc.label } : {}),
    });
  });
  return out ?? { response: {} as Api.Response, value, changed: false, submitted: false, cleared: false };
}
