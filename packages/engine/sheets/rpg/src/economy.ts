// Game-owned examples: the components render these decisions, not rules of
// their own. The scenarios distinguish legal actions, failed requirements,
// loading, pending transactions, empty collections and long content.
export const ECONOMY = {
  craftRecipes: [
    { id: "draught", name: "Healing draught", icon: "potion", description: "Restores health. Produces 2 draughts.", canCraft: true, reason: "" },
    { id: "elixir", name: "Storm elixir", icon: "bolt", description: "A charged catalyst for the next expedition.", canCraft: false, reason: "Requires Alchemy 4" },
  ],
  craftIngredients: [
    { id: "herb", name: "Silverleaf", quantity: "6 / 2", sufficient: true },
    { id: "water", name: "Pure water", quantity: "4 / 1", sufficient: true },
  ],
  craftMissing: [{ id: "crystal", name: "Storm crystal", quantity: "0 / 1", sufficient: false }],
  tradeGoods: [
    { id: "potion", name: "Healing draught ×2", icon: "potion", stock: "8 in stock", description: "Two draughts for the road.", price: "20 gold", available: true, reason: "" },
    { id: "gem", name: "Storm crystal", icon: "gem", stock: "1 in stock", description: "A charged crafting catalyst.", price: "240 gold", available: false, reason: "Not enough gold" },
  ],
  tradeOwned: [{ id: "gem", name: "Storm crystal", icon: "gem", stock: "3 owned", description: "Sell one crystal.", price: "80 gold", available: true, reason: "" }],
  lootItems: [
    { id: "gold", name: "Gold", icon: "coin", count: 120, description: "Recovered from the chest.", available: true, reason: "" },
    { id: "relic", name: "The Abbey Bell", icon: "gem", count: 1, description: "Unique relic", available: false, reason: "Inventory full" },
  ],
  lootOverflow: Array.from({ length: 12 }, (_, i) => ({ id: `relic-${i}`, name: `Silver reliquary from the farthest chamber of the ruined abbey ${i + 1}`, icon: "gem", count: i + 1, description: "A long description that should wrap cleanly instead of pushing the collection action beyond the panel.", available: true, reason: "" })),
  historyLines: [
    { id: "marissa", speaker: "Lady Marissa", text: "The bell remembers every promise. Bring it home, and the abbey may finally rest.", canReplay: true },
    { id: "player", speaker: "Brenna", text: "Then we will finish what we started.", canReplay: true },
    { id: "narration", speaker: "", text: "Beyond the gate, the marsh falls silent.", canReplay: false },
  ],
  historyOverflow: Array.from({ length: 16 }, (_, i) => ({ id: `line-${i}`, speaker: i % 2 ? "Brenna" : "Lady Marissa", text: "The road through the marsh has many turns. Keep the lantern close, and remember the promise we made at the gate.", canReplay: i % 2 == 0 })),
};
