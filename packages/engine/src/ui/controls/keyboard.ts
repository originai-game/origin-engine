// The on-screen keyboard (docs/ui.md §2): a floating panel at
// the screen's bottom for pads and touch, the text with its caret, a
// candidate bar while composing (an IME's), and rows of keys a pad walks
// (focus wraps by rows) and a finger presses. Letters with shift and caps,
// symbols, a number pad and the kana table with its voicing marks.

import type * as Api from "@origingame/engine";
import { floating, floatingContent, elevation } from "../containers/base.ts";
import { byText, clamp, control, drawn, frame, grow, kit, mix, paint, px, role, still, text, TOUCH, xy } from "./base.ts";

type Layout = Api.KeyboardLayout;
type Key =
  | { kind: "char"; ch: string; wide?: number }
  | { kind: "shift" | "back" | "space" | "enter" | "voice" | "semi" | "small" | "left" | "right"; wide?: number }
  | { kind: "layout"; to: Layout; label: string; wide?: number };

const ch = (s: string): Key[] => [...s].map((c) => ({ kind: "char", ch: c }));

/** The kana table in rows: the gojūon read across by consonant (a row is a vowel), as a pad keyboard lays it out. */
const KANA_ROWS = ["あかさたなはまやらわ", "いきしちにひみ\u3000り\u3000", "うくすつぬふむゆるを", "えけせてねへめ\u3000れん", "おこそとのほもよろー"];

/** The keys of a layout (rows, the last one the action row), letters in the shift state given. */
/** The keys of a layout. `digits`: a masked number (a PIN) takes digits only, no sign, point or caret keys. */
export function layoutRows(layout: Layout, shifted: boolean, kana: boolean, digits = false): Key[][] {
  const up = (s: string) => (shifted ? s.toUpperCase() : s);
  const bottom = (to: Layout, label: string): Key[] => [
    { kind: "layout", to, label, wide: 1.5 },
    ...(kana && layout !== "kana" ? [{ kind: "layout", to: "kana", label: "かな", wide: 1.25 } as Key] : []),
    ...(layout === "kana" ? [{ kind: "layout", to: "letters", label: "ABC", wide: 1.25 } as Key] : []),
    { kind: "left" },
    { kind: "space", wide: layout === "kana" ? 3 : 4 },
    { kind: "right" },
    { kind: "enter", wide: 2 },
  ];
  switch (layout) {
    case "symbols":
      return [ch("1234567890"), ch("@#$%&*-+()"), [...ch("!\"':;/?,."), { kind: "back", wide: 1.5 }], bottom("letters", "ABC")];
    case "numeric":
      if (digits) return [ch("123"), ch("456"), ch("789"), [{ kind: "char", ch: "0", wide: 2 }, { kind: "back" }], [{ kind: "enter", wide: 3 }]];
      return [ch("123"), ch("456"), ch("789"), [{ kind: "char", ch: "-" }, { kind: "char", ch: "0" }, { kind: "char", ch: "." }], [{ kind: "left" }, { kind: "back" }, { kind: "right" }], [{ kind: "enter", wide: 3 }]];
    case "kana":
      return [
        ...KANA_ROWS.map((r) => ch(r)),
        [{ kind: "voice" }, { kind: "semi" }, { kind: "small" }, ...ch("、。？！「」"), { kind: "back", wide: 2 }],
        bottom("symbols", "?123"),
      ];
    default:
      return [ch(up("qwertyuiop")), ch(up("asdfghjkl")), [{ kind: "shift", wide: 1.5 }, ...ch(up("zxcvbnm")), { kind: "back", wide: 1.5 }], bottom("symbols", "?123")];
  }
}

const DAKUTEN: Record<string, string> = Object.fromEntries(
  [..."かきくけこさしすせそたちつてとはひふへほ"].map((c) => [c, String.fromCharCode(c.charCodeAt(0) + 1)]).concat([["う", "ゔ"]]),
);
const HANDAKUTEN: Record<string, string> = Object.fromEntries([..."はひふへほ"].map((c) => [c, String.fromCharCode(c.charCodeAt(0) + 2)]));
const SMALL: Record<string, string> = Object.fromEntries([..."あいうえおつやゆよわ"].map((c) => [c, String.fromCharCode(c.charCodeAt(0) - 1)]));

