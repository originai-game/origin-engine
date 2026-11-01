// Charts (docs/ui.md §11, Data): line, area, bar (grouped or stacked),
// radar, pie and donut. Marks are drawn with og.ui.setDraw (paths, filled
// paths, arcs, polygons) in the style's series colours (§12.2); the axes,
// the legend and the tooltip are labels and parts a style dresses
// (`Chart.tooltip`, `Chart.legend`). Every category is a target the
// pointer, a touch, keys and a pad reach, so a reader walks the values
// one by one; the figure says what it shows in its description.

import type * as Api from "@origingame/engine";
import {
  clamp, fit, flags, frame, grow, kit, mix, over, paint, pointerFrom, px, rememberPath, role, rtl, series, spring, still, surface, text, token, xy,
  type Rgba,
} from "../containers/base.ts";
import { dataState } from "./table.ts";
import { formatNumber, niceTicks } from "./format.ts";

const PAD = 10;
/** Past this many points values stop springing one by one (the reveal still plays). */
const ANIMATED_POINTS = 240;

type Kind = "line" | "area" | "bar" | "radar" | "pie" | "donut";

interface Ctx {
  /** What its parts' paths are remembered under (`chart:<id>`). */
  key: string;
  desc: Api.ChartDesc;
  kind: Kind;
  values: number[][];
  colours: Rgba[];
  names: string[];
  cats: readonly string[];
  n: number;
  /** The category under the pointer or the focus last tick (-1: none). */
  hot: number;
  /** 0 to 1 as the marks first appear. */
  reveal: number;
  fmt: (v: number) => string;
}

/**
 * A chart of one or more series over categories. Line and area charts
 * draw smooth (monotone) curves that never overshoot the data, an area
 * filled with its series' colour fading to the baseline; bar charts group
 * the series side by side or stack them. A radar draws each series as a
 * polygon over rings; a pie or a donut draws the first series' values as
 * wedges with a legend that lists them. Hovering or focusing a category
 * (a touch on a phone) shows its values in a tooltip and a crosshair, a
 * press answers it. The marks grow in on the panel spring the first time
 * and move to new values on the ui spring; reduced motion shows them at
 * once. Empty, loading and error states replace the plot.
 */
export function chart(id: string, desc: Api.ChartDesc): Api.ChartResult {
  const { t, phone } = frame();
  const kind = (desc.kind ?? "line") as Kind;
  const list = desc.series ?? [];
  const cats = desc.categories ?? [];
  const n = Math.max(cats.length, ...list.map((s) => s.values?.length ?? 0), 0);
  const colours = list.map((s, k) => (s.color ? hexOr(s.color, series(k)) : series(k)));
  const values = list.map((s) => Array.from({ length: n }, (_, i) => Number(s.values?.[i] ?? 0) || 0));
  const empty = n === 0 || list.length === 0 || ((kind === "pie" || kind === "donut") && (values[0] ?? []).every((v) => v <= 0));
  const height = desc.height ?? (phone ? 200 : 240);
  let hovered: number | null = null;
  let clicked: number | null = null;
  let retry = false;
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: t.sm, cross: "stretch", part: `Chart.frame${flags(kind)}`, partClass: "panel" });
  try {
    if (desc.title || desc.subtitle) {
      kit.column("Header", { width: grow(), gap: 2 }, () => {
        if (desc.title) text("Title", desc.title, { step: "base", heading: true, width: grow(), nowrap: true, ellipsis: true });
        if (desc.subtitle) text("Subtitle", desc.subtitle, { step: "caption", color: "inkMuted", width: grow(), nowrap: true, ellipsis: true });
      });
    }
    if (desc.loading || desc.error || empty) {
      kit.open("Body", { direction: "column", width: grow(), height: px(height), align: "center", cross: "stretch" });
      retry = dataState("State", { ...desc, emptyLabel: desc.emptyLabel || "No data yet" }, 4, 28);
      kit.close();
    } else {
      const hot = Math.round(kit.memory("~hot", -1));
      const W = kit.memory("~w", 0);
      const reveal = still() ? 1 : kit.animate("~reveal", W > 0 ? 1 : 0, spring("panel"));
      const cx: Ctx = {
        key: `chart:${id}`, desc, kind, values, colours, cats, n, hot, reveal,
        names: list.map((s, k) => s.label || `${k + 1}`),
        fmt: (v) => formatNumber(v, desc.format, desc.digits ?? 0, desc.unit ?? ""),
      };
      if ((desc.legend ?? list.length > 1) && kind !== "pie" && kind !== "donut") legend("Legend", cx);
      const r = kind === "pie" || kind === "donut" ? pie(cx, W, height) : kind === "radar" && n >= 3 ? radar(cx, W, height) : cartesian(cx, W, height);
      hovered = r.hovered;
      clicked = r.clicked;
      kit.remember("~hot", hovered ?? -1);
    }
  } finally {
    kit.close();
  }
  kit.semantics({ role: "figure", name: desc.label || desc.title || "", description: summary(kind, list, values, desc), busy: desc.loading ?? false });
  if (clicked !== null) kit.feedback("select", -1);
  return { hovered, clicked, retry };
}

