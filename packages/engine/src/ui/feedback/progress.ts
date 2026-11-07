// og.ui.progressBar, og.ui.progressRing and og.ui.skeletonGroup: progress
// and loading (docs/ui.md §2 "progress and spinners, skeletons"). Read
// against the download and install bars of console stores (a buffer ahead
// of the fill, the value in words at the end), Material's indeterminate
// ring (the arc grows and shrinks as it turns, so it reads as working at
// any size) and the skeleton screens of shop and social tabs (one sheen
// sweeping the whole group, not each piece on a clock of its own).

import type * as Api from "@origingame/engine";
import { decimalNumber } from "../data/format.ts";
import { TAU, a11y, absolute, arc, bp, box, clamp01, edge, frame, groups, inner, kit, leaf, lit, open, close, pct, phase, px, reduced, role, sel, solid, sound, spring, stroke, text, toneRole, token, vgrad, glow, type Rgba, ts, apply, tsFit } from "./core.ts";

function toneInk(tone: Api.Tone | undefined): Rgba {
  return role(toneRole(tone, "primary"));
}

export function progressBar(id: string, desc: Api.ProgressBarDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const style: Api.ProgressStyle = desc.style ?? "bar";
  const target = clamp01(desc.value ?? 0);
  const thick = desc.thickness || (bp().phone ? 6 : 8);
  const ink = toneInk(desc.tone);
  const r = thick / 2;
  const showValue = desc.showValue || !!desc.valueText;
  const valueText = desc.valueText || `${decimalNumber(Math.round(target * 100), 0)}%`;
  kit.column(id, { gap: tk.xs, width: desc.width ?? { kind: "grow" } }, () => {
    const v = style === "indeterminate" ? 0 : clamp01(spring("value", target));
    const done = style !== "indeterminate" && target >= 1;
    const e = edge("done", done ? 1 : 0);
    sound("success", e.changed && done);
    if (desc.label || showValue) {
      kit.row("Head", { gap: tk.sm, cross: "end", width: { kind: "grow" } }, () => {
        // Outlined, so a bar laid over a bright scene still reads (4.5:1 against its own outline).
        text("Label", desc.label ?? "", { step: ts(14), color: token("ink"), width: { kind: "grow" }, nowrap: true, ellipsis: true, outline: 0.8 });
        if (showValue && style !== "indeterminate") text("Value", valueText, { step: ts(14), font: "mono", nowrap: true, outline: 0.8 });
      });
    }
    if (style === "steps") {
      const n = Math.max(1, desc.steps || 5);
      kit.row("Track", { gap: 3, width: { kind: "grow" } }, () => {
        for (let i = 0; i < n; i++) {
          const part = clamp01(v * n - i);
          box(kit.idi("Step", i), { width: { kind: "grow" }, height: px(thick), part: sel("ProgressBar", "step", part >= 1 ? "filled" : null), partClass: "bar" }, {
            shape: { kind: "rect", radii: [r, r, r, r] },
            fills: [solid(token("controlStrong"))],
            effects: [inner(1, 2, token("shadow", 0.35))],
            clip: "shape",
          }, () => {
            if (part > 0) leaf("Fill", { width: pct(part), height: pct(1) }, { fills: [vgrad([[0, lit(ink)], [1, lit(ink, 0.82)]])] });
          });
        }
      });
    } else {
      const trackPath = open("Track", { width: { kind: "grow" }, height: px(thick), part: sel("ProgressBar", "track", style), partClass: "bar" }, {
        shape: { kind: "rect", radii: [r, r, r, r] },
        fills: [solid(token("controlStrong"))],
        effects: [inner(1, 2, token("shadow", 0.4))],
        clip: "shape",
      });
      const w = kit.bounds(trackPath)?.width ?? 240;
      if (style === "indeterminate") {
        // A band sweeping along the track, easing at each end.
        const p = phase(0.7);
        const band = 0.36;
        const x = (p * (1 + band) - band) * w;
        leaf("Fill", { width: pct(band), height: pct(1), part: sel("ProgressBar", "fill", desc.tone ?? null), partClass: "bar" }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [vgrad([[0, lit(ink, 0)], [0.5, lit(ink)], [1, lit(ink, 0)]], 90)],
        }, absolute({ start: 0, top: 0 }, { transform: { x: reduced() ? w * 0.32 : x } }));
      } else {
        if ((desc.buffer ?? 0) > v) {
          leaf("Buffer", { width: pct(clamp01(desc.buffer ?? 0)), height: pct(1), part: sel("ProgressBar", "buffer"), partClass: "bar" }, {
            shape: { kind: "rect", radii: [r, r, r, r] },
            fills: [solid(lit(ink, 0.3))],
          }, absolute({ start: 0, top: 0 }));
        }
        const flash = e.since < 30 && done ? 1 - e.since / 30 : 0;
        leaf("Fill", { width: pct(v), height: pct(1), part: sel("ProgressBar", "fill", desc.tone ?? null, done ? "done" : null), partClass: "bar" }, {
          shape: { kind: "rect", radii: [r, r, r, r] },
          fills: [vgrad([[0, lit(ink)], [0.55, lit(ink, 0.9)], [1, lit(ink, 0.78)]])],
          strokes: [stroke(1, token("sheen", 0.35), "inside")],
          effects: flash > 0 ? [glow(10 * flash, lit(ink, flash))] : [],
        }, absolute({ start: 0, top: 0 }));
        if (desc.sheen !== false && !done && v > 0.04 && !reduced()) {
          const sw = Math.max(24, w * 0.18);
          const x = phase(0.45) * (w * v + sw) - sw;
          leaf("Sheen", { width: px(sw), height: pct(1), part: sel("ProgressBar", "sheen") }, {
            fills: [vgrad([[0, token("sheen", 0)], [0.5, token("sheen", 0.45)], [1, token("sheen", 0)]], 90)],
            blend: "screen",
          }, absolute({ start: 0, top: 0 }, { transform: { x }, opacity: x + sw > w * v ? 0 : 1 }));
        }
      }
      close();
    }
  });
  apply({
    a11y: a11y("progressbar", desc.label || "Progress", style === "indeterminate"
      ? { busy: true }
      : { value: valueText, valueMin: 0, valueMax: 100, valueNow: Math.round(target * 100) }),
    ...groups({ progress: target >= 1 ? "done" : "working", mode: style === "indeterminate" ? "indeterminate" : "determinate" }, { progress: ["working", "done"], mode: ["determinate", "indeterminate"] }),
  });
}

