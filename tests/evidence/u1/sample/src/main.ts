// The U1 sample screen (docs/ui.md §5): a reward screen whose every
// surface is the UI pass's, set as data through og.ui.setDraw.
//
// - the ground: a radial gradient with a diamond glow over it;
// - the panel: glass (a backdrop blur) with squircle corners, two drop
//   shadows and a gradient border stroke;
// - the title: a gradient fill with an outline and a shadow;
// - the medallion: a circle clipping a conic sweep, an outer glow, and the
//   cooked emblem (a `type vector`) over it;
// - three reward cards: a hexagon with an inner glow, a star badge, and a
//   "legendary" card composited through the hologram material;
// - a portrait through the custom duotone material (a `type ui-material`);
// - a Bézier connector with dashes between the cards, a cooldown arc
//   with round caps over its track, and a capsule button with a gradient.

import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";

const fixed = (px: number) => ({ kind: "fixed", px }) as const;
const token = (t: string, alpha = 1) => ({ token: t, alpha }) as never;
const rgb = (r: number, g: number, b: number, alpha = 1) => ({ color: { r, g, b }, alpha }) as never;
const stops = (...s: [number, unknown][]) => s.map(([offset, color]) => ({ offset, color })) as never;

let emblem = 0;
let duotone = 0;

/** One element: opened, drawn with `draw`, its children built, closed. */
function box(id: string, desc: Record<string, unknown>, draw: Record<string, unknown>, children?: () => void): void {
  og.ui.open(id, desc as never);
  og.ui.setDraw(draw as never);
  children?.();
  og.ui.close();
}

/** A reward card: a shape, a caption, what draws it, and its transform (og.ui.props). */
function card(id: string, caption: string, draw: Record<string, unknown>, inner: Record<string, unknown>, transform?: Record<string, number>): void {
  box(id, { width: fixed(150), height: fixed(190), direction: "column", align: "center", cross: "center", gap: 14 }, draw, () => {
    box(`${id}Gem`, { width: fixed(84), height: fixed(84) }, inner);
    og.ui.label(`${id}Caption`, caption, { step: "lg", color: token("ink") } as never);
  });
  if (transform) og.ui.props({ transform } as never);
}