function hexOr(s: string, fallback: Rgba): Rgba {
  const h = s.replace("#", "");
  if (!/^[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(h)) return fallback;
  const n = (k: number) => parseInt(h.slice(k, k + 2), 16) / 255;
  return { r: n(0), g: n(2), b: n(4), a: h.length >= 8 ? n(6) : 1 };
}

/** What the figure shows, for a screen reader: the kind, and each series' range and last value. */
function summary(kind: string, list: readonly Api.ChartSeries[], values: number[][], desc: Api.ChartDesc): string {
  const f = (v: number) => formatNumber(v, desc.format, desc.digits ?? 0, desc.unit ?? "");
  const name = { line: "Line chart", area: "Area chart", bar: "Bar chart", radar: "Radar chart", pie: "Pie chart", donut: "Donut chart" }[kind] ?? "Chart";
  const parts = list.map((s, k) => {
    const v = values[k] ?? [];
    if (v.length === 0) return s.label ?? "";
    return `${s.label ?? ""}: ${f(Math.min(...v))} to ${f(Math.max(...v))}, last ${f(v[v.length - 1]!)}`;
  });
  return [name, ...parts].join("; ");
}

// ── legend and tooltip ──────────────────────────────────────────────────

function swatch(id: string, c: Rgba, size = 10, round = false): void {
  kit.open(id, { width: px(size), height: px(size), radius: round ? size / 2 : Math.min(3, size / 3), background: paint(c), part: "Chart.swatch", partClass: "chip" });
  kit.close();
}

function legend(id: string, cx: Ctx): void {
  const { t } = frame();
  kit.row(id, { width: grow(), gap: t.md, wrap: true, cross: "center", part: "Chart.legend", partClass: "panel" }, () => {
    cx.names.forEach((name, k) => {
      kit.row(kit.idi("Key", k), { gap: t.xs, cross: "center" }, () => {
        swatch("Swatch", cx.colours[k]!, 10, cx.kind === "line");
        text("Label", name, { step: "caption", color: "inkMuted", nowrap: true });
      });
    });
  });
}

/** The values of category `i`: its name, then each series' swatch, name and value. Placed by the caller. */
function tooltip(id: string, cx: Ctx, i: number, only?: number): void {
  const { t } = frame();
  kit.open(id, { direction: "column", gap: t.xxs, padding: xy(t.sm, t.xs), radius: t.control, background: token("panelRaised"), part: "Chart.tooltip", partClass: "tooltip" });
  text("Category", cx.cats[i] ?? `${i + 1}`, { step: "caption", color: "inkMuted", nowrap: true });
  cx.values.forEach((v, k) => {
    if (only !== undefined && k !== only) return;
    kit.row(kit.idi("Value", k), { gap: t.xs, cross: "center" }, () => {
      swatch("Swatch", cx.colours[k]!, 8, true);
      if (cx.values.length > 1) text("Name", cx.names[k]!, { step: "caption", color: "inkMuted", nowrap: true });
      text("Number", cx.fmt(v[i] ?? 0), { step: "sm", font: "mono", nowrap: true });
    });
  });
  kit.close();
}

/** A label in a box of `width` centred on a start-relative x, at `top`, taking no input. */
function tag(id: string, value: string, xStart: number, top: number, width: number, align: Api.TextAlign = "center"): void {
  text(id, value, { step: "caption", color: "inkMuted", nowrap: true, ellipsis: true, align, width: px(width), font: "mono" });
  over({ insets: { start: align === "center" ? xStart - width / 2 : align === "end" ? xStart - width : xStart, top }, inert: true });
}

/** An element covering the plot, positioned at its start, drawn with `draw`. */
function layer(id: string, W: number, H: number, draw: Api.DrawDesc, z = 0, props: Api.ElementProps = {}): void {
  kit.open(id, { width: px(W), height: px(H) });
  kit.setDraw(draw);
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true, z, ...props });
}

// ── line, area, bar ─────────────────────────────────────────────────────

