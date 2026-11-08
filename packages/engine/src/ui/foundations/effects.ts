// Foundations (docs/ui.md §2): the effect layer. It is decoration: it takes
// no input and says nothing to a screen reader; its sweep stands still under
// reduced motion.

import type * as Api from "@origingame/engine";
import { frame, kit, still } from "../controls/base.ts";

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
