// A TSX HUD padded by the HUD insets keeps out of the web shell's Back
// (Brasswick's finding #15, Viewport.covered given here through
// og.ui.previewViewport), on the native host, on a desktop, a phone in
// portrait (Back inside its notch's safe area) and a phone in landscape: no
// readout of the HUD lies under the control, and moving past it never lands
// a readout on another of the HUD's readouts more than they met without it.
// The rule is og-ui's (env(hud-inset-*) folds the covered region into one
// edge's inset), so each HUD here puts a different readout in the top-left
// corner under Back. The command channel's `cover` lays a native run out the
// same way (Viewport.covered without a host).
//
// Ways it could fail: a health, resource or timer readout under Back; a
// readout moved onto its neighbour; the insets not given to the new core's
// tree, or not re-read when the surface or the covered region changes;
// `cover` refused, ignored or never cleared.
import { expect, test } from "bun:test";
import { og } from "@origingame/engine";
import { loadNativeHost } from "../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { Text, View, mount, renderers } from "../../src/ui/index.ts";

type Rect = { x: number; y: number; width: number; height: number };
type Element = Rect & { id: string; parent: string; text: string };
type Place = "left" | "middle" | "right" | "bottom";

type Readout = { label: string; value: string; place: Place };

// A HUD readout: a label and its value, as wide as a health bar or a clock.
const readout = (r: Readout) =>
  View({ class: `flex-row gap-2 items-center py-1.5 px-2${r.place === "middle" ? " w-[200px]" : ""}`, children: [Text({ children: r.label }), Text({ class: "w-[140px]", children: r.value })] });

const HUDS: Record<string, Readout[]> = {
  action: [
    { label: "HP", value: "80 / 100", place: "left" },
    { label: "Time", value: "1:20", place: "middle" },
    { label: "Coins", value: "1,200", place: "right" },
    { label: "Ammo", value: "12 / 90", place: "bottom" },
  ],
  resources: [{ label: "Gold", value: "123,456", place: "left" }, { label: "Wave", value: "0:40", place: "right" }],
  timer: [{ label: "Lap", value: "0:03", place: "left" }, { label: "Boost", value: "40", place: "bottom" }],
};

// The corners as a game's HUD writes them: 16 px inside the HUD insets.
const PAD = "pt-[calc(env(hud-inset-top)+16px)] pr-[calc(env(hud-inset-right)+16px)] pb-[calc(env(hud-inset-bottom)+16px)] pl-[calc(env(hud-inset-left)+16px)]";

function Hud(readouts: Readout[]) {
  const at = (place: Place) => readouts.filter((r) => r.place === place).map(readout);
  return View({
    class: `flex-1 flex-col justify-between ${PAD}`,
    children: [
      View({ class: "flex-row justify-between items-start", children: [...at("left"), ...at("middle"), ...at("right")] }),
      View({ class: "flex-row justify-end", children: at("bottom") }),
    ],
  });
}

const SURFACES = {
  desktop: { viewport: { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" }, back: { x: 12, y: 12, width: 40, height: 40 } },
  phone: { viewport: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" }, back: { x: 12, y: 59, width: 40, height: 40 } },
  landscape: { viewport: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" }, back: { x: 59, y: 12, width: 40, height: 40 } },
};

function hud(name: string): Engine {
  loadNativeHost();
  const r = createEngine(og.game({
    ui(world) {
      mount(world, () => Hud(HUDS[name]!), {});
    },
  }) as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function close(engine: Engine): void {
  renderers.delete((engine.world as unknown as { id: number }).id);
  engine.destroy();
}

const meet = (a: Rect, b: Rect): number => Math.max(0, Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y));

/** The HUD's readouts, by label: each the box that holds its label's text. */
function readouts(engine: Engine, name: string): Map<string, Rect> {
  const ui = (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
  const out = new Map<string, Rect>();
  for (const { label } of HUDS[name]!) {
    const text = ui.find((e) => e.text === label);
    const box = text && ui.find((e) => e.id === text.parent);
    if (box) out.set(label, box);
  }
  return out;
}

/** How much the HUD's readouts cover each other, in CSS px². */
function crowding(boxes: Map<string, Rect>): number {
  const all = [...boxes.values()];
  let sum = 0;
  for (let i = 0; i < all.length; i++) for (let j = i + 1; j < all.length; j++) sum += meet(all[i]!, all[j]!);
  return sum;
}

for (const name of Object.keys(HUDS)) {
  test(`the ${name} HUD keeps out of the shell's Back on a desktop and on a phone`, () => {
    const engine = hud(name);
    try {
      for (const [surface, { viewport, back }] of Object.entries(SURFACES)) {
        og.ui.previewViewport(engine.world, viewport as never);
        engine.advance(3);
        const free = readouts(engine, name);
        expect(free.size).toBe(HUDS[name]!.length);
        // Without Back, the top-left readout sits under where it will be: the test means something.
        expect([...free.values()].some((e) => meet(e, back) > 0)).toBe(true);
        og.ui.previewViewport(engine.world, { ...viewport, covered: back } as never);
        engine.advance(3);
        const covered = readouts(engine, name);
        const under = [...covered].filter(([, e]) => meet(e, back) > 0).map(([label]) => label);
        expect(`${surface}: ${under.join(", ")}`).toBe(`${surface}: `);
        expect(crowding(covered)).toBeLessThanOrEqual(crowding(free) + 0.5);
      }
    } finally {
      close(engine);
    }
  });
}

test("the command channel's cover lays a native run out as the published page (Viewport.covered without a host)", () => {
  const engine = hud("timer");
  try {
    const lap = () => readouts(engine, "timer").get("Lap")!;
    engine.advance(2);
    const free = lap();
    expect(free.x).toBeLessThan(40);
    // The page's Back: 40 CSS px square at (12, 12) on a desktop page (shell.css).
    const covered = engine.command("cover", ["12,12,40,40"]);
    expect(covered.ok).toBeTrue();
    engine.advance(2);
    const moved = lap();
    expect(moved.x).toBeGreaterThanOrEqual(52);
    expect(moved.y).toBeCloseTo(free.y, 3);
    // Cleared, the readout comes back; a malformed rectangle is refused with the usage.
    expect(engine.command("cover", ["none"]).ok).toBeTrue();
    engine.advance(2);
    expect(lap().x).toBeCloseTo(free.x, 3);
    const refused = engine.command("cover", ["12,12,40"]);
    expect(refused.ok).toBeFalse();
    if (!refused.ok) expect(refused.reason).toContain("x,y,width,height");
  } finally {
    close(engine);
  }
});