/** A kana's voiced (゛), semi-voiced (゜) or small form, cycling back to the plain one; null when it has none. */
export function kanaMark(c: string, mark: "voice" | "semi" | "small"): string | null {
  const inv = (m: Record<string, string>) => Object.fromEntries(Object.entries(m).map(([a, b]) => [b, a]));
  const table = mark === "voice" ? DAKUTEN : mark === "semi" ? HANDAKUTEN : SMALL;
  if (table[c]) return table[c]!;
  const back = inv(table)[c];
  if (back) return back;
  // A voiced kana takes the semi mark (ば → ぱ) and back.
  if (mark === "semi") {
    const plain = inv(DAKUTEN)[c];
    if (plain && HANDAKUTEN[plain]) return HANDAKUTEN[plain]!;
  }
  if (mark === "voice") {
    const plain = inv(HANDAKUTEN)[c];
    if (plain && DAKUTEN[plain]) return DAKUTEN[plain]!;
  }
  return null;
}

/** Hiragana as katakana (the candidate the kana layout always offers). */
export function toKatakana(s: string): string {
  return [...s].map((c) => {
    const k = c.charCodeAt(0);
    return k >= 0x3041 && k <= 0x3096 ? String.fromCharCode(k + 0x60) : c;
  }).join("");
}

/** What a key types (its accessible name and its glyph). */
function keyName(k: Key): string {
  switch (k.kind) {
    case "char": return k.ch === "\u3000" ? "" : k.ch;
    case "layout": return k.label;
    case "space": return "Space";
    case "back": return "Backspace";
    case "enter": return "Done";
    case "shift": return "Shift";
    case "voice": return "゛";
    case "semi": return "゜";
    case "small": return "小";
    case "left": return "Caret left";
    case "right": return "Caret right";
  }
}

const drafts = new Map<string, string>();

