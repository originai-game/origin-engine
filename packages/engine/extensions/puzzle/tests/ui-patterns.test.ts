import { expect, test } from "bun:test";

import type * as Api from "@origingame/engine";

import { puzzle } from "@origingame/engine/extensions/puzzle/index.ts";

import { saga } from "./support/patterns.ts";

import { byPath, tapOn, centre, run, elements } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("sagaMap: it opens with the current level in view, far down the path", () => {
  // Wrong: the map opening at its first level, with the player's level off screen.
  for (const vp of [undefined, { width: 390, height: 844, breakpoint: "phonePortrait", coarsePointer: true }] as const) {
    const s = saga(vp as Partial<Api.Viewport> | undefined);
    const view = byPath(s.engine, "Saga/View")!;
    const cur = byPath(s.engine, "Path/l21")!;
    expect(cur.x).toBeGreaterThanOrEqual(view.x - 1);
    expect(cur.x + cur.width).toBeLessThanOrEqual(view.x + view.width + 1);
    expect(cur.y).toBeGreaterThanOrEqual(view.y - 1);
    expect(cur.y + cur.height).toBeLessThanOrEqual(view.y + view.height + 1);
    expect(cur.name).toContain("You are here");
    // The first level is far behind it, scrolled out of the view (the probe lists only what the view shows).
    expect(byPath(s.engine, "Path/l1") === undefined).toBeTrue();
  }
});

test("sagaMap: an open level is picked; a locked one is refused and said locked; stars are said", () => {
  const s = saga();
  const tapOn = (end: string) => {
    s.engine.probe.commands(`0:touch_move:${centre(byPath(s.engine, end)!)};0.05:touch_start;0.1:touch_end`);
    s.engine.advance(14);
  };
  tapOn("Path/l20");
  expect(s.results.some((r) => r.picked === "l20")).toBeTrue();
  tapOn("Path/l22");
  expect(s.results.some((r) => r.picked === "l22")).toBeFalse();
  expect(s.results.some((r) => r.refused === "l22")).toBeTrue();
  expect(byPath(s.engine, "Path/l22")!.name).toContain("Locked");
  expect(byPath(s.engine, "Path/l20")!.name).toContain("2 of 3 stars");
});

test("goalChips: a goal met says done; its count never shows past the need", () => {
  const engine = run(() => {
    puzzle.goalChips("Goals", { goals: [{ key: "blue", label: "Blue Gems", have: 14, need: 20 }, { key: "ice", label: "Ice", have: 7, need: 6 }] });
  });
  expect(byPath(engine, "Goals/blue")!.name).toBe("Blue Gems, 14 of 20");
  expect(byPath(engine, "Goals/ice")!.name).toBe("Ice, 6 of 6, Done");
  expect(elements(engine).some((e) => e.text === "7 / 6")).toBeFalse();
});
