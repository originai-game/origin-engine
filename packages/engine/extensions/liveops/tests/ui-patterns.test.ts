import { expect, test } from "bun:test";

import { og } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { liveops } from "@origingame/engine/extensions/liveops/index.ts";

import { store, pass } from "./support/patterns.ts";

import { byPath, elements, run } from "../../../tests/support/patterns.ts";

import "../../../tests/support/patterns.ts";

test("storeFront: an offer the purse pays is chosen; short or sold out it is refused and says why; owned it is viewed", () => {
  // Wrong: a short offer chosen (the game asked to charge what the purse cannot pay), a sold-out one sold again, or
  // an owned one bought twice.
  const s = store(500);
  s.press("Offers/potion");
  expect(s.results.some((r) => r.chosen === "potion")).toBeTrue();
  s.press("Offers/sword");
  expect(s.results.some((r) => r.chosen === "sword")).toBeFalse();
  expect(s.results.some((r) => r.refused === "sword")).toBeTrue();
  expect(byPath(s.engine, "Offers/sword")!.name).toContain("not enough");
  s.press("Offers/key");
  expect(s.results.some((r) => r.chosen === "key")).toBeFalse();
  expect(s.results.some((r) => r.refused === "key")).toBeTrue();
  expect(byPath(s.engine, "Offers/key")!.name).toContain("Sold out");
  s.press("Offers/hat");
  expect(s.results.some((r) => r.viewed === "hat")).toBeTrue();
  expect(s.results.some((r) => r.chosen === "hat")).toBeFalse();
});

test("storeFront: a discount shows its saving in whole percent; a limit says what is left; a plus asks to top up", () => {
  // Wrong: the saving worked from the wrong price (−67 % for 1,200 against 2,000), or the limit showing the bought count.
  const s = store(5000);
  expect(elements(s.engine).some((e) => e.text === "−40 %")).toBeTrue();
  expect(byPath(s.engine, "Offers/potion")!.name).toContain("2 left");
  s.press("Wallet/gems/TopUp");
  expect(s.results.some((r) => r.topUp === "gems")).toBeTrue();
  expect(byPath(s.engine, "Wallet/gems/TopUp")!.name).toBe("Get more Gems");
});

test("battlePass: a reward up to the level is claimed once; past the level, or premium while locked, it is refused", () => {
  // Wrong: a reward past the level granted, the premium track granted without the pass, or a claimed one granted again.
  const p = pass(false);
  p.press("Tiers/L2/Free");
  expect(p.claims()).toEqual(["2f"]);
  p.press("Tiers/L2/Free");
  expect(p.claims()).toEqual(["2f"]);
  expect(byPath(p.engine, "Tiers/L2/Free")!.name).toContain("Claimed");
  p.press("Tiers/L2/Premium");
  expect(p.claims()).toEqual(["2f"]);
  expect(p.results.some((r) => r.refused?.level === 2 && r.refused.premium)).toBeTrue();
  p.press("Tiers/L4/Free");
  expect(p.claims()).toEqual(["2f"]);
  expect(byPath(p.engine, "Tiers/L4/Free")!.name).toContain("Reach level 4");
}, 20000);

test("battlePass: Claim all takes every ready reward once, free and premium, and goes when none is left", () => {
  // Wrong: Claim all granting rewards past the level or already claimed, or staying after it emptied the track.
  const p = pass(true);
  expect(byPath(p.engine, "Actions/ClaimAll")!.name).toBe("Claim all (5)");
  p.press("Actions/ClaimAll");
  expect(p.claims().sort()).toEqual(["1p", "2f", "2p", "3f", "3p"]);
  p.engine.advance(4);
  expect(byPath(p.engine, "Actions/ClaimAll") === undefined).toBeTrue();
  expect(byPath(p.engine, "Actions/Unlock") === undefined).toBeTrue();
});

test("battlePass: the track opens on the player's tier, far along it, in a row and stacked", () => {
  // Wrong: the track opening at tier 1 with the player's tier off screen.
  for (const vp of [undefined, { width: 390, height: 844, breakpoint: "phonePortrait", coarsePointer: true }] as const) {
    const p = (() => {
      const claimed: string[] = [];
      const engine = run(() => {
        og.ui.column("Box", { width: { kind: "fixed", px: vp ? 360 : 1180 }, height: { kind: "fixed", px: vp ? 700 : 600 } } as never, () => {
          liveops.battlePass("Pass", { level: 24, xpNext: 1000, tiers: Array.from({ length: 30 }, (_, i) => ({ level: i + 1, free: { key: `f${i}`, name: `R${i}` } })) });
        });
      }, vp as Partial<Api.Viewport> | undefined);
      engine.advance(6);
      return { engine, claimed };
    })();
    const view = byPath(p.engine, "Pass/Track/View")!;
    const cur = byPath(p.engine, "Tiers/L24")!;
    expect(cur.x).toBeGreaterThanOrEqual(view.x - 1);
    expect(cur.x + cur.width).toBeLessThanOrEqual(view.x + view.width + 1);
    expect(cur.y).toBeGreaterThanOrEqual(view.y - 1);
    expect(cur.y + cur.height).toBeLessThanOrEqual(view.y + view.height + 1);
    expect(byPath(p.engine, "Tiers/L1") === undefined).toBeTrue();
  }
});

test("battlePass: inside a scroll the track keeps its height, and only the tiers in view are built", () => {
  // Wrong: the track sized from its own view, a step shorter each tick in a scroll that leaves it no room; or every tier built.
  const engine = run(() => {
    og.ui.scrollArea("Box", { width: { kind: "fixed", px: 1180 }, height: { kind: "fixed", px: 500 } } as never, () => {
      liveops.battlePass("Pass", { level: 12, xpNext: 1000, tiers: Array.from({ length: 30 }, (_, i) => ({ level: i + 1, free: { key: `f${i}`, name: `R${i}` }, premium: { key: `p${i}`, name: `P${i}` } })) });
    });
  });
  engine.advance(6);
  const h6 = byPath(engine, "Pass/Track")!.height;
  engine.advance(30);
  const track = byPath(engine, "Pass/Track")!;
  expect(track.height).toBeGreaterThanOrEqual(270);
  expect(track.height).toBe(h6);
  const built = elements(engine).filter((e) => /\/Tiers\/L\d+$/.test(e.path)).length;
  expect(built).toBeGreaterThan(0);
  expect(built).toBeLessThan(15);
});
