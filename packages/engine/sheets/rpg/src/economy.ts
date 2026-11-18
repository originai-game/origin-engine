// Game-owned examples: the components render these decisions, not rules of
// their own. The scenarios distinguish legal actions, failed requirements,
// loading, pending transactions, empty collections and long content.
const bagItem = (id: string, name: string, icon: string, rarity: string, count = 1, more: Partial<{ equipped: boolean; fresh: boolean; locked: boolean; broken: boolean }> = {}) =>
  ({ id, name, icon: `item-${icon}`, count, rarity, equipped: false, fresh: false, locked: false, broken: false, ...more });

/** The backpack's other states, swapped in by the scenario command on the inventory page. */
export const BAGS: Record<string, { items: ReturnType<typeof bagItem>[]; tabs: { id: string; label: string; count?: number }[]; tab: string; title: string; weight: string; load: number; gold: string; capacity: string; selected: string }> = {
  empty: { title: "Backpack", tab: "all", tabs: [{ id: "all", label: "All" }], items: [], weight: "0 / 120", load: 0, gold: "0", capacity: "0 / 48 slots", selected: "" },
  over: {
    title: "Backpack", tab: "all", tabs: [{ id: "all", label: "All", count: 3 }, { id: "craft", label: "Materials", count: 2 }],
    items: [bagItem("anvil", "Field Anvil", "mace", "common"), bagItem("ore", "Iron Ore", "moonstone", "common", 40), bagItem("axe", "Woodsman's Axe", "axe", "rare", 1, { broken: true })],
    weight: "131.0 / 120", load: 1.09, gold: "12", capacity: "3 / 48 slots", selected: "ore",
  },
  overflow: {
    title: "The Bottomless Satchel of the Wandering Quartermaster", tab: "craft",
    tabs: [{ id: "all", label: "All", count: 1240 }, { id: "weapons", label: "Weapons", count: 312 }, { id: "armour", label: "Armour", count: 208 }, { id: "trinkets", label: "Accessories", count: 96 }, { id: "use", label: "Consumables", count: 420 }, { id: "craft", label: "Crafting materials", count: 204 }],
    items: [bagItem("ore", "Moonstone", "moonstone", "common", 99999, { fresh: true, locked: true }), bagItem("blade", "Emberwake Blade", "blade", "mythic", 1, { equipped: true, fresh: true, locked: true, broken: true }), bagItem("herb", "Silverleaf", "herb", "uncommon", 1234)],
    weight: "1,234.5 / 2,000", load: 0.62, gold: "1,234,567,890", capacity: "1,240 / 1,500 slots", selected: "blade",
  },
};

