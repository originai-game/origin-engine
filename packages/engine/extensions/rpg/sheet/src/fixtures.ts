// The sheet's fixtures (docs/ui.md §6.1): a party's worth of plausible,
// long-worded data. Portraits and the map are texture handles main.ts
// loads; `art` names them here.
export type Art = Record<"hero" | "mage" | "rogue" | "cleric" | "speaker" | "map", number>;

export const inventory = () => ({
  title: "Backpack",
  filter: "all",
  filters: [
    { id: "all", label: "All", count: 14 },
    { id: "gear", label: "Weapons & Armour", count: 7 },
    { id: "use", label: "Consumables", count: 4 },
    { id: "quest", label: "Quest", count: 2 },
    { id: "misc", label: "Curios", count: 1 },
  ],
  items: [
    { id: "blade", name: "Emberwake Blade", icon: "sword", quantity: 1, rarity: "epic", kind: "Longsword", equipped: true, fresh: false, locked: false, broken: false },
    { id: "shield", name: "Oakheart Shield", icon: "shield", quantity: 1, rarity: "rare", kind: "Shield", equipped: false, fresh: true, locked: false, broken: false },
    { id: "crown", name: "Circlet of the Drowned King", icon: "crown", quantity: 1, rarity: "legendary", kind: "Head", equipped: false, fresh: true, locked: false, broken: false },
    { id: "potion", name: "Healing Draught", icon: "potion", quantity: 12, rarity: "common", kind: "Potion", equipped: false, fresh: false, locked: false, broken: false },
    { id: "bomb", name: "Alchemist’s Fire", icon: "bomb", quantity: 3, rarity: "uncommon", kind: "Thrown", equipped: false, fresh: false, locked: false, broken: false },
    { id: "key", name: "Ferryman’s Key", icon: "key", quantity: 1, rarity: "uncommon", kind: "Quest item", equipped: false, fresh: false, locked: true, broken: false },
    { id: "gem", name: "Moonstone", icon: "gem", quantity: 3, rarity: "mythic", kind: "Gem", equipped: false, fresh: false, locked: false, broken: false },
    { id: "bow", name: "Splintered Yew Bow", icon: "bolt", quantity: 1, rarity: "common", kind: "Bow", equipped: false, fresh: false, locked: false, broken: true },
    { id: "apple", name: "Marsh Apple", icon: "apple", quantity: 6, rarity: "common", kind: "Food", equipped: false, fresh: false, locked: false, broken: false },
    { id: "leaf", name: "Bitterleaf", icon: "leaf", quantity: 9, rarity: "common", kind: "Herb", equipped: false, fresh: false, locked: false, broken: false },
    { id: "drop", name: "Vial of Black Water", icon: "drop", quantity: 1, rarity: "rare", kind: "Quest item", equipped: false, fresh: false, locked: true, broken: false },
    { id: "trophy", name: "Heron Feather Charm", icon: "trophy", quantity: 1, rarity: "uncommon", kind: "Trinket", equipped: false, fresh: false, locked: false, broken: false },
    { id: "music", name: "Bone Flute", icon: "music", quantity: 1, rarity: "common", kind: "Curio", equipped: false, fresh: false, locked: false, broken: false },
    { id: "fire", name: "Ember Shard", icon: "fire", quantity: 2, rarity: "epic", kind: "Reagent", equipped: false, fresh: false, locked: false, broken: false },
  ],
  selected: "shield",
  weight: "42.5 / 60",
  load: 0.71,
  gold: "1,284",
  detailName: "Oakheart Shield",
  detailIcon: "shield",
  detailRarity: "rare",
  detailKind: "Shield · Rare",
  detailText: "Bands of bog-oak bound in black iron. It drinks the first blow of every fight.",
  detailStats: [
    { id: "armour", label: "Armour", value: "38", delta: "+6", change: "better" },
    { id: "block", label: "Block chance", value: "22%", delta: "+4%", change: "better" },
    { id: "weight", label: "Weight", value: "6.0", delta: "+1.5", change: "worse" },
  ],
  detailValue: "Worth 240 gold",
  canUse: false,
  canEquip: true,
  equipLabel: "",
  canDrop: true,
});

