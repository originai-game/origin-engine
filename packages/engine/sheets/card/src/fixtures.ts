// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning). TABLE is a mid-combat turn: six cards in hand, two the
// game says cannot be played (with its reason), one picked up. BUILDER is a
// deck of 13 of 15 over a collection with owned, capped, refused and
// unowned cards, the two agreeing card by card. RUN is the run map with
// every room state and a reward of two cards and a relic.

export const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

/** The sheet's painted card art (art/card-art.png, 4 × 3, sample art only). */
export const ART_NAMES = [
  "fire", "shield", "axe", "moonstone", "potion", "ember",
  "feather", "bow", "torch", "staff", "lantern", "gem",
];

const card = (id: string, name: string, cost: string, icon: string, type: string, typeLabel: string, rarity: string, text: string) =>
  ({ id, name, cost, icon: `art-${icon}`, art: 0, type, typeLabel, rarity, text });

// The cards of the mockup (one deck, one collection, one reward): the hand,
// the collection and the deck list agree about every card.
const STRIKE = card("strike", "Ember Strike", "1", "fire", "attack", "Attack", "common", "Deal 6 damage. Apply 1 Burn.");
const GUARD = card("guard", "Iron Guard", "1", "shield", "skill", "Skill", "common", "Gain 8 Block.");
const CLEAVE = card("cleave", "Cleave", "1", "axe", "attack", "Attack · Uncommon", "uncommon", "Deal 8 damage to ALL enemies.");
const FOCUS = card("focus", "Moon Focus", "2", "moonstone", "power", "Power · Rare", "rare", "At the start of your turn, draw 1 extra card.");
const WIND = card("wind", "Second Wind", "0", "potion", "skill", "Skill · Uncommon", "uncommon", "Gain 1 energy. Exhaust.");
const FLAME = card("flame", "Phoenix Flame", "3", "ember", "attack", "Attack · Legendary", "legendary", "Deal 24 damage. If this kills, return it to your hand.");
const DOUBT = card("doubt", "Doubt", "", "feather", "curse", "Curse", "common", "Unplayable. At the end of your turn, gain 1 Weak.");
const QUICK = card("quick", "Quickdraw", "0", "bow", "attack", "Attack", "common", "Deal 3 damage. Draw 1 card.");
const TORCH = card("torch", "Torchlight", "1", "torch", "skill", "Skill · Uncommon", "uncommon", "Look at the top 3 cards of your draw pile.");
const STAR = card("star", "Starfall", "2", "staff", "power", "Power · Epic", "epic", "Whenever you play a Power, deal 5 damage to ALL enemies.");

// Three energy left: Moon Focus costs more than that; Doubt is unplayable.
const HAND = [STRIKE, GUARD, CLEAVE, FOCUS, WIND, DOUBT];
const REASON: Record<string, string> = { focus: "No energy", doubt: "Unplayable" };

export const TABLE = {
  // The game spreads the fan: each card's place from -1 to 1 in hand order.
  handCards: HAND.map((c, i) => ({ ...c, playable: !REASON[c.id], reason: REASON[c.id] ?? "", at: (i / (HAND.length - 1)) * 2 - 1 })),
  handSelected: "cleave",
  handFan: 8,
  drawPiles: [{ id: "draw", kind: "draw", label: "Draw", icon: "bag", count: 14, enabled: true }],
  spentPiles: [
    { id: "discard", kind: "discard", label: "Discard", icon: "back", count: 6, enabled: true },
    { id: "exhaust", kind: "exhaust", label: "Exhaust", icon: "fire", count: 0, enabled: false },
  ],
};

// reason is the game's word for a card it will not add; empty leaves the kit's own (Not owned, Cannot be added).
const owned = (c: ReturnType<typeof card>, owned: number, inDeck: number, canAdd: boolean, locked = false, reason = "") => ({ ...c, owned, inDeck, canAdd, locked, reason });

