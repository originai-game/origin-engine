// The racing kit's fixtures: one race's data in the shapes its components
// take, for the review sheet (sheet/), the skill's examples and a game's
// first draft. Plain data: copy and edit it.

import type { GarageDesc, LapTimerDesc, RaceBannerDesc, SpeedometerDesc, StandingsTowerDesc } from "../types.ts";

/** The field: name, gap to the leader in seconds, livery colour. */
export const FIELD: ReadonlyArray<readonly [string, number, string]> = [
  ["Okafor", 0, "#e6372e"], ["Lindqvist", 1.84, "#2a7de1"], ["Moreau", 3.02, "#f2b705"], ["You", 4.37, "#1fb36b"],
  ["Tanaka", 5.11, "#9b51e0"], ["Reyes", 7.66, "#f2f2f2"], ["Haddad", 9.4, "#ff7a1a"],
];

const hex = (h: string) => ({ color: { r: parseInt(h.slice(1, 3), 16) / 255, g: parseInt(h.slice(3, 5), 16) / 255, b: parseInt(h.slice(5, 7), 16) / 255, a: 1 } });

export function standings(): StandingsTowerDesc {
  const you = FIELD.find(([n]) => n === "You")![1];
  return {
    entries: FIELD.map(([name, gap, c]) => ({ key: name, name, color: hex(c), gap: gap - you, player: name === "You" })),
    relative: true, ahead: 2, behind: 2, leaderLabel: "Leader", pitLabel: "Pit", outLabel: "Out", fastestLabel: "Fastest lap", anchor: null,
  };
}

export function lap(show: "head" | "times"): LapTimerDesc {
  return {
    show, boxless: show === "head", compact: show === "times", position: 4, positionTotal: FIELD.length, lap: 2, lapTotal: 3,
    currentMs: 51_480, bestMs: 72_904, lastMs: 73_377, deltaMs: -420, sectors: ["best", "personal", "none"],
    positionLabel: "Pos", lapLabel: "Lap", finalLabel: "Final lap", currentLabel: "Current", bestLabel: "Best", lastLabel: "Last", splitLabel: "Split",
    anchor: null,
  };
}

export function tach(size: number): SpeedometerDesc {
  return { style: "tach", speed: 238, maxSpeed: 340, unit: "km/h", rpm: 0.81, maxRpm: 8000, rpmStep: 1000, redline: 0.85, shiftAt: 0.925, gear: "5", boost: 0.62, size, anchor: null };
}

export function checkpoint(at: number): RaceBannerDesc {
  return {
    event: "checkpoint", at, count: 3, deltaMs: -420, index: 2, best: false, place: 4, placeText: "4th", timeMs: 221_287,
    goLabel: "GO", checkpointLabel: "Checkpoint", wrongWayLabel: "Wrong way", finalLapLabel: "Final lap", finishLabel: "Finish",
  };
}

const TIERS = [["Stock", 0], ["Street", 6400], ["Sport", 18_500], ["Race", 42_000], ["Works", 68_000]] as const;
const RARITY = ["common", "uncommon", "rare", "epic", "legendary"] as const;

export function garage(): GarageDesc {
  return {
    balance: 24_800,
    categories: ["Engine", "Wheels", "Suspension", "Exhaust", "Brakes", "Aero", "Livery"].map((name) => ({ key: name.toLowerCase(), name, livery: name === "Livery", count: name === "Livery" ? "3" : "2/5" })),
    category: "engine",
    parts: TIERS.map(([tier, price], i) => ({
      key: `engine.${tier.toLowerCase()}`, name: `${tier} engine`, rarity: RARITY[i], rarityLabel: tier, tier: i + 1, tierMax: TIERS.length, price,
      owned: i < 2, equipped: i === 1, locked: tier === "Works", lockedHint: tier === "Works" ? "Needs class S" : "", description: "More power through the whole rev range.",
    })),
    selected: "engine.sport",
    stats: [["power", "Power", 6.6, 7.4], ["handling", "Handling", 5.4, -1], ["braking", "Braking", 5.8, -1], ["launch", "Launch", 6.6, 7.0], ["speed", "Top speed", 7.1, 7.6]].map(([key, label, value, preview]) => ({ key: key as string, label: label as string, value: value as number, preview: preview as number, max: 10 })),
    ratingClass: "A", rating: 712, ratingPreview: 739,
    performanceLabel: "Performance", classLabel: "Class", currentLabel: "Current", previewLabel: "Preview", equipLabel: "Equip", equippedLabel: "Equipped",
  };
}