export const equipment = (art: Art) => ({
  name: "Brenna Ashford",
  subtitle: "Level 7 Fighter · Human",
  portrait: art.hero,
  selected: "off",
  slots: [
    { id: "head", label: "Head", icon: "crown", name: "Warden’s Helm", rarity: "uncommon", empty: false, locked: false },
    { id: "neck", label: "Neck", icon: "", name: "", rarity: "common", empty: true, locked: false },
    { id: "chest", label: "Chest", icon: "shield", name: "Brigandine", rarity: "rare", empty: false, locked: false },
    { id: "hands", label: "Hands", icon: "", name: "", rarity: "common", empty: true, locked: false },
    { id: "main", label: "Main hand", icon: "sword", name: "Emberwake Blade", rarity: "epic", empty: false, locked: false },
    { id: "off", label: "Off hand", icon: "", name: "", rarity: "common", empty: true, locked: false },
    { id: "ring1", label: "Ring", icon: "gem", name: "Moonstone Band", rarity: "legendary", empty: false, locked: true },
    { id: "ring2", label: "Ring", icon: "", name: "", rarity: "common", empty: true, locked: false },
  ],
  stats: [
    { id: "ac", label: "Armour class", value: "17", delta: "+2", change: "better" },
    { id: "hp", label: "Hit points", value: "64 / 64", delta: "", change: "same" },
    { id: "speed", label: "Speed", value: "9 m", delta: "−1.5 m", change: "worse" },
    { id: "init", label: "Initiative", value: "+2", delta: "", change: "same" },
  ],
});

export const journal = () => ({
  title: "Journal",
  filter: "active",
  filters: [
    { id: "active", label: "Active", count: 6 },
    { id: "done", label: "Completed", count: 23 },
    { id: "failed", label: "Failed", count: 2 },
  ],
  quests: [
    { id: "ferry", name: "The Drowned Ferryman", region: "Saltmarsh", level: "Lv 8", kind: "main", tracked: true, fresh: false, state: "active" },
    { id: "bells", name: "Bells Beneath the Abbey of Saint Aldric the Unremembered", region: "Greyfen Abbey", level: "Lv 6", kind: "side", tracked: false, fresh: true, state: "active" },
    { id: "pelts", name: "Wolf Pelts for Marta", region: "Hollin", level: "", kind: "task", tracked: false, fresh: false, state: "active" },
    { id: "heron", name: "What the Heron Saw", region: "Saltmarsh", level: "Lv 9", kind: "side", tracked: false, fresh: false, state: "active" },
    { id: "toll", name: "An Old Toll, Unpaid", region: "Causeway", level: "Lv 7", kind: "side", tracked: false, fresh: false, state: "done" },
    { id: "lamp", name: "The Lamplighter’s Daughter", region: "Hollin", level: "Lv 5", kind: "side", tracked: false, fresh: false, state: "failed" },
  ],
  selected: "ferry",
  detailName: "The Drowned Ferryman",
  detailMeta: "Main quest · Saltmarsh · Lv 8",
  detailText: "The ferryman’s lantern still burns over black water, though no one has seen his boat in a year. Old Ysolde says he took a debt to the bottom with him, and that the marsh will not give it back until its beacons burn again.",
  objectives: [
    { id: "o1", text: "Speak with Old Ysolde at the ferry landing", progress: "", done: true, failed: false, optional: false },
    { id: "o2", text: "Light the marsh beacons", progress: "2 / 3", done: false, failed: false, optional: false },
    { id: "o3", text: "Find the ferryman’s ledger", progress: "", done: false, failed: false, optional: true },
    { id: "o4", text: "Keep the toll-keeper alive", progress: "", done: false, failed: true, optional: true },
  ],
  rewards: [
    { id: "xp", icon: "star", label: "850 XP" },
    { id: "gold", icon: "coin", label: "120 gold" },
    { id: "item", icon: "key", label: "Ferryman’s Oar" },
  ],
  tracked: true,
  canTrack: true,
  canLocate: true,
});

export const tracker = () => ({
  quest: "The Drowned Ferryman",
  distance: "240 m",
  objectives: [
    { id: "o2", text: "Light the marsh beacons", progress: "2 / 3", done: false, optional: false },
    { id: "o3", text: "Find the ferryman’s ledger", progress: "", done: false, optional: true },
  ],
  updated: "o2",
});