/** Monotone cubic (Fritsch–Carlson) through the points as a path's curve points: smooth, never past the data. */
function monotone(p: Array<{ x: number; y: number }>): Array<{ x: number; y: number }> {
  const n = p.length;
  if (n < 3) return n === 2 ? [p[0]!, p[0]!, p[1]!, p[1]!] : [...p];
  const d: number[] = [];
  for (let i = 0; i < n - 1; i++) d.push((p[i + 1]!.y - p[i]!.y) / Math.max(1e-6, p[i + 1]!.x - p[i]!.x));
  const m: number[] = [d[0]!];
  for (let i = 1; i < n - 1; i++) m.push(d[i - 1]! * d[i]! <= 0 ? 0 : (d[i - 1]! + d[i]!) / 2);
  m.push(d[n - 2]!);
  for (let i = 0; i < n - 1; i++) {
    if (d[i] === 0) {
      m[i] = 0;
      m[i + 1] = 0;
      continue;
    }
    const a = m[i]! / d[i]!;
    const b = m[i + 1]! / d[i]!;
    const s = a * a + b * b;
    if (s > 9) {
      const k = 3 / Math.sqrt(s);
      m[i] = k * a * d[i]!;
      m[i + 1] = k * b * d[i]!;
    }
  }
  const out = [p[0]!];
  for (let i = 0; i < n - 1; i++) {
    const dx = (p[i + 1]!.x - p[i]!.x) / 3;
    out.push({ x: p[i]!.x + dx, y: p[i]!.y + m[i]! * dx }, { x: p[i + 1]!.x - dx, y: p[i + 1]!.y - m[i + 1]! * dx }, p[i + 1]!);
  }
  return out;
}

/** Straight segments as curve points (each segment a cubic whose controls sit on its ends). */
function straight(p: Array<{ x: number; y: number }>): Array<{ x: number; y: number }> {
  const out = p.length ? [p[0]!] : [];
  for (let i = 1; i < p.length; i++) out.push(p[i - 1]!, p[i]!, p[i]!);
  return out;
}

