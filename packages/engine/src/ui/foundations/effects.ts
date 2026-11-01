// Foundations (docs/ui.md §11): the particle emitter and the effect layer.
// Both are decoration: they take no input and say nothing to a screen
// reader; the emitter stops under reduced motion, the layer's sweep stands
// still.

import type * as Api from "@origingame/engine";
import { frame, grow, kit, over, still, token } from "../controls/base.ts";

type Preset = Omit<Api.ParticlesDesc, "rate" | "colors" | "fire" | "burst">;

const UP = -Math.PI / 2;
const DOWN = Math.PI / 2;

/** Each look's particles at size 1 and its rate a second. */
const PRESETS: Record<Api.EmitterPreset, { rate: number; colors: Api.ThemeColor[]; p: Preset }> = {
  sparkle: { rate: 5, colors: ["highlight", "gold", "primary"], p: { lifetimeMs: 1400, shape: "diamond", size: { x: 5, y: 5 }, travelMin: 6, travelMax: 22, direction: UP, spread: 1.2, gravity: -6, spin: 1, from: "area", endScale: 0, fade: true } },
  confetti: { rate: 18, colors: ["primary", "highlight", "success", "info", "gold"], p: { lifetimeMs: 2600, shape: "square", size: { x: 7, y: 4 }, travelMin: 30, travelMax: 90, direction: DOWN, spread: 0.6, gravity: 140, spin: 4, from: "edges", endScale: 0.9, fade: true } },
  embers: { rate: 10, colors: ["warn", "gold", "danger"], p: { lifetimeMs: 1800, shape: "circle", size: { x: 3, y: 3 }, travelMin: 40, travelMax: 110, direction: UP, spread: 0.5, gravity: -30, spin: 0, from: "area", endScale: 0.2, fade: true } },
  snow: { rate: 8, colors: ["ink"], p: { lifetimeMs: 4200, shape: "circle", size: { x: 4, y: 4 }, travelMin: 60, travelMax: 140, direction: DOWN, spread: 0.5, gravity: 10, spin: 0, from: "area", endScale: 0.8, fade: true } },
  bubbles: { rate: 4, colors: ["info", "primarySoft"], p: { lifetimeMs: 2400, shape: "circle", size: { x: 8, y: 8 }, travelMin: 30, travelMax: 80, direction: UP, spread: 0.4, gravity: -20, spin: 0, from: "area", endScale: 1.6, fade: true } },
  dust: { rate: 3, colors: ["inkFaint", "sheen"], p: { lifetimeMs: 5000, shape: "circle", size: { x: 2, y: 2 }, travelMin: 8, travelMax: 30, direction: UP, spread: Math.PI * 2, gravity: 0, spin: 0, from: "area", endScale: 1, fade: true } },
};

export function emitter(id: string, desc: Api.EmitterDesc = {}): void {
  const preset = PRESETS[desc.preset ?? "sparkle"] ?? PRESETS.sparkle;
  const k = desc.size && desc.size > 0 ? desc.size : 1;
  // A layer over its parent's area, taking no input: what the particles fly from.
  kit.open(id, { width: grow(), height: grow() });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
  if (still() || desc.active === false) return;
  const colors = (desc.colors && desc.colors.length > 0 ? desc.colors : preset.colors).map((c) => token(c));
  kit.particles({
    ...preset.p,
    size: { x: (preset.p.size?.x ?? 4) * k, y: (preset.p.size?.y ?? 4) * k },
    rate: desc.rate && desc.rate > 0 ? desc.rate : preset.rate,
    colors,
  });
}

/** One amount (0–1) as each built-in material's parameters, with its colour. */
function params(m: Api.MaterialName, k: number, t: { r: number; g: number; b: number }): number[] {
  const col = [t.r, t.g, t.b, 1];
  switch (m) {
    case "dissolve": return [k, 0.06, 18, 0, ...col];
    case "shine": return [k * 1.4 - 0.2, 0.18, -0.6, 0.85, ...col];
    case "hologram": return [3, 0.15 + 0.6 * k, 1.5 + 2 * k, 0.35, ...col];
    case "glitch": return [k, 8, 6 * k, 12];
    case "ripple": return [6 * k, 48, 60, 0, 0.5, 0.5, 0, 0];
    case "pixelate": return [Math.max(1, Math.round(1 + 15 * k))];
    case "colorAdjust": return [0, 1 - k, 1 - 0.25 * k, k];
    case "outline": return [1 + 3 * k, 0, 0, 0, ...col];
    case "blur": return [12 * k];
    default: return [];
  }
}

export function effectLayer(id: string, desc: Api.EffectLayerDesc, build: Api.UiBuild): void {
  const m = desc.material ?? "plain";
  let k = Math.max(0, Math.min(1, desc.amount ?? 0));
  if (desc.sweep && !still()) {
    const period = Math.max(200, desc.periodMs ?? 2400) / 1000;
    const x = (kit.seconds() % period) / period;
    k = x < 0.5 ? x * 2 : 2 - x * 2;
  }
  const colorName = desc.color ?? "highlight";
  const hexs = frame().theme.palette[colorName] ?? "#ffffff";
  const n = (o: number) => parseInt(hexs.slice(1 + o, 3 + o), 16) / 255;
  kit.open(id, { direction: "column", width: null, height: null, cross: "stretch", part: "Layer.frame", partClass: "panel" });
  const layer: Api.LayerDesc = m === "custom"
    ? { material: "custom", ...(desc.custom ? { custom: desc.custom } : {}), params: [...(desc.params ?? [])] }
    : { material: m, params: params(m, k, { r: n(0), g: n(2), b: n(4) }) };
  kit.setDraw({ layer });
  try {
    build();
  } finally {
    kit.close();
  }
}