export const dialogue = (art: Art) => ({
  speaker: "Old Ysolde",
  epithet: "Ferry warden",
  portrait: art.speaker,
  line: "He rowed out with the tithe and never came back. Light the beacons, and the water gives up what it keeps — but it always takes something in trade.",
  choices: [
    { id: "c2", key: "1", text: "You know more than you are telling me.", check: "Insight · DC 12", seen: false, locked: false, ends: false },
    { id: "c3", key: "2", text: "Pay me first, and I will see what the water keeps.", check: "Requires 50 gold", seen: false, locked: true, ends: false },
    { id: "c4", key: "3", text: "Who lit the beacons last?", check: "", seen: true, locked: false, ends: false },
    { id: "c5", key: "4", text: "I will see to it. Farewell.", check: "", seen: false, locked: false, ends: true },
  ],
  selected: "c2",
  canContinue: false,
  canLog: true,
});

export const party = (art: Art) => ({
  selected: "brenna",
  orientation: "vertical",
  compact: false,
  members: [
    { id: "brenna", name: "Brenna", portrait: art.hero, level: 7, health: 48, healthMax: 64, resource: 0, resourceMax: 0, status: "", active: true, down: false },
    { id: "idris", name: "Idris", portrait: art.mage, level: 7, health: 21, healthMax: 38, resource: 14, resourceMax: 30, status: "Concentrating", active: false, down: false },
    { id: "maren", name: "Maren", portrait: art.cleric, level: 6, health: 9, healthMax: 52, resource: 4, resourceMax: 12, status: "Poisoned · Blessed", active: false, down: false },
    { id: "sela", name: "Sela", portrait: art.rogue, level: 6, health: 0, healthMax: 42, resource: 0, resourceMax: 0, status: "Unconscious", active: false, down: true },
  ],
});

export const turns = (art: Art) => ({
  round: 3,
  selected: "",
  entries: [
    { id: "brenna", name: "Brenna", portrait: art.hero, side: "ally", current: true, done: false },
    { id: "oars1", name: "Drowned Oarsman", portrait: 0, side: "enemy", current: false, done: false },
    { id: "idris", name: "Idris", portrait: art.mage, side: "ally", current: false, done: false },
    { id: "heron", name: "Marsh Heron", portrait: 0, side: "neutral", current: false, done: false },
    { id: "oars2", name: "Drowned Oarsman", portrait: 0, side: "enemy", current: false, done: false },
    { id: "maren", name: "Maren", portrait: art.cleric, side: "ally", current: false, done: false },
    { id: "lurker", name: "Bog Lurker", portrait: 0, side: "enemy", current: false, done: false },
    { id: "sela", name: "Sela", portrait: art.rogue, side: "ally", current: false, done: true },
  ],
});

export const map = (art: Art) => ({
  map: art.map,
  region: "Saltmarsh",
  selected: "abbey",
  detailName: "Greyfen Abbey",
  detailText: "Waypoint · 1.2 km",
  canTravel: true,
  canTrack: true,
  markers: [
    { id: "you", x: 0.47, y: 0.55, icon: "person", label: "You", kind: "player", tracked: false, discovered: true },
    { id: "beacon", x: 0.63, y: 0.38, icon: "flag", label: "Marsh beacon", kind: "quest", tracked: true, discovered: true },
    { id: "abbey", x: 0.22, y: 0.46, icon: "home", label: "Greyfen Abbey", kind: "waypoint", tracked: false, discovered: true },
    { id: "hollin", x: 0.36, y: 0.78, icon: "home", label: "Hollin", kind: "town", tracked: false, discovered: true },
    { id: "lair", x: 0.8, y: 0.72, icon: "skull", label: "Drowned Hollow", kind: "danger", tracked: false, discovered: false },
  ],
  filters: [
    { id: "quest", label: "Quests", icon: "flag", on: true },
    { id: "waypoint", label: "Waypoints", icon: "home", on: true },
    { id: "danger", label: "Dangers", icon: "skull", on: false },
  ],
});

export const compass = () => ({
  facing: "NE",
  points: [
    { id: "n", x: 0.12, label: "N" },
    { id: "ne", x: 0.5, label: "NE" },
    { id: "e", x: 0.88, label: "E" },
  ],
  markers: [
    { id: "abbey", x: 0.28, icon: "home", kind: "waypoint", distance: "1.2 km", above: false, below: false },
    { id: "beacon", x: 0.62, icon: "flag", kind: "quest", distance: "240 m", above: false, below: false },
    { id: "lair", x: 0.84, icon: "skull", kind: "danger", distance: "", above: false, below: true },
  ],
});
