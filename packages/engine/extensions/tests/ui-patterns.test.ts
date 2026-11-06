import { expect, test } from "bun:test";

import { og, defineSystem } from "@origingame/engine";

import { createEngine, type GameToken } from "../../src/runtime.ts";

import { racing } from "@origingame/engine/extensions/racing/index.ts";

import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

import { systemIds, type Element, centre, byPath, run, elements } from "../../tests/support/patterns.ts";

import { BAG } from "../rpg/tests/support/patterns.ts";

import "../../tests/support/patterns.ts";

test("long press: an item's menu is og-ui's recogniser, timed in ms whatever the tick rate", () => {
  // Wrong: a hand-counted 30 ticks (a second at 30 Hz, not og-ui's 500 ms), so a 600 ms hold opens nothing.
  const menus: string[] = [];
  const sys = defineSystem(`UpLong${systemIds.next++}`, {}, () => {
    og.ui.anchor("Root", { anchor: "topLeft", direction: "column", padding: { left: 24, top: 24, right: 24, bottom: 24 } } as never, () => {
      const r = rpg.inventoryGrid("Bag", { columns: 8, rows: 4, cellSize: 50, items: BAG, label: "Backpack" });
      if (r.menu) menus.push(r.menu);
    });
  });
  const made = createEngine(og.game({ tick: 30, systems: [sys] }) as unknown as GameToken);
  if (!made.ok) throw new Error(made.reason);
  const engine = made.value;
  og.ui.previewViewport(engine.world, { width: 1280, height: 720, dpr: 1, coarsePointer: false, breakpoint: "desktop", safeArea: { left: 0, top: 0, right: 0, bottom: 0 } } as never);
  engine.advance(4);
  const holdOn = (el: Element): void => {
    engine.probe.commands(`0:touch_move:${centre(el)};0.05:touch_start;0.7:touch_end`);
    engine.advance(18);
  };
  holdOn(byPath(engine, "Cells/helm")!);
  expect(menus).toEqual(["helm"]);
});

test("a screen-filling pattern given less than the safe area (under a game's header) lays out within its box, its action inside", () => {
  // Wrong: the pattern sizes its parts from its first guess (the whole safe area), its box grows to fit them, and the
  // grown box is kept as its room from then on, so its foot stands past the parent's on every tick (the kit's phone
  // character select had Lock in 42 px under the safe area).
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const cases: Array<[string, () => void]> = [
    ["Action", () => racing.garage("Garage", {
      title: "Garage", balance: 3000, categories: [{ key: "engine", name: "Engine" }], category: "engine",
      parts: ["stock", "street", "sport", "race", "works"].map((k, i) => ({ key: k, name: `${k} engine`, owned: i < 2, equipped: i === 1, price: 400 * i, tier: Math.min(3, i + 1) })),
      selected: "race", stats: [{ key: "power", label: "Power", value: 5, preview: 7, max: 10 }],
    })],
  ];
  for (const [action, build] of cases) {
    const engine = run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 600 } }, build), phone);
    engine.advance(20);
    const box = byPath(engine, "Root/Box")!;
    const a = elements(engine).find((e) => e.path.endsWith(action) && e.role === "button")!;
    expect(a).toBeDefined();
    expect(a.y + a.height).toBeLessThanOrEqual(box.y + box.height + 0.5);
  }
});

test("on a phone in portrait the map's legend is one row", () => {
  // Wrong: the legend's chips wrap into rows over the map.
  const phone = { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } } as const;
  const box = (build: () => void) => run(() => og.ui.column("Box", { width: { kind: "fixed", px: 374 }, height: { kind: "fixed", px: 691 } }, build), phone);
  const kinds = ["quest", "objective", "fastTravel", "shop", "poi", "boss"];
  const m = box(() => rpg.worldMap("Map", { markers: kinds.map((k, i) => ({ key: k, kind: k, name: k, x: 0.1 + i * 0.15, y: 0.5 })) }));
  m.advance(10);
  expect(byPath(m, "Side/Legend")!.height).toBeLessThanOrEqual(60);
});
