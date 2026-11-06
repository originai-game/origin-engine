// What a game puts in a screen document's slots, for the catalogue's screen
// thumbnails: a screen pushed with its viewmodel alone draws its frame and
// none of its lists, maps or trees. Each entry fills the slots after
// og.ui.screen, with the kits' own fixtures where a kit has them
// (packages/engine/extensions/*/fixtures) and the ui-kit template's sample
// data otherwise. Screens not listed draw from their document alone.
import { og, type Entity, type World } from "@origingame/engine";
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";
import { shooter } from "@origingame/engine/extensions/shooter/index.ts";
import { gacha } from "@origingame/engine/extensions/gacha/index.ts";
import { racing } from "@origingame/engine/extensions/racing/index.ts";
import { QUESTS, bag, conversation, gear, party } from "@origingame/engine/extensions/rpg/fixtures/party.ts";
import { loadout, scoreboard } from "@origingame/engine/extensions/shooter/fixtures/match.ts";
import { banner, pull } from "@origingame/engine/extensions/gacha/fixtures/banner.ts";
import { puzzle } from "@origingame/engine/extensions/puzzle/index.ts";
import { goals, saga } from "@origingame/engine/extensions/puzzle/fixtures/levels.ts";
import { garage } from "@origingame/engine/extensions/racing/fixtures/race.ts";

const grow = { kind: "grow" } as const;
const nobody = 0 as Entity;

/** Fill `<screen>/<slot>` with `build` when the document built that slot this tick. */
function fill(screen: string, slot: string, build: () => void): void {
  if (!og.ui.openSlot(`${screen}/${slot}`)) return;
  build();
  og.ui.close();
}

const speaker = (id: string, name: string) => ({ id, name, entity: nobody, portrait: "", expression: "", side: "left" });
const line = (seq: number, who: ReturnType<typeof speaker>, text: string, chosen: string | null = null) =>
  ({ seq, node: `n${seq}`, speaker: who, key: "", args: null, text, voice: "", seen: true, chosen: chosen ? { index: 0, text: chosen } : null });
const wren = speaker("wren", "Old Wren"), you = speaker("you", "You");

const ROSTER = ["Aria", "Kael", "Mira", "Orrin", "Vex", "Sable", "Juno", "Bram", "Lio", "Tess"].map((name, i) => ({
  key: name.toLowerCase(), name, role: ["Vanguard", "Striker", "Support"][i % 3], description: `${name}'s kit: a dash, a guard and a signature strike.`, taken: i === 3, locked: i === 9, lockedHint: "Reach rank 10",
}));

const SONGS = [["Neon Tide", "Kiryu", 174], ["Paper Moon", "Ash & Ivy", 128], ["Overclock", "DJ Halcyon", 190], ["Lantern Walk", "Mio", 96], ["Static Bloom", "Verse", 150]].map(([title, artist, bpm], i) => ({
  key: `s${i}`, title: title as string, artist: artist as string, bpm: bpm as number, seconds: 132 + i * 17, best: 912_340 - i * 41_000, grade: ["S", "A", "A", "B", "—"][i], accuracy: 0.98 - i * 0.03, isNew: i === 4,
  difficulties: [{ key: "easy", name: "Easy", level: 3 + i }, { key: "hard", name: "Hard", level: 8 + i }, { key: "expert", name: "Expert", level: 11 + i, locked: i > 2 }],
}));


const RARITY = ["common", "uncommon", "rare", "epic", "legendary"] as const;
const SWORD = [{ label: "Damage", current: 42, next: 51, max: 80, icon: "sword" }, { label: "Speed", current: 1.2, next: 1.1, max: 2, digits: 1, icon: "clock" }, { label: "Defence", current: 18, next: 18, max: 60, icon: "shield" }];
const kv = (id: string, rows: Array<[string, string]>, title = "") => og.ui.keyValueRows(id, { title, rows: rows.map(([label, value]) => ({ label, value })), width: grow } as never);
const stepper = (value: number, label: string) => og.ui.numberStepper("Amount", value, { min: 0, max: 9999, step: 10, label } as never);
const LISTINGS = [["Starsteel ingot", 34, 1200, "2h"], ["Moonblade", 40, 18500, "11h"], ["Ember potion ×5", 12, 320, "20m"], ["Warden's helm", 38, 7400, "1d"], ["Saltmarsh map", 20, 950, "4h"], ["Gilded lute", 25, 2600, "6h"]]
  .map(([name, level, price, left]) => ({ name, level, price, left }));

