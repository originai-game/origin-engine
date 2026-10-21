// Every kit HUD keeps out of the web shell's Back (Brasswick's finding #15,
// Viewport.covered given here through og.ui.previewViewport), on the
// native host, on a desktop, a phone in portrait (Back inside its notch's
// safe area) and a phone in landscape: no element of the HUD lies under
// the control, and moving past it never lands a group on another of the
// HUD's groups more than they met without it (on a phone the shooter's
// vitals and the puzzle's level and moves step down past Back, where
// moving right would put them under the kills or the score).
//
// Ways it could fail: a heart, a place or a health bar under Back; a group
// moved right onto its neighbour.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

type Rect = { x: number; y: number; width: number; height: number };
type Element = Rect & { id: string; parent: string };

const HUDS: Record<string, () => void> = {
  platformer: () => og.ui.platformerHud("Hud", { lives: 2, maxLives: 3, coins: 4, score: 120, level: "1-1", timeLeft: 80, touch: true }),
  shooter: () => og.ui.shooterHud("Hud", { health: 80, maxHealth: 100, ammo: 12, magazine: 30, reserve: 90, kills: 3, touch: true, abilities: [{ id: "dash", key: "Q" }] }),
  puzzle: () => og.ui.puzzleHud("Hud", { score: 400, target: 1000, moves: 12, level: "Level 3", timeLeft: 40 }),
  idle: () => og.ui.idleHud("Hud", { currencies: [{ id: "Gold", amount: 123456, perSecond: 12, icon: "coin" }, { id: "Gems", amount: 40 }], generators: [{ id: "mine", name: "Mine", level: 2, cost: 10, affordable: true, progress: 0.4 }] }),
  racing: () => og.ui.racingHud("Hud", { position: 1, racers: 8, lap: 1, laps: 3, time: 3, touch: true }),
};

const SURFACES = {
  desktop: { viewport: { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" }, back: { x: 12, y: 12, width: 40, height: 40 } },
  phone: { viewport: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" }, back: { x: 12, y: 59, width: 40, height: 40 } },
  landscape: { viewport: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" }, back: { x: 59, y: 12, width: 40, height: 40 } },
};

const meet = (a: Rect, b: Rect): number => Math.max(0, Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y));

function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

/** How much the HUD's top-level groups cover each other, in CSS px². */
function crowding(ui: Element[]): number {
  const groups = ui.filter((e) => e.parent === "");
  let sum = 0;
  for (let i = 0; i < groups.length; i++) for (let j = i + 1; j < groups.length; j++) sum += meet(groups[i]!, groups[j]!);
  return sum;
}

for (const [name, build] of Object.entries(HUDS)) {
  test(`og.ui.${name}Hud keeps out of the shell's Back on a desktop and on a phone`, () => {
    const Hud = defineSystem(`UcHud${name}`, {}, () => build());
    const r = createEngine(og.game({ systems: [Hud] }) as GameToken);
    if (!r.ok) throw new Error(r.reason);
    const engine = r.value;
    for (const [surface, { viewport, back }] of Object.entries(SURFACES)) {
      og.ui.previewViewport(engine.world, viewport as never);
      engine.advance(3);
      const free = elements(engine);
      og.ui.previewViewport(engine.world, { ...viewport, covered: back } as never);
      engine.advance(3);
      const ui = elements(engine);
      const under = ui.filter((e) => e.width > 0 && e.height > 0 && meet(e, back) > 0).map((e) => `${e.parent}/${e.id}`);
      expect(`${surface}: ${under.join(", ")}`).toBe(`${surface}: `);
      expect(crowding(ui)).toBeLessThanOrEqual(crowding(free) + 0.5);
    }
  });
}
