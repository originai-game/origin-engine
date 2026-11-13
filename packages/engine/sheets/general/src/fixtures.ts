// The sheet's fixture data (docs/ui.md §3.1: fixture values are data, not
// visual tuning), at the density of a live game: a shop over three
// currencies with every verdict, an achievement list with hidden and
// claimable entries, a week of login rewards, a busy guild chat and a
// friends list with requests and every presence. Prices, counts, dates and
// times are already formatted, as the game hands them in.

export const INK_NAMES = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

const offer = (id: string, name: string, icon: string, detail: string, more: Partial<{ badge: string; featured: boolean; soldOut: boolean; owned: boolean; affordable: boolean }> = {}) =>
  ({ id, name, icon, detail, badge: "", featured: false, soldOut: false, owned: false, affordable: true, ...more });
const price = (offer: string, currency: "gold" | "gems" | "tokens", amount: string, more: Partial<{ was: string; short: boolean }> = {}) => ({
  id: `${offer}-${currency}`, offer, amount, was: "", short: false, ...more,
  icon: { gold: "coin", gems: "gem", tokens: "star" }[currency],
  label: { gold: "Gold", gems: "Gems", tokens: "Tokens" }[currency],
});

export const STORE = {
  shopTitle: "",
  shopWallet: [
    { id: "gold", icon: "coin", label: "Gold", amount: "12,450" },
    { id: "gems", icon: "gem", label: "Gems", amount: "320" },
    { id: "tokens", icon: "star", label: "Tokens", amount: "8" },
  ],
  shopTabs: [
    { id: "featured", label: "Featured" }, { id: "items", label: "Items" }, { id: "looks", label: "Looks" },
    { id: "bundles", label: "Bundles" }, { id: "gems", label: "Gems" },
  ],
  shopTab: "featured",
  shopOffers: [
    offer("bundle", "Founder Bundle", "crown", "3 items and 500 gems", { badge: "-30%", featured: true }),
    offer("potion", "Potion Pack", "potion", "x10"),
    offer("cape", "Ember Cape", "fire", "Look", { badge: "New", owned: true }),
    offer("key", "Vault Key", "key", "x1", { soldOut: true }),
    offer("mount", "Dragon Mount", "star", "Mount", { affordable: false }),
    offer("elixir", "Elixir of Swiftness", "drop", "x3, 10 minutes each"),
    offer("shield", "Oak Shield", "shield", "Gear", { badge: "2 left" }),
    offer("bombs", "Powder Bombs", "bomb", "x5"),
  ],
  shopPrices: [
    price("bundle", "gems", "700", { was: "1,000" }),
    price("potion", "gold", "1,200"), price("potion", "gems", "40"),
    price("cape", "gems", "250"),
    price("key", "gold", "5,000"),
    price("mount", "gems", "2,400", { short: true }), price("mount", "tokens", "12", { short: true }),
    price("elixir", "gold", "900"),
    price("shield", "gold", "3,200"), price("shield", "tokens", "2"),
    price("bombs", "gold", "650"),
  ],
  shopSelected: "potion",

  confirmName: "Potion Pack",
  confirmIcon: "potion",
  confirmDetail: "Ten healing potions, each restoring 50 health.",
  confirmPrices: [
    { id: "gold", icon: "coin", label: "Gold", amount: "1,200", was: "", after: "12,450 → 11,250", short: false },
  ],
};

const achievement = (id: string, name: string, about: string, icon: string, state: string, more: Partial<{ progress: number; progressText: string; date: string; hidden: boolean; rarity: string; tier: string; claimable: boolean; claimed: boolean }> = {}) =>
  ({ id, name, about, icon, state, progress: state === "unlocked" ? 1 : 0, progressText: "", date: "", hidden: false, rarity: "", tier: "common", claimable: false, claimed: false, ...more });

export const PROGRESS = {
  achSummary: "38 / 120",
  achDone: 0.32,
  achList: [
    achievement("first", "First Steps", "Finish the tutorial.", "flag", "unlocked", { date: "12 Nov 2026", rarity: "92.1%", claimable: true }),
    achievement("hunter", "Monster Hunter", "Defeat 100 monsters.", "sword", "progress", { progress: 0.64, progressText: "64 / 100", rarity: "18.4%", tier: "rare" }),
    achievement("rich", "Tycoon", "Hold a million gold at once.", "coin", "locked", { rarity: "1.2%", tier: "legendary" }),
    achievement("secret", "The Truth Beneath", "Find what lies under the abbey.", "skull", "locked", { hidden: true, rarity: "0.4%", tier: "epic" }),
    achievement("pet", "Best Friends", "Pet the dog.", "heart", "unlocked", { date: "3 Nov 2026", rarity: "71.0%", claimed: true }),
    achievement("bard", "Wandering Bard", "Play every song on the flute.", "music", "progress", { progress: 0.25, progressText: "3 / 12", rarity: "6.3%", tier: "rare" }),
    achievement("chef", "Hearth Chef", "Cook 50 meals.", "apple", "locked", { rarity: "12%" }),
  ],

  questsRefresh: "Resets in 4h 12m",
  quests: [
    { id: "win", name: "Win 3 matches", icon: "trophy", progress: 0.67, progressText: "2 / 3", reward: "50", rewardIcon: "gem", state: "active" },
    { id: "gather", name: "Gather 20 herbs", icon: "leaf", progress: 1, progressText: "20 / 20", reward: "500", rewardIcon: "coin", state: "done" },
    { id: "slay", name: "Defeat 5 elite foes", icon: "sword", progress: 0.2, progressText: "1 / 5", reward: "1,000", rewardIcon: "coin", state: "active" },
    { id: "login", name: "Log in", icon: "check", progress: 1, progressText: "1 / 1", reward: "100", rewardIcon: "coin", state: "claimed" },
  ],

  rewardsRefresh: "Next reward in 9h 40m",
  rewardsDays: [
    { id: "d1", label: "Day 1", icon: "coin", amount: "x100", state: "claimed", milestone: false },
    { id: "d2", label: "Day 2", icon: "potion", amount: "x3", state: "claimed", milestone: false },
    { id: "d3", label: "Day 3", icon: "gem", amount: "x20", state: "missed", milestone: false },
    { id: "d4", label: "Day 4", icon: "coin", amount: "x300", state: "today", milestone: false },
    { id: "d5", label: "Day 5", icon: "key", amount: "x1", state: "upcoming", milestone: false },
    { id: "d6", label: "Day 6", icon: "gem", amount: "x50", state: "upcoming", milestone: false },
    { id: "d7", label: "Day 7", icon: "crown", amount: "Legend chest", state: "upcoming", milestone: true },
  ],
};

