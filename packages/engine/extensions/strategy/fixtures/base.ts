// The strategy kit's fixtures: a selection, its orders and production, the
// economy and the waves to come in the shapes the kit's components take,
// for the review sheet (sheet/), the skill's examples and a game's first
// draft. Plain data: copy and edit it. Resources, unit kinds and the
// number of waves are the game's own: add or drop entries freely.

import type { AbilitySlotDesc } from "@origingame/engine";
import type { ResourceBarDesc, RtsCommandsDesc, WavePreviewDesc } from "../types.ts";

const order = (key: string, name: string, icon: string, hotkey: string, more: Partial<AbilitySlotDesc> = {}): AbilitySlotDesc => ({ key, name, icon, hotkey, ...more });

export const ORDERS: AbilitySlotDesc[] = [
  order("move", "Move", "arrow-right", "M"), order("stop", "Stop", "square", "S"), order("hold", "Hold", "shield", "H"), order("patrol", "Patrol", "refresh", "P"),
  order("attack", "Attack", "sword", "A"), order("build", "Build", "hammer", "B"), order("repair", "Repair", "wrench", "R", { disabled: true }),
  order("stim", "Stim", "bolt", "T", { cooldown: 4, cooldownMax: 10 }),
];

export function commands(): RtsCommandsDesc {
  return {
    page: "main", targetLabel: "Choose a target", cancelLabel: "Cancel", queueLabel: "Queue", backLabel: "Back",
    pages: [{ key: "main", commands: ORDERS }, { key: "build", commands: [order("barracks", "Barracks", "home", "B"), order("forge", "Forge", "anvil", "F"), order("tower", "Tower", "castle", "T")] }],
    console: {
      map: { zoom: 0.5, markers: [{ key: "me", kind: "player", x: 0.3, y: 0.62 }, { key: "e1", kind: "enemy", x: 0.78, y: 0.2 }, { key: "o", kind: "objective", x: 0.55, y: 0.45 }] },
      selection: {
        focus: "u1", group: "1",
        groups: [{ key: "1", label: "1", count: 6 }, { key: "2", label: "2", count: 2 }, { key: "3", label: "3", count: 1 }],
        units: [
          { key: "u1", name: "Marine", kind: "infantry", icon: "user", health: 45, healthMax: 55 },
          { key: "u2", name: "Marine", kind: "infantry", icon: "user", health: 55, healthMax: 55 },
          { key: "u3", name: "Medic", kind: "support", icon: "heart", health: 30, healthMax: 60, energy: 80, energyMax: 200 },
          { key: "u4", name: "Siege tank", kind: "vehicle", icon: "truck", health: 140, healthMax: 175, shield: 20, shieldMax: 40 },
          { key: "u5", name: "Marine", kind: "infantry", icon: "user", health: 12, healthMax: 55 },
          { key: "u6", name: "Engineer", kind: "worker", icon: "wrench", health: 40, healthMax: 40, progress: 0.4 },
        ],
        stats: [{ key: "dmg", icon: "sword", label: "Damage", value: "6" }, { key: "arm", icon: "shield", label: "Armour", value: "1" }, { key: "rng", icon: "target", label: "Range", value: "5" }],
        queue: { items: [{ key: "q1", name: "Marine", icon: "user", remaining: 6, duration: 18, count: 3 }, { key: "q2", name: "Medic", icon: "heart", duration: 30 }], slots: 5 },
      },
      commands: { columns: 4 },
    },
  };
}

export function resources(): ResourceBarDesc {
  return {
    rates: true, clock: 754, anchor: null,
    items: [
      { key: "gold", icon: "coin", name: "Gold", amount: 1240, rate: 14 },
      { key: "wood", icon: "tree", name: "Wood", amount: 860, rate: 9 },
      { key: "stone", icon: "cube", name: "Stone", amount: 310, rate: -2 },
      { key: "supply", icon: "users", name: "Supply", amount: 46, capacity: 46 },
    ],
  };
}

export function waves(): WavePreviewDesc {
  return {
    title: "Next waves", current: 7, max: 20, anchor: null,
    waves: [
      { key: "w8", number: 8, name: "Goblin runners", icon: "skull", count: 24 },
      { key: "w9", number: 9, name: "Armoured trolls", icon: "shield", count: 6 },
      { key: "w10", number: 10, name: "The Ogre King", icon: "crown", count: 1, boss: true },
    ],
  };
}
