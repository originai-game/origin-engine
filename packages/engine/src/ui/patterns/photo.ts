// Photo mode (docs/ui.md §11 genre patterns, Appendix B: photo mode). Read
// against Horizon Forbidden West's, Ghost of Tsushima's and Forza Horizon's
// (a glass panel at the side over the paused shot, its tabs, its sliders
// with their values, filter tiles showing the shot through each, the
// shutter large at the foot, the controls hidden for a clean shot). The
// camera, the filters' look, the frames' art and the capture are the
// game's: this draws the controls and reports what was asked.

import type * as Api from "@origingame/engine";
import {
  artSlot, clamp, eyebrow, flags, frame, glow, grow, hex, kit, lastInteraction, line, over, paint, pointer, px, rememberRoom, role, safeHeight, safeWidth,
  shadow, st, still, surface, symbol, text, token, u, vgrad, xy, type Mut, type SymbolKind,
} from "./core.ts";
import { valueSlider } from "../controls/ranges.ts";
import { tps } from "../feedback/core.ts";

const LOOK_TABS = new Set(["filters", "frames"]);

/**
 * Photo mode's controls (Horizon Forbidden West, Ghost of Tsushima, Forza
 * Horizon): see `fn photoMode` in crates/og-api/api/ui.ogt.
 */
