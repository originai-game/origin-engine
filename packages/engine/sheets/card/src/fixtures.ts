// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning). TABLE is a mid-combat turn: seven cards in hand, two the
// game says cannot be played, one picked up. BUILDER is a deck in progress
// over a collection with owned, capped and locked cards, the deck one card
// over its limit. RUN is the run map with every room state and a reward of
// three cards, one refused.

export const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

const card = (id: string, name: string, cost: string, icon: string, type: string, typeLabel: string, rarity: string, text: string) =>
  ({ id, name, cost, icon, art: 0, type, typeLabel, rarity, text });

const HAND = [
  card("strike", "Ember Strike", "1", "fire", "attack", "Attack", "common", "Deal 6 damage. Apply 1 Burn."),
  card("guard", "Iron Guard", "1", "shield", "skill", "Skill", "common", "Gain 5 Block."),
  card("cleave", "Cleave", "1", "sword", "attack", "Attack", "uncommon", "Deal 8 damage to all enemies."),
  card("lament", "Lament of the Drowned Cathedral", "4", "skull", "power", "Power", "legendary", "Each turn, every enemy loses 2 Strength."),
  card("mend", "Mend", "2", "heart", "skill", "Skill", "uncommon", "Heal 6. Exhaust."),
  card("surge", "Arcane Surge", "3", "bolt", "power", "Power", "rare", "Draw 2 more cards each turn."),
  card("wound", "Wound", "", "drop", "status", "Status", "common", "Unplayable."),
];
// Arcane Surge costs more energy than the player has; Wound is unplayable.
const PLAYABLE = new Set(["strike", "guard", "cleave", "lament", "mend"]);

export const TABLE = {
  // The game spreads the fan: each card's place from -1 to 1 in hand order.
  handCards: HAND.map((c, i) => ({ ...c, playable: PLAYABLE.has(c.id), at: HAND.length > 1 ? (i / (HAND.length - 1)) * 2 - 1 : 0 })),
  handSelected: "cleave",
  handFan: 8,
  drawPiles: [{ id: "draw", kind: "draw", label: "Draw", icon: "bag", count: 17, enabled: true }],
  spentPiles: [
    { id: "discard", kind: "discard", label: "Discard", icon: "back", count: 4, enabled: true },
    { id: "exhaust", kind: "exhaust", label: "Exhaust", icon: "fire", count: 0, enabled: false },
  ],
};

// reason is the game's word for a card it will not add; empty leaves the kit's own (Not owned, Cannot be added).
const owned = (c: ReturnType<typeof card>, owned: number, inDeck: number, canAdd: boolean, locked = false, reason = "") => ({ ...c, owned, inDeck, canAdd, locked, reason });

export const BUILDER = {
  colTitle: "Collection",
  colFilter: "all",
  colFilters: [
    { id: "all", label: "All" }, { id: "attack", label: "Attacks" }, { id: "skill", label: "Skills" },
    { id: "power", label: "Powers" }, { id: "cost0", label: "Cost 0–1" }, { id: "cost2", label: "Cost 2+" },
  ],
  colSelected: "cleave",
  colCards: [
    owned(HAND[0]!, 4, 3, false, false, "Three copies at most"),
    owned(HAND[1]!, 3, 2, true),
    owned(HAND[2]!, 2, 1, true),
    owned(card("quick", "Quickstep", "0", "leaf", "skill", "Skill", "uncommon", "Draw 1 card."), 2, 2, false),
    owned(HAND[4]!, 1, 0, true),
    owned(HAND[5]!, 1, 1, false),
    owned(card("bash", "Bash", "2", "sword", "attack", "Attack", "common", "Deal 8 damage. Apply 2 Vulnerable."), 3, 0, true),
    owned(card("ward", "Ward", "1", "shield", "skill", "Skill", "common", "Gain 8 Block."), 2, 0, true),
    owned(HAND[3]!, 0, 0, false, true),
    owned(card("crown", "Hollow Crown", "2", "crown", "power", "Power", "mythic", "Your attacks deal double damage."), 0, 0, false, true),
    owned(card("ember", "Ember Rain", "2", "fire", "attack", "Attack", "rare", "Deal 3 damage to a random enemy 4 times."), 1, 0, true),
    owned(card("flask", "Bottled Light", "1", "potion", "skill", "Skill", "epic", "Retain. Gain 2 energy next turn."), 1, 0, true),
  ],
  deckTitle: "Ember Deck",
  deckCount: 31,
  deckLimit: 30,
  deckSelected: "cleave",
  deckConfirm: "Save deck",
  deckEntries: [
    { id: "quick", name: "Quickstep", cost: "0", type: "skill", rarity: "uncommon", copies: 2, canAdd: false, canRemove: true },
    { id: "strike", name: "Ember Strike", cost: "1", type: "attack", rarity: "common", copies: 3, canAdd: false, canRemove: true },
    { id: "guard", name: "Iron Guard", cost: "1", type: "skill", rarity: "common", copies: 2, canAdd: false, canRemove: true },
    { id: "cleave", name: "Cleave", cost: "1", type: "attack", rarity: "uncommon", copies: 1, canAdd: false, canRemove: true },
    { id: "surge", name: "Arcane Surge", cost: "3", type: "power", rarity: "rare", copies: 1, canAdd: false, canRemove: true },
    { id: "rest", name: "Twenty-two cards of the starter set, kept together", cost: "1", type: "skill", rarity: "common", copies: 22, canAdd: false, canRemove: false },
  ],
};

