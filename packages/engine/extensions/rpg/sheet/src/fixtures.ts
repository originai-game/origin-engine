// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning), at the density of the reference boards: a full backpack
// over every rarity, a grouped quest log with finished quests, a party with
// one member down and five combatants in the turn order.

export const ITEM_NAMES = [
  "blade", "shield", "circlet", "potion", "fire", "key", "moonstone", "bow",
  "apple", "herb", "vial", "feather", "flute", "ember", "purse", "scroll",
  "helm", "amulet", "brigandine", "gloves", "belt", "boots", "ring", "cloak",
  "lantern", "axe", "mace", "staff", "dagger", "bread", "torch", "map",
];

export const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

export const PICTURES = ["brenna", "idris", "maren", "sela", "ysolde", "oarsman", "lurker", "abbey", "causeway", "map"] as const;
type Pictures = Record<string, number>;

const item = (id: string, name: string, icon: string, rarity: string, count = 1, more: Partial<{ equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }> = {}) =>
  ({ id, name, icon: `item-${icon}`, count, rarity, equipped: false, fresh: false, locked: false, broken: false, ...more });

export const INVENTORY = (p: Pictures) => ({
  bagTitle: "Backpack",
  bagTab: "all",
  bagTabs: [
    { id: "all", label: "All" }, { id: "weapons", label: "Weapons" }, { id: "armour", label: "Armour" },
    { id: "trinkets", label: "Accessories" }, { id: "use", label: "Consumables" }, { id: "craft", label: "Materials" },
  ],
  bagItems: [
    item("emberwake", "Emberwake Blade", "blade", "legendary", 1, { equipped: true }),
    item("cleaver", "Woodsman's Axe", "axe", "rare"),
    item("rod", "Stormglass Staff", "staff", "epic"),
    item("cloak", "Fernshade Cloak", "cloak", "uncommon"),
    item("brig", "Riveted Brigandine", "brigandine", "rare", 1, { equipped: true }),
    item("gloves", "Hide Gloves", "gloves", "common"),
    item("helm", "Gilded Helm", "helm", "rare"),
    item("ward", "Oakheart Shield", "shield", "epic"),
    item("boots", "Wayfarer Boots", "boots", "uncommon"),
    item("ring", "Ruby Signet", "ring", "common"),
    item("ring2", "Sapphire Band", "ring", "rare", 1, { fresh: true }),
    item("amulet", "Sunstone Amulet", "amulet", "legendary"),
    item("potion", "Healing Draught", "potion", "common", 5),
    item("vial", "Moonwater Vial", "vial", "common", 3),
    item("ember", "Ember Salts", "ember", "common", 2),
    item("scroll", "Scroll of Return", "scroll", "common", 4),
    item("moon", "Moonstone Shard", "moonstone", "epic", 12),
    item("herb", "Silverleaf", "herb", "uncommon", 8),
    item("apple", "Red Apple", "apple", "uncommon", 6),
    item("bread", "Travel Bread", "bread", "common", 14),
    item("fire", "Fire Crystal", "fire", "legendary", 4),
    item("feather", "Griffin Feather", "feather", "common", 7),
    item("key", "Crypt Key", "key", "common", 2, { locked: true }),
    item("circlet", "Silver Circlet", "circlet", "epic"),
    item("bow", "Ashen Longbow", "bow", "common"),
    item("mace", "Iron Mace", "mace", "common", 1, { broken: true }),
    item("dagger", "Thorn Dagger", "dagger", "uncommon"),
    item("flute", "Bone Flute", "flute", "rare"),
    item("purse", "Coin Purse", "purse", "common"),
    item("lantern", "Hooded Lantern", "lantern", "common"),
    item("torch", "Pitch Torch", "torch", "common", 3),
    item("map", "Old Chart", "map", "rare", 1, { fresh: true }),
    item("belt", "Ranger's Belt", "belt", "uncommon"),
    item("apple2", "Golden Apple", "apple", "mythic"),
    item("vial2", "Venom Vial", "vial", "uncommon", 2),
    item("herb2", "Nightcap", "herb", "rare", 3),
  ],
  bagSelected: "emberwake",
  bagWeight: "92.4 / 120",
  bagLoad: 0.77,
  bagGold: "4,327",

  cardName: "Emberwake Blade",
  cardIcon: "item-blade",
  cardRarity: "legendary",
  cardKind: "Legendary · Longsword",
  cardLead: [{ id: "dmg", icon: "sword", label: "Damage", value: "12–20", delta: "+4", trend: "up", current: "8–16" }],
  cardStats: [
    { id: "atk", icon: "star", label: "Attack bonus", value: "+7", delta: "+2", trend: "up", current: "+5" },
    { id: "crit", icon: "bolt", label: "Critical chance", value: "12%", delta: "+4%", trend: "up", current: "8%" },
    { id: "fire", icon: "fire", label: "Fire damage", value: "+6", delta: "+6", trend: "up", current: "—" },
    { id: "res", icon: "shield", label: "Spell resistance", value: "+10%", delta: "−5%", trend: "down", current: "+15%" },
  ],
  cardLore: "“Quenched in the last ember of a dying hearth, it still remembers the warmth it was made to guard.”",
  cardPrimary: "Equip",
  cardSecondary: "Drop",

  charName: "Brenna Vale",
  charLine: "Ranger · Level 12",
  charPortrait: p.brenna,
  charFigure: p.brenna,
  charSlots: [
    { id: "hand", side: "start", label: "Weapon", icon: "item-blade", rarity: "legendary" },
    { id: "head", side: "start", label: "Head", icon: "item-helm", rarity: "rare" },
    { id: "body", side: "start", label: "Body", icon: "item-brigandine", rarity: "rare" },
    { id: "hands", side: "start", label: "Hands", icon: "item-gloves", rarity: "uncommon" },
    { id: "feet", side: "start", label: "Feet", icon: "item-boots", rarity: "uncommon" },
    { id: "neck", side: "end", label: "Amulet", icon: "item-amulet", rarity: "legendary" },
    { id: "finger", side: "end", label: "Ring", icon: "item-ring", rarity: "common" },
    { id: "waist", side: "end", label: "Belt", icon: "item-belt", rarity: "uncommon" },
    { id: "back", side: "end", label: "Cloak", icon: "item-cloak", rarity: "epic" },
    { id: "off", side: "end", label: "Off hand", icon: "", rarity: "common" },
  ],
  charSelected: "hand",
  charAttributes: [
    { id: "vit", group: "a", icon: "heart", label: "Vitality", value: "142" },
    { id: "arm", group: "a", icon: "shield", label: "Armour", value: "212" },
    { id: "eva", group: "a", icon: "bolt", label: "Evasion", value: "18%" },
    { id: "res", group: "a", icon: "drop", label: "Resistance", value: "15%" },
    { id: "str", group: "b", icon: "sword", label: "Strength", value: "14" },
    { id: "dex", group: "b", icon: "leaf", label: "Dexterity", value: "20" },
    { id: "int", group: "b", icon: "star", label: "Intellect", value: "12" },
    { id: "wil", group: "b", icon: "gem", label: "Willpower", value: "16" },
  ],
});