const msg = (id: string, kind: string, sender: string, time: string, text: string) => ({ id, kind, sender, time, text });

export const SOCIAL = {
  chatChannels: [
    { id: "world", label: "World", unread: 0 }, { id: "guild", label: "Order of the Silver Dawn", unread: 0 },
    { id: "party", label: "Party", unread: 3 }, { id: "trade", label: "Trade", unread: 128 },
  ],
  chatChannel: "guild",
  chatMessages: [
    msg("m1", "system", "", "19:58", "Message of the day: the raid starts at nine. Bring food for three hours."),
    msg("m2", "player", "Aerin", "20:01", "Anyone have a spare frost ward? Mine broke on the last pull."),
    msg("m3", "player", "Maximilian the Unbreakable", "20:02", "Supercalifragilisticexpialidocious-and-a-word-that-will-not-break-anywhere-at-all"),
    msg("m4", "own", "Brenna", "20:02", "I can craft one. Meet me at the forge before we go, and bring two moonstone shards."),
    msg("m5", "player", "Lys", "20:03", "ok"),
    msg("m6", "system", "", "20:05", "Sela joined the guild."),
    msg("m7", "player", "Sela", "20:05", "Hello everyone! Glad to be here."),
    msg("m8", "own", "Brenna", "20:06", "Welcome, Sela."),
  ],
  chatDraft: "On my way to the forge",
  chatNotice: "",

  friendsRequests: [
    { id: "idris", name: "Idris", avatar: "person", note: "2 mutual friends" },
    { id: "bart", name: "Bartholomew Fitzgerald-Montgomery", avatar: "person", note: "" },
  ],
  friendsList: [
    { id: "aerin", name: "Aerin", avatar: "person", status: "ingame", activity: "Ranked match, round 3", canInvite: false, invited: false },
    { id: "lys", name: "Lys", avatar: "person", status: "online", activity: "In the lobby", canInvite: true, invited: false },
    { id: "maren", name: "Maren", avatar: "person", status: "away", activity: "", canInvite: true, invited: true },
    { id: "max", name: "Maximilian the Unbreakable, Lord of the Northern Wastes", avatar: "person", status: "online", activity: "", canInvite: true, invited: false },
    { id: "sela", name: "Sela", avatar: "person", status: "offline", activity: "Last seen 2 d ago", canInvite: false, invited: false },
    { id: "oarsman", name: "The Oarsman", avatar: "person", status: "offline", activity: "Last seen 3 weeks ago", canInvite: false, invited: false },
  ],
  friendsSelected: "lys",
};

/** The states page: each member empty, loading, failed or refused. */
export const STATES = {
  none: [] as { id: string }[],
  // Literal props a page passes through stores: the cook checks a literal on a
  // component tag against the core attribute of the same name (selected, loading).
  blank: "",
  no: false,
  yes: true,
  world: "world",
  shopError: "The shop could not be reached. Check your connection.",
  confirmError: "The purchase did not go through. Nothing was charged.",
  chatMutedChannels: [{ id: "world", label: "World", unread: 0 }],
  chatMuted: [msg("x1", "system", "", "20:20", "You are muted for 5 minutes.")],
  chatMutedNotice: "You are muted for 5 minutes.",
  rewardsClaimedDays: [
    { id: "c1", label: "Day 1", icon: "coin", amount: "x100", state: "claimed", milestone: false },
    { id: "c2", label: "Day 2", icon: "potion", amount: "x3", state: "claimed", milestone: false },
    { id: "c3", label: "Day 3", icon: "gem", amount: "x20", state: "claimed", milestone: true },
    { id: "c4", label: "Day 4", icon: "coin", amount: "x300", state: "upcoming", milestone: false },
  ],
  rewardsClaimedRefresh: "Next reward in 23h 10m",
  questsEmptyRefresh: "Resets in 23h 59m",
};
