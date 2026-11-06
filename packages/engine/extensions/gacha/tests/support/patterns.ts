import { gacha, type GachaBannerResult, type PullItem, type PullSequenceResult } from "@origingame/engine/extensions/gacha/index.ts";

import { run, centre, byPath } from "../../../../tests/support/patterns.ts";

// ── gacha banner and pull ───────────────────────────────────────────────

export function banner(balance: number) {
  let current = "event";
  const results: GachaBannerResult[] = [];
  const engine = run(() => {
    const r = gacha.gachaBanner("Banner", {
      banners: [{ key: "event", name: "Event" }, { key: "weapon", name: "Weapon" }, { key: "std", name: "Standard" }],
      current, title: "Moonlit Tide", balance, single: 160, multi: 1600, pityMax: 90, pity: 72,
      featured: [{ name: "Aria", stars: 5, rarity: "legendary" }],
    }, () => {});
    results.push(r);
    if (r.banner) current = r.banner;
  }, { width: 1280, height: 720 });
  const press = (end: string) => {
    engine.probe.commands(`0:touch_move:${centre(byPath(engine, end)!)};0.05:touch_start;0.1:touch_end`);
    engine.advance(14);
  };
  return { engine, results, press, current: () => current };
}

export const PULL: PullItem[] = [
  { key: "p1", name: "Iron Sword", stars: 3, rarity: "common" },
  { key: "p2", name: "Aria", stars: 5, rarity: "legendary", isNew: true },
  { key: "p3", name: "Bow", stars: 4, rarity: "epic", extra: "+2 Star Shards" },
];

export function pull(items: PullItem[] = PULL) {
  let token = 0;
  const results: PullSequenceResult[] = [];
  const engine = run(() => {
    const r = gacha.pullSequence("Pull", { token, items, againPrice: 1600 });
    results.push(r);
    if (r.done) token = 0;
  });
  const tap = () => {
    engine.probe.commands("0:touch_move:640,360;0.05:touch_start;0.1:touch_end");
    engine.advance(14);
  };
  return { engine, results, tap, start: (t: number) => { token = t; engine.advance(2); }, last: () => results.at(-1)! };
}
