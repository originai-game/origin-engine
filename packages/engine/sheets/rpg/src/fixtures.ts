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

export const SKILL_NAMES = ["fire", "shield", "arcane", "nature", "blood", "dash", "frost", "mend"];
export const CMD_NAMES = ["attack", "skills", "spells", "items", "defend", "flee", "health", "mana"];

export const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

export const PICTURES = ["blade", "brenna-full", "brenna", "idris", "maren", "sela", "ysolde", "oarsman", "lurker", "abbey", "causeway", "map", "scene"] as const;
type Pictures = Record<string, number>;

const item = (id: string, name: string, icon: string, rarity: string, count = 1, more: Partial<{ equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }> = {}) =>
  ({ id, name, icon: `item-${icon}`, count, rarity, equipped: false, fresh: false, locked: false, broken: false, ...more });

export const INVENTORY = (p: Pictures) => ({
  bagTitle: "Backpack",
  bagKicker: "Inventory",
  bagCapacity: "36 / 48 slots",
  bagTab: "all",
  bagTabs: [
    { id: "all", label: "All", count: 36 }, { id: "weapons", label: "Weapons", count: 7 }, { id: "armour", label: "Armour", count: 7 },
    { id: "trinkets", label: "Accessories", count: 4 }, { id: "use", label: "Consumables", count: 11 }, { id: "craft", label: "Materials" },
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
  cardArt: p.blade,
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
  cardSecondaryEquip: "Details",
  cardUnequip: "Unequip",

  charName: "Brenna Vale",
  charLine: "Ranger · Level 12",
  charPortrait: p.brenna,
  charFigure: p["brenna-full"],
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
    { id: "res", group: "a", icon: "drop", label: "Resist", value: "15%" },
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
    { id: "lantern", group: "main", name: "The Sunken Bell", tracked: true, fresh: false },
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

  questName: "The Sunken Bell",
  questSummary: "The abbey's bell fell silent the night the causeway flooded. The brothers say it still rings beneath the mere, and that whoever raises it will hold the crossing.",
  questObjectives: [
    { id: "o1", text: "Cross the flooded causeway", done: false, optional: false, step: 0 },
    { id: "o2", text: "Speak with the oarsman", done: true, optional: false, step: 1 },
    { id: "o3", text: "Find the sunken belfry", done: false, optional: false, step: 1 },
    { id: "o4", text: "Bring salt for the lurker", done: false, optional: true, step: 1 },
  ],
  questRewards: [
    { id: "gold", icon: "coin", label: "900 Gold", detail: "" },
    { id: "xp", icon: "gem", label: "320 XP", detail: "" },
    { id: "relic", icon: "crown", label: "The Abbey Bell", detail: "Unique relic" },
  ],
  questScenery: p.abbey,
  questLabel: "Main quest",

  mapTexture: p.map,
  mapMarkers: [
    { id: "abbey", kind: "quest", x: 0.42, y: 0.2, label: "Saltmere Abbey" },
    { id: "w1", kind: "waypoint", x: 0.15, y: 0.37, label: "Reedford" },
    { id: "w2", kind: "waypoint", x: 0.7, y: 0.53, label: "Gullstone" },
    { id: "w3", kind: "waypoint", x: 0.13, y: 0.62, label: "Hollin Mill" },
    { id: "w4", kind: "waypoint", x: 0.47, y: 0.6, label: "Ferry Steps" },
    { id: "d1", kind: "danger", x: 0.73, y: 0.29, label: "Drowned Barrows" },
    { id: "d2", kind: "danger", x: 0.88, y: 0.66, label: "Lurker's Reach" },
    { id: "me", kind: "player", x: 0.34, y: 0.5, label: "You" },
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
    { id: "r1", number: 1, text: "Yes. The bell must ring again.", used: false },
    { id: "r2", number: 2, text: "Tell me first what the brothers fear.", used: false },
    { id: "r3", number: 3, text: "Not yet. I need to gather supplies.", used: true },
  ],
  talkSelected: "r1",

  // Compass: heading and bearings in radians clockwise from north.
  compassHeading: 0,
  compassPois: [
    { id: "q", bearing: 0.5, distance: 240, kind: "objective", icon: "star", label: "Quest" },
    { id: "camp", bearing: 5.6, distance: 85, kind: "poi", icon: "flag", label: "Camp" },
  ],

  partyMembers: [
    { id: "brenna", name: "Brenna", portrait: p.brenna, level: 8, health: 62, healthMax: 72, mana: 48, manaMax: 72, down: false },
    { id: "idris", name: "Idris", portrait: p.idris, level: 8, health: 74, healthMax: 82, mana: 36, manaMax: 54, down: false },
    { id: "maren", name: "Maren", portrait: p.maren, level: 7, health: 0, healthMax: 60, mana: 0, manaMax: 80, down: true },
    { id: "sela", name: "Sela", portrait: p.sela, level: 8, health: 58, healthMax: 70, mana: 66, manaMax: 84, down: false },
  ],
  partySelected: "brenna",

  exploreScene: p.scene,

  turnRound: 3,
  turns: [
    { id: "t-brenna", name: "Brenna", portrait: p.brenna, side: "ally", acting: true, down: false },
    { id: "t-idris", name: "Idris", portrait: p.idris, side: "ally", acting: false, down: false },
    { id: "t-maren", name: "Maren", portrait: p.maren, side: "ally", acting: false, down: true },
    { id: "t-sela", name: "Sela", portrait: p.sela, side: "ally", acting: false, down: false },
    { id: "t-lurker", name: "The Lurker", portrait: p.lurker, side: "enemy", acting: false, down: false },
  ],

  logLines: [
    { id: "l1", text: "Round 3 begins.", tone: "status", detail: "" },
    { id: "l2", text: "Brenna's Storm Arrow strikes the Lurker.", tone: "damage", detail: "→ 42 damage" },
    { id: "l3", text: "The Lurker is Soaked.", tone: "status", detail: "" },
    { id: "l4", text: "The Lurker's Undertow hits Maren.", tone: "damage", detail: "→ 31 damage" },
    { id: "l5", text: "Maren falls.", tone: "crit", detail: "" },
    { id: "l6", text: "Sela casts Mend on Idris.", tone: "heal", detail: "→ 24 health" },
  ],
  objectiveName: "Objective",
  objectiveSteps: [
    { id: "g1", text: "Defeat the Lurker", done: false },
    { id: "g2", text: "Keep Sela standing", done: false },
  ],
  enemyName: "The Lurker",
  enemyPortrait: p.lurker,
  enemyLevel: 14,
  enemyHealth: 418,
  enemyHealthMax: 760,
  enemyEffects: [
    { id: "e1", icon: "drop", label: "Soaked", tone: "harm" },
    { id: "e2", icon: "bolt", label: "Weak to lightning", tone: "weak" },
    { id: "e3", icon: "fire", label: "Resists fire", tone: "strong" },
  ],
  commands: [
    { id: "attack", label: "Attack", icon: "cmd-attack", enabled: true },
    { id: "skills", label: "Skills", icon: "cmd-skills", enabled: true },
    { id: "spells", label: "Spells", icon: "cmd-spells", enabled: true },
    { id: "items", label: "Items", icon: "cmd-items", enabled: true },
    { id: "defend", label: "Defend", icon: "cmd-defend", enabled: true },
    { id: "flee", label: "Flee", icon: "cmd-flee", enabled: false },
  ],
  commandSelected: "attack",
});

export const EQUIPMENT = () => ({
  menuTabs: [
    { id: "character", label: "Character" }, { id: "inventory", label: "Inventory" },
    { id: "skills", label: "Skills" }, { id: "quests", label: "Quests" }, { id: "map", label: "Map" },
  ],
  menuTab: "character",
  equipTitle: "Equipment",
  equipSlots: [
    { id: "head", side: "start", label: "Head", icon: "item-helm", rarity: "rare" },
    { id: "neck", side: "start", label: "Amulet", icon: "item-amulet", rarity: "legendary" },
    { id: "body", side: "start", label: "Body", icon: "item-brigandine", rarity: "rare" },
    { id: "hands", side: "start", label: "Hands", icon: "item-gloves", rarity: "uncommon" },
    { id: "hand", side: "start", label: "Main hand", icon: "item-blade", rarity: "legendary" },
    { id: "back", side: "end", label: "Cloak", icon: "item-cloak", rarity: "epic" },
    { id: "finger", side: "end", label: "Ring", icon: "item-ring", rarity: "common" },
    { id: "waist", side: "end", label: "Belt", icon: "item-belt", rarity: "uncommon" },
    { id: "feet", side: "end", label: "Feet", icon: "item-boots", rarity: "uncommon" },
    { id: "off", side: "end", label: "Off hand", icon: "item-shield", rarity: "epic" },
  ],
  noAttributes: [],
  statsTitle: "Stats",
  statGroups: [
    { id: "res", column: "a" }, { id: "off", column: "a" }, { id: "def", column: "a" },
    { id: "resist", column: "b" }, { id: "load", column: "b" },
  ],
  statRows: [
    { id: "hp", group: "res", icon: "heart", label: "Health", value: "286 / 340" },
    { id: "mp", group: "res", icon: "drop", label: "Mana", value: "128 / 180" },
    { id: "st", group: "res", icon: "bolt", label: "Stamina", value: "96 / 120" },
    { id: "atk", group: "off", icon: "sword", label: "Attack power", value: "184" },
    { id: "crit", group: "off", icon: "star", label: "Critical chance", value: "12%" },
    { id: "cdmg", group: "off", icon: "fire", label: "Critical damage", value: "160%" },
    { id: "arm", group: "def", icon: "shield", label: "Armour", value: "212" },
    { id: "eva", group: "def", icon: "leaf", label: "Dodge chance", value: "14%" },
    { id: "cdr", group: "def", icon: "clock", label: "Cooldown cut", value: "8%" },
    { id: "rf", group: "resist", icon: "fire", label: "Fire resist", value: "20%" },
    { id: "rc", group: "resist", icon: "gem", label: "Frost resist", value: "10%" },
    { id: "rs", group: "resist", icon: "drop", label: "Shadow resist", value: "12%" },
    { id: "rp", group: "resist", icon: "skull", label: "Poison resist", value: "15%" },
    { id: "ra", group: "resist", icon: "star", label: "Arcane resist", value: "6%" },
    { id: "wt", group: "load", icon: "bag", label: "Equip load", value: "92 / 120" },
    { id: "bal", group: "load", icon: "person", label: "Balance", value: "64%" },
    { id: "luck", group: "load", icon: "leaf", label: "Luck", value: "9%" },
    { id: "gold", group: "load", icon: "coin", label: "Gold find", value: "18%" },
    { id: "move", group: "load", icon: "flag", label: "Move speed", value: "104%" },
  ],
  footBack: "Back",
  footUnequip: "Unequip",
  footAuto: "Auto-equip",
  footDetails: "Details",
});

// The fixtures' player-facing text goes through the string tables as a
// game's own data would: a top-level text store is the key `sheet.<store>`,
// a row's text field `sheet.<store>.<id>.<field>`. The English in this file
// is the source of assets/strings/en.ogt; `tr` answers each key in the active
// locale. Ids, icons, tones, figures and selections stay as they are.
const ROW_TEXT = new Set(["label", "name", "text", "detail"]);
const NOT_TEXT = /(Selected|Tab|Icon|Rarity|Weight|Gold|Distance|Heading)$|^page$/;

export function localized<T extends Record<string, unknown>>(data: T, tr: (key: string, en: string) => string): T {
  const out: Record<string, unknown> = {};
  for (const [store, v] of Object.entries(data)) {
    if (typeof v === "string") out[store] = v && !NOT_TEXT.test(store) ? tr(`sheet.${store}`, v) : v;
    else if (Array.isArray(v)) out[store] = v.map((row: Record<string, unknown>) => Object.fromEntries(Object.entries(row).map(([f, x]) =>
      [f, typeof x === "string" && x && ROW_TEXT.has(f) ? tr(`sheet.${store}.${row.id}.${f}`, x) : x])));
    else out[store] = v;
  }
  return out as T;
}

// EXPLORE is the action HUD mid-fight, laid out as the approved mockup: the
// leader's vitals and effects over the companions, the boss's bar, the
// minimap with the place and the tracked quest, the lock-on with the damage
// dealt and a heal, the combo, the loot feed, the finisher prompt over the
// skill bar, the input hints and the experience line. A field naming sample
// art (portrait, icon, map) holds its file under assets/art (hud-<name>.jpg),
// which main.ts loads and replaces with the texture. It is sheet data, kept
// in English (it does not go through the string tables).
export const EXPLORE = {
  leader: { name: "Ysolde", level: 42, portrait: "ysolde", health: 2184, healthMax: 3420, ghost: 2668, mana: 72, manaMax: 100, stamina: 5, staminaMax: 8, segment: 1, low: 0.25 },
  effects: [
    { id: "fury", name: "Fury", icon: "fury", glyph: "bolt", remaining: 0.7, label: "0:12", harmful: false },
    { id: "ward", name: "Rune ward", icon: "ward", glyph: "shield", remaining: 0.42, label: "×3", harmful: false },
    { id: "hex", name: "Hex", icon: "hex", glyph: "skull", remaining: 0.3, label: "4.2", harmful: true },
  ],
  companions: [
    { id: "brannoch", name: "Brannoch", portrait: "brannoch", health: 91, healthMax: 100, ghost: 0, mana: 40, manaMax: 100, key: "2", fallen: false },
    { id: "sylvenne", name: "Sylvenne", portrait: "sylvenne", health: 18, healthMax: 100, ghost: 44, mana: 88, manaMax: 100, key: "3", fallen: false },
    { id: "caelum", name: "Caelum", portrait: "caelum", health: 0, healthMax: 100, ghost: 0, mana: 0, manaMax: 100, key: "4", fallen: true },
  ],
  boss: { name: "Ashen Colossus", epithet: "Warden of the Ninefold Gate", value: 5800, max: 10000, ghost: 7100, phase: "storm", stagger: 0.7, staggerCells: 10 },
  bossPhases: [
    { id: "shell", label: "Shell", complete: true },
    { id: "storm", label: "Storm", complete: false },
    { id: "core", label: "Core", complete: false },
  ],
  weaknesses: [
    { id: "fire", name: "Fire", icon: "fire", glyph: "fire" },
    { id: "frost", name: "Frost", icon: "ice", glyph: "drop" },
  ],
  minimap: { map: "map", centerX: 0.5, centerY: 0.5, zoom: 0.5, heading: 0, place: "Ashen Vale", clock: "06:40" },
  blips: [
    { id: "boss", kind: "boss", x: 0.56, y: 0.4, label: "Ashen Colossus" },
    { id: "gate", kind: "objective", x: 0.44, y: 0.37, label: "The Ninefold Gate" },
    { id: "brannoch", kind: "ally", x: 0.46, y: 0.55, label: "Brannoch" },
    { id: "sylvenne", kind: "ally", x: 0.53, y: 0.57, label: "Sylvenne" },
  ],
  quest: { label: "", name: "The Ninefold Gate", distance: "" },
  questSteps: [
    { id: "ruins", text: "Reach the highland ruins", done: true, tag: "" },
    { id: "guard", text: "Break the Colossus’s guard", done: false, tag: "1 / 3" },
    { id: "pillars", text: "Light the three rune pillars", done: false, tag: "" },
  ],
  target: "Ashen Colossus",
  hits: [
    { id: "h3", amount: 2481, kind: "critical", call: "Critical · Weak point", word: "" },
    { id: "h2", amount: 642, kind: "normal", call: "", word: "" },
    { id: "h1", amount: 118, kind: "burn", call: "", word: "burn" },
  ],
  heals: [{ id: "r1", amount: 320, kind: "heal", call: "", word: "" }],
  combo: { count: 27, word: "", remaining: 0.6, multiplier: 1.6 },
  loot: [
    { id: "blade", name: "Ember Blade", icon: "slash", glyph: "sword", rarity: "epic", label: "Epic", quantity: 0, currency: false, fading: false },
    { id: "gold", name: "Gold", icon: "gold", glyph: "coin", rarity: "legendary", label: "", quantity: 1240, currency: true, fading: false },
    { id: "shard", name: "Rune shard", icon: "shard", glyph: "gem", rarity: "common", label: "", quantity: 3, currency: false, fading: true },
  ],
  prompt: { key: "F", text: "Finisher: Sunder the guard", disabled: false, reason: "" },
  slots: [
    { id: "tonic", name: "Healing tonic", icon: "tonic", glyph: "potion", key: "Z", kind: "item", cooldown: 0, time: "", count: 5, ready: false, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 0 },
    { id: "ether", name: "Ether", icon: "ether", glyph: "drop", key: "X", kind: "item", cooldown: 0, time: "", count: 2, ready: false, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 0 },
    { id: "slash", name: "Ember slash", icon: "slash", glyph: "sword", key: "1", kind: "skill", cooldown: 0, time: "", count: 0, ready: true, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 0 },
    { id: "frost", name: "Frost lance", icon: "frost", glyph: "bolt", key: "2", kind: "skill", cooldown: 0.65, time: "8", count: 0, ready: false, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 0 },
    { id: "arc", name: "Arc step", icon: "arc", glyph: "star", key: "3", kind: "skill", cooldown: 0, time: "", count: 0, ready: false, unaffordable: false, reason: "", charges: 2, chargesMax: 3, charge: 0 },
    { id: "radiance", name: "Radiance", icon: "radiance", glyph: "star", key: "4", kind: "skill", cooldown: 0, time: "", count: 0, ready: false, unaffordable: true, reason: "No mana", charges: 0, chargesMax: 0, charge: 0 },
    { id: "gale", name: "Gale", icon: "gale", glyph: "map", key: "Q", kind: "skill", cooldown: 0.18, time: "2", count: 0, ready: false, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 0 },
    { id: "meteor", name: "Cataclysm", icon: "meteor", glyph: "fire", key: "R", kind: "ultimate", cooldown: 0, time: "", count: 0, ready: false, unaffordable: false, reason: "", charges: 0, chargesMax: 0, charge: 1 },
  ],
  hints: [
    { id: "target", key: "Tab", label: "Target" },
    { id: "dodge", key: "Space", label: "Dodge" },
  ],
  xp: { level: 42, value: 0.62 },
};

/** The explore page's sample art under assets/art (hud-<name>.jpg; sheet only, never installed). */
export const HUD_ART = ["stage", "map", "ysolde", "brannoch", "sylvenne", "caelum", "slash", "frost", "arc", "radiance", "gale", "meteor", "fury", "ward", "hex", "fire", "ice", "tonic", "ether", "gold", "shard"] as const;
