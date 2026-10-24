// The racing HUD and the results screen for a time trial (Meridian Loop's
// findings #8, #10 and #18), on the native host: every word the HUD draws is
// its desc's (a German table's, no English left), a time trial hides the
// place and the boost meter and shows its live delta and splits, the
// results headline is a formatted time under its label with the lower time
// as the best, and the HUD keeps out of the web shell's Back
// (Viewport.covered, given here through og.ui.previewViewport).
//
// Ways it could fail: `lap`, `Best`, `Pause` or `Boost` drawn whatever the
// desc says; `1/1` or an empty boost meter in a time trial; a delta or a
// split not shown, or signed with a glyph the packs lack (U+2212); 96.233 s
// shown as "96"; the place under Back.
import { beforeAll, expect, test } from "bun:test";
import { og, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

type Element = { id: string; parent: string; text: string; x: number; y: number; width: number; height: number };

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function elements(engine: Engine): Element[] {
  return (engine.probe.dump({ what: "ui" }) as unknown as { elements: Element[] }).elements;
}

const German = {
  lapLabel: "Runde",
  bestLabel: "Bestzeit {best}",
  pauseLabel: "Pausieren",
  boostLabel: "Turbo",
  unit: "km/h",
};

const Trial = defineResource("UrTrial", { covered: u32() });
const TimeTrial = defineSystem("UrTimeTrial", { writes: [Trial] }, (_q, _t, world: World) => {
  og.ui.racingHud("Hud", {
    ...German,
    speed: 142,
    gear: "4",
    lap: 2,
    laps: 3,
    racers: 0,
    boostMeter: false,
    time: 47.5,
    delta: -0.412,
    best: 96.233,
    splits: [
      { label: "S1", time: 31.2, delta: -0.412 },
      { label: "S2", time: 30.05, delta: 1.05 },
      { label: "S3" },
    ],
  });
});

test("og.ui.racingHud: a time trial's words are the desc's, with its delta and splits and no place or boost", () => {
  const engine = start(og.game({ resources: [Trial], systems: [TimeTrial] }));
  engine.advance(2);
  const ui = elements(engine);
  const texts = ui.map((e) => e.text).filter((t) => t !== "");
  for (const english of ["lap", "Pause", "Boost"]) expect(texts).not.toContain(english);
  expect(texts.some((t) => /^Best /.test(t))).toBeFalse();
  expect(texts).toContain("Runde");
  expect(texts).toContain("Bestzeit 1:36.233");
  expect(ui.some((e) => e.id === "Pause" && (e.text === "Pausieren" || ui.some((c) => c.parent === "Pause" && c.text === "Pausieren")))).toBeTrue();
  // racers 0: no place; boostMeter false: no meter.
  expect(ui.some((e) => e.id === "Place" || e.id === "Position")).toBeFalse();
  expect(ui.some((e) => e.id === "Boost")).toBeFalse();
  // The live delta and the splits, signed with the ASCII minus.
  expect(ui.find((e) => e.id === "Delta" && e.parent === "Hud.times")?.text).toBe("-0.412");
  const split = (i: number) => ui.filter((e) => e.parent === `Split#${i}`).map((e) => e.text);
  expect(split(0)).toEqual(["S1", "0:31.200", "-0.412"]);
  expect(split(1)).toEqual(["S2", "0:30.050", "+1.050"]);
  expect(split(2)).toEqual(["S3", "-:--.---"]);
});

test("og.ui.racingHud: its place keeps out of the shell's Back (Viewport.covered)", () => {
  const Race = defineSystem("UrRace", {}, (_q, _t, _world: World) => {
    og.ui.racingHud("Hud", { position: 1, racers: 1, lap: 1, laps: 3, time: 3 });
  });
  const engine = start(og.game({ systems: [Race] }));
  const viewport = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
  og.ui.previewViewport(engine.world, viewport as never);
  engine.advance(2);
  const free = elements(engine).find((e) => e.id === "Hud.place")!;
  expect(free.x).toBeLessThan(40);
  og.ui.previewViewport(engine.world, { ...viewport, covered: { x: 12, y: 12, width: 40, height: 40 } } as never);
  engine.advance(2);
  const moved = elements(engine).find((e) => e.id === "Hud.place")!;
  expect(moved.x).toBeGreaterThanOrEqual(52);
  expect(moved.y).toBeCloseTo(free.y, 3);
  expect(og.ui.frameViewport).toBeDefined();
});

test("the command channel's cover lays a native run out as the published page (Viewport.covered without a host)", () => {
  const Race = defineSystem("UcRace", {}, (_q, _t, _world: World) => {
    og.ui.racingHud("Hud", { position: 1, racers: 1, lap: 1, laps: 3, time: 3 });
  });
  const engine = start(og.game({ systems: [Race] }));
  engine.advance(2);
  const free = elements(engine).find((e) => e.id === "Hud.place")!;
  expect(free.x).toBeLessThan(40);
  // The page's Back: 40 CSS px square at (12, 12) on a desktop page (shell.css).
  const covered = engine.command("cover", ["12,12,40,40"]);
  expect(covered.ok).toBeTrue();
  engine.advance(2);
  const moved = elements(engine).find((e) => e.id === "Hud.place")!;
  expect(moved.x).toBeGreaterThanOrEqual(52);
  expect(moved.y).toBeCloseTo(free.y, 3);
  // Cleared, the place comes back; a malformed rectangle is refused with the usage.
  expect(engine.command("cover", ["none"]).ok).toBeTrue();
  engine.advance(2);
  expect(elements(engine).find((e) => e.id === "Hud.place")!.x).toBeCloseTo(free.x, 3);
  const refused = engine.command("cover", ["12,12,40"]);
  expect(refused.ok).toBeFalse();
  if (!refused.ok) expect(refused.reason).toContain("x,y,width,height");
});

const Results = defineSystem("UrResults", {}, (_q, _t, _world: World) => {
  og.ui.resultsScreen("Results", {
    open: true,
    title: "Ziel",
    format: "time",
    scoreLabel: "Gesamtzeit",
    score: 96.233,
    best: 95.1,
    bestLabel: "Bestzeit {best}",
    maxStars: 0,
  });
});

test("og.ui.resultsScreen: a time trial's result is its headline, a time under its label, the lower time the best", () => {
  const engine = start(og.game({ systems: [Results] }));
  engine.advance(80);
  const ui = elements(engine);
  expect(ui.find((e) => e.id === "ScoreLabel")?.text).toBe("Gesamtzeit");
  expect(ui.find((e) => e.id === "Score")?.text).toBe("1:36.233");
  expect(ui.find((e) => e.id === "Best")?.text).toBe("Bestzeit 1:35.100");
});

test("og.ui.racingHud: the best lap reads as the results read it, never a millisecond short by a float's last bit", () => {
  const Best = defineSystem("UrBest", {}, (_q, _t, _world: World) => {
    og.ui.racingHud("Hud", { racers: 0, boostMeter: false, time: 2, best: 30.717 });
  });
  const engine = start(og.game({ systems: [Best] }));
  engine.advance(2);
  // 30.717 - 30 is 0.716999…: the running clock's floor would say 0:30.716.
  expect(elements(engine).find((e) => e.id === "Best")?.text).toBe("Best 0:30.717");
});
