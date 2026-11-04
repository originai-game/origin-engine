// The RPG kit's fixtures: a party's bags, gear, quests, a conversation and
// its frames in the shapes the kit's components take, for the review sheet
// (sheet/), the skill's examples and a game's first draft. Plain data: copy
// and edit it. A game's quests and lines come from og.story (og.story.quests,
// og.story.view); these stand in for them.

import type { DialogueView, Entity, QuestView } from "@origingame/engine";
import type { CastBarDesc, GridItem, InventoryGridDesc, PaperdollDesc, PartyFramesDesc, RaidFramesDesc, TurnOrderDesc, UnitFrameDesc } from "../types.ts";

export const ITEMS: GridItem[] = [
  { key: "sword", name: "Dawnsteel blade", icon: "sword", x: 0, y: 0, w: 1, h: 3, rarity: "epic", equipped: true },
  { key: "shield", name: "Oak buckler", icon: "shield", x: 1, y: 0, w: 2, h: 2, rarity: "uncommon" },
  { key: "potion", name: "Healing draught", icon: "flask", x: 3, y: 0, count: 7, maxStack: 20, rarity: "common" },
  { key: "ether", name: "Ether", icon: "droplet", x: 4, y: 0, count: 3, maxStack: 20, rarity: "rare", fresh: true },
  { key: "ring", name: "Ring of embers", icon: "ring", x: 3, y: 1, rarity: "legendary" },
  { key: "scroll", name: "Scroll of warding", icon: "scroll", x: 4, y: 1, rarity: "rare", tag: "Quest" },
  { key: "bow", name: "Hunter's bow", icon: "bow", x: 1, y: 2, w: 3, h: 1, rarity: "uncommon", unusable: true },
  { key: "key", name: "Crypt key", icon: "key", x: 5, y: 3, locked: true },
];

export function bag(): InventoryGridDesc {
  return { columns: 6, rows: 4, items: ITEMS, selected: "ring", label: "Backpack", capacity: 24, emptyLabel: "Empty", lockedLabel: "Locked", unusableLabel: "Can't use", rotateLabel: "Rotate" };
}

export function gear(): PaperdollDesc {
  const slot = (key: string, label: string, x: number, y: number, item: GridItem | null = null) => ({ key, label, x, y, item, accept: [key] });
  return {
    title: "Cassia", subtitle: "Level 24 Spellblade", rating: "412", ratingLabel: "Gear score",
    slots: [slot("head", "Head", 0.5, 0.06), slot("weapon", "Weapon", 0.08, 0.45, ITEMS[0]!), slot("chest", "Chest", 0.5, 0.36), slot("offhand", "Off hand", 0.92, 0.45), slot("ring", "Ring", 0.08, 0.78), slot("feet", "Feet", 0.5, 0.9)],
    stats: [{ label: "Attack", value: "184", delta: 12 }, { label: "Defence", value: "96" }, { label: "Crit", value: "18%", delta: -2, deltaText: "-2%" }, { label: "Speed", value: "1.05" }],
    statsTitle: "Stats", emptyLabel: "Empty", lockedLabel: "Locked",
  };
}

const quest = (id: string, title: string, status: string, objectives: Array<[string, number, number]>, tracked = false): QuestView => ({
  id, title, summary: "The lighthouse keeper has not lit the lamp for three nights. Find out why before the fleet returns.",
  category: "main", stage: "search", status, tracked, isNew: id === "q3", updated: false, updatedTick: 0, giver: "Mira", region: "Saltmarsh coast",
  marker: null, rewards: [{ name: "Gold", amount: 250, image: "", rarity: "common" }, { name: "Tide charm", amount: 1, image: "", rarity: "rare" }],
  objectives: objectives.map(([text, count, target], i) => ({ id: `${id}.${i}`, text, count, target, done: count >= target, failed: false, optional: i === 2, hidden: false })),
});

export const QUESTS: QuestView[] = [
  quest("q1", "The dark lighthouse", "active", [["Reach the lighthouse", 1, 1], ["Search the keeper's quarters", 0, 1], ["Collect lamp oil", 2, 5]], true),
  quest("q2", "Salt and iron", "active", [["Deliver ore to the smith", 3, 6]]),
  quest("q3", "Whispers in the reeds", "active", [["Speak to the ferryman", 0, 1]]),
  quest("q4", "A hound's errand", "completed", [["Find the lost hound", 1, 1]]),
];