export const Screen = defineSystem("Screen", { reads: [og.Input] }, () => {
  const t = og.ui.seconds();
  // The ground.
  box(
    "Ground",
    { width: { kind: "percent", fraction: 1 }, height: { kind: "percent", fraction: 1 }, align: "center", cross: "center" },
    {
      fills: [
        { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.35 }, to: { x: 0.8, y: 0.9 }, stops: stops([0, rgb(0.16, 0.12, 0.34)], [1, rgb(0.02, 0.02, 0.06)]) },
        { kind: "gradient", type: "diamond", from: { x: 0.5, y: 0.3 }, to: { x: 0.35, y: 0.5 }, stops: stops([0, rgb(1, 0.7, 0.2, 0.18)], [1, rgb(1, 0.7, 0.2, 0)]) },
      ],
    },
    () => {
      // The panel: glass with squircle corners, shadows and a gradient border.
      box(
        "Panel",
        { width: fixed(900), direction: "column", cross: "center", gap: 18, padding: { left: 36, top: 26, right: 36, bottom: 30 } },
        {
          shape: { kind: "rect", radii: [36, 36, 36, 36], corner: "squircle", squircle: 4 },
          fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 0, y: 1 }, stops: stops([0, rgb(0.24, 0.22, 0.4, 0.55)], [1, rgb(0.08, 0.07, 0.16, 0.75)]) }],
          strokes: [
            {
              width: 2,
              align: "inside",
              paint: { kind: "gradient", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: stops([0, rgb(1, 0.85, 0.4)], [0.5, rgb(0.9, 0.4, 1, 0.6)], [1, rgb(0.3, 0.9, 1)]) },
            },
          ],
          effects: [
            { kind: "backdrop", blur: 18, saturation: 1.3, brightness: 0.9, tint: rgb(0.1, 0.08, 0.2, 0.2) },
            { kind: "dropShadow", offset: { x: 0, y: 4 }, blur: 10, color: rgb(0, 0, 0, 0.45) },
            { kind: "dropShadow", offset: { x: 0, y: 30 }, blur: 60, color: rgb(0, 0, 0, 0.55) },
          ],
        },
        () => {
          og.ui.open("Header", { direction: "row", cross: "center", gap: 28 } as never);
          // The medallion: a circle clip over a turning conic sweep, glowing, the emblem on top.
          box(
            "Medallion",
            { width: fixed(132), height: fixed(132), align: "center", cross: "center" },
            {
              shape: { kind: "ellipse" },
              fills: [{ kind: "gradient", type: "conic", from: { x: 0.5, y: 0.5 }, to: { x: t * 0.8, y: 0 }, stops: stops([0, rgb(1, 0.75, 0.2)], [0.33, rgb(0.9, 0.3, 0.6)], [0.66, rgb(0.3, 0.6, 1)], [1, rgb(1, 0.75, 0.2)]) }],
              strokes: [{ width: 3, align: "inside", paint: { kind: "solid", color: rgb(1, 0.95, 0.8) } }],
              effects: [{ kind: "outerGlow", blur: 26, spread: 2, color: rgb(1, 0.7, 0.25, 0.8) }],
              clip: "shape",
            },
            () => box("Emblem", { width: fixed(104), height: fixed(104) }, { fills: [{ kind: "vector", vector: emblem }] }),
          );
          og.ui.open("Titles", { direction: "column", gap: 4 } as never);
          og.ui.label("Title", "VICTORY", { step: "displayXl", font: "display" } as never);
          og.ui.setDraw({
            text: {
              fill: { from: { x: 0, y: 0 }, to: { x: 0, y: 1 }, stops: stops([0, rgb(1, 0.97, 0.7)], [0.55, rgb(1, 0.75, 0.15)], [1, rgb(0.9, 0.35, 0.05)]) },
              outline: { width: 2.5, color: rgb(0.25, 0.12, 0.05), align: "outside" },
              shadows: [{ offset: { x: 0, y: 5 }, blur: 8, color: rgb(0, 0, 0, 0.7) }],
            },
          } as never);
          // Leaned like an italic: the outline, fill and shadow take the skew.
          og.ui.props({ transform: { skewX: -0.2 } } as never);
          og.ui.label("Subtitle", "Stage 7 cleared: three rewards", { step: "lg", color: token("inkMuted") } as never);
          og.ui.close();
          og.ui.close();
          // The cards, a connector behind them.
          og.ui.open("Cards", { direction: "row", gap: 40, cross: "center" } as never);
          card(
            "Hex",
            "Gem ×3",
            { shape: { kind: "rect", radii: [20, 20, 20, 20], corner: "bevel" }, fills: [{ kind: "solid", color: rgb(0.1, 0.12, 0.22, 0.8) }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: rgb(0.4, 0.8, 1, 0.5) } }] },
            {
              shape: { kind: "polygon", sides: 6, rounding: 6 },
              fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: stops([0, rgb(0.4, 0.95, 1)], [1, rgb(0.1, 0.35, 0.9)]) }],
              effects: [{ kind: "innerGlow", blur: 14, color: rgb(1, 1, 1, 0.8) }, { kind: "dropShadow", offset: { x: 0, y: 6 }, blur: 10, color: rgb(0, 0, 0, 0.5) }],
            },
            { rotate: -0.12 },
          );
          card(
            "Star",
            "Gold ×250",
            { shape: { kind: "rect", radii: [20, 20, 20, 20], corner: "notch" }, fills: [{ kind: "solid", color: rgb(0.1, 0.12, 0.22, 0.8) }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: rgb(1, 0.8, 0.3, 0.6) } }] },
            {
              shape: { kind: "polygon", sides: 5, inner: 0.48, rounding: 3, rotation: 0 },
              fills: [{ kind: "gradient", type: "radial", from: { x: 0.5, y: 0.4 }, to: { x: 0.6, y: 0.6 }, stops: stops([0, rgb(1, 0.98, 0.7)], [1, rgb(1, 0.6, 0.05)]) }],
              strokes: [{ width: 2, paint: { kind: "solid", color: rgb(0.5, 0.25, 0.02) } }],
              effects: [{ kind: "outerGlow", blur: 18, color: rgb(1, 0.75, 0.1, 0.9) }],
            },
            { skewX: -0.22 },
          );
          card(
            "Legend",
            "Legendary",
            {
              shape: { kind: "rect", radii: [20, 20, 20, 20] },
              fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: stops([0, rgb(0.45, 0.1, 0.7)], [1, rgb(0.05, 0.6, 0.65)]) }],
              layer: { material: "hologram", params: [4, 0.15, 1.5, 0.35, 0.4, 1, 1, 1] },
            },
            {
              shape: { kind: "arc", start: -Math.PI / 2, sweep: 1.4 * Math.PI, thickness: 10, cap: "round" },
              fills: [{ kind: "gradient", type: "conic", from: { x: 0.5, y: 0.5 }, to: { x: -Math.PI / 2, y: 0 }, stops: stops([0, rgb(0.6, 1, 0.9)], [0.7, rgb(1, 1, 1)]) }],
            },
            { rotate: 0.1 },
          );
          og.ui.close();
          // A footer: the portrait through the duotone material, a dashed
          // Bézier, the cooldown ring and the button.
          og.ui.open("Footer", { direction: "row", gap: 30, cross: "center" } as never);
          box(
            "Portrait",
            { width: fixed(96), height: fixed(96), align: "center", cross: "center" },
            {
              shape: { kind: "rect", radii: [48, 48, 48, 48] },
              fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: stops([0, rgb(1, 1, 1)], [1, rgb(0.15, 0.15, 0.2)]) }],
              layer: { material: "custom", custom: duotone, params: [0.03, 0.02, 0.12, 0, 1, 0.72, 0.25, 0, 0.18] },
              clip: "shape",
            },
            () => box("Face", { width: fixed(70), height: fixed(70) }, { fills: [{ kind: "vector", vector: emblem }] }),
          );
          box(
            "Path",
            { width: fixed(180), height: fixed(60) },
            {
              shape: { kind: "path", points: [{ x: 4, y: 50 }, { x: 60, y: -20 }, { x: 120, y: 80 }, { x: 176, y: 10 }], curves: true, width: 4, cap: "round" },
              fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 1, y: 0 }, stops: stops([0, rgb(0.3, 0.9, 1)], [1, rgb(1, 0.4, 0.8)]) }],
              strokes: [],
            },
          );
          box(
            "Cooldown",
            { width: fixed(72), height: fixed(72) },
            {
              shape: { kind: "arc", start: 0, sweep: 2 * Math.PI, thickness: 8 },
              fills: [{ kind: "solid", color: rgb(1, 1, 1, 0.12) }],
            },
            () => {
              og.ui.open("CooldownFill", { width: fixed(72), height: fixed(72) } as never);
              og.ui.setDraw({
                shape: { kind: "arc", start: -Math.PI / 2, sweep: 1.3 * Math.PI, thickness: 8, cap: "round" },
                fills: [{ kind: "gradient", type: "conic", from: { x: 0.5, y: 0.5 }, to: { x: -Math.PI / 2, y: 0 }, stops: stops([0, rgb(0.3, 1, 0.6)], [0.65, rgb(1, 0.9, 0.3)]) }],
              } as never);
              og.ui.close();
            },
          );
          box(
            "Continue",
            { width: fixed(240), height: fixed(64), align: "center", cross: "center", interactive: true },
            {
              shape: { kind: "rect", radii: [32, 32, 32, 32] },
              fills: [{ kind: "gradient", from: { x: 0, y: 0 }, to: { x: 0, y: 1 }, stops: stops([0, rgb(1, 0.82, 0.35)], [1, rgb(0.95, 0.45, 0.1)]) }],
              strokes: [{ width: 2, align: "inside", paint: { kind: "solid", color: rgb(1, 1, 1, 0.5) } }],
              effects: [
                { kind: "dropShadow", offset: { x: 0, y: 6 }, blur: 14, color: rgb(0.6, 0.2, 0, 0.6) },
                { kind: "innerShadow", offset: { x: 0, y: -4 }, blur: 6, color: rgb(0.5, 0.15, 0, 0.5) },
              ],
            },
            () => {
              og.ui.label("ContinueLabel", "Continue", { step: "xl", color: rgb(0.2, 0.08, 0.02) } as never);
            },
          );
          og.ui.close();
        },
      );
    },
  );
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Screen],
  start(world: World) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0, g: 0, b: 0 };
    const settings = world.resource(og.RenderSettings);
    if (settings.ok) {
      settings.value.toneMapped = false;
      settings.value.sky = false;
    }
    const font = og.load.font(world, "fonts/Inter-Display.ttf");
    if (!font.ok) throw new Error(font.reason);
    og.ui.configure(world, { font: font.value, displayFont: font.value } as never);
    const v = og.load.vector(world, "ui/emblem.ogt");
    if (!v.ok) throw new Error(v.reason);
    emblem = v.value as unknown as number;
    const m = og.load.uiMaterial(world, "ui/duotone.ogt");
    if (!m.ok) throw new Error(m.reason);
    duotone = m.value as unknown as number;
  },
});