export function photoMode(id: string, desc: Api.PhotoModeDesc = {}): Api.PhotoModeResult {
  const out: Mut<Api.PhotoModeResult> = { tab: null, set: null, filter: null, strength: null, frame: null, grid: false, hide: false, capture: false, reset: false, exit: false };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const W = kit.memory(`~ph.w.${id}`, 0) || safeWidth();
  const H = kit.memory(`~ph.h.${id}`, 0) || safeHeight();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const tabs = desc.tabs ?? [];
  const tab = tabs.find((x) => x.key === desc.tab) ?? tabs[0] ?? null;
  const hidden = !!desc.hidden;
  // Read and written outside the pattern's element: a memory key is the open element's.
  const shotAt = kit.memory(`~ph.shot.${id}`, -1);

  const capture = (): void => {
    out.capture = true;
    kit.feedback("confirm", -1);
  };

  // Hidden: any press or key brings the controls back; confirm takes the shot.
  if (hidden) {
    if (kit.action("confirm")) capture();
    else if (kit.back() || kit.action("details") || pointer().pressed || kit.action("cancel")) {
      out.hide = true;
      kit.feedback("toggle", -1);
    }
  } else {
    const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
    if (step !== 0 && tabs.length > 1) {
      const i = Math.max(0, tabs.findIndex((x) => x.key === tab?.key));
      out.tab = tabs[(i + step + tabs.length) % tabs.length]!.key ?? null;
      kit.feedback("tab", -1);
    }
    if (kit.action("details")) {
      out.hide = true;
      kit.feedback("toggle", -1);
    } else if (kit.back()) out.exit = true;
  }

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 8 : 16), part: "Photo.frame", partClass: "panel" });

  // The rule of thirds over the whole picture.
  if (desc.grid) {
    kit.open("Grid", { width: grow(), height: grow(), part: "Photo.grid", partClass: "badge" });
    const w = Math.max(1, u(1));
    const ln = (k: string, vertical: boolean, at: number): void => {
      kit.open(k, vertical ? { width: px(w), height: grow() } : { width: grow(), height: px(w) });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("ink", 0.42) }], effects: [shadow(0, u(2), 0.35)] });
      kit.close();
      over({ insets: vertical ? { start: Math.round(W * at), top: 0, bottom: 0 } : { top: Math.round(H * at), start: 0, end: 0 }, inert: true });
    };
    ln("V1", true, 1 / 3);
    ln("V2", true, 2 / 3);
    ln("H1", false, 1 / 3);
    ln("H2", false, 2 / 3);
    kit.close();
    over({ insets: { start: 0, end: 0, top: 0, bottom: 0 }, inert: true, z: 1 });
  }
  // The chosen frame's art over the whole picture.
  if (desc.frameImage) {
    artSlot("Frame", "Photo.frame", { texture: desc.frameImage, kind: "overlay", fit: "cover", width: grow(), height: grow(), label: "" });
    over({ insets: { start: 0, end: 0, top: 0, bottom: 0 }, inert: true, z: 2 });
  }

  // A round tool of the bar: its symbol in a glass disc, its name under it.
  const tool = (key: string, label: string, sym: SymbolKind, on: boolean | null): boolean => {
    const d = Math.max(44, u(phone ? 44 : 52));
    let clicked = false;
    kit.column(key, { gap: u(4), cross: "center", width: px(Math.round(Math.max(d, u(phone ? 60 : 84)) * Math.min(1.3, ts))) }, () => {
      const r = surface("Button", {
        part: `Photo.tool${flags(!!on && "on")}`,
        cls: "button",
        fill: on ? { ...role("primary"), a: 0.3 } : { ...role("shadow"), a: 0.55 },
        washBy: 0.14,
        width: px(d),
        height: px(d),
        align: "center",
        cross: "center",
        radius: 999,
        semantics: { role: on === null ? "button" : "switch", name: label, ...(on === null ? {} : { checked: on }) } as never,
      }, () => symbol("Mark", sym, u(phone ? 20 : 24), on ? role("primary") : role("ink"), { icon: sym }));
      clicked = r.clicked;
      text("Name", label, { step: st(phone ? 12 : 13), color: "inkMuted", nowrap: true, ellipsis: true, align: "center", width: grow() });
    });
    return clicked;
  };

  // The shutter: a ring around a lit disc, the one dominant control of the bar.
  const shutter = (): void => {
    const d = Math.max(64, u(phone ? 72 : 88));
    const r = surface("Shutter", {
      part: "Photo.shutter",
      cls: "button",
      fill: null,
      width: px(d),
      height: px(d),
      align: "center",
      cross: "center",
      radius: 999,
      semantics: { role: "button", name: desc.captureLabel ?? "Capture" },
    }, () => {
      const ix = lastInteraction("Shutter");
      const inner = Math.round(d * 0.76);
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("shadow", 0.45) }], strokes: [line(Math.max(3, u(4)), paint(hex("#ffffff")))], effects: [shadow(u(4), u(18), 0.5)] });
      kit.open("Disc", { width: px(inner), height: px(inner), align: "center", cross: "center" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [vgrad([[0, paint(hex("#ffffff"))], [1, ix.pressed ? token("ink", 0.7) : token("ink", 0.88)]])], effects: ix.hovered || ix.focused ? [glow(u(10), token("primary", 0.55), 0)] : [] });
      symbol("Mark", "camera", Math.round(inner * 0.42), role("shadow"), { icon: "camera" });
      kit.close();
    });
    if (r.clicked) capture();
  };

  // The bar: Grid and Hide UI, the shutter, Reset and Exit.
  const bar = (): void => {
    kit.row("Bar", { width: grow(), gap: u(phone ? 4 : 18), align: "center", cross: "center", part: "Photo.bar", partClass: "panel" }, () => {
      if (tool("Grid", desc.gridLabel ?? "Grid", "grid", !!desc.grid)) {
        out.grid = true;
        kit.feedback("toggle", -1);
      }
      if (tool("Hide", desc.hideLabel ?? "Hide UI", "eye", null)) {
        out.hide = true;
        kit.feedback("toggle", -1);
      }
      shutter();
      if (tool("Reset", desc.resetLabel ?? "Reset", "rotate", null)) {
        out.reset = true;
        kit.feedback("confirm", -1);
      }
      if (tool("Exit", desc.exitLabel ?? "Exit", "cross", null)) out.exit = true;
    });
  };

  // A tab: its word, and a lit underline under the one shown (a chip on a phone).
  const tabEntry = (x: Api.PhotoTab): void => {
    const k = x.key ?? "";
    const on = !!tab && tab.key === k;
    const r = surface(k, {
      part: `Photo.tab${flags(on && "selected")}`,
      cls: portrait ? "chip" : "listItem",
      fill: portrait && on ? { ...role("primary"), a: 0.22 } : null,
      washBy: 0.12,
      direction: "column",
      gap: u(4),
      cross: "center",
      padding: xy(u(portrait ? 12 : 10), u(portrait ? 8 : 6)),
      radius: portrait ? 999 : Math.max(4, t.small),
      semantics: { role: "tab", name: x.label ?? k, selected: on },
    } as never, () => {
      text("Label", (x.label ?? k).toLocaleUpperCase(), { step: st(phone ? 13 : 15), font: "display", nowrap: true, color: on ? "ink" : "inkMuted" });
      if (!portrait) {
        kit.open("Line", { width: grow(), height: px(Math.max(2, u(3))) });
        kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0) }], ...(on ? { effects: [glow(u(6), token("primary", 0.5), 0)] } : {}) });
        kit.close();
      }
    });
    if (r.clicked && !on) {
      out.tab = k;
      kit.feedback("tab", -1);
    }
  };

  // A slider: its name and its reading (the control's own head) over the track.
  const controlRow = (c: Api.PhotoControl): void => {
    const k = c.key ?? "";
    const unit = c.unit ?? "";
    const r = valueSlider(k, c.value ?? 0, {
      min: c.min ?? 0,
      max: c.max ?? 1,
      step: c.step ?? 0,
      label: c.label ?? k,
      format: unit === "%" ? { percent: true } : { decimals: c.digits ?? 0, unit: /^[A-Za-z]/.test(unit) ? ` ${unit}` : unit },
      width: grow(),
      ...(c.origin != null ? { origin: c.origin } : {}),
    });
    if (r.changed) out.set = { key: k, value: r.value };
  };

  // A look's tile: the shot through it, its name in its own row under it, the chosen one rimmed and ticked.
  const lookTile = (l: Api.PhotoLook, chosen: string, size: number, which: "filter" | "frame"): void => {
    const k = l.key ?? "";
    const on = chosen === k;
    const r = surface(k, {
      part: `Photo.tile${flags(on && "selected")}`,
      cls: "listItem",
      fill: null,
      direction: "column",
      gap: u(4),
      cross: "center",
      width: px(size),
      padding: xy(0, 0),
      radius: Math.max(4, t.small),
      semantics: { role: "radio", name: l.name ?? k, checked: on } as never,
    } as never, () => {
      const ix = lastInteraction(k);
      const th = Math.round(size * 0.72);
      artSlot("Thumb", "Photo.thumb", { texture: l.image ?? null, kind: "thumbnail", width: px(size), height: px(th), radius: Math.max(4, t.small), label: "" }, () => {
        if (l.tint) {
          kit.open("Tint", { width: grow(), height: grow() });
          kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: paint(hex(l.tint)) }] });
          kit.close();
        }
        if (on) {
          kit.open("Tick", { width: px(u(22)), height: px(u(22)), align: "center", cross: "center" });
          kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("primary") }] });
          symbol("Mark", "check", u(14), role("onPrimary"));
          kit.close();
          over({ insets: { end: u(4), top: u(4) }, inert: true });
        }
      });
      kit.setDraw({ shape: { kind: "rect", radii: [t.small, t.small, t.small, t.small] }, fills: [], strokes: on ? [line(u(2), token("focus"))] : ix.hovered || ix.focused ? [line(u(1.5), token("ink", 0.4))] : [], effects: on ? [glow(u(8), token("focus", 0.4), 0)] : [] } as never);
      text("Name", l.name ?? k, { step: st(phone ? 12 : 13), nowrap: true, ellipsis: true, align: "center", width: grow(), color: on ? "ink" : "inkMuted" });
    });
    if (r.clicked && !on) {
      if (which === "filter") out.filter = k;
      else out.frame = k;
      kit.feedback("select", -1);
    }
  };

  // The tab's body: its sliders, or its tiles in whole rows (and the filter's strength).
  const body = (innerW: number): void => {
    const k = tab?.key ?? "";
    kit.openScroll("Rows", { width: grow(), height: grow(), gap: u(phone ? 10 : 14), padding: xy(u(2), u(2)), fade: 24, fadeInto: "shadow" } as never);
    if (LOOK_TABS.has(k)) {
      const looks = k === "filters" ? desc.filters ?? [] : desc.frames ?? [];
      const chosen = k === "filters" ? desc.filter ?? "" : desc.frame ?? "";
      const gap = u(8);
      const want = u(phone ? 96 : 108) * Math.min(1.25, ts);
      const cols = Math.max(2, Math.floor((innerW + gap) / (want + gap)));
      const size = Math.floor((innerW - gap * (cols - 1)) / cols);
      // Whole rows, straight in the scroll (a column between them would be squeezed to its height).
      for (let r0 = 0; r0 < looks.length; r0 += cols) {
        kit.row(kit.idi("Line", r0 / cols), { gap }, () => looks.slice(r0, r0 + cols).forEach((l) => lookTile(l, chosen, size, k === "filters" ? "filter" : "frame")));
      }
      if (k === "filters" && desc.strength != null) {
        const r = valueSlider("Strength", clamp(desc.strength ?? 0, 0, 1), { min: 0, max: 1, step: 0.05, format: { percent: true }, width: grow(), label: desc.strengthLabel ?? "Strength", disabled: !desc.filter || desc.filter === (desc.filters ?? [])[0]?.key });
        if (r.changed) out.strength = r.value;
      }
    } else for (const c of desc.controls ?? []) if ((c.tab ?? "") === k) controlRow(c);
    kit.close();
    if (LOOK_TABS.has(k)) kit.semantics({ role: "radiogroup", name: tab?.label ?? k });
  };

  // The panel: glass at the end side on a desktop, a sheet over the lower third on a phone in portrait.
  const panel = (w: number, h: Api.Size): void => {
    kit.column("Panel", { width: px(w), height: h, gap: u(phone ? 8 : 14), padding: xy(u(phone ? 14 : 20), u(phone ? 12 : 18)), part: "Photo.panel", partClass: "panel" }, () => {
      const rr = Math.max(8, t.panel);
      kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("panel", 0.78) }, vgrad([[0, token("sheen", 0.08)], [1, token("sheen", 0)]])], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(8), u(28), 0.5)] });
      if (!portrait) eyebrow("Title", desc.title ?? "Photo mode");
      if (portrait) {
        kit.openScroll("Tabs", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal" } as never);
        kit.open("Chips", { direction: "row", gap: u(6) });
        tabs.forEach(tabEntry);
        kit.close();
        kit.close();
      } else kit.row("Tabs", { width: grow(), gap: u(4), wrap: true } as never, () => tabs.forEach(tabEntry));
      kit.semantics({ role: "tablist", name: desc.title ?? "Photo mode" });
      body(w - 2 * u(phone ? 14 : 20) - 4);
    });
    kit.semantics({ role: "region", name: desc.title ?? "Photo mode" });
  };

  if (hidden) {
    kit.spacer("Gap", grow());
    kit.row("Hint", { gap: u(8), cross: "center", padding: xy(u(14), u(8)), part: "Photo.hint", partClass: "badge" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.4) }] });
      symbol("Eye", "eye", u(14), role("inkMuted"), { icon: "eye" });
      text("Text", desc.hiddenHint ?? "Press any key to show the controls", { step: st(13), color: "inkMuted", nowrap: true, ellipsis: true });
    });
    kit.props({ align: "center", position: "absolute", insets: { centerX: true, bottom: u(phone ? 12 : 24) }, inert: true, z: 3 } as never);
    kit.semantics({ role: "status", name: desc.hiddenHint ?? "Press any key to show the controls" });
  } else if (portrait) {
    kit.spacer("Gap", grow());
    panel(Math.round(W), px(Math.round(Math.max(H * 0.34, u(250) * Math.min(1.4, ts)))));
    bar();
  } else {
    kit.row("Top", { width: grow(), height: grow(), gap: 0 }, () => {
      kit.spacer("Gap", grow());
      panel(Math.round(Math.min(u(400), W * 0.3) * Math.min(1.3, ts)), grow());
    });
    bar();
  }

  // The shutter's flash: a white wash fading over flashMs (held still under reduced motion).
  const flashTicks = Math.max(1, Math.round(((desc.flashMs ?? 260) / 1000) * tps()));
  if (shotAt >= 0 && tick - shotAt < flashTicks || out.capture) {
    const f = out.capture ? 1 : 1 - (tick - shotAt) / flashTicks;
    kit.open("Flash", { width: grow(), height: grow(), part: "Photo.flash", partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { r: 1, g: 1, b: 1, a: still() ? 0.5 : 0.85 * clamp(f, 0, 1) } as never }] });
    kit.close();
    over({ insets: { start: 0, end: 0, top: 0, bottom: 0 }, inert: true, z: 8 });
  }

  kit.close();
  kit.semantics({ role: "region", name: desc.title ?? "Photo mode" });
  rememberRoom("ph", id);
  kit.remember(`~ph.shot.${id}`, out.capture ? tick : shotAt);
  return out;
}
