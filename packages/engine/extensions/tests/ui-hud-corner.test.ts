// A HUD of anchored components keeps out of the web shell's Back
// (Brasswick's finding #15, Viewport.covered given here through
// og.ui.previewViewport), on the native host, on a desktop, a phone in
// portrait (Back inside its notch's safe area) and a phone in landscape: no
// element of the HUD lies under the control, and moving past it never lands
// a group on another of the HUD's groups more than they met without it.
// The rule is og-ui's (an anchored group's keep-out), so each HUD here puts
// a different component in the top-left corner under Back. The command
// channel's `cover` lays a native run out the same way (Viewport.covered
// without a host).
//
// Ways it could fail: a health bar, a resource or a timer under Back; a
// group moved right onto its neighbour; `cover` refused, ignored or never
// cleared.
import { beforeAll, expect, test } from "bun:test";
import { og, defineSystem } from "@origingame/engine";
import { loadNativeHost } from "../../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../../src/runtime.ts";
import { shooter } from "@origingame/engine/extensions/shooter/index.ts";
import { strategy } from "@origingame/engine/extensions/strategy/index.ts";

beforeAll(() => {
  loadNativeHost();
});

type Rect = { x: number; y: number; width: number; height: number };
type Element = Rect & { id: string; parent: string };

const HUDS: Record<string, () => void> = {
  action: () => {
    og.ui.statBar("HP", { value: 80, max: 100, label: "HP", showValue: true, anchor: "topLeft" });
    og.ui.hudTimer("Clock", { seconds: 80, label: "Time", anchor: "top" });
    strategy.resourceBar("Res", { items: [{ key: "coins", name: "Coins", amount: 1200 }], anchor: "topRight" });
    shooter.ammoCounter("Ammo", { magazine: 12, reserve: 90, anchor: "bottomRight" });
  },
  strategy: () => {
    strategy.resourceBar("Res", { items: [{ key: "gold", name: "Gold", amount: 123456, rate: 12 }, { key: "gems", name: "Gems", amount: 40 }], anchor: "topLeft" });
    og.ui.hudTimer("Clock", { seconds: 40, label: "Wave", anchor: "topRight" });
  },
  timer: () => {
    og.ui.hudTimer("Clock", { seconds: 3, label: "Lap", anchor: "topLeft" });
    og.ui.statBar("Boost", { value: 40, max: 100, label: "Boost", anchor: "bottomRight" });
  },
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
  test(`the ${name} HUD keeps out of the shell's Back on a desktop and on a phone`, () => {
    const Hud = defineSystem(`UcHud${name}`, {}, () => build());
    const r = createEngine(og.game({ systems: [Hud] }) as GameToken);
    if (!r.ok) throw new Error(r.reason);
    const engine = r.value;
    for (const [surface, { viewport, back }] of Object.entries(SURFACES)) {
      og.ui.previewViewport(engine.world, viewport as never);
      engine.advance(3);
      const free = elements(engine);
      // Without Back, the top-left group sits under where it will be: the test means something.
      expect(free.some((e) => e.width > 0 && e.height > 0 && meet(e, back) > 0)).toBe(true);
      og.ui.previewViewport(engine.world, { ...viewport, covered: back } as never);
      engine.advance(3);
      const ui = elements(engine);
      const under = ui.filter((e) => e.width > 0 && e.height > 0 && meet(e, back) > 0).map((e) => `${e.parent}/${e.id}`);
      expect(`${surface}: ${under.join(", ")}`).toBe(`${surface}: `);
      expect(crowding(ui)).toBeLessThanOrEqual(crowding(free) + 0.5);
    }
  });
}

test("the command channel's cover lays a native run out as the published page (Viewport.covered without a host)", () => {
  const Hud = defineSystem("UcHudCover", {}, () => {
    og.ui.hudTimer("Clock", { seconds: 3, label: "Lap", anchor: "topLeft" });
  });
  const r = createEngine(og.game({ systems: [Hud] }) as GameToken);
  if (!r.ok) throw new Error(r.reason);
  const engine = r.value;
  const group = () => elements(engine).find((e) => e.parent === "" && e.width > 0)!;
  engine.advance(2);
  const free = group();
  expect(free.x).toBeLessThan(40);
  // The page's Back: 40 CSS px square at (12, 12) on a desktop page (shell.css).
  const covered = engine.command("cover", ["12,12,40,40"]);
  expect(covered.ok).toBeTrue();
  engine.advance(2);
  const moved = group();
  expect(moved.x).toBeGreaterThanOrEqual(52);
  expect(moved.y).toBeCloseTo(free.y, 3);
  // Cleared, the group comes back; a malformed rectangle is refused with the usage.
  expect(engine.command("cover", ["none"]).ok).toBeTrue();
  engine.advance(2);
  expect(group().x).toBeCloseTo(free.x, 3);
  const refused = engine.command("cover", ["12,12,40"]);
  expect(refused.ok).toBeFalse();
  if (!refused.ok) expect(refused.reason).toContain("x,y,width,height");
});