function cartesian(cx: Ctx, W: number, H: number): { hovered: number | null; clicked: number | null } {
  const { t, phone } = frame();
  const { desc, kind, n, values, hot } = cx;
  const bar = kind === "bar" || kind === "radar";
  const stacked = (desc.stacked ?? false) && kind !== "line";
  const flip = rtl();
  // The value range: stacked sums for stacked charts, and zero kept in view for bars and areas.
  let lo = Infinity;
  let hi = -Infinity;
  for (let i = 0; i < n; i++) {
    if (stacked) {
      let up = 0;
      let down = 0;
      for (const v of values) (v[i]! >= 0 ? (up += v[i]!) : (down += v[i]!));
      lo = Math.min(lo, down);
      hi = Math.max(hi, up);
    } else for (const v of values) (lo = Math.min(lo, v[i]!), hi = Math.max(hi, v[i]!));
  }
  if (bar || kind === "area") (lo = Math.min(0, lo), hi = Math.max(0, hi));
  const tk = niceTicks(desc.min ?? lo, desc.max ?? hi, phone ? 3 : 4);
  const yLo = desc.min ?? tk.lo;
  const yHi = desc.max ?? tk.hi;
  const ticks = tk.ticks.filter((v) => v >= yLo - 1e-9 && v <= yHi + 1e-9);
  const tickDigits = Math.max(desc.digits ?? 0, tk.step < 1 ? Math.min(3, Math.ceil(-Math.log10(tk.step))) : 0);
  const tickText = (v: number) => formatNumber(v, desc.format === "plain" ? "grouped" : desc.format, desc.format === "percent" ? Math.max(0, tickDigits - 2) : tickDigits, "");
  let hovered: number | null = null;
  let clicked: number | null = null;
  // Room above the top line and below the bottom one for half a tick label (more at a large text size).
  const padY = Math.max(PAD, Math.ceil((frame().theme.typeRamp.caption?.lineHeight ?? 18) / 2));
  kit.row("Plot", { width: grow(), height: px(H), gap: t.xs, cross: "stretch" }, () => {
    // The value axis: the tick labels, top to bottom, each centred on its line.
    if (desc.axis !== false) {
      kit.column("ValueAxis", { height: grow(), align: "spaceBetween", cross: "end", padding: xy(0, padY) }, () => {
        [...ticks].reverse().forEach((v, k) => {
          kit.row(kit.idi("Tick", k), { height: px(0), cross: "center" }, () => text("Text", tickText(v), { step: "caption", color: "inkFaint", nowrap: true, font: "mono" }));
        });
      });
    }
    kit.open("Area", { direction: "column", width: grow(), height: grow() });
    const Wi = Math.max(0, W - 2 * PAD);
    const Hi = Math.max(0, H - 2 * padY);
    const y = (v: number) => padY + ((yHi - v) / Math.max(1e-9, yHi - yLo)) * Hi;
    const band = n > 0 ? Wi / n : Wi;
    const xs = (i: number) => (bar ? PAD + band * (i + 0.5) : n <= 1 ? PAD + Wi / 2 : PAD + (i * Wi) / (n - 1));
    const phys = (x: number) => (flip ? W - x : x);
    if (W > 0) {
      // Grid lines, the zero line stronger.
      ticks.forEach((v, k) => {
        kit.open(kit.idi("Grid", k), { width: px(Wi), height: px(1), background: token(Math.abs(v) < 1e-9 ? "lineStrong" : "line", Math.abs(v) < 1e-9 ? 1 : desc.grid === false ? 0 : 0.7), part: `Chart.grid${flags(Math.abs(v) < 1e-9 && "zero")}`, partClass: "bar" });
        kit.close();
        over({ insets: { start: PAD, top: Math.round(y(v)) }, inert: true });
      });
      const base = y(clamp(0, yLo, yHi));
      const animated = n * values.length <= ANIMATED_POINTS;
      const shown = values.map((v, s) => v.map((val, i) => {
        const to = animated && !still() ? kit.animate(`~v${s}.${i}`, val, spring("ui")) : val;
        const b0 = clamp(0, yLo, yHi);
        return b0 + (to - b0) * cx.reveal;
      }));
      if (bar) bars(cx, shown, stacked, band, xs, y, base);
      else {
        // A stacked area's series sit on the ones before them.
        const tops = stacked ? shown.map((_, s) => shown[0]!.map((_, i) => shown.slice(0, s + 1).reduce((a, v) => a + v[i]!, 0))) : shown;
        let below: Array<{ x: number; y: number }> | null = null;
        tops.forEach((v, s) => {
          const c = cx.colours[s]!;
          const pts = v.map((val, i) => ({ x: phys(xs(i)), y: y(val) }));
          if (flip) pts.reverse();
          const curve = (desc.smooth ?? true) ? monotone(pts) : straight(pts);
          if (kind === "area") {
            const first = pts[0]!;
            const last = pts[pts.length - 1]!;
            const floor = below ?? straight([{ x: first.x, y: base }, { x: last.x, y: base }]);
            const back = [...floor].reverse();
            // Down from the last point, back along what it sits on, and up to the first.
            const outline = [...curve, last, back[0]!, back[0]!, ...back.slice(1), back[back.length - 1]!, first, first];
            below = curve;
            layer(kit.idi("Fill", s), W, H, {
              shape: { kind: "path", points: outline, curves: true, filled: true, width: 0 },
              fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: paint(c, 0.34) }, { offset: 1, color: paint(c, 0.02) }] }],
            }, s);
          }
          layer(kit.idi("Line", s), W, H, {
            shape: { kind: "path", points: curve, curves: true, width: phone ? 2.5 : 2.25, cap: "round", join: "round" },
            fills: [{ kind: "solid", color: paint(c) }],
          }, 10 + s);
          // Points: every one on a short series, the hovered one always, the last one on a long series.
          const dots = desc.dots ?? n <= 16;
          v.forEach((val, i) => {
            const on = i === hot;
            if (!dots && !on && i !== n - 1) return;
            const r = on ? 6 : 4;
            kit.open(kit.idi(`Dot${s}.`, i), { width: px(2 * r), height: px(2 * r) });
            kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("panelRaised") }], strokes: [{ width: on ? 3 : 2, align: "inside", paint: { kind: "solid", color: paint(c) } }] });
            kit.close();
            over({ insets: { start: xs(i) - r, top: y(val) - r }, inert: true, z: 20 + s, motion: spring("ui") });
          });
        });
        if (hot >= 0 && hot < n) {
          kit.open("Crosshair", { width: px(1), height: px(Hi), background: token("lineStrong"), part: "Chart.crosshair", partClass: "bar" });
          kit.close();
          over({ insets: { start: Math.round(xs(hot)), top: padY }, inert: true, z: 5 });
        }
      }
      // A target per category: hover, touch, focus and press.
      const cw = bar ? band : n <= 1 ? Wi : Wi / (n - 1);
      for (let i = 0; i < n; i++) {
        const x0 = clamp(xs(i) - cw / 2, 0, W);
        const x1 = clamp(xs(i) + cw / 2, 0, W);
        const r = surface(kit.idi("Category", i), {
          part: "Chart.category",
          cls: "slot",
          flags: [i === hot ? "hover" : "", desc.selected === i ? "selected" : ""],
          selected: desc.selected === i,
          fill: null,
          washBy: bar ? 0.06 : 0,
          width: px(Math.max(1, x1 - x0)),
          height: px(H),
          radius: 0,
          semantics: { role: "button", name: `${cx.cats[i] ?? i + 1}: ${values.map((v, k) => `${values.length > 1 ? `${cx.names[k]} ` : ""}${cx.fmt(v[i] ?? 0)}`).join(", ")}`, selected: desc.selected === i },
          layer: { insets: { start: x0, top: 0 }, z: 30 },
        }, () => {});
        if (r.hovered || r.focused || r.pressed) hovered = i;
        if (r.clicked) clicked = i;
      }
      if (hot >= 0 && hot < n) {
        const tw = kit.memory("~tw", 120);
        const right = xs(hot) + 14 + tw <= W - 2;
        const at = right ? xs(hot) + 14 : Math.max(2, xs(hot) - 14 - tw);
        tooltip("Tooltip", cx, hot);
        over({ insets: { start: at, top: PAD }, inert: true, z: 40, enter: { values: [{ prop: "opacity", value: 0 }], transition: spring("ui") } });
        const b = kit.bounds(kit.lastPath());
        if (b) kit.remember("~tw", b.width);
      }
    }
    kit.close();
  });
  // The plot's width, in the chart's memory (read before the row next tick).
  const area = kit.bounds(`${kit.lastPath()}/Area`);
  kit.remember("~w", area ? area.width : 0);
  // The category axis: as many labels as fit, evenly skipped.
  if (desc.axis !== false && cx.cats.length > 0) {
    const W = kit.memory("~w", 0);
    const axisStart = kit.memory("~ax", 0);
    kit.row("CategoryAxis", { width: grow(), cross: "start" }, () => {
      kit.spacer("Inset", px(axisStart));
      kit.open("Labels", { width: grow(), direction: "row" });
      text("Strut", "\u00a0", { step: "caption", width: px(1) });
      if (W > 0) {
        const Wi = W - 2 * PAD;
        const longest = Math.max(...cx.cats.map((c) => c.length));
        const each = Math.max(36, longest * 7 + 12);
        const fit = Math.max(1, Math.floor(Wi / each));
        const step = Math.max(1, Math.ceil(n / fit));
        const xs = (i: number) => (bar ? PAD + (Wi / n) * (i + 0.5) : n <= 1 ? PAD + Wi / 2 : PAD + (i * Wi) / (n - 1));
        for (let i = 0; i < n; i += step) {
          const w = Math.min(each, bar ? (Wi / n) * step : each);
          tag(kit.idi("Label", i), cx.cats[i] ?? "", clamp(xs(i), w / 2, W - w / 2), 0, w);
        }
      }
      kit.close();
    });
    // Where the plot starts after the value axis, for the next tick.
    const plot = kit.bounds(`${kit.lastPath().replace(/\/CategoryAxis$/, "")}/Plot/Area`);
    const row = kit.bounds(`${kit.lastPath().replace(/\/CategoryAxis$/, "")}/Plot`);
    if (plot && row) kit.remember("~ax", flip ? row.x + row.width - (plot.x + plot.width) : plot.x - row.x);
  }
  return { hovered, clicked };
}