export const BUILDER = {
  colTitle: "Collection",
  colFilter: "all",
  colFilters: [
    { id: "all", label: "All", count: 42 }, { id: "attack", label: "Attack", count: 18 }, { id: "skill", label: "Skill", count: 15 },
    { id: "power", label: "Power", count: 6 }, { id: "curse", label: "Curse", count: 3 },
  ],
  colSelected: "strike",
  // Owned and in-deck counts match the deck list: 13 cards of 15.
  colCards: [
    owned(STRIKE, 4, 3, true),
    owned(GUARD, 4, 4, false, false, "All copies in the deck"),
    owned(CLEAVE, 2, 1, true),
    owned(FOCUS, 2, 2, false, false, "All copies in the deck"),
    owned(WIND, 2, 1, false, false, "Deck holds 1 at most"),
    owned(FLAME, 1, 1, false, false, "All copies in the deck"),
    owned(DOUBT, 1, 1, false, false, "All copies in the deck"),
    owned(QUICK, 2, 0, true),
    owned(TORCH, 1, 0, true),
    owned(STAR, 0, 0, false, true),
  ],
  deckTitle: "Ashen Vanguard",
  deckCount: 13,
  deckLimit: 15,
  deckSelected: "strike",
  deckConfirm: "Save deck",
  deckEntries: [
    { id: "wind", name: "Second Wind", cost: "0", type: "skill", rarity: "uncommon", copies: 1, canAdd: false, canRemove: true },
    { id: "strike", name: "Ember Strike", cost: "1", type: "attack", rarity: "common", copies: 3, canAdd: true, canRemove: true },
    { id: "guard", name: "Iron Guard", cost: "1", type: "skill", rarity: "common", copies: 4, canAdd: false, canRemove: true },
    { id: "cleave", name: "Cleave", cost: "1", type: "attack", rarity: "uncommon", copies: 1, canAdd: true, canRemove: true },
    { id: "focus", name: "Moon Focus", cost: "2", type: "power", rarity: "rare", copies: 2, canAdd: false, canRemove: true },
    { id: "flame", name: "Phoenix Flame", cost: "3", type: "attack", rarity: "legendary", copies: 1, canAdd: false, canRemove: true },
    { id: "doubt", name: "Doubt", cost: "", type: "curse", rarity: "common", copies: 1, canAdd: false, canRemove: false },
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
    { id: "here", x: 115, y: 60, label: "Event", icon: "flag", state: "current", shape: "circle", team: "" },
    { id: "alt", x: 115, y: 180, label: "Shop", icon: "coin", state: "locked", shape: "circle", team: "" },
    { id: "e1", x: 200, y: 35, label: "Elite", icon: "skull", state: "available", shape: "diamond", team: "" },
    { id: "e2", x: 200, y: 125, label: "Rest", icon: "fire", state: "available", shape: "circle", team: "" },
    { id: "t", x: 275, y: 80, label: "Treasure", icon: "gem", state: "locked", shape: "circle", team: "" },
    { id: "boss", x: 335, y: 150, label: "Boss", icon: "crown", state: "locked", shape: "diamond", team: "" },
  ],
  mapLegend: [
    { id: "fight", icon: "sword", label: "Fight", shape: "circle" }, { id: "elite", icon: "skull", label: "Elite", shape: "diamond" },
    { id: "rest", icon: "fire", label: "Rest", shape: "circle" }, { id: "shop", icon: "coin", label: "Shop", shape: "circle" },
    { id: "event", icon: "flag", label: "Event", shape: "circle" }, { id: "treasure", icon: "gem", label: "Treasure", shape: "circle" },
    { id: "boss", icon: "crown", label: "Boss", shape: "diamond" },
  ],
  rewardTitle: "Choose a reward",
  rewardKicker: "Victory · Floor 7",
  rewardSelected: "cleave",
  rewardSkip: "Skip",
  rewardChoices: [
    { ...CLEAVE, relic: false, enabled: true },
    { ...FOCUS, relic: false, enabled: true },
    { id: "lantern", relic: true, name: "Lantern of Dusk", cost: "", icon: "art-lantern", art: 0, type: "relic", typeLabel: "Relic · Rare", rarity: "rare", text: "At the start of each combat, gain 1 energy.", enabled: true },
  ],
};

// The sheet's fixture text goes through the string tables as a game's own
// data would: a top-level text store is the key `sheet.<store>`, a row's
// text field `sheet.<store>.<id>.<field>`. The English here is the source of
// assets/strings/en.ogt; `tr` answers each key in the active locale. Ids,
// icons, types, rarities, states and figures stay.
const ROW_TEXT = new Set(["label", "name", "typeLabel", "text", "reason"]);
const NOT_TEXT = /(Selected|Filter|View|Icon)$/;
const WORDS = /\p{L}{2}/u;

export function localized<T extends Record<string, unknown>>(data: T, tr: (key: string) => string): T {
  const out: Record<string, unknown> = {};
  for (const [store, v] of Object.entries(data)) {
    if (typeof v === "string") out[store] = WORDS.test(v) && !NOT_TEXT.test(store) ? tr(`sheet.${store}`) : v;
    else if (Array.isArray(v)) out[store] = v.map((row: Record<string, unknown>) => Object.fromEntries(Object.entries(row).map(([f, x]) =>
      [f, typeof x === "string" && WORDS.test(x) && ROW_TEXT.has(f) ? tr(`sheet.${store}.${row.id}.${f}`) : x])));
    else out[store] = v;
  }
  return out as T;
}
