// Text fields (docs/ui.md §11, Controls): the field and the area framed
// with a label, an icon, a suffix, a clear button and a helper, error or
// counter line; the search box with its suggestions. The editing is
// og.ui.textField's (IME, selection, clipboard, undo); the frame is this
// component's `Input.frame` part, dressed by focus, validity and hover.

import type * as Api from "@origingame/engine";
import {
  bgValues, byText, control, drawn, fieldChrome, flags, frame, glyphIcon, grow, kit, paint, parentOf, pct, px, role, still, text, TOUCH, validity, vstate, xy, defineTimelines,
} from "./base.ts";
import { elevation, overProps } from "../containers/base.ts";

/** Accents and case dropped: "Épée" matches "epee". */
export function fold(s: string): string {
  return s.normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
}

/** Suggestions for a query: those containing it, those starting with it first, then by length. */
export function filterSuggestions(all: readonly string[], query: string, max: number): string[] {
  const q = fold(query.trim());
  if (!q) return [];
  const hits = all.map((s) => ({ s, f: fold(s) })).filter((x) => x.f.includes(q));
  hits.sort((a, b) => Number(!a.f.startsWith(q)) - Number(!b.f.startsWith(q)) || a.s.length - b.s.length);
  return hits.slice(0, max).map((x) => x.s);
}

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
  const h = byText(Math.max(t.controlMd, frame().touch ? TOUCH : 0));
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

/** A suggestion's text with the query's match bold (markup for og.ui.richText). */
function boldMatch(s: string, query: string): string {
  const esc = (x: string) => x.replace(/\[/g, "[[");
  const f = fold(s);
  const q = fold(query.trim());
  const at = q ? f.indexOf(q) : -1;
  // NFD folding keeps one character per base character for Latin text, so the offsets line up.
  if (at < 0 || f.length !== s.length) return esc(s);
  return `${esc(s.slice(0, at))}[b]${esc(s.slice(at, at + q.length))}[/b]${esc(s.slice(at + q.length))}`;
}

export function searchBox(id: string, query: string, desc: Api.SearchBoxDesc = {}): Api.SearchResult {
  const { t } = frame();
  const max = desc.maxSuggestions ?? 6;
  const list = desc.filter ? filterSuggestions(desc.suggestions ?? [], query, max) : (desc.suggestions ?? []).slice(0, max);
  const showing = query.trim() ? list : (desc.recent ?? []).slice(0, max);
  // Open while the field or a suggestion had the focus last tick (UI memory), with something to show.
  const wasFocused = kit.memory(`${id}~open`, 0) > 0;
  // Back closes the list and leaves the field focused; typing or a press on the field opens it again.
  const shut = kit.memory(`${id}~shut`, 0) > 0;
  let closing = false;
  let chosen: string | null = null;
  let focusedAny = false;
  let res: Api.InputResult | null = null;
  let listed = false;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: t.xxs, cross: "stretch" });
  try {
    res = framed("Frame", query, {
      placeholder: desc.placeholder || "Search",
      icon: "search",
      clearable: true,
      disabled: desc.disabled ?? false,
      part: "Search.frame",
      name: desc.label || desc.placeholder || "Search",
    });
    // Down from the field reaches the list by spatial navigation (a one-line field keeps no Up or Down).
    focusedAny = res.response.focused;
    if (wasFocused && !shut && showing.length > 0 && !(desc.disabled ?? false)) {
      closing = kit.back();
      listed = true;
      kit.open("List", {
        direction: "column",
        width: pct(1),
        padding: xy(0, t.xxs),
        radius: t.control,
        background: paint(role("panelRaised")),
        part: "Search.list",
        partClass: "tooltip",
      });
      kit.setDraw({ strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: { token: "line" } } }], effects: elevation("floating") });
      if (!query.trim()) text("Recent", "Recent", { step: "caption", color: "inkMuted", nowrap: true });
      showing.forEach((s, i) => {
        const o = control(kit.idi("Option", i), {
          part: "Search.option",
          cls: "button",
          width: grow(),
          height: px(byText(Math.max(36, frame().touch ? TOUCH : 0))),
          padding: xy(t.md, 0),
          gap: t.sm,
          align: "start",
          fill: null,
          washBy: 0.08,
          lift: 0,
          press: 0.99,
          radius: 0,
          semantics: { role: "option", name: s },
        }, () => {
          drawn("Lead", query.trim() ? "search" : "reset", byText(14), role("inkMuted"));
          kit.richText("Text", boldMatch(s, query), { step: "body", width: grow(), nowrap: true });
        });
        if (o.r.focused) focusedAny = true;
        if (o.r.clicked) chosen = s;
      });
      kit.close();
      kit.semantics({ role: "listbox", name: desc.label || "Suggestions" });
      // Over what follows the field (never pushing it down, so a press beside it lands where it was aimed),
      // above it in the stacking; it overlaps the field's neighbours by design.
      const self = kit.lastPath();
      const fb = kit.bounds(`${parentOf(self)}/Frame`);
      const fieldH = fb?.height ?? byText(Math.max(t.controlMd, frame().touch ? TOUCH : 0));
      // Above the field when below it the list would run past the screen's bottom (a field low on a phone)
      // and above it there is room, as a select's list flips (last tick's sizes).
      const { viewport: vp } = frame();
      // Its height from its rows (its bounds are what the page's clip leaves of it).
      const listH = showing.length * byText(Math.max(36, frame().touch ? TOUCH : 0)) + 2 * t.xxs
        + (query.trim() ? 0 : frame().theme.typeRamp.caption?.lineHeight ?? 16);
      const up = fb !== null
        && fb.y + fb.height + t.xxs + listH > vp.height - vp.safeArea.bottom
        && fb.y - t.xxs - listH >= vp.safeArea.top;
      kit.props(overProps({
        insets: { start: 0, top: up ? -(listH + t.xxs) : fieldH + t.xxs },
        z: 20,
        overlaps: [parentOf(parentOf(self))].filter((x) => x),
        enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: still() ? 0 : up ? 4 : -4 }], transition: { kind: "tween", durationMs: 120, ease: "out" } },
        reduced: "fade",
      }));
    }
  } finally {
    kit.close();
  }
  // A z orders only siblings: the list's lifts it over the field, the search's own over the fields after it.
  if (listed) kit.props({ z: 20 });
  if (chosen !== null || closing) {
    kit.feedback(closing ? "close" : "select", -1);
    // Back in the field (with the chosen text), the list closed.
    kit.setFocus(`${kit.lastPath()}/Frame/Field`);
  }
  kit.remember(`${id}~open`, focusedAny && chosen === null ? 1 : 0);
  // Shut until the player types, presses the field or leaves it.
  const reopen = !focusedAny || (res?.changed ?? false) || (res?.response.clicked ?? false);
  kit.remember(`${id}~shut`, closing || (shut && !reopen) ? 1 : 0);
  const q = chosen ?? res?.value ?? query;
  return {
    query: q,
    changed: q !== query,
    submitted: res?.submitted ?? false,
    chosen,
    cleared: res?.cleared ?? false,
    focused: focusedAny,
  };
}

