// Pickers (docs/ui.md §11, Controls): the dropdown (a select) and the colour
// picker. The dropdown's list is a floating layer (a focus scope that Back
// and a press outside close, the focus returning to its trigger) opening
// at the chosen option; the colour picker is a saturation and brightness
// square, a hue strip, an opacity strip, swatches and a hex field.

import type * as Api from "@origingame/engine";
import { beside, floating, floatingContent, elevation } from "../containers/base.ts";
import {
  byText, chevron, clamp, control, drawn, fieldChrome, frame, glyphIcon, grow, kit, mix, over, paint, px, recallPath, rememberPath, role, rtl, still, text, TOUCH, xy,
} from "./base.ts";
import { fold, inputField } from "./fields.ts";

// ── dropdown ────────────────────────────────────────────────────────────

/** The options a filter keeps, by index: those whose label contains it (case and accents ignored). */
export function keptOptions(options: readonly Api.DropdownOption[], filter: string): number[] {
  const q = fold(filter.trim());
  return options.map((_, i) => i).filter((i) => !q || fold(options[i]?.label ?? "").includes(q));
}

export function dropdown(id: string, desc: Api.DropdownDesc = {}): Api.DropdownResult {
  const { t } = frame();
  const options = desc.options ?? [];
  const before = desc.selected ?? -1;
  let selected = before;
  const disabled = desc.disabled ?? false;
  const invalid = !!desc.error;
  let open = kit.memory(`${id}~open`, 0) > 0 && !disabled;
  const wasOpen = open;
  const h = byText(Math.max(t.controlMd, frame().touch ? TOUCH : 0));
  const chosen = selected >= 0 ? options[selected] : undefined;
  const searchable = desc.searchable ?? options.length > 10;
  let filter = "";
  const key = `Dropdown~${id}`;
  fieldChrome(`${id}`, {
    ...(desc.label ? { label: desc.label } : {}),
    ...(desc.helper ? { helper: desc.helper } : {}),
    ...(desc.error ? { error: desc.error } : {}),
    width: desc.width ?? grow(),
  }, () => {
    const trig = control("Trigger", {
      part: "Dropdown.trigger",
      cls: "field",
      flags: [open && "open", invalid && "invalid"],
      disabled,
      direction: "row",
      width: grow(),
      height: px(h),
      padding: xy(t.md, 0),
      gap: t.sm,
      fill: open ? role("panel") : role("well"),
      washBy: 0.05,
      lift: 0,
      press: 0.99,
      semantics: { role: "combobox", name: desc.label || desc.placeholder || id, value: chosen?.label ?? "", expanded: open },
    }, () => {
      kit.setDraw({ strokes: [{ width: invalid || open ? 2 : 1, align: "inside", paint: { kind: "solid", color: { token: invalid ? "danger" : open ? "focus" : "lineStrong" } } }] });
      if (chosen?.icon) glyphIcon("Icon", chosen.icon, byText(18), role("ink"));
      text("Value", chosen?.label ?? (desc.placeholder || "Select…"), { step: "body", color: chosen ? (disabled ? "inkFaint" : "ink") : "inkFaint", nowrap: true, ellipsis: true, width: grow() });
      chevron("Chevron", "down", byText(18), disabled ? role("inkFaint") : role("inkMuted"), { group: "expansion", current: open ? "expanded" : "collapsed", turns: { collapsed: 0, expanded: 2 } }, 2.2);
    });
    rememberPath(key, trig.path);
    if (trig.r.clicked && !disabled) {
      open = !open;
      kit.feedback(open ? "open" : "close", -1);
    }
  });
  const anchor = kit.bounds(recallPath(key)) ?? null;
  const listId = `${id}.list`;
  const rowH = byText(Math.max(40, frame().touch ? TOUCH : 0));
  let dismissed = false;
  if (anchor) {
    const f = floating(listId, open, { width: px(anchor.width) }, (size) => beside(anchor, size, "below", 4, "start"), () => {
      floatingContent({
        direction: "column",
        width: px(anchor.width),
        padding: xy(0, t.xs),
        radius: t.control,
        background: paint(role("panelRaised")),
        part: "Dropdown.list",
        partClass: "tooltip",
      }, () => {
        kit.setDraw({ strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: { token: "line" } } }], effects: elevation("floating") });
        if (searchable) {
          kit.open("Filter", { direction: "column", width: grow(), padding: xy(t.sm, t.xxs) });
          filter = inputField("Search", kit.memory(`${id}~f`, 0) ? (filterText.get(id) ?? "") : "", { placeholder: desc.searchPlaceholder || "Filter", icon: "search", clearable: true }).value;
          kit.close();
          filterText.set(id, filter);
          kit.remember(`${id}~f`, filter ? 1 : 0);
        }
        const kept = keptOptions(options, filter);
        const tall = kept.length * rowH > (desc.maxHeight ?? 320);
        const body = (): void => {
          let group = "";
          kept.forEach((i) => {
            const o = options[i]!;
            if ((o.group ?? "") !== group) {
              group = o.group ?? "";
              if (group) {
                kit.open(kit.idi("Group", i), { direction: "row", width: grow(), padding: { left: t.md, right: t.md, top: t.sm, bottom: t.xxs }, part: "Dropdown.group", partClass: "panel" });
                text("Text", group, { step: "caption", color: "inkMuted", nowrap: true });
                kit.close();
              }
            }
            const on = i === selected;
            const off = o.disabled ?? false;
            const b = control(kit.idi("Option", i), {
              part: "Dropdown.option",
              cls: "button",
              flags: [on && "selected"],
              disabled: off,
              width: grow(),
              height: o.description ? null : px(rowH),
              padding: xy(t.md, o.description ? t.xs : 0),
              gap: t.sm,
              align: "start",
              fill: on ? mix(role("panelRaised"), role("primary"), 0.1) : null,
              washBy: 0.08,
              lift: 0,
              press: 0.99,
              radius: 0,
              semantics: { role: "option", name: o.label ?? "", selected: on },
            }, () => {
              if (o.icon) glyphIcon("Icon", o.icon, byText(18), off ? role("inkFaint") : role("ink"));
              kit.open("Text", { direction: "column", width: grow(), gap: 2 });
              text("Label", o.label ?? "", { step: "body", color: off ? "inkFaint" : "ink", nowrap: true, ellipsis: true, width: grow() });
              if (o.description) text("Description", o.description, { step: "sm", color: "inkMuted", width: grow(), maxLines: 2, ellipsis: true });
              kit.close();
              // The chosen one's tick: never colour alone.
              if (on) drawn("Tick", "tick", byText(18), role("primary"));
              else kit.spacer("Tick", px(byText(18)));
            });
            if (b.r.clicked && !off) {
              selected = i;
              open = false;
            }
          });
          if (kept.length === 0) {
            kit.open("Empty", { direction: "row", width: grow(), height: px(rowH), padding: xy(t.md, 0), cross: "center" });
            text("Text", "—", { step: "body", color: "inkMuted" });
            kit.close();
          }
        };
        if (tall) kit.scroll("List", { width: grow(), height: px(desc.maxHeight ?? 320), gap: 0 }, body);
        else {
          kit.open("List", { direction: "column", width: grow() });
          body();
          kit.close();
        }
        kit.semantics({ role: "listbox", name: desc.label || id });
      });
    }, (at) => {
      const k = at.openness;
      return { transform: { y: (1 - k) * -6 }, opacity: Math.min(1, k * 1.6) };
    });
    if (f.dismissed) {
      dismissed = true;
      open = false;
    }
    // Opening, the list takes the focus at the chosen option (the filter when it has one).
    if (open && !wasOpen) {
      const list = `${listId}/Panel/Content`;
      const at = searchable ? `${list}/Filter/Search/Frame/Field` : `${list}/List/${kit.idi("Option", Math.max(0, selected))}`;
      kit.setFocus(at);
    }
  }
  if (selected !== before) kit.feedback("select", -1);
  else if (dismissed && wasOpen) kit.feedback("close", -1);
  if (!open) filterText.delete(id);
  kit.remember(`${id}~open`, open ? 1 : 0);
  return { selected, changed: selected !== before, open };
}