function bars(cx: Ctx, shown: number[][], stacked: boolean, band: number, xs: (i: number) => number, y: (v: number) => number, base: number): void {
  const S = shown.length;
  const groupW = Math.min(band * (stacked || S === 1 ? 0.6 : 0.76), stacked || S === 1 ? 56 : 28 * S);
  const gap = stacked || S === 1 ? 0 : Math.min(3, groupW * 0.06);
  const bw = stacked ? groupW : Math.max(1, (groupW - gap * (S - 1)) / S);
  const radius = Math.min(4, bw / 2);
  for (let i = 0; i < cx.n; i++) {
    let up = 0;
    let down = 0;
    // The topmost segment of a stack is the one rounded.
    const lastUp = stacked ? shown.map((v) => v[i]!).map((v, s) => (v > 0 ? s : -1)).reduce((a, b) => Math.max(a, b), -1) : -1;
    const lastDown = stacked ? shown.map((v) => v[i]!).map((v, s) => (v < 0 ? s : -1)).reduce((a, b) => Math.max(a, b), -1) : -1;
    shown.forEach((v, s) => {
      const val = v[i]!;
      let top: number;
      let bottom: number;
      if (stacked) {
        if (val >= 0) (bottom = y(up), up += val, top = y(up));
        else (top = y(down), down += val, bottom = y(down));
      } else (top = Math.min(y(val), base), bottom = Math.max(y(val), base));
      const h = Math.max(0, bottom - top);
      if (h < 0.5) return;
      const x = xs(i) - groupW / 2 + (stacked ? 0 : s * (bw + gap));
      const round = !stacked || (val >= 0 ? s === lastUp : s === lastDown);
      const r = round ? Math.min(radius, h) : 0;
      const dim = cx.hot >= 0 && cx.hot !== i;
      kit.open(kit.idi(`Bar${s}.`, i), { width: px(bw), height: px(h), part: `Chart.bar${flags(cx.hot === i && "hover")}`, partClass: "chip" });
      kit.setDraw({
        shape: { kind: "rect", radii: val >= 0 ? [r, r, 0, 0] : [0, 0, r, r] },
        fills: [{ kind: "solid", color: paint(cx.hot === i ? mix(cx.colours[s]!, { r: 1, g: 1, b: 1, a: 1 }, 0.1) : cx.colours[s]!) }],
      });
      kit.close();
      over({ insets: { start: x, top }, inert: true, z: 2, opacity: dim ? 0.45 : 1, motion: spring("ui") });
    });
  }
}