export function progressRing(id: string, desc: Api.ProgressRingDesc = {}): void {
  const size = desc.size || 48;
  const thick = desc.thickness || Math.max(3, size / 10);
  const ink = toneInk(desc.tone);
  const spinning = desc.value === null || desc.value === undefined;
  const target = clamp01(desc.value ?? 0);
  const name = desc.name || (spinning ? "Loading" : "Progress");
  box(id, { width: px(size), height: px(size), align: "center", cross: "center" }, undefined, () => {
    leaf("Track", { width: px(size), height: px(size), part: sel("ProgressRing", "track"), partClass: "bar" }, {
      shape: arc(1, thick),
      fills: [solid(token("controlStrong"))],
    }, absolute({ start: 0, top: 0 }));
    let shape: Api.DrawShape;
    let opacity = 1;
    if (spinning) {
      if (reduced()) {
        shape = arc(0.3, thick);
        opacity = 0.55 + 0.45 * Math.cos(kit.seconds() * TAU * 0.8) ** 2;
      } else {
        // The arc grows and shrinks as it turns (Material's indeterminate ring).
        const turn = phase(0.75) * TAU;
        const g = phase(0.42);
        const sweep = 0.08 + 0.62 * (0.5 - 0.5 * Math.cos(g * TAU));
        shape = { kind: "arc", start: turn + g * TAU * 0.5 - Math.PI / 2, sweep: sweep * TAU, thickness: thick, cap: "round" };
      }
    } else {
      shape = arc(clamp01(spring("value", target)), thick);
    }
    leaf("Fill", { width: px(size), height: px(size), part: sel("ProgressRing", "fill", desc.tone ?? null, spinning ? "spinning" : null), partClass: "bar" }, {
      shape,
      fills: [solid(lit(ink))],
    }, absolute({ start: 0, top: 0 }, { opacity }));
    const label = desc.label ?? (spinning ? "" : `${decimalNumber(Math.round(target * 100), 0)}%`);
    if (label && size >= 40) text("Label", label, { step: tsFit(size >= 72 ? 18 : 13, size * 0.3), font: "mono", nowrap: true, outline: 0.8 });
  }, {
    a11y: a11y("progressbar", name, spinning ? { busy: true } : { value: `${decimalNumber(Math.round(target * 100), 0)}%`, valueMin: 0, valueMax: 100, valueNow: Math.round(target * 100) }),
    ...groups({ mode: spinning ? "indeterminate" : "determinate" }, { mode: ["determinate", "indeterminate"] }),
  });
}