/** A dropdown's filter text while its list is open: names of the frame's UI only, dropped when it closes. */
const filterText = new Map<string, string>();

// ── colour picker ───────────────────────────────────────────────────────

/** HSV (h 0–360, s and v 0–1) from sRGB 0–1. */
export function rgbToHsv(c: { r: number; g: number; b: number }): { h: number; s: number; v: number } {
  const mx = Math.max(c.r, c.g, c.b);
  const mn = Math.min(c.r, c.g, c.b);
  const d = mx - mn;
  let h = 0;
  if (d > 1e-9) {
    if (mx === c.r) h = 60 * (((c.g - c.b) / d) % 6);
    else if (mx === c.g) h = 60 * ((c.b - c.r) / d + 2);
    else h = 60 * ((c.r - c.g) / d + 4);
  }
  return { h: (h + 360) % 360, s: mx > 0 ? d / mx : 0, v: mx };
}

/** sRGB 0–1 from HSV. */
export function hsvToRgb(h: number, s: number, v: number): { r: number; g: number; b: number } {
  const c = v * s;
  const hh = (((h % 360) + 360) % 360) / 60;
  const x = c * (1 - Math.abs((hh % 2) - 1));
  const [r, g, b] = hh < 1 ? [c, x, 0] : hh < 2 ? [x, c, 0] : hh < 3 ? [0, c, x] : hh < 4 ? [0, x, c] : hh < 5 ? [x, 0, c] : [c, 0, x];
  const m = v - c;
  return { r: r + m, g: g + m, b: b + m };
}