export function onScreenKeyboard(id: string, open: boolean, desc: Api.KeyboardDesc = {}): Api.KeyboardResult {
  const { t, viewport } = frame();
  const kana = desc.kana ?? false;
  const startLayout: Layout = desc.layout ?? "letters";
  // The layout shown, the shift (0 off, 1 once, 2 caps), the caret and the composition persist while it is open.
  const LAYOUTS: Layout[] = ["letters", "symbols", "numeric", "kana"];
  const opened = kit.memory(`${id}~was`, 0) > 0;
  // The layout the game asks for: opening shows it, and so does the game asking for another while it is open.
  const asked = LAYOUTS.indexOf(startLayout);
  const askedBefore = kit.memory(`${id}~asked`, asked);
  // The game switched the layout while it was open (its own layout key's switch, from outside).
  const switched = open && opened && asked !== askedBefore;
  if (open && !opened) {
    kit.remember(`${id}~caret`, -1);
    kit.remember(`${id}~shift`, 0);
    drafts.set(id, "");
  }
  if (open && (!opened || asked !== askedBefore)) {
    kit.remember(`${id}~layout`, asked);
    kit.remember(`${id}~shift`, 0);
  }
  kit.remember(`${id}~asked`, asked);
  kit.remember(`${id}~was`, open ? 1 : 0);
  let layout = LAYOUTS[clamp(kit.memory(`${id}~layout`, LAYOUTS.indexOf(startLayout)), 0, 3)] ?? "letters";
  if (layout === "kana" && !kana) layout = "letters";
  let shift = kit.memory(`${id}~shift`, 0);
  let value = desc.value ?? "";
  const chars = [...value];
  let caret = kit.memory(`${id}~caret`, -1);
  caret = caret < 0 ? chars.length : clamp(caret, 0, chars.length);
  let comp = drafts.get(id) ?? "";
  const composing = layout === "kana" || (desc.compose ?? false);
  const max = desc.maxChars ?? 0;
  let changed = false;
  let submitted = false;
  let closed = false;

  const insert = (s: string) => {
    const cs = [...value];
    if (max > 0 && cs.length + [...s].length > max) {
      kit.feedback("error", -1);
      return;
    }
    cs.splice(caret, 0, ...s);
    value = cs.join("");
    caret += [...s].length;
    changed = true;
  };
  const commit = (s: string) => {
    comp = "";
    insert(s);
  };
  // Switched by the game to a layout that does not compose: the composition is committed, as the layout key does.
  if (switched && comp && !composing) commit(comp);
  const press = (k: Key) => {
    switch (k.kind) {
      case "char": {
        if (k.ch === "\u3000") return;
        if (composing && layout !== "symbols" && layout !== "numeric" && /[\p{L}ー]/u.test(k.ch)) comp += k.ch;
        else {
          if (comp) commit(comp);
          insert(k.ch);
        }
        if (shift === 1) shift = 0;
        break;
      }
      case "voice":
      case "semi":
      case "small": {
        const last = [...comp].pop();
        const to = last ? kanaMark(last, k.kind) : null;
        if (to) comp = [...comp].slice(0, -1).join("") + to;
        else kit.feedback("error", -1);
        break;
      }
      case "space":
        if (comp) commit(comp);
        insert(" ");
        break;
      case "back":
        if (comp) comp = [...comp].slice(0, -1).join("");
        else if (caret > 0) {
          const cs = [...value];
          cs.splice(caret - 1, 1);
          value = cs.join("");
          caret -= 1;
          changed = true;
        }
        break;
      case "left":
        if (!comp) caret = Math.max(0, caret - 1);
        break;
      case "right":
        if (!comp) caret = Math.min([...value].length, caret + 1);
        break;
      case "enter":
        if (comp) commit(comp);
        else submitted = true;
        break;
      case "shift": {
        const now = kit.seconds();
        const last = kit.memory(`${id}~shiftAt`, -10);
        shift = shift === 0 ? (now - last < 0.35 ? 2 : 1) : shift === 1 && now - last < 0.35 ? 2 : 0;
        kit.remember(`${id}~shiftAt`, now);
        break;
      }
      case "layout":
        if (comp) commit(comp);
        layout = k.to;
        shift = 0;
        break;
    }
  };

  const rows = layoutRows(layout, shift > 0, kana, desc.password ?? false);
  const wide = viewport.width >= 720;
  const panelW = Math.min(viewport.width - viewport.safeArea.left - viewport.safeArea.right, wide ? 980 : viewport.width);
  // A key's height: a finger's 44 on touch, larger on a wide screen, with the text size, but the rows
  // never taller than about three fifths of the screen. Its width: the widest row fills the panel at
  // most, so a phone's ten letters fit across (narrower than tall, as every phone keyboard's are).
  const base = Math.max(frame().touch ? TOUCH : 40, Math.min(56, (Math.min(viewport.width, 980) - 2 * t.md) / 11.5 - t.xxs));
  const gap = t.xxs;
  const keysRoom = (viewport.height - viewport.safeArea.top - viewport.safeArea.bottom) * 0.6;
  const keyH = Math.round(Math.max(frame().touch ? TOUCH : 32, Math.min(byText(base), (keysRoom - gap * (rows.length - 1)) / rows.length)));
  const units = Math.max(...rows.map((row) => row.reduce((a, k) => a + (k.wide ?? 1), 0)));
  const fits = (panelW - 2 * t.md - gap * (units - 1)) / units;
  const unit = Math.floor(Math.min(byText(base) * (layout === "numeric" ? 2.4 : 1), fits));
  const title = desc.title ?? "";
  const placeholder = desc.placeholder ?? "";
  const shown = desc.password ? "•".repeat([...value].length) : value;
  const candidates = composing && comp ? [...new Set([comp, ...(layout === "kana" ? [toKatakana(comp)] : []), ...(desc.candidates ?? [])])] : [];

  const f = floating(id, open, { scrim: true, transition: "slideUp", width: px(panelW), spring: "panel" }, (size) => ({
    x: Math.round((viewport.width - size.width) / 2),
    y: Math.round(viewport.height - viewport.safeArea.bottom - size.height),
  }), (at) => {
    floatingContent({
      direction: "column",
      width: px(panelW),
      padding: { left: t.md, right: t.md, top: t.md, bottom: t.md },
      gap: t.sm,
      radius: t.panel,
      background: paint(role("panelRaised")),
      part: "Keyboard.panel",
      partClass: "panel",
    }, () => {
      kit.setDraw({ strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: { token: "line" } } }], effects: elevation("overlay") });
      kit.semantics({ role: "dialog", name: title || placeholder || "Keyboard" });
      if (title) text("Title", title, { step: "sm", color: "inkMuted", nowrap: true, ellipsis: true });
      // The text with its caret (and the composition underlined at it).
      kit.open("Preview", {
        direction: "row",
        width: grow(),
        height: px(Math.max(t.controlMd, 44)),
        padding: xy(t.md, 0),
        cross: "center",
        radius: t.control,
        background: paint(role("well")),
        part: "Keyboard.preview",
        partClass: "field",
      });
      kit.setDraw({ strokes: [{ width: 2, align: "inside", paint: { kind: "solid", color: { token: "focus" } } }] });
      const cs = [...shown];
      const before = cs.slice(0, caret).join("");
      const after = cs.slice(caret).join("");
      if (before) text("Before", before, { step: "body", color: "ink", nowrap: true });
      if (comp) {
        kit.open("Composition", { direction: "column", cross: "stretch" });
        text("Text", comp, { step: "body", color: "primary", nowrap: true });
        kit.open("Underline", { width: grow(), height: px(2), background: paint(role("primary")) });
        kit.close();
        kit.close();
      }
      const blink = still() ? 1 : (Math.floor(kit.seconds() * 1.9) % 2 === 0 ? 1 : 0);
      kit.open("Caret", { width: px(2), height: px(byText(22)), background: paint(role("focus")) });
      kit.close();
      kit.props({ opacity: blink });
      // Empty, the caret waits at the start with the placeholder after it, as a field's does.
      if (!value && !comp && placeholder) text("Placeholder", placeholder, { step: "body", color: "inkFaint", nowrap: true });
      if (after) text("After", after, { step: "body", color: "ink", nowrap: true });
      if (max > 0) {
        kit.spacer("Gap", grow());
        text("Count", `${[...value].length}/${max}`, { step: "caption", color: [...value].length >= max ? "danger" : "inkMuted", nowrap: true });
      }
      kit.close();
      kit.semantics({ role: "textbox", name: title || placeholder || "Text", value: desc.password ? "" : value });
      if (candidates.length > 0) {
        kit.open("Candidates", { direction: "row", width: grow(), gap: t.xs, scrollX: true, part: "Keyboard.candidates", partClass: "panel" });
        candidates.forEach((c, i) => {
          const b = control(kit.idi("Candidate", i), {
            part: "Keyboard.candidate",
            cls: "button",
            height: px(byText(Math.max(36, frame().touch ? TOUCH : 0))),
            padding: xy(t.md, 0),
            fill: i === 0 ? mix(role("control"), role("primary"), 0.16) : role("control"),
            washBy: 0.1,
            semantics: { role: "button", name: c },
          }, () => text("Text", c, { step: "body", color: "ink", nowrap: true }));
          if (b.r.clicked) {
            commit(c);
            kit.feedback("select", -1);
          }
        });
        kit.close();
      }
      // The keys: a row is a row of buttons whose widths are units of the narrowest.
      kit.open("Keys", { direction: "column", width: grow(), gap, cross: "center" });
      const firstPath = { p: "" };
      rows.forEach((row, r) => {
        kit.open(kit.idi("Row", r), { direction: "row", gap, align: "center" });
        row.forEach((k, i) => {
          const name = keyName(k);
          const blank = k.kind === "char" && k.ch === "\u3000";
          const action = k.kind !== "char";
          const active = (k.kind === "shift" && shift > 0) || (k.kind === "layout" && k.to === layout);
          const w = unit * (k.wide ?? 1) + gap * ((k.wide ?? 1) - 1);
          const primary = k.kind === "enter";
          const b = control(kit.idi("Key", i), {
            part: "Keyboard.key",
            cls: "button",
            flags: [(k.wide ?? 1) > 1 && "wide", action && "action", active && "active", primary && "primary"],
            disabled: blank,
            width: px(Math.round(w)),
            height: px(keyH),
            // A finger's target spans the gap to its neighbours.
            touch: 0,
            props: { hitSlop: Math.ceil(gap / 2) },
            fill: primary ? role("primary") : active ? mix(role("control"), role("primary"), 0.25) : action ? role("controlStrong") : role("control"),
            washBy: 0.12,
            press: 0.94,
            lift: 0,
            dim: blank ? 0 : 0.5,
            semantics: { role: "button", name: shift === 2 && k.kind === "shift" ? "Caps lock" : name },
          }, (ix) => {
            if (blank) return;
            const ink = primary ? role("onPrimary") : role("ink");
            const icon = byText(20);
            switch (k.kind) {
              case "back": drawn("Icon", "backspace", icon, ink); break;
              case "shift":
                drawn("Icon", "shift", icon, shift > 0 ? role("primary") : ink);
                if (shift === 2) {
                  kit.open("Lock", { width: px(byText(10)), height: px(2), background: paint(role("primary")) });
                  kit.close();
                }
                break;
              case "enter": text("Label", desc.submitLabel || (comp ? "Enter" : "Done"), { step: "body", color: ink, nowrap: true }); break;
              case "space": text("Label", layout === "kana" ? "空白" : "space", { step: "sm", color: role("inkMuted"), nowrap: true }); break;
              case "left": drawn("Icon", "chevronStart", icon, ink); break;
              case "right": drawn("Icon", "chevronEnd", icon, ink); break;
              default: text("Label", name, { step: k.kind === "char" ? "base" : "sm", color: ink, nowrap: true, align: "center" });
            }
            // A finger's press shows the letter above it (it is under the finger).
            if (k.kind === "char" && ix === "pressed" && frame().touch && !still()) {
              kit.open("Popup", {
                width: px(Math.round(Math.max(unit * 1.2, keyH * 0.9))),
                height: px(Math.round(keyH * 1.25)),
                align: "center",
                cross: "center",
                radius: t.control,
                background: paint(role("panelRaised")),
                part: "Keyboard.popup",
                partClass: "tooltip",
              });
              kit.setDraw({ effects: elevation("floating") });
              text("Text", name, { step: "xl", color: "ink", nowrap: true });
              kit.close();
              kit.props({ transform: { y: -Math.round(keyH * 1.15) }, z: 5 });
            }
          });
          if (!firstPath.p && !blank) firstPath.p = b.path;
          if (b.r.clicked && !blank) {
            press(k);
            kit.feedback(k.kind === "back" ? "back" : k.kind === "enter" ? "confirm" : "press", -1);
          }
        });
        kit.close();
      });
      kit.close();
      // What opens takes the focus: the first key, once.
      if (at.measured && kit.memory(`${id}~focused`, 0) === 0 && firstPath.p) {
        kit.setFocus(firstPath.p);
        kit.remember(`${id}~focused`, 1);
      }
      // A pad's shortcuts: Y (the details action) erases, the bumpers move the caret.
      if (open) {
        if (kit.action("details")) press({ kind: "back" });
        if (kit.action("tabPrev")) press({ kind: "left" });
        if (kit.action("tabNext")) press({ kind: "right" });
      }
    });
  });
  if (!open) kit.remember(`${id}~focused`, 0);
  if (f.dismissed && open) {
    closed = true;
    kit.feedback("back", -1);
  }
  kit.remember(`${id}~layout`, LAYOUTS.indexOf(layout));
  kit.remember(`${id}~caret`, caret);
  kit.remember(`${id}~shift`, shift);
  drafts.set(id, comp);
  return { value, composition: comp, changed, submitted, closed, layout };
}