// ── radar ───────────────────────────────────────────────────────────────

function radar(cx: Ctx, W: number, H: number): { hovered: number | null; clicked: number | null } {
  const { t, phone } = frame();
  const { n, values, hot, desc } = cx;
  const flip = rtl();
  let hovered: number | null = null;
  let clicked: number | null = null;
  kit.open("Plot", { direction: "column", width: grow(), height: px(H) });
  if (W > 0) {
    const room = phone ? 44 : 64;
    const R = Math.max(10, Math.min(W / 2 - room, H / 2 - 22));
    const c = { x: W / 2, y: H / 2 };
    const top = desc.max ?? niceTicks(0, Math.max(...values.flat(), 0), 4).hi;
    const ang = (k: number) => -Math.PI / 2 + (k * Math.PI * 2) / n;
    const at = (k: number, f: number) => {
      const a = ang(k);
      return { x: c.x + Math.cos(a) * R * f * (flip ? -1 : 1), y: c.y + Math.sin(a) * R * f };
    };
    // Rings and spokes.
    [0.25, 0.5, 0.75, 1].forEach((f, k) => {
      kit.open(kit.idi("Ring", k), { width: px(2 * R * f), height: px(2 * R * f) });
      kit.setDraw({ shape: { kind: "polygon", sides: n }, fills: k === 3 ? [{ kind: "solid", color: token("control", 0.35) }] : [], strokes: [{ width: 1, align: "center", paint: { kind: "solid", color: token(k === 3 ? "lineStrong" : "line") } }] });
      kit.close();
      over({ insets: { start: W / 2 - R * f, top: c.y - R * f }, inert: true });
    });
    const spokes: Array<{ x: number; y: number }> = [];
    for (let k = 0; k < n; k++) spokes.push(c, at(k, 1));
    layer("Spokes", W, H, { shape: { kind: "path", points: spokes, width: 1, cap: "butt", join: "round" }, fills: [{ kind: "solid", color: token("line") }] }, 1);
    values.forEach((v, s) => {
      const col = cx.colours[s]!;
      const pts = v.map((val, k) => {
        const to = still() ? val : kit.animate(`~v${s}.${k}`, val, spring("ui"));
        return at(k, clamp(to / Math.max(1e-9, top), 0, 1.1) * cx.reveal);
      });
      layer(kit.idi("Shape", s), W, H, {
        shape: { kind: "path", points: pts, closed: true, filled: true, width: 0, join: "round" },
        fills: [{ kind: "solid", color: paint(col, 0.2) }],
        strokes: [{ width: 2, align: "center", paint: { kind: "solid", color: paint(col) } }],
      }, 2 + s);
      pts.forEach((p, k) => {
        const on = k === hot;
        const r = on ? 5 : 3;
        kit.open(kit.idi(`Dot${s}.`, k), { width: px(2 * r), height: px(2 * r), radius: r, background: paint(col) });
        kit.close();
        over({ insets: { start: (flip ? W - p.x : p.x) - r, top: p.y - r }, inert: true, z: 10 + s });
      });
    });
    // The axes' labels, each a target: hover or focus shows its values.
    for (let k = 0; k < n; k++) {
      const p = at(k, 1);
      const a = ang(k);
      const dx = Math.cos(a) * (flip ? -1 : 1);
      const w = phone ? 72 : 96;
      const physRight = dx > 0.25;
      const physLeft = dx < -0.25;
      const lx = p.x + dx * 10;
      const ly = p.y + Math.sin(a) * 12;
      const physX = physRight ? lx : physLeft ? lx - w : lx - w / 2;
      const align: Api.TextAlign = physRight ? (flip ? "end" : "start") : physLeft ? (flip ? "start" : "end") : "center";
      const r = surface(kit.idi("Axis", k), {
        part: "Chart.axis",
        cls: "chip",
        flags: [k === hot ? "hover" : ""],
        fill: null,
        washBy: 0.06,
        width: px(w),
        height: fit(28),
        padding: xy(4, 0),
        align: align === "center" ? "center" : align === "end" ? "end" : "start",
        semantics: { role: "button", name: `${cx.cats[k] ?? k + 1}: ${values.map((v, s) => `${values.length > 1 ? `${cx.names[s]} ` : ""}${cx.fmt(v[k] ?? 0)}`).join(", ")}` },
        layer: { insets: { start: flip ? W - physX - w : physX, top: ly - 14 + (Math.sin(a) > 0.5 ? 6 : Math.sin(a) < -0.5 ? -6 : 0) }, z: 30 },
      }, () => text("Label", cx.cats[k] ?? "", { step: "caption", color: k === hot ? "ink" : "inkMuted", nowrap: true, ellipsis: true, align }));
      if (r.hovered || r.focused || r.pressed) hovered = k;
      if (r.clicked) clicked = k;
    }
    if (hot >= 0 && hot < n) {
      tooltip("Tooltip", cx, hot);
      over({ insets: { start: t.xs, top: t.xs }, inert: true, z: 40, enter: { values: [{ prop: "opacity", value: 0 }], transition: spring("ui") } });
    }
  }
  kit.close();
  const b = kit.bounds(kit.lastPath());
  kit.remember("~w", b ? b.width : 0);
  return { hovered, clicked };
}

