// The gacha kit's fixtures: an event banner and a ten-pull in the shapes the
// kit's components take, for the review sheet (sheet/), the skill's
// examples and a game's first draft. Plain data: copy and edit it. Rarity
// here is the engine's Rarity scale; a game with its own tiers passes their
// words in rarityLabel and their star counts in stars.

import type { Rarity } from "@origingame/engine";
import type { GachaBannerDesc, PullSequenceDesc } from "../types.ts";

export function banner(): GachaBannerDesc {
  return {
    banners: [
      { key: "event", name: "Moonlit Tide", endsIn: 3 * 86_400 + 5 * 3600, fresh: true },
      { key: "standard", name: "Wanderlust" },
      { key: "weapon", name: "Epitome", endsIn: 9 * 86_400 },
    ],
    current: "event", title: "Moonlit Tide", subtitle: "Event wish · Limited",
    featured: [{ name: "Nerissa", title: "Tide Oracle", stars: 5, rarity: "legendary" }, { name: "Kai", stars: 4, rarity: "epic" }, { name: "Lumi", stars: 4, rarity: "epic" }],
    rateText: "5★ 0.6% · 4★ 5.1%", endsIn: 3 * 86_400 + 5 * 3600, pity: 62, pityMax: 90, pityLabel: "Guaranteed in {n}",
    currency: "Crystals", balance: 2400, single: 160, multi: 1600, multiCount: 10,
    singleLabel: "Wish ×1", multiLabel: "Wish ×10", shortLabel: "Not enough", detailsLabel: "Details", historyLabel: "History",
    daysLabel: "d", hoursLabel: "h", minutesLabel: "m",
  };
}

const NAMES: Array<[string, string, number, Rarity]> = [
  ["Nerissa", "character", 5, "legendary"], ["Kai", "character", 4, "epic"], ["Coral bow", "weapon", 3, "rare"], ["Iron blade", "weapon", 3, "rare"],
  ["Lumi", "character", 4, "epic"], ["Tide charm", "weapon", 3, "rare"], ["Iron blade", "weapon", 3, "rare"], ["Sea glass", "weapon", 3, "rare"],
  ["Kelp staff", "weapon", 3, "rare"], ["Iron blade", "weapon", 3, "rare"],
];

export function pull(token: number): PullSequenceDesc {
  return {
    token, againPrice: 1600, currency: "Crystals", affordable: true,
    againLabel: "Wish ×10 again", doneLabel: "Done", skipLabel: "Skip", tapLabel: "Tap to reveal", continueLabel: "Continue", newLabel: "New",
    items: NAMES.map(([name, kind, stars, rarity], i) => ({ key: `p${i}`, name, kind, stars, rarity, isNew: i < 2, title: i === 0 ? "Tide Oracle" : "" })),
  };
}