export const JOURNAL = (p: Pictures) => ({
  logTitle: "Quest Journal",
  logGroups: [
    { id: "main", label: "Main Quests", done: false },
    { id: "side", label: "Side Quests", done: false },
    { id: "done", label: "Completed", done: true },
  ],
  logQuests: [
    { id: "lantern", group: "main", name: "The Drowned Lantern", tracked: true, fresh: false },
    { id: "tithe", group: "main", name: "A Tithe of Salt", tracked: false, fresh: true },
    { id: "heron", group: "side", name: "The Grey Heron", tracked: false, fresh: false },
    { id: "ferry", group: "side", name: "Toll for the Ferryman", tracked: false, fresh: false },
    { id: "bells", group: "side", name: "Bells Under the Mere", tracked: false, fresh: false },
    { id: "moss", group: "side", name: "Moss and Mortar", tracked: false, fresh: false },
    { id: "oath", group: "done", name: "The Abbot's Oath", tracked: false, fresh: false },
    { id: "nets", group: "done", name: "Mending the Nets", tracked: false, fresh: false },
    { id: "eel", group: "done", name: "An Eel for the Prior", tracked: false, fresh: false },
  ],
  logSelected: "lantern",

  questName: "The Drowned Lantern",
  questSummary: "The abbey's lantern went dark the night the causeway flooded. The brothers say it still burns somewhere beneath the mere, and that whoever relights it will hold the crossing.",
  questObjectives: [
    { id: "o1", text: "Cross the flooded causeway", done: false, optional: false, step: 0 },
    { id: "o2", text: "Speak with the oarsman", done: true, optional: false, step: 1 },
    { id: "o3", text: "Find the sunken belfry", done: false, optional: false, step: 1 },
    { id: "o4", text: "Bring salt for the lurker", done: false, optional: true, step: 1 },
  ],
  questRewards: [
    { id: "gold", icon: "coin", label: "900 Gold", detail: "" },
    { id: "xp", icon: "gem", label: "320 XP", detail: "" },
    { id: "relic", icon: "crown", label: "The Abbey Lantern", detail: "Unique relic" },
  ],
  questScenery: p.abbey,

  mapTexture: p.map,
  mapMarkers: [
    { id: "abbey", kind: "quest", x: 0.47, y: 0.3, label: "Saltmere Abbey" },
    { id: "w1", kind: "waypoint", x: 0.22, y: 0.18, label: "Reedford" },
    { id: "w2", kind: "waypoint", x: 0.18, y: 0.6, label: "Gullstone" },
    { id: "w3", kind: "waypoint", x: 0.72, y: 0.22, label: "Hollin Mill" },
    { id: "w4", kind: "waypoint", x: 0.62, y: 0.58, label: "Ferry Steps" },
    { id: "d1", kind: "danger", x: 0.36, y: 0.38, label: "Drowned Barrows" },
    { id: "d2", kind: "danger", x: 0.84, y: 0.4, label: "Lurker's Reach" },
    { id: "me", kind: "player", x: 0.4, y: 0.62, label: "You" },
  ],
  mapSelected: "abbey",
  mapLegend: [
    { id: "quest", kind: "quest", label: "Quest" },
    { id: "waypoint", kind: "waypoint", label: "Waypoint" },
    { id: "danger", kind: "danger", label: "Danger" },
  ],
  mapHere: "Current location",

  placeName: "Saltmere Abbey",
  placeAbout: "Its belfry leans over the mere, half a bell's height under water at high tide.",
  placePicture: p.abbey,

  talkSpeaker: "Ysolde of the Mere",
  talkPortrait: p.ysolde,
  talkLine: "The water remembers every bell it swallowed. Are you certain you want to hear them ring again?",
  talkReplies: [
    { id: "r1", number: 1, text: "Yes. The lantern must burn again.", used: false },
    { id: "r2", number: 2, text: "Tell me first what the brothers fear.", used: false },
    { id: "r3", number: 3, text: "Not yet. I need to gather supplies.", used: true },
  ],
  talkSelected: "r1",

  compassHeading: "N",
  compassMarks: [
    { id: "w", kind: "cardinal", label: "W", at: 0.04 },
    { id: "t1", kind: "tick", label: "", at: 0.15 },
    { id: "t2", kind: "tick", label: "", at: 0.27 },
    { id: "t3", kind: "tick", label: "", at: 0.38 },
    { id: "n", kind: "cardinal", label: "N", at: 0.44 },
    { id: "t4", kind: "tick", label: "", at: 0.62 },
    { id: "t5", kind: "tick", label: "", at: 0.73 },
    { id: "e", kind: "cardinal", label: "E", at: 0.85 },
    { id: "t6", kind: "tick", label: "", at: 0.94 },
    { id: "q", kind: "quest", label: "", at: 0.6 },
  ],

  partyMembers: [
    { id: "brenna", name: "Brenna", portrait: p.brenna, level: 8, health: 62, healthMax: 72, mana: 48, manaMax: 72, down: false },
    { id: "idris", name: "Idris", portrait: p.idris, level: 8, health: 74, healthMax: 82, mana: 36, manaMax: 54, down: false },
    { id: "maren", name: "Maren", portrait: p.maren, level: 7, health: 0, healthMax: 60, mana: 0, manaMax: 80, down: true },
    { id: "sela", name: "Sela", portrait: p.sela, level: 8, health: 58, healthMax: 70, mana: 66, manaMax: 84, down: false },
  ],
  partySelected: "brenna",

  turnRound: 3,
  turns: [
    { id: "t-brenna", name: "Brenna", portrait: p.brenna, side: "ally", acting: true, down: false },
    { id: "t-idris", name: "Idris", portrait: p.idris, side: "ally", acting: false, down: false },
    { id: "t-maren", name: "Maren", portrait: p.maren, side: "ally", acting: false, down: true },
    { id: "t-sela", name: "Sela", portrait: p.sela, side: "ally", acting: false, down: false },
    { id: "t-lurker", name: "The Lurker", portrait: p.lurker, side: "enemy", acting: false, down: false },
  ],
});