/** `#rrggbb`, or `#rrggbbaa` with alpha under 1. */
export function toHex(c: Api.PickedColor): string {
  const b = (x: number) => Math.round(clamp(x, 0, 1) * 255).toString(16).padStart(2, "0");
  const a = c.a ?? 1;
  return `#${b(c.r ?? 0)}${b(c.g ?? 0)}${b(c.b ?? 0)}${a < 1 ? b(a) : ""}`;
}

/** `#rgb`, `#rrggbb` or `#rrggbbaa` (the `#` optional) as a colour; null for anything else. */
export function parseHex(s: string): Api.PickedColor | null {
  let h = s.trim().replace(/^#/, "");
  if (!/^([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(h)) return null;
  if (h.length === 3) h = [...h].map((x) => x + x).join("");
  const n = (k: number) => parseInt(h.slice(k, k + 2), 16) / 255;
  return { r: n(0), g: n(2), b: n(4), a: h.length === 8 ? n(6) : 1 };
}

const solid = (r: number, g: number, b: number, a = 1): Api.Paint => ({ color: { r, g, b }, alpha: a });

export function colorPicker(id: string, color: Api.PickedColor, desc: Api.ColorPickerDesc = {}): Api.ColorPickerResult {
  const { t } = frame();
  const disabled = desc.disabled ?? false;
  const c0: Required<Api.PickedColor> = { r: color.r ?? 0, g: color.g ?? 0, b: color.b ?? 0, a: color.a ?? 1 };
  const hsv = rgbToHsv(c0);
  // The hue kept while the colour is grey (no hue of its own): UI memory.
  const keptHue = kit.memory(`${id}~hue`, hsv.h);
  let h = hsv.s < 1e-4 || hsv.v < 1e-4 ? keptHue : hsv.h;
  let s = hsv.s;
  let v = hsv.v;
  let a = c0.a;
  let committed = false;
  const ptr = kit.framePointer() as Api.UiPointer;
  const width = desc.width ?? px(280);
  // Which part a held press drags (1 square, 2 hue, 3 alpha) and their boxes last tick.
  const drag = kit.memory(`${id}~drag`, 0);
  const box = (part: string) => kit.bounds(recallPath(`ColorPicker~${id}~${part}`));
  const across = (b: Api.UiRect | null) => (b ? clamp((ptr.x - b.x) / Math.max(1, b.width), 0, 1) : 0);
  const down = (b: Api.UiRect | null) => (b ? clamp((ptr.y - b.y) / Math.max(1, b.height), 0, 1) : 0);
  const apply = (which: number): void => {
    if (which === 1) {
      const b = box("square");
      s = across(b);
      v = 1 - down(b);
    } else if (which === 2) h = across(box("hue")) * 360;
    else if (which === 3) a = across(box("alpha"));
  };
  if (drag && ptr.down && !disabled) apply(drag);
  let pressed = 0;
  let hexEdit: string | null = null;
  let swatchHit: Api.PickedColor | null = null;
  const thumb = byText(frame().touch ? 26 : 20);
  const stripH = byText(frame().touch ? 28 : 20);
  kit.open(id, { direction: "column", width, gap: t.sm, cross: "stretch" });
  try {
    if (desc.label) text("Label", desc.label, { step: "sm", color: "inkMuted", nowrap: true });
    const hue = hsvToRgb(h, 1, 1);
    // The square: the hue, white across, black down; a ring at the colour.
    const sq = control("Square", {
      part: "ColorPicker.square",
      cls: "slot",
      disabled,
      adjustable: true,
      width: grow(),
      height: px(byText(180)),
      fill: null,
      wash: null,
      lift: 0,
      press: 1,
      radius: t.control,
      semantics: { role: "slider", name: "Saturation and brightness", value: toHex({ ...hsvToRgb(h, s, v), a }), valueMin: 0, valueMax: 100, valueNow: Math.round(s * 100) },
    }, (ix) => {
      kit.setDraw({
        shape: { kind: "rect", radii: [t.control, t.control, t.control, t.control] },
        clip: "shape",
        fills: [
          { kind: "solid", color: solid(hue.r, hue.g, hue.b) },
          { kind: "gradient", type: "linear", from: { x: 0, y: 0.5 }, to: { x: 1, y: 0.5 }, stops: [{ offset: 0, color: solid(1, 1, 1, 1) }, { offset: 1, color: solid(1, 1, 1, 0) }] },
          { kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: solid(0, 0, 0, 0) }, { offset: 1, color: solid(0, 0, 0, 1) }] },
        ],
        strokes: [{ width: ix === "focused" ? 2 : 1, align: "inside", paint: { kind: "solid", color: { token: ix === "focused" ? "focus" : "line" } } }],
      } as Api.DrawDesc);
      const b = box("square");
      const w = b ? b.width : 0;
      const hgt = b ? b.height : 0;
      ringAt("Ring", s * w - thumb / 2, (1 - v) * hgt - thumb / 2, thumb, hsvToRgb(h, s, v), ix === "focused" || drag === 1, w);
    });
    rememberPath(`ColorPicker~${id}~square`, sq.path);
    if (sq.r.pressed && drag === 0) pressed = 1;
    if (sq.r.adjust) {
      s = clamp(s + 0.05 * sq.r.adjust * (rtl() ? -1 : 1), 0, 1);
      committed = true;
    }
    const strip = (sid: string, part: string, which: number, f: number, fills: Api.Fill[], name: string, value: string): void => {
      const b = control(sid, {
        part: `ColorPicker.${part}`,
        cls: "bar",
        disabled,
        adjustable: true,
        width: grow(),
        height: px(stripH),
        fill: null,
        wash: null,
        lift: 0,
        press: 1,
        radius: stripH / 2,
        semantics: { role: "slider", name, value, valueMin: 0, valueMax: which === 2 ? 360 : 100, valueNow: Math.round(which === 2 ? h : a * 100) },
      }, (ix) => {
        kit.setDraw({ shape: { kind: "rect", radii: [stripH / 2, stripH / 2, stripH / 2, stripH / 2] }, clip: "shape", fills, strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: { token: "line" } } }] } as Api.DrawDesc);
        const bb = box(part);
        ringAt("Thumb", f * (bb ? bb.width : 0) - thumb / 2, (stripH - thumb) / 2, thumb, which === 2 ? hue : hsvToRgb(h, s, v), ix === "focused" || drag === which, bb ? bb.width : 0);
      });
      rememberPath(`ColorPicker~${id}~${part}`, b.path);
      if (b.r.pressed && drag === 0) pressed = which;
      if (b.r.adjust) {
        // Physical strips: Right raises them in every locale (adjust is the reading direction's).
        const dir = b.r.adjust * (rtl() ? -1 : 1);
        if (which === 2) h = (h + 5 * dir + 360) % 360;
        else a = clamp(a + 0.05 * dir, 0, 1);
        committed = true;
      }
    };
    const hueStops = [0, 60, 120, 180, 240, 300, 360].map((d, i) => {
      const c = hsvToRgb(d, 1, 1);
      return { offset: i / 6, color: solid(c.r, c.g, c.b) };
    });
    strip("Hue", "hue", 2, h / 360, [{ kind: "gradient", type: "linear", from: { x: 0, y: 0.5 }, to: { x: 1, y: 0.5 }, stops: hueStops }], "Hue", `${Math.round(h)}°`);
    if (desc.alpha) {
      const cc = hsvToRgb(h, s, v);
      strip("Alpha", "alpha", 3, a, [
        // A checkerboard's grey under the ramp, so transparency reads as transparency.
        { kind: "gradient", type: "linear", from: { x: 0, y: 0 }, to: { x: 0.04, y: 0.04 }, spread: "repeat", stops: [{ offset: 0.5, color: solid(0.8, 0.8, 0.8) }, { offset: 0.5, color: solid(0.55, 0.55, 0.55) }] },
        { kind: "gradient", type: "linear", from: { x: 0, y: 0.5 }, to: { x: 1, y: 0.5 }, stops: [{ offset: 0, color: solid(cc.r, cc.g, cc.b, 0) }, { offset: 1, color: solid(cc.r, cc.g, cc.b, 1) }] },
      ] as Api.Fill[], "Opacity", `${Math.round(a * 100)}%`);
    }
    // Before and after, and the hex field.
    kit.open("Bottom", { direction: "row", width: grow(), gap: t.sm, cross: "end" });
    kit.open("Preview", { direction: "row", width: px(byText(56)), height: px(byText(40)), radius: t.control, part: "ColorPicker.preview", partClass: "slot" });
    const now = hsvToRgb(h, s, v);
    kit.setDraw({ shape: { kind: "rect", radii: [t.control, t.control, t.control, t.control] }, clip: "shape", strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: { token: "line" } } }] } as Api.DrawDesc);
    kit.open("Was", { width: grow(), height: grow(), background: solid(c0.r, c0.g, c0.b, c0.a) });
    kit.close();
    kit.open("Now", { width: grow(), height: grow(), background: solid(now.r, now.g, now.b, a) });
    kit.close();
    kit.close();
    kit.semantics({ role: "image", name: `Was ${toHex(c0)}, now ${toHex({ ...now, a })}` });
    if (desc.hex ?? true) {
      kit.open("Hex", { direction: "column", width: grow() });
      const shown = toHex({ ...now, a });
      const typed = hexDrafts.get(id);
      const r = inputField("Input", typed ?? shown, { placeholder: "#rrggbb", clearable: false, maxChars: 9, ...(typed !== undefined && parseHex(typed) === null ? { error: " " } : {}), disabled });
      if (r.changed) hexDrafts.set(id, r.value);
      if (r.submitted || (!r.response.focused && typed !== undefined)) {
        const p = parseHex(r.value);
        if (p) hexEdit = r.value;
        hexDrafts.delete(id);
      }
      kit.close();
    }
    kit.close();
    const sw = desc.swatches ?? [];
    if (sw.length > 0) {
      kit.open("Swatches", { direction: "row", width: grow(), gap: t.xs, wrap: true });
      const cur = toHex({ ...hsvToRgb(h, s, v), a: 1 }).toLowerCase();
      sw.forEach((hx, i) => {
        const p = parseHex(hx);
        if (!p) return;
        const on = toHex({ ...p, a: 1 }).toLowerCase() === cur;
        const size = byText(frame().touch ? 36 : 28);
        const b = control(kit.idi("Swatch", i), {
          part: "ColorPicker.swatch",
          cls: "chip",
          flags: [on && "selected", "round"],
          disabled,
          width: px(size),
          height: px(size),
          radius: size / 2,
          fill: { r: p.r ?? 0, g: p.g ?? 0, b: p.b ?? 0, a: 1 },
          wash: null,
          lift: -1,
          press: 0.9,
          semantics: { role: "radio", name: hx, checked: on },
        }, () => {
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: solid(p.r ?? 0, p.g ?? 0, p.b ?? 0) }], strokes: [{ width: on ? 2 : 1, align: "outside", paint: { kind: "solid", color: { token: on ? "focus" : "line" } } }] } as Api.DrawDesc);
          if (on) drawn("Tick", "tick", Math.round(size * 0.55), contrastInk(p));
        });
        if (b.r.clicked) swatchHit = p;
      });
      kit.close();
    }
  } finally {
    kit.close();
  }
  if (!disabled) {
    if (pressed) apply(pressed);
    if (swatchHit) {
      const p = rgbToHsv(swatchHit);
      if (p.s > 1e-4) h = p.h;
      s = p.s;
      v = p.v;
      committed = true;
    }
    if (hexEdit) {
      const p = parseHex(hexEdit)!;
      const q = rgbToHsv({ r: p.r ?? 0, g: p.g ?? 0, b: p.b ?? 0 });
      if (q.s > 1e-4) h = q.h;
      s = q.s;
      v = q.v;
      a = p.a ?? 1;
      committed = true;
    }
  }
  const held = pressed || (drag && ptr.down ? drag : 0);
  if (drag && !held) committed = true;
  kit.remember(`${id}~drag`, held);
  kit.remember(`${id}~hue`, h);
  const out = { ...hsvToRgb(h, s, v), a };
  const same = Math.abs(out.r - c0.r) < 1e-6 && Math.abs(out.g - c0.g) < 1e-6 && Math.abs(out.b - c0.b) < 1e-6 && Math.abs(out.a - c0.a) < 1e-6;
  return { color: out, hex: toHex(out), changed: !same, committed };
}