export const ECONOMY = {
  craftRecipes: [
    { id: "draught", name: "Healing draught", icon: "potion", description: "Restores 240 health over 4 s. Produces 2 draughts.", canCraft: true, reason: "", rarity: "uncommon", held: "You hold 5" },
    { id: "elixir", name: "Storm elixir", icon: "bolt", description: "A charged catalyst for the next expedition.", canCraft: false, reason: "Requires Alchemy 4", rarity: "epic", held: "" },
    { id: "ember", name: "Ember flask", icon: "fire", description: "Bursts into flame where it lands.", canCraft: false, reason: "Missing ingredients", rarity: "legendary", held: "You hold 1" },
  ],
  craftIngredients: [
    { id: "herb", name: "Silverleaf", quantity: "6 / 2", sufficient: true },
    { id: "water", name: "Pure water", quantity: "4 / 1", sufficient: true },
  ],
  craftMissing: [{ id: "crystal", name: "Storm crystal", quantity: "0 / 1", sufficient: false }],
  tradeGoods: [
    { id: "potion", name: "Healing draught ×2", icon: "potion", stock: "8 in stock", description: "Two draughts for the road.", price: "20 gold", was: "", after: "120 → 100", available: true, reason: "", rarity: "uncommon" },
    { id: "bread", name: "Travel bread", icon: "apple", stock: "20 in stock", description: "Keeps for a week.", price: "4 gold", was: "", after: "120 → 116", available: true, reason: "", rarity: "common" },
    { id: "lamp", name: "Hooded lantern", icon: "fire", stock: "2 in stock", description: "Burns through the night.", price: "45 gold", was: "60 gold", after: "120 → 75", available: true, reason: "", rarity: "rare" },
    { id: "gem", name: "Storm crystal", icon: "gem", stock: "1 in stock", description: "A charged crafting catalyst.", price: "240 gold", was: "", after: "", available: false, reason: "Not enough gold", rarity: "epic" },
  ],
  tradeOwned: [{ id: "gem", name: "Storm crystal", icon: "gem", stock: "3 owned", description: "Sell one crystal.", price: "80 gold", was: "", after: "120 → 200", available: true, reason: "", rarity: "epic" }],
  lootItems: [
    { id: "ember", name: "Heart of the Ember", icon: "fire", count: 1, description: "Crafting reagent", available: true, reason: "", rarity: "legendary" },
    { id: "ring", name: "Moonlit Band", icon: "star", count: 1, description: "Ring · +4 Intellect", available: true, reason: "", rarity: "rare" },
    { id: "gold", name: "Gold", icon: "coin", count: 120, description: "Recovered from the chest.", available: true, reason: "", rarity: "common" },
    { id: "relic", name: "The Abbey Bell", icon: "gem", count: 1, description: "Unique relic", available: false, reason: "Inventory full", rarity: "epic" },
  ],
  lootOverflow: Array.from({ length: 12 }, (_, i) => ({ id: `relic-${i}`, name: `Silver reliquary from the farthest chamber of the ruined abbey ${i + 1}`, icon: "gem", count: i + 1, description: "A long description that should wrap cleanly instead of pushing the collection action beyond the panel.", available: true, reason: "" })),
  historyLines: [
    { id: "marissa", speaker: "Lady Marissa", text: "The bell remembers every promise. Bring it home, and the abbey may finally rest.", canReplay: true },
    { id: "player", speaker: "Brenna", text: "Then we will finish what we started.", canReplay: true },
    { id: "narration", speaker: "", text: "Beyond the gate, the marsh falls silent.", canReplay: false },
  ],
  historyOverflow: Array.from({ length: 16 }, (_, i) => ({ id: `line-${i}`, speaker: i % 2 ? "Brenna" : "Lady Marissa", text: "The road through the marsh has many turns. Keep the lantern close, and remember the promise we made at the gate.", canReplay: i % 2 == 0 })),
};

/** Item card states the scenario command swaps in: a refused downgrade and a plain consumable. */
export const CARDS: Record<string, Record<string, unknown>> = {
  refused: {
    cardName: "Staff of the Drowned Choir and the Last Hymn", cardIcon: "item-staff", cardArt: 0, cardRarity: "epic", cardKind: "Epic · Staff", cardLine: "Two-handed",
    cardLead: [{ id: "sp", icon: "star", label: "Spell power", value: "38", delta: "−6", trend: "down", current: "44" }],
    cardStats: [
      { id: "mana", icon: "bolt", label: "Mana regeneration", value: "4 / s", delta: "−1 / s", trend: "down", current: "5 / s" },
      { id: "int", icon: "star", label: "Intellect", value: "+18", delta: "+2", trend: "up", current: "+16" },
    ],
    cardLore: "", cardPrimary: "Equip", cardSecondary: "Drop", cardCanPrimary: false, cardReason: "Requires level 40",
  },
  plain: {
    cardName: "Bread", cardIcon: "item-bread", cardArt: 0, cardRarity: "common", cardKind: "Common · Food", cardLine: "Restores 40 health over 10 s",
    cardLead: [], cardStats: [], cardLore: "", cardPrimary: "Use", cardSecondary: "", cardCanPrimary: true, cardReason: "",
  },
};
