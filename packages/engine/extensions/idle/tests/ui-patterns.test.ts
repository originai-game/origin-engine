import { expect, test } from "bun:test";

import { idle, type PrestigeResult } from "@origingame/engine/extensions/idle/index.ts";

import { generator } from "./support/patterns.ts";

import { centre, byPath, run } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("idleGenerator: a press buys when the purse pays; short, it is refused and said so", () => {
  let can = true;
  const g = generator(() => can);
  const buy = centre(byPath(g.engine, "Solar/Buy")!);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start;0.1:touch_end`);
  g.engine.advance(14);
  expect(g.bought()).toBe(1);
  can = false;
  g.engine.advance(2);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start;0.1:touch_end`);
  g.engine.advance(14);
  expect(g.bought()).toBe(1);
  expect(g.results.some((r) => r.short)).toBeTrue();
  expect(byPath(g.engine, "Solar/Buy")!.name).toContain("short");
});

test("idleGenerator: Buy held repeats after a beat while the purse pays", () => {
  // Wrong: a hold that buys once, or one that buys every tick from the first.
  const g = generator(() => true);
  const buy = centre(byPath(g.engine, "Solar/Buy")!);
  g.engine.probe.commands(`0:touch_move:${buy};0.05:touch_start`);
  g.engine.advance(12);
  expect(g.bought()).toBe(0);
  g.engine.advance(60);
  const held = g.bought();
  expect(held).toBeGreaterThanOrEqual(4);
  expect(held).toBeLessThanOrEqual(12);
});

test("prestigePanel: a tap does not reset the run; a full hold does; unavailable it cannot be held", () => {
  // Wrong: a reset on a press (a slip throws a run away), or one taken while nothing would be gained.
  let available = false;
  const results: PrestigeResult[] = [];
  const engine = run(() => {
    results.push(idle.prestigePanel("Prestige", { lead: "Ascend for", rewardText: "+128 Shards", points: ["Resets generators"], available, holdMs: 600 }));
  }, { width: 1280, height: 720 });
  const at = () => centre(byPath(engine, "Action/Button")!);
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;1.2:touch_end`);
  engine.advance(80);
  expect(results.some((r) => r.confirmed)).toBeFalse();
  available = true;
  engine.advance(2);
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;0.15:touch_end`);
  engine.advance(14);
  expect(results.some((r) => r.confirmed)).toBeFalse();
  engine.probe.commands(`0:touch_move:${at()};0.05:touch_start;1.0:touch_end`);
  engine.advance(70);
  expect(results.filter((r) => r.confirmed).length).toBe(1);
});
