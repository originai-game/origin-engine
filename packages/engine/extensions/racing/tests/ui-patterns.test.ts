import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import { racing, type GarageResult } from "@origingame/engine/extensions/racing/index.ts";

import { run, tapOn, byPath, elements, key } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

// ── garage ──────────────────────────────────────────────────────────────

test("garage: a fitted part is not bought again; a short or locked part is refused, an affordable one bought, an owned one equipped", () => {
  // Wrong: a part dearer than the purse answered as bought, a locked one answered, an owned one sold again.
  const results: GarageResult[] = [];
  const st = { selected: "short" };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = racing.garage("Garage", {
        title: "Garage",
        balance: 300,
        categories: [{ key: "engine", name: "Engine" }, { key: "paint", name: "Livery", livery: true }],
        category: "engine",
        parts: [
          { key: "stock", name: "Stock", owned: true, equipped: true, tier: 1 },
          { key: "owned", name: "Street", owned: true, tier: 2 },
          { key: "short", name: "Race", price: 900, tier: 3 },
          { key: "cheap", name: "Sport", price: 200, tier: 2 },
          { key: "locked", name: "Works", price: 50, locked: true, lockedHint: "Class A", tier: 3 },
        ],
        selected: st.selected,
        stats: [{ key: "power", label: "Power", value: 5, preview: 7, max: 10 }],
      });
      results.push(r);
      if (r.selected) st.selected = r.selected;
    });
  });
  engine.advance(10);
  const act = (): void => {
    tapOn(engine, byPath(engine, "Side/Action")!);
    engine.advance(4);
  };
  act();
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["short"]);
  for (const k of ["locked", "cheap", "owned", "stock"]) {
    tapOn(engine, byPath(engine, `List/${k}`)!);
    engine.advance(4);
    expect(st.selected).toBe(k);
    act();
  }
  expect(results.filter((r) => r.refused).map((r) => r.refused)).toEqual(["short", "locked"]);
  expect(results.filter((r) => r.buy).map((r) => r.buy)).toEqual(["cheap"]);
  expect(results.filter((r) => r.equip).map((r) => r.equip)).toEqual(["owned"]);
});

test("garage: on a phone at 200 % text the selected part scrolls into the list's view, clear of its fade", () => {
  // Wrong: the chosen part low in the list stayed under the scroll's foot and its fade, so the screen showed no
  // selected row at all (U8's text200-garage).
  const engine = run(() => {
    // The room a phone's screen leaves it: the safe area less the root's padding.
    og.ui.column("Box", { width: { kind: "fixed", px: 342 }, height: { kind: "fixed", px: 715 } }, () => racing.garage("Garage", {
      title: "Garage",
      balance: 3000,
      categories: [{ key: "engine", name: "Engine" }],
      category: "engine",
      parts: ["stock", "street", "sport", "race", "works"].map((k, i) => ({ key: k, name: `${k} engine`, owned: i < 2, equipped: i === 1, price: 400 * i, tier: Math.min(3, i + 1) })),
      selected: "race",
      stats: [{ key: "power", label: "Power", value: 5, preview: 7, max: 10 }],
    }));
  }, { width: 390, height: 844, dpr: 3, coarsePointer: true, breakpoint: "phonePortrait", safeArea: { left: 0, top: 47, right: 0, bottom: 34 } });
  og.ui.configure(engine.world, { textScale: 1.5 });
  engine.advance(30);
  const els = elements(engine);
  const view = els.find((e) => e.path.endsWith("Garage/Body"))!;
  const row = els.find((e) => e.path.endsWith("Parts/race"));
  expect(row).toBeDefined();
  expect(row!.y).toBeGreaterThanOrEqual(view.y - 0.5);
  // Whole, and above the 40 px fade at the view's foot.
  expect(row!.y + row!.height).toBeLessThanOrEqual(view.y + view.height - 40 + 0.5);
});

test("garage: the livery's layers answer their own eye and order; the first layer cannot go up nor the last down; the next-tab action wraps to the first category", () => {
  // Wrong: the eye or the arrows selecting the row instead, or a first layer answered as moved up.
  const results: GarageResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      results.push(racing.garage("Garage", {
        balance: 0,
        categories: [{ key: "engine", name: "Engine" }, { key: "paint", name: "Livery", livery: true }],
        category: "paint",
        layers: [
          { key: "base", name: "Base", kind: "Paint", color: { r: 0.8, g: 0.1, b: 0.1, a: 1 } },
          { key: "stripe", name: "Stripe", kind: "Decal", color: { r: 1, g: 1, b: 1, a: 1 } },
        ],
        layer: "base",
      }));
    });
  });
  engine.advance(10);
  for (const p of ["stripe/Up", "base/Down", "stripe/Eye"]) {
    tapOn(engine, byPath(engine, `List/${p}`)!);
    engine.advance(4);
  }
  expect(results.filter((r) => r.layerMove).map((r) => [r.layerMove!.key, r.layerMove!.by])).toEqual([["stripe", -1], ["base", 1]]);
  expect(results.filter((r) => r.layerVisible).map((r) => r.layerVisible)).toEqual(["stripe"]);
  expect(results.some((r) => r.layer)).toBeFalse();
  for (const p of ["base/Up", "stripe/Down"]) {
    tapOn(engine, byPath(engine, `List/${p}`)!);
    engine.advance(4);
  }
  expect(results.filter((r) => r.layerMove).length).toBe(2);
  key(engine, "KeyE");
  expect(results.filter((r) => r.category).map((r) => r.category)).toEqual(["engine"]);
  tapOn(engine, byPath(engine, "List/stripe")!);
  engine.advance(4);
  expect(results.filter((r) => r.layer).map((r) => r.layer)).toEqual(["stripe"]);
});