export const SCREEN_FILLS: Record<string, (world: World) => void> = {
  dialogue: () => {
    fill("dialogue", "Conversation", () => rpg.conversation("Talk", { view: conversation(), charsPerSecond: 0 } as never));
  },
  backlog: () => fill("backlog", "Log", () => og.ui.backlog("Log", {
    title: "Backlog", closeLabel: "Close", replayLabel: "Replay", emptyLabel: "Nothing read yet",
    lines: [
      line(1, wren, "You came by the coast road? Then you saw the lamp."),
      line(2, you, "It was dark. Nobody had lit it in days."),
      line(3, wren, "The lamp went dark the night the tide turned red."),
      line(4, you, "", "Who was singing?"),
      line(5, wren, "I heard singing under the rocks, and then nothing at all."),
      line(6, you, "Then I'll go down to the rocks tonight."),
    ],
  } as never)),
  scoreboard: () => fill("scoreboard", "Board", () => shooter.scoreboard("Board", scoreboard())),
  settings: (world) => fill("settings", "Tree", () => og.ui.settingsTree(world, "Tree", {
    title: "Settings", page: "audio",
    pages: [
      { key: "video", label: "Video", entries: [{ key: "mode", kind: "choice", label: "Display mode", options: ["Windowed", "Borderless", "Fullscreen"], index: 1, display: true }] },
      { key: "audio", label: "Audio", entries: [
        { key: "master", kind: "slider", label: "Master volume", section: "Volume", value: 0.8, max: 1, unit: "%" },
        { key: "music", kind: "slider", label: "Music", section: "Volume", value: 0.6, max: 1, unit: "%" },
        { key: "effects", kind: "slider", label: "Effects", section: "Volume", value: 0.75, max: 1, unit: "%" },
        { key: "voice", kind: "choice", label: "Voice language", section: "Language", options: ["English", "日本語", "Français"], index: 0 },
        { key: "captions", kind: "toggle", label: "Captions", section: "Language", on: true, description: "Show who is speaking and what they say." },
        { key: "mono", kind: "toggle", label: "Mono audio", section: "Accessibility", on: false },
      ] },
      { key: "controls", label: "Controls", entries: [{ key: "jump", kind: "binding", label: "Jump", action: "jump" }] },
      { key: "access", label: "Accessibility", entries: [{ key: "text", kind: "slider", label: "Text size", value: 1, min: 0.8, max: 1.6, step: 0.1 }] },
    ],
  } as never, () => {})),
  "hud-editor": () => {
    for (const [anchor, label] of [["topLeft", "Health"], ["topRight", "Minimap"], ["bottomLeft", "Abilities"], ["bottomRight", "Ammo"]] as const) {
      og.ui.anchor(`Hud${label}`, { anchor, margin: 24 }, () => og.ui.progressBar("Bar", { value: 0.7, label, showValue: true } as never));
    }
    fill("hud-editor", "Editor", () => og.ui.hudLayoutEditor("Editor", { title: "Edit HUD", sizeLabel: "Size", opacityLabel: "Opacity", nextLabel: "Next", resetLabel: "Reset", doneLabel: "Done" }));
  },
  "character-select": () => fill("character-select", "Select", () => og.ui.characterSelect("Chars", {
    roster: ROSTER as never, secondsLeft: 24, lockLabel: "Lock in", lockedInLabel: "Locked in", takenLabel: "Taken", randomLabel: "Random",
    cursors: [{ key: "p1", name: "You", entry: "aria", you: true, team: 0 }, { key: "p2", name: "Riven", entry: "kael", team: 1, confirmed: true }],
  } as never)),
  "quest-log": () => fill("quest-log", "Log", () => rpg.questLog("Log", { quests: QUESTS, selected: "q1", title: "Journal" } as never)),
  flowchart: () => fill("flowchart", "Chart", () => og.ui.routeChart("Chart", {
    chapters: [{ key: "c1", label: "Chapter 1" }, { key: "c2", label: "Chapter 2" }, { key: "c3", label: "Chapter 3" }],
    scenes: [
      { key: "a", title: "The letter", column: 0, lane: 1, seen: true },
      { key: "b", title: "The harbour", column: 1, lane: 0, seen: true },
      { key: "c", title: "The lighthouse", column: 1, lane: 2, seen: true, current: true },
      { key: "d", title: "Tide's end", column: 2, lane: 0, ending: "Ending A", good: true, seen: true },
      { key: "e", title: "The singing rocks", column: 2, lane: 2 },
      { key: "f", title: "???", column: 2, lane: 1, locked: true },
    ],
    links: [{ from: "a", to: "b", taken: true, label: "Go to the docks" }, { from: "a", to: "c", taken: true, label: "Climb the hill" }, { from: "b", to: "d", taken: true }, { from: "c", to: "e" }, { from: "c", to: "f" }],
    selected: "c", lockedLabel: "Locked", jumpLabel: "Jump here", currentLabel: "You are here", seenLabel: "Seen",
  } as never)),
  management: () => fill("management", "Ledger", () => og.ui.ledger("Ledger", {
    tabs: [{ key: "month", label: "Month" }, { key: "quarter", label: "Quarter" }, { key: "year", label: "Year" }], tab: "month",
    figures: [{ key: "treasury", label: "Treasury", value: 1_284_600, delta: 18_900 }, { key: "income", label: "Income", value: 117_000, delta: 4_200 }, { key: "expenses", label: "Expenses", value: 98_100, delta: -2_300, lowerIsBetter: true }],
    sections: [{ key: "in", name: "Income" }, { key: "out", name: "Expenses", expense: true }],
    lines: [
      { key: "tax", section: "in", name: "Taxes", amount: 92_000, previous: 88_500, funding: 0.6 },
      { key: "trade", section: "in", name: "Trade", amount: 25_000, previous: 24_300 },
      { key: "police", section: "out", name: "Police", amount: 14_200, previous: 13_900, funding: 1 },
      { key: "fire", section: "out", name: "Fire service", amount: 8_850, previous: 9_100, funding: 0.75, warning: "Understaffed" },
      { key: "health", section: "out", name: "Health care", amount: 17_220, previous: 16_400, funding: 1 },
    ],
    periods: ["Jan", "Feb", "Mar", "Apr", "May", "Jun"],
    history: [{ label: "Income", values: [98, 102, 104, 110, 113, 117] }, { label: "Expenses", values: [90, 92, 95, 96, 100, 98] }],
    graph: "line", currency: "$",
  } as never)),
  "world-map": () => {
    fill("world-map", "Map", () => og.ui.worldMap("Map", {
      aspect: 16 / 9, playerX: 0.42, playerY: 0.55, heading: 0.6, selected: "mill", title: "Greywater Marsh",
      regions: [{ key: "marsh", name: "Greywater Marsh", x: 0.4, y: 0.55, radius: 0.22, discovered: true }, { key: "peaks", name: "Ashen Peaks", x: 0.75, y: 0.25, radius: 0.18, discovered: true }, { key: "coast", name: "Salt Coast", x: 0.15, y: 0.3, radius: 0.15 }],
      markers: [
        { key: "mill", kind: "town", name: "Old mill", description: "A drowned mill where the river splits.", x: 0.38, y: 0.6, discovered: true, tracked: true },
        { key: "gate", kind: "quest", name: "Cliff gate", description: "Sealed since the flood.", x: 0.7, y: 0.32, discovered: true },
        { key: "camp", kind: "camp", name: "Hunter's camp", x: 0.55, y: 0.7, discovered: true },
        { key: "cave", kind: "dungeon", name: "Singing rocks", x: 0.2, y: 0.35 },
      ],
      kindLabels: { town: "Town", quest: "Quest", camp: "Camp", dungeon: "Dungeon" }, travelLabel: "Fast travel", trackLabel: "Track", untrackLabel: "Untrack", undiscoveredLabel: "Undiscovered", closeLabel: "Close",
    } as never));
    fill("world-map", "Tracker", () => rpg.questTracker("Tracker", { quests: QUESTS.filter((q) => q.status === "active"), max: 2 } as never));
  },
  versus: () => fill("versus", "Versus", () => og.ui.versus("Vs", {
    left: { name: "Lumen", character: "Aria", detail: "128 W · 64 L", team: 0 }, right: { name: "Riven", character: "Kael", detail: "211 W · 97 L", team: 1 },
    versusLabel: "VS", stage: "Golden Valley", progress: 0.62, skipLabel: "Skip",
  })),
  gallery: () => fill("gallery", "Gallery", () => og.ui.cgGallery("Cg", {
    chapters: [{ key: "c1", label: "Chapter 1" }, { key: "c2", label: "Chapter 2" }, { key: "c3", label: "Chapter 3" }], chapter: "c1",
    entries: ["First meeting", "The festival", "Rain at the pier", "The lantern", "Two letters", "Night train"].map((title, i) => ({ key: `cg${i}`, title, chapter: "c1", locked: i > 3, hint: i > 3 ? "Chapter 4" : "", fresh: i === 2 })),
    countLabel: "Seen", lockedLabel: "Locked", newLabel: "New", closeLabel: "Close",
  } as never)),
  "song-select": () => fill("song-select", "Select", () => og.ui.songSelect("Wheel", {
    songs: SONGS as never, selected: "s1", difficulty: "hard", preview: { playing: true, position: 0.35, peaks: Array.from({ length: 48 }, (_, i) => 0.3 + 0.6 * Math.abs(Math.sin(i * 0.7))) },
  } as never)),
  "gacha-banner": () => fill("gacha-banner", "Banner", () => gacha.gachaBanner("Banner", banner(), () => {})),
  lobby: () => fill("lobby", "Lobby", () => og.ui.lobby("Party", {
    slots: 5, estimateSeconds: 90, canStart: true,
    members: [
      { key: "you", name: "Lumen", level: 42, title: "Captain", ready: true, leader: true, you: true },
      { key: "m2", name: "Riven", level: 37, ready: true },
      { key: "m3", name: "Sable", level: 29, ready: false, status: "In menus" },
    ],
    mode: { name: "Ranked", detail: "5v5 · Best of 3" },
    readyLabel: "Ready", notReadyLabel: "Not ready", readyUpLabel: "Ready up", startLabel: "Find match", cancelLabel: "Cancel", inviteLabel: "Invite", levelLabel: "Lv", leaderLabel: "Leader", modeLabel: "Mode",
  } as never)),
  garage: () => fill("garage", "Garage", () => racing.garage("Garage", { ...garage(), title: "" } as never)),
  "photo-mode": () => fill("photo-mode", "Controls", () => og.ui.photoMode("Mode", {
    title: "Photo mode", tab: "camera",
    tabs: [{ key: "camera", label: "Camera" }, { key: "light", label: "Lighting" }, { key: "filter", label: "Filters" }, { key: "frame", label: "Frames" }],
    controls: [
      { key: "fov", tab: "camera", label: "Field of view", value: 55, min: 20, max: 110, step: 1, unit: "°" },
      { key: "roll", tab: "camera", label: "Roll", value: -4, min: -45, max: 45, step: 1, unit: "°", origin: 0 },
      { key: "focus", tab: "camera", label: "Focus distance", value: 6.5, min: 0.5, max: 50, step: 0.5, unit: "m", digits: 1 },
      { key: "aperture", tab: "camera", label: "Aperture", value: 2.8, min: 1.4, max: 22, step: 0.1, digits: 1 },
    ],
    grid: true, hiddenHint: "Press any button to show the controls",
  } as never)),
  "battle-pass": () => {
    fill("battle-pass", "Pass", () => og.ui.battlePass("Pass", {
      title: "Season 4 · Tides", endsIn: 18 * 86400 + 6 * 3600, level: 7, xp: 640, xpNext: 1000, premium: false, unlockPriceText: "$9.99",
      tiers: Array.from({ length: 12 }, (_, i) => ({ level: i + 1, milestone: (i + 1) % 10 === 0, freeClaimed: i < 6,
        free: i % 3 === 2 ? null : { key: `f${i}`, name: ["Coins", "Gems", "XP boost"][i % 3], amountText: "×200", rarity: RARITY[i % 2] },
        premium: { key: `p${i}`, name: ["Skin", "Emote", "Banner", "Frame"][i % 4], rarity: RARITY[(i % 3) + 1] } })),
      label: "Battle pass", freeLabel: "Free", premiumLabel: "Premium", levelLabel: "Level", xpLabel: "XP", claimLabel: "Claim", claimAllLabel: "Claim all", claimedLabel: "Claimed", unlockLabel: "Unlock premium", lockedLabel: "Premium", reachLabel: "Reach level", daysLabel: "d", hoursLabel: "h",
    } as never));
    fill("battle-pass", "ChallengeList", () => puzzle.goalChips("Goals", goals()));
  },
  character: () => {
    fill("character", "Paperdoll", () => rpg.paperdoll("Gear", gear(), () => {}));
    fill("character", "Compare", () => og.ui.statCompare("Compare", { rows: SWORD, width: grow } as never));
    fill("character", "Loadout", () => shooter.loadout("Loadout", loadout()));
  },
  "deck-builder": () => fill("deck-builder", "Builder", () => og.ui.deckBuilder("Builder", {
    deckName: "Frost tempo", deckSize: 24, copyLimit: 2,
    cards: ["Frost bolt", "Glacier wall", "Ice sprite", "Rime knight", "Blizzard", "Shatter", "Snow owl", "Cold snap", "Winter crown", "Hoarfrost", "Thaw", "Aurora"].map((name, i) => ({
      key: `c${i}`, name, cost: (i % 6) + 1, kind: ["unit", "spell", "relic"][i % 3], typeLine: ["Unit · Elemental", "Spell", "Relic"][i % 3], text: "Freeze an enemy; draw a card.", rarity: RARITY[i % 5], owned: 2, count: i < 8 ? (i % 2) + 1 : 0, limit: 2,
    })),
    rarities: RARITY.map((rarity) => ({ rarity, label: rarity[0]!.toUpperCase() + rarity.slice(1) })),
    kinds: ["unit", "spell", "relic"].map((key) => ({ key, label: key[0]!.toUpperCase() + key.slice(1) })),
    sorts: ["cost", "name", "rarity"].map((key) => ({ key, label: key })),
    allLabel: "All", searchLabel: "Search cards", sortLabel: "Sort", collectionLabel: "Collection", deckLabel: "Deck", curveLabel: "Mana curve", kindsLabel: "Types", doneLabel: "Done", fullLabel: "Deck full", limitLabel: "Limit", unownedLabel: "Not owned", emptyLabel: "No cards",
  } as never)),
  draft: () => fill("draft", "Draft", () => og.ui.draft("Draft", {
    roster: ROSTER.map((e) => ({ ...e, locked: false, taken: false })) as never, phase: "ban", secondsLeft: 18, seconds: 30, yourTurn: true, hovered: "mira",
    teams: [
      { name: "Lumen", team: 0, bans: ["orrin"], picks: [{ player: "You", entry: "aria", locked: true, you: true }, { player: "Sable", active: true }, { player: "Juno" }] },
      { name: "Riven", team: 1, bans: ["vex"], picks: [{ player: "Riven", entry: "kael", locked: true }, { player: "Bram" }, { player: "Tess" }] },
    ],
    lockLabel: "Lock in", banLabel: "Ban", waitingLabel: "Waiting", bansLabel: "Bans", bannedLabel: "Banned", pickedLabel: "Picked",
  } as never)),
  "pull-results": () => fill("pull-results", "Sequence", () => gacha.pullSequence("Pull", pull(1))),
  "reward-choice": () => {
    fill("reward-choice", "Choice", () => og.ui.rewardChoice("Choice", {
      title: "Choose a reward", subtitle: "Pick one", picks: 1, selected: ["ember"], rerolls: 2, rerollCost: "50 gold", banishes: 1,
      options: [
        { key: "ember", title: "Ember heart", kind: "Relic", text: "Burning enemies take 20% more damage.", icon: "fire", rarity: "rare", tag: "Synergy" },
        { key: "frost", title: "Frost step", kind: "Skill", text: "Dashing leaves a trail that slows.", icon: "drop", rarity: "uncommon" },
        { key: "crown", title: "Gilded crown", kind: "Relic", text: "Gain 1 gold for every 10 you hold.", icon: "crown", rarity: "epic", tag: "New" },
      ],
      skipLabel: "Skip", rerollLabel: "Reroll", banishLabel: "Banish", confirmLabel: "Take",
    } as never));
    fill("reward-choice", "Context", () => kv("Build", [["Deck", "18 cards"], ["Relics", "4"], ["Gold", "212"]]));
  },
  "run-map": () => fill("run-map", "Graph", () => og.ui.nodeGraph("Map", {
    nodes: [[0, 2, "current"], [1, 1], [1, 3], [2, 0], [2, 2], [2, 4], [3, 1], [3, 3], [4, 2]].map(([x, y, state], i) => ({
      id: `n${i}`, x: 60 + (x as number) * 150, y: 40 + (y as number) * 70, icon: ["home", "sword", "coin", "potion", "sword", "key", "gem", "sword", "skull"][i], label: ["Start", "Fight", "Shop", "Rest", "Fight", "Treasure", "Event", "Elite", "Boss"][i], state: (state as string) ?? "available",
    })),
    links: [[0, 1], [0, 2], [1, 3], [1, 4], [2, 4], [2, 5], [3, 6], [4, 6], [4, 7], [5, 7], [6, 8], [7, 8]].map(([a, b]) => ({ from: `n${a}`, to: `n${b}` })),
    width: grow, height: grow, label: "Act 1",
  } as never)),
  stash: () => {
    fill("stash", "StashGrid", () => rpg.inventoryGrid("Stash", bag()));
    fill("stash", "BagGrid", () => rpg.inventoryGrid("Bag", bag()));
    fill("stash", "Amount", () => stepper(250, "Gold"));
  },
  inventory: () => {
    fill("inventory", "Grid", () => rpg.inventoryGrid("Bag", bag()));
    fill("inventory", "Detail", () => og.ui.statCompare("Compare", { rows: SWORD, width: grow } as never));
    fill("inventory", "Equip", () => rpg.paperdoll("Gear", gear(), () => {}));
  },
  vendor: () => {
    fill("vendor", "Bag", () => rpg.inventoryGrid("Bag", bag()));
    fill("vendor", "Compare", () => og.ui.statCompare("Compare", { rows: SWORD, width: grow } as never));
    fill("vendor", "Amount", () => stepper(1, "Quantity"));
  },
  "auction-house": () => {
    fill("auction-house", "Listings", () => og.ui.table("Listings", {
      rows: LISTINGS, width: grow, height: grow, label: "Listings",
      columns: [{ key: "name", label: "Item" }, { key: "level", label: "Level", numeric: true, width: 70 }, { key: "price", label: "Buyout", numeric: true, format: "grouped", width: 100 }, { key: "left", label: "Time left", width: 90 }],
    } as never));
    fill("auction-house", "History", () => og.ui.chart("History", { kind: "line", title: "Price, 7 days", categories: ["M", "T", "W", "T", "F", "S", "S"], series: [{ label: "Starsteel ingot", values: [1100, 1150, 1180, 1240, 1210, 1190, 1200] }] } as never));
  },
  store: () => fill("store", "Store", () => og.ui.storeFront("Store", {
    wallet: [{ key: "gems", name: "Gems", amount: 4800, topUp: true }, { key: "coins", name: "Coins", amount: 12_450, topUp: true }],
    sections: [
      { key: "featured", title: "Featured", endsIn: 2 * 86400 + 4 * 3600, offers: [
        { key: "tide", name: "Moonlit Tide bundle", kind: "Bundle", rarity: "legendary", price: 1980, currency: "gems", original: 2800, badge: "-30%", featured: true, contents: ["Nerissa", "Tide glider", "800 gems"] },
        { key: "crown", name: "Gilded crown", kind: "Cosmetic", rarity: "epic", price: 980, currency: "gems" },
      ] },
      { key: "daily", title: "Daily", endsIn: 7 * 3600, offers: [
        { key: "potion", name: "Ember potion ×5", kind: "Item", rarity: "common", price: 300, currency: "coins", limit: 3, bought: 1 },
        { key: "key", name: "Vault key", kind: "Item", rarity: "rare", price: 120, currency: "gems" },
        { key: "free", name: "Daily gift", kind: "Gift", rarity: "uncommon", price: 0 },
      ] },
      { key: "packs", title: "Gem packs", offers: [{ key: "g1", name: "500 gems", priceText: "$4.99", rarity: "rare" }, { key: "g2", name: "1,200 gems", priceText: "$9.99", rarity: "epic", badge: "Best value" }] },
    ],
    label: "Store", freeLabel: "Free", ownedLabel: "Owned", soldOutLabel: "Sold out", leftLabel: "left", saveLabel: "Save", refreshLabel: "Refreshes in", topUpLabel: "Top up", daysLabel: "d", hoursLabel: "h",
  } as never)),
  training: () => {
    fill("training", "Bars", () => og.ui.row("Meters", { width: grow, gap: 24 }, () => {
      og.ui.statBar("P1", { value: 740, max: 1000, label: "Aria", showValue: true, width: grow } as never);
      og.ui.statBar("P2", { value: 1000, max: 1000, label: "Dummy", showValue: true, reverse: true, width: grow } as never);
    }));
    fill("training", "Inputs", () => kv("History", [["↓ ↘ → + LP", "12f"], ["→ + HP", "18f"], ["↓ + MK", "9f"], ["↓ ↙ ← + HK", "21f"]], "Inputs"));
    fill("training", "Dummy", () => kv("Dummy", [["Stance", "Standing"], ["Block", "After first hit"], ["Recovery", "Neutral"], ["Counter hit", "Off"]], "Dummy"));
  },
  tutorial: () => {
    og.ui.anchor("Target", { anchor: "center", width: { kind: "fixed", px: 220 }, height: { kind: "fixed", px: 56 } }, () => og.ui.pushButton("Jump", "Jump"));
    fill("tutorial", "Hint", () => og.ui.actionHints("Hints", { hints: [{ action: "jump", hint: "Jump" }, { action: "confirm", hint: "Continue" }] } as never));
  },
  "creator-tools": () => {
    fill("creator-tools", "Hierarchy", () => og.ui.tree("Scene", {
      items: [{ id: "world", label: "World" }, { id: "terrain", parent: "world", label: "Terrain" }, { id: "lights", parent: "world", label: "Lights" }, { id: "sun", parent: "lights", label: "Sun" }, { id: "lantern", parent: "world", label: "Lantern_03" }, { id: "player", parent: "world", label: "Player" }, { id: "camera", parent: "player", label: "Camera" }],
      expanded: ["world", "lights", "player"], selected: "lantern", label: "Hierarchy",
    } as never));
    fill("creator-tools", "Inspector", () => kv("Fields", [["Position", "12.0, 1.5, -4.0"], ["Rotation", "0, 45, 0"], ["Scale", "1, 1, 1"], ["Light", "Warm · 2.4"], ["Range", "6 m"]], "Lantern_03"));
  },
  "saga-map": () => fill("saga-map", "Map", () => puzzle.sagaMap("Map", saga())),
  "skill-tree": () => {
    fill("skill-tree", "Graph", () => og.ui.nodeGraph("Tree", {
      nodes: [[2, 0, "current", "Ember"], [1, 1, "available", "Kindle"], [3, 1, "available", "Ward"], [0, 2, "available", "Wildfire"], [2, 2, "available", "Cinder"], [4, 2, "available", "Aegis"], [2, 3, "available", "Phoenix"]].map(([x, y, state, label], i) => ({
        id: `k${i}`, x: 80 + (x as number) * 120, y: 40 + (y as number) * 90, icon: ["fire", "bolt", "shield", "fire", "star", "shield", "crown"][i], label, state,
      })),
      links: [[0, 1], [0, 2], [1, 3], [1, 4], [2, 4], [2, 5], [4, 6]].map(([a, b]) => ({ from: `k${a}`, to: `k${b}`, active: a === 0 })),
      width: grow, height: grow, label: "Pyromancy",
    } as never));
    fill("skill-tree", "Detail", () => kv("Rank", [["Ember", "Rank 2 / 5"], ["Burn damage", "+12% → +18%"], ["Cost", "1 point"]], "Ember"));
  },
  replay: () => {
    fill("replay", "Players", () => rpg.partyFrames("Party", party()));
    fill("replay", "Overlay", () => shooter.scoreboard("Board", scoreboard()));
  },
  prestige: () => fill("prestige", "Records", () => kv("Run", [["Best floor", "38"], ["Gold earned", "1.2M"], ["Time played", "4h 12m"], ["Prestiges", "3"]], "This run")),
};
