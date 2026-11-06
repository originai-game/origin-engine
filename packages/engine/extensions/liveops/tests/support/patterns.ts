import { og } from "@origingame/engine";

import type * as Api from "@origingame/engine";

import { liveops, type BattlePassResult, type StoreFrontResult } from "@origingame/engine/extensions/liveops/index.ts";

import { run, centre, byPath } from "../../../../tests/support/patterns.ts";

// ── store ───────────────────────────────────────────────────────────────

export function store(gems: number) {
  const results: StoreFrontResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1180 } } as never, () => {
      results.push(liveops.storeFront("Store", {
        wallet: [{ key: "gems", name: "Gems", amount: gems, topUp: true }],
        sections: [{
          key: "daily",
          title: "Daily",
          endsIn: 3600 * 7,
          offers: [
            { key: "sword", name: "Sword", kind: "Weapon", price: 1200, original: 2000, currency: "gems", rarity: "epic", affordable: gems >= 1200 },
            { key: "potion", name: "Potion", price: 100, currency: "gems", limit: 3, bought: 1, affordable: gems >= 100 },
            { key: "key", name: "Key", price: 100, currency: "gems", limit: 1, bought: 1 },
            { key: "hat", name: "Hat", price: 900, currency: "gems", owned: true },
          ],
        }],
      }));
    });
  });
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  return { engine, results, press };
}

// ── battle pass ─────────────────────────────────────────────────────────

export function pass(premium: boolean, viewport?: Partial<Api.Viewport>) {
  const claimed = new Set<string>(["1f"]);
  const results: BattlePassResult[] = [];
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: viewport ? 360 : 1180 }, height: { kind: "fixed", px: viewport ? 700 : 600 } } as never, () => {
      const r = liveops.battlePass("Pass", {
        title: "Season 4",
        level: 3,
        xp: 400,
        xpNext: 1000,
        premium,
        tiers: Array.from({ length: 8 }, (_, i) => ({
          level: i + 1,
          free: { key: `f${i + 1}`, name: `Coins ${i + 1}` },
          premium: { key: `p${i + 1}`, name: `Outfit ${i + 1}` },
          freeClaimed: claimed.has(`${i + 1}f`),
          premiumClaimed: claimed.has(`${i + 1}p`),
        })),
      });
      for (const c of r.claimed) claimed.add(`${c.level}${c.premium ? "p" : "f"}`);
      results.push(r);
    });
  }, viewport);
  engine.advance(6);
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  const claims = () => results.flatMap((r) => r.claimed.map((c) => `${c.level}${c.premium ? "p" : "f"}`));
  return { engine, results, press, claims };
}