/** A hex field's text while it is being typed: the frame's UI names only. */
const hexDrafts = new Map<string, string>();

/** Black or white, whichever reads on a colour. */
function contrastInk(p: Api.PickedColor): { r: number; g: number; b: number; a: number } {
  const l = 0.2126 * (p.r ?? 0) + 0.7152 * (p.g ?? 0) + 0.0722 * (p.b ?? 0);
  return l > 0.55 ? { r: 0, g: 0, b: 0, a: 1 } : { r: 1, g: 1, b: 1, a: 1 };
}

/** A picker's ring: white with a dark edge (it reads on any colour), the colour inside; larger while held or focused. */
function ringAt(id: string, x: number, y: number, size: number, fill: { r: number; g: number; b: number }, big: boolean, across: number): void {
  const k = big && !still() ? 1.15 : 1;
  kit.open(id, { width: px(size), height: px(size), part: "ColorPicker.thumb", partClass: "chip" });
  kit.setDraw({
    shape: { kind: "ellipse" },
    fills: [{ kind: "solid", color: solid(fill.r, fill.g, fill.b) }],
    strokes: [{ width: 3, align: "inside", paint: { kind: "solid", color: solid(1, 1, 1) } }, { width: 1, align: "outside", paint: { kind: "solid", color: solid(0, 0, 0, 0.45) } }],
    effects: [{ kind: "dropShadow", offset: { x: 0, y: 1 }, blur: 3, spread: 0, color: solid(0, 0, 0, 0.35) }],
  } as Api.DrawDesc);
  kit.close();
  // The square and the strips are physical (red is at the left in every locale): placed from the left.
  over({ insets: { start: Math.round(rtl() ? across - x - size : x), top: Math.round(y) }, inert: true, z: 3, transform: { scaleX: k, scaleY: k }, motion: { kind: "token", token: "ui" } });
}