export const RUN = {
  mapTitle: "The Sunken Road",
  mapFloor: "Floor 3 of 8",
  mapView: "0 0 360 240",
  mapSelected: "e2",
  mapLinks: [
    { id: "l1", d: "M30 120 L 115 60", active: true },
    { id: "l2", d: "M30 120 L 115 180", active: false },
    { id: "l3", d: "M115 60 L 200 35", active: false },
    { id: "l4", d: "M115 60 L 200 125", active: false },
    { id: "l5", d: "M115 180 L 200 125", active: false },
    { id: "l6", d: "M200 35 L 275 80", active: false },
    { id: "l7", d: "M200 125 L 275 80", active: false },
    { id: "l8", d: "M275 80 L 335 150", active: false },
  ],
  mapNodes: [
    { id: "start", x: 30, y: 120, label: "Fight", icon: "sword", state: "done", shape: "circle", team: "" },
    { id: "here", x: 115, y: 60, label: "Event", icon: "flag", state: "current", shape: "hex", team: "" },
    { id: "alt", x: 115, y: 180, label: "Shop", icon: "coin", state: "locked", shape: "square", team: "" },
    { id: "e1", x: 200, y: 35, label: "Elite", icon: "skull", state: "available", shape: "diamond", team: "" },
    { id: "e2", x: 200, y: 125, label: "Rest", icon: "fire", state: "available", shape: "square", team: "" },
    { id: "t", x: 275, y: 80, label: "Treasure", icon: "gem", state: "locked", shape: "hex", team: "" },
    { id: "boss", x: 335, y: 150, label: "Boss", icon: "crown", state: "locked", shape: "diamond", team: "" },
  ],
  mapLegend: [
    { id: "fight", icon: "sword", label: "Fight" }, { id: "elite", icon: "skull", label: "Elite" },
    { id: "rest", icon: "fire", label: "Rest" }, { id: "shop", icon: "coin", label: "Shop" },
    { id: "event", icon: "flag", label: "Event" }, { id: "boss", icon: "crown", label: "Boss" },
  ],
  rewardTitle: "Choose a card",
  rewardSelected: "ember",
  rewardSkip: "Skip",
  rewardChoices: [
    { ...card("ember", "Ember Rain", "2", "fire", "attack", "Attack", "rare", "Deal 3 damage to a random enemy 4 times."), relic: false, enabled: true },
    { ...card("ward", "Ward", "1", "shield", "skill", "Skill", "common", "Gain 8 Block."), relic: false, enabled: true },
    { ...card("crown", "Hollow Crown", "2", "crown", "power", "Power", "mythic", "Your attacks deal double damage."), relic: false, enabled: false },
    { id: "lantern", relic: true, name: "Lantern of Dusk", cost: "", icon: "star", art: 0, type: "relic", typeLabel: "", rarity: "rare", text: "At the start of each combat, gain 1 energy.", enabled: true },
  ],
};