const choice = (index: number, id: string, text: string, more: Partial<DialogueView["choices"][number]> = {}): DialogueView["choices"][number] => ({
  index, id, text, key: "", args: null, enabled: true, reason: "", reasonKey: "", reasonArgs: null, visited: false, kind: "", check: null, ...more,
});

export function conversation(): DialogueView {
  return {
    conversation: 1, node: "keeper", seq: 3, seen: false, key: "", args: null, voice: "", voiceTicks: 0, voiceLeft: 0,
    text: "The lamp went dark the night the tide turned red. I heard singing under the rocks, and then nothing at all.",
    speaker: { id: "keeper", name: "Old Wren", entity: 0 as Entity, portrait: "", expression: "", side: "left" },
    shot: "", timer: null, lastCheck: null, ended: false,
    choices: [
      choice(0, "ask", "Who was singing?"),
      choice(1, "calm", "You're safe now. Tell me everything.", { kind: "check", check: { stat: "persuasion", target: 12, chance: 0.65, kind: "skill" } }),
      choice(2, "bribe", "Here's twenty gold for your trouble.", { enabled: false, reason: "Needs 20 gold" }),
      choice(3, "leave", "I'll look for myself.", { visited: true }),
    ],
  };
}

export function hero(): UnitFrameDesc {
  return {
    name: "Cassia", level: "24", health: 0.64, resource: 0.48, resourceKind: "mana", combat: true, leader: true, size: "large", anchor: null,
    effects: [{ key: "regen", icon: "heart", name: "Regeneration", remaining: 12, duration: 20 }, { key: "burn", icon: "flame", name: "Burning", stacks: 2, remaining: 4, duration: 8, debuff: true }],
  };
}

export function target(): UnitFrameDesc {
  return { name: "Vorthul", title: "Drowned Prophet", level: "??", health: 0.37, hostile: true, unitClass: "boss", castName: "Tidal wail", castProgress: 0.55, targeted: true, size: "large", reverse: true, anchor: null };
}

export function party(): PartyFramesDesc {
  return {
    anchor: null,
    members: [
      { key: "a", name: "Bram", level: "23", health: 0.82, resource: 0.3, resourceKind: "primary", size: "small" },
      { key: "b", name: "Isolde", level: "24", health: 0.21, resource: 0.9, resourceKind: "mana", size: "small" },
      { key: "c", name: "Tamsin", level: "22", health: 0, dead: true, size: "small" },
    ],
  };
}

export function raid(): RaidFramesDesc {
  const roles = ["tank", "healer", "damage", "damage", "damage"] as const;
  const names = ["Bram", "Isolde", "Tamsin", "Corvo", "Ysra", "Halden", "Pell", "Nyx", "Odo", "Rue"];
  return {
    columns: 5, anchor: null,
    members: names.map((name, i) => ({
      key: name, name, role: roles[i % 5], health: [1, 0.72, 0.4, 0.18, 0.9][i % 5]!, absorb: i === 1 ? 0.2 : 0, incomingHeal: i === 3 ? 0.25 : 0,
      dead: i === 7, offline: i === 9, outOfRange: i === 6, aggro: i === 0, dispel: i === 4 ? "magic" : "", selected: i === 2,
    })),
  };
}

export function cast(): CastBarDesc {
  return { name: "Frost lance", icon: "snowflake", progress: 0.62, remaining: 0.7, anchor: null };
}

export function turns(): TurnOrderDesc {
  return {
    anchor: null, nowLabel: "Now", hostileLabel: "Enemy",
    units: [
      { key: "c", name: "Cassia", current: true }, { key: "s", name: "Skeleton", team: 1 }, { key: "b", name: "Bram" },
      { key: "w", name: "Wraith", team: 1, delayed: true }, { key: "i", name: "Isolde" }, { key: "s2", name: "Skeleton", team: 1, dead: true },
    ],
  };
}