// ── pie and donut ───────────────────────────────────────────────────────

function pie(cx: Ctx, W0: number, H: number): { hovered: number | null; clicked: number | null } {
  const { t, phone } = frame();
  const { desc, hot } = cx;
  const v = (cx.values[0] ?? []).map((x) => Math.max(0, x));
  const total = v.reduce((a, b) => a + b, 0);
  const donut = cx.kind === "donut";
  const flip = rtl();
  const D = Math.round(Math.min(H, phone ? Math.max(160, W0) : Math.max(160, W0 * 0.5)));
  const R = D / 2 - 8;
  const hole = donut ? clamp(desc.hole ?? 0.62, 0.2, 0.9) : 0;
  const colour = (k: number) => (cx.values.length > 1 ? cx.colours[k % cx.colours.length]! : series(k));
  const pct = (x: number) => formatNumber(total > 0 ? x / total : 0, "percent", (desc.digits ?? 0) > 0 ? 1 : 0);
  let hovered: number | null = null;
  let clicked: number | null = null;
  const stack = phone;
  // Every wedge's start and sweep, in the reading direction from 12 o'clock (mirrored under a
  // right-to-left locale).
  const sweepAll = Math.PI * 2 * cx.reveal;
  const first = -Math.PI / 2 - (1 - cx.reveal) * 0.6;
  const wedges: Array<{ a0: number; sweep: number }> = [];
  {
    let a = first;
    for (const val of v) {
      const sweep = total > 0 ? (val / total) * sweepAll : 0;
      wedges.push({ a0: a, sweep });
      a += sweep;
    }
  }
  const inner = donut ? R * hole : 0;
  // The pointer picks a wedge by its angle and distance from the centre, so anywhere inside a
  // wedge is that wedge (rectangles would overlap across its edges).
  const ptr = pointerFrom(`${cx.key}~plot`);
  let under = -1;
  if (ptr && ptr.d >= inner && ptr.d <= R + 6) {
    const phys = Math.atan2(ptr.dy, ptr.dx);
    const ang = flip ? Math.PI - phys : phys;
    const rel = (((ang - first) % (Math.PI * 2)) + Math.PI * 2) % (Math.PI * 2);
    let a = 0;
    under = wedges.findIndex((w) => {
      const hit = w.sweep > 0 && rel >= a && rel < a + w.sweep;
      a += w.sweep;
      return hit;
    });
  }
  kit.open("Body", { direction: stack ? "column" : "row", width: grow(), gap: t.lg, cross: "center" });
  kit.open("Plot", { width: px(D), height: px(D), interactive: true });
  {
    const c = D / 2;
    const gapA = v.filter((x) => x > 0).length > 1 ? Math.min(0.04, 3 / R) : 0;
    v.forEach((_val, k) => {
      const { a0, sweep } = wedges[k]!;
      if (sweep <= 0) return;
      const mid = a0 + sweep / 2;
      const on = k === hot;
      const pull = on && !still() ? 6 : 0;
      // Clockwise in the reading direction: mirrored under a right-to-left locale.
      const start = flip ? Math.PI - (a0 + sweep) : a0;
      const physMid = flip ? Math.PI - mid : mid;
      kit.open(kit.idi("Wedge", k), { width: px(2 * R), height: px(2 * R), part: `Chart.wedge${flags(on && "hover")}`, partClass: "chip" });
      kit.setDraw({ shape: { kind: "arc", start: start + gapA / 2, sweep: Math.max(0.001, sweep - gapA), thickness: donut ? R * (1 - hole) : R, cap: "butt" }, fills: [{ kind: "solid", color: paint(colour(k)) }] });
      kit.close();
      over({ insets: { start: c - R, top: c - R }, inert: true, z: on ? 2 : 1, opacity: hot >= 0 && !on ? 0.55 : 1, transform: { x: Math.cos(physMid) * pull * (flip ? -1 : 1), y: Math.sin(physMid) * pull }, motion: spring("ui") });
      kit.semantics({ role: "image", name: `${cx.cats[k] ?? k + 1}, ${cx.fmt(cx.values[0]![k]!)}, ${pct(v[k]!)}` });
    });
    if (donut) {
      const k = hot >= 0 && hot < v.length ? hot : -1;
      kit.column("Centre", { width: px(2 * R * hole - 12), height: px(2 * R * hole - 12), align: "center", cross: "center", gap: 0 }, () => {
        text("Value", k >= 0 ? cx.fmt(cx.values[0]![k]!) : cx.fmt(total), { step: phone ? "lg" : "xl", font: "display", nowrap: true, ellipsis: true, align: "center", width: grow() });
        text("Label", k >= 0 ? `${cx.cats[k] ?? ""} · ${pct(v[k]!)}` : desc.totalLabel || "Total", { step: "caption", color: "inkMuted", nowrap: true, ellipsis: true, align: "center", width: grow() });
      });
      over({ insets: { start: c - R * hole + 6, top: c - R * hole + 6 }, inert: true, z: 3 });
    }
  }
  const plot = kit.close() as Api.Response;
  rememberPath(`${cx.key}~plot`, kit.lastPath());
  if (under >= 0 && (plot.hovered || plot.pressed)) hovered = under;
  if (plot.clicked && under >= 0) clicked = under;
  // The legend: every wedge's swatch, name, value and share; each row a target keys and pads walk.
  kit.column("Legend", { width: stack ? grow() : grow(), gap: 0, cross: "stretch", part: "Chart.legend", partClass: "panel" }, () => {
    v.forEach((val, k) => {
      const on = k === hot;
      const r = surface(kit.idi("Row", k), {
        part: "Chart.legendRow",
        cls: "chip",
        flags: [on ? "hover" : ""],
        fill: null,
        washBy: 0.08,
        width: grow(),
        height: fit(phone ? 44 : 36),
        padding: xy(t.sm, 0),
        gap: t.sm,
        semantics: { role: "button", name: `${cx.cats[k] ?? k + 1}, ${cx.fmt(cx.values[0]![k]!)}, ${pct(val)}`, selected: desc.selected === k },
      }, () => {
        swatch("Swatch", colour(k), 10);
        text("Label", cx.cats[k] ?? `${k + 1}`, { step: "sm", nowrap: true, ellipsis: true, width: grow(), color: on ? "ink" : "inkMuted" });
        text("Value", cx.fmt(cx.values[0]![k]!), { step: "sm", font: "mono", nowrap: true });
        text("Share", pct(val), { step: "caption", font: "mono", color: "inkFaint", nowrap: true, width: px(44), align: "end" });
      });
      if (r.hovered || r.focused || r.pressed) hovered = k;
      if (r.clicked) clicked = k;
    });
  });
  kit.close();
  const b = kit.bounds(kit.lastPath());
  kit.remember("~w", b ? b.width : 0);
  return { hovered, clicked };
}