/** One skeleton piece, its fill a band of the group's sheen at the piece's place in the group. */
function piece(id: string, size: { w: Api.Size; h: number }, radius: number, shape: Api.SkeletonShape, parent: string, group: string, band: number): void {
  const pathOf = parent ? `${parent}/${id}` : id;
  const g = kit.bounds(group);
  const p = kit.bounds(pathOf);
  const base = token("controlStrong");
  const shine = token("sheen", 0.5);
  let fills: Api.Fill[] = [solid(base)];
  if (g && p && p.width > 0 && band >= -1) {
    const c = (g.x + band * g.width - p.x) / p.width;
    const bw = (g.width * 0.18) / p.width;
    const s = [c - bw, c, c + bw].map(clamp01) as [number, number, number];
    if (s[2] > 0 && s[0] < 1) fills = [solid(base), vgrad([[s[0], token("sheen", 0)], [s[1], shine], [s[2], token("sheen", 0)]], 90)];
  }
  leaf(id, { width: size.w, height: px(size.h), part: sel("Skeleton", "piece", shape), partClass: "card" }, {
    shape: { kind: "rect", radii: [radius, radius, radius, radius] },
    fills,
  });
}

export function skeletonGroup(id: string, desc: Api.SkeletonGroupDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const shape: Api.SkeletonShape = desc.shape ?? "lines";
  const count = Math.max(1, desc.count || 1);
  const lineH = Math.round((f.theme.typeRamp.body?.size ?? 16) * 0.75);
  // The sheen crosses the group once a second and a half, then rests; under reduced motion the group breathes instead.
  const still = reduced();
  const band = still ? -2 : phase(0.55) * 1.6 - 0.3;
  const breath = still ? 0.6 + 0.4 * Math.cos(kit.seconds() * TAU * 0.5) ** 2 : 1;
  let n = 0;
  const next = () => kit.idi("Piece", n++);
  const path = open(id, { direction: "column", gap: tk.md, width: desc.width ?? { kind: "grow" } });
  for (let k = 0; k < count; k++) {
    kit.column(kit.idi("Item", k), { gap: tk.sm, width: { kind: "grow" } }, () => {
      const item = `${path}/${kit.idi("Item", k)}`;
      if (shape === "avatar") piece(next(), { w: px(desc.height || 48), h: desc.height || 48 }, 999, shape, item, path, band);
      else if (shape === "block") piece(next(), { w: { kind: "grow" }, h: desc.height || 120 }, tk.control, shape, item, path, band);
      else if (shape === "row") {
        kit.row("Row", { gap: tk.md, cross: "center", width: { kind: "grow" } }, () => {
          const row = `${item}/Row`;
          piece(next(), { w: px(40), h: 40 }, 999, shape, row, path, band);
          kit.column("Lines", { gap: tk.xs, width: { kind: "grow" } }, () => {
            piece(next(), { w: { kind: "percent", fraction: 0.6 }, h: lineH }, lineH / 2, shape, `${row}/Lines`, path, band);
            piece(next(), { w: { kind: "percent", fraction: 0.85 }, h: lineH * 0.8 }, lineH / 2, shape, `${row}/Lines`, path, band);
          });
        });
      } else if (shape === "card") {
        piece(next(), { w: { kind: "grow" }, h: desc.height || 120 }, tk.control, shape, item, path, band);
        piece(next(), { w: { kind: "percent", fraction: 0.55 }, h: lineH * 1.2 }, lineH / 2, shape, item, path, band);
        piece(next(), { w: { kind: "grow" }, h: lineH }, lineH / 2, shape, item, path, band);
        piece(next(), { w: { kind: "percent", fraction: 0.7 }, h: lineH }, lineH / 2, shape, item, path, band);
      } else {
        const lines = Math.max(1, desc.lines || 3);
        for (let i = 0; i < lines; i++) piece(next(), { w: i === lines - 1 && lines > 1 ? { kind: "percent", fraction: 0.62 } : { kind: "grow" }, h: lineH }, lineH / 2, shape, item, path, band);
      }
    });
  }
  close({ opacity: breath, a11y: a11y("progressbar", desc.name ?? "Loading", { busy: true }) });
}
