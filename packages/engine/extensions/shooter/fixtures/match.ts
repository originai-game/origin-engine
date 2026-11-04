// The shooter kit's fixtures: one match's data in the shapes its components
// take, for the review sheet (sheet/), the skill's examples and a game's
// first draft. Plain data: copy and edit it.

import type { AmmoDesc, KillfeedEntry, LoadoutDesc, ScoreboardDesc } from "../types.ts";

export const FEED: KillfeedEntry[] = [
  { key: "k1", killer: "Vex", killerTeam: 0, victim: "Orrin", victimTeam: 1, weaponText: "Raven", headshot: true },
  { key: "k2", killer: "Sable", killerTeam: 1, victim: "Juno", victimTeam: 0, weaponText: "Frag" },
  { key: "k3", killer: "You", killerTeam: 0, victim: "Brask", victimTeam: 1, weaponText: "Raven", local: true, assist: "Vex" },
];

export function ammo(): AmmoDesc {
  return { magazine: 7, magazineMax: 30, reserve: 120, weapon: "Raven", fireMode: "Auto", reloading: null, reloadLabel: "Reloading", reloadHint: "Reload", anchor: "bottomRight" };
}

export function scoreboard(): ScoreboardDesc {
  return {
    title: "Bind · Competitive", subtitle: "Round 14 · 7–6",
    columns: [{ key: "k", label: "K" }, { key: "d", label: "D" }, { key: "a", label: "A" }, { key: "s", label: "Score", width: 72 }],
    teams: [
      { team: 0, name: "Attackers", score: "7", rows: [
        { key: "a", name: "Jett", values: ["21", "9", "4", "6,120"], local: true, ping: 24, mvp: true },
        { key: "b", name: "Sova", values: ["14", "11", "12", "4,410"], ping: 118 },
        { key: "c", name: "Sage", values: ["6", "12", "15", "2,380"], dead: true, ping: 260 },
      ] },
      { team: 1, name: "Defenders", score: "6", rows: [
        { key: "d", name: "Cypher", values: ["18", "13", "3", "5,040"], ping: 40 },
        { key: "e", name: "Killjoy", values: ["9", "15", "8", "2,990"], ping: 66 },
      ] },
    ],
  };
}

const SLOTS = [["primary", "Primary", "Raven AR", "Level 14"], ["secondary", "Secondary", "Mk 4 pistol", "Level 6"], ["melee", "Melee", "Combat knife", ""], ["lethal", "Lethal", "Frag", ""], ["tactical", "Tactical", "Stun", ""], ["field", "Field upgrade", "Recon drone", "Unlocks at 20"]] as const;
const POINTS = [["optic", "Optic", "Holo 2x", 0.42, 0.22, "rare"], ["laser", "Laser", "Tac laser", 0.74, 0.3, "epic"], ["magazine", "Magazine", "40-round mag", 0.52, 0.68, "rare"], ["underbarrel", "Underbarrel", "Vertical grip", 0.67, 0.51, "uncommon"], ["muzzle", "Muzzle", "Suppressor", 0.94, 0.37, "rare"]] as const;

export function loadout(): LoadoutDesc {
  return {
    loadouts: ["Loadout 1", "Loadout 2", "Loadout 3"], loadout: 0, balance: 1250,
    slots: SLOTS.map(([key, label, name, detail]) => ({ key, label, name, detail, rarity: key === "primary" ? "rare" : "common", locked: key === "field" })),
    slot: "primary", name: "Raven AR", subtitle: ["Assault rifle", "Rare", "Level 14"], rarity: "rare", rarityWord: 1,
    description: "A steady full-auto rifle that rewards controlled bursts.",
    attachments: POINTS.map(([key, label, name, x, y, rarity]) => ({ key, label, name, rarity, rarityLabel: rarity, x, y })),
    capacity: POINTS.length,
    stats: [["damage", "Damage", 32, 4], ["range", "Range", 64, 2], ["fireRate", "Fire rate", 75, -3], ["mobility", "Mobility", 64, 4], ["control", "Control", 54, 3]].map(([key, label, value, delta]) => ({ key: key as string, label: label as string, value: value as number, delta: delta as number, max: 100 })),
    statsLabel: "Stats", attachmentsLabel: "Attachments", emptyLabel: "Empty", removeLabel: "Remove", lockedLabel: "Locked",
  };
}
