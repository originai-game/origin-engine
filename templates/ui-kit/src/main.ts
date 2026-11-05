// The UI kit demo: every og.ui template and the game widget set on one
// gallery. Pick a screen from the list (or `show:<screen>` on the command
// channel); Back (Escape, gamepad B, the Gallery button) returns.
//
// It is also the reference source of crates/og-ui/references: the
// playtests in tests/ show each screen at desktop (1280×720) and phone
// portrait (390×844 at 3×, with a notch and a home indicator) and compare
// the frames, and drive the screens with touch_move_to, move_to_text and
// keyboard and gamepad focus.
//
// Commands: show:<screen>, viewport:<desktop|tablet|phone|landscape>,
// pad:<up|down|left|right|next|prev|activate|back>, toast:<text>,
// damage:<hp>, heal:<hp>, style:<a STYLES name: default, a pack (pixel, ...) or a seed (gilded, ...)>
// (a built-in style pack, @styles/<name>), transition:<none|fade|wipe|iris|dissolve|slide>
// (screen changes go through og.ui.screenTransition), sounds:<on|off> (on by default),
// mixed:<on|off> (a label of CJK, Latin and Arabic under the locale screens),
// offline:<on|off> (the leaderboard's rows fail to load: its error and Retry).

import { og, defineComponent, defineResource, defineSystem, bool, f32, f64, str, u32 } from "@origingame/engine";
import type { Entity, Viewport, World } from "@origingame/engine";
import { dialogueScreen, loadStory, questsScreen, vnScreen } from "./story.ts";
import { charactersScreen, draftScreen, lobbyScreen, mapScreen, songsScreen, versusScreen } from "./genres.ts";
import { chatScreen, cityMapScreen, defineGenres2Actions, garageScreen, ledgerScreen, rtsScreen } from "./genres2.ts";
import { cgGalleryScreen, cgViewing, deckScreen, defineGenres3Actions, loadoutScreen, photoHidden, photoScreen, restoreSettingsTree, routeChartScreen, settingsTreeScreen } from "./genres3.ts";
import { art, boardScreen, chromeRoom, L, cardsScreen, gachaScreen, gearScreen, generatorsScreen, loadPatternArt, passScreen, patternScene, patternsCommand, rewardsScreen, sagaScreen, storeScreen } from "./patterns.ts";
import { chartsScreen, containersScreen, dataScreen, demoCommand, graphScreen, navigationScreen, overlaysScreen } from "./components.ts";
import { bindingsScreen, controlsDemo, controlsScreen, fieldsScreen, foundationsScreen, rangesScreen } from "./controls.ts";
import { abilityCardsScreen, hangarScreen, loadStarshipArt, sectorMapScreen, shipStatusScreen, starshipCommand, turnOrderScreen } from "./starship.ts";
import { buildRaceScene, RaceView } from "./raceScene.ts";
import { chromeAnchor, chromeInDocument, chromeSlide, loadShowcases, showcaseCommand, showcaseNext, showcaseOf, showcaseRow, showcaseScreen, SHOWCASES, SHOWCASE_SCREENS, syncDocs } from "./showcases.ts";
import { defineHudActions, gameScene } from "./huds.ts";
import { packGround } from "./ground.ts";
import { styleNames, stylePaths } from "../tools/style-names.js" with { type: "macro" };

export const SCREENS = [
  "gallery", "widgets", "toasts",
  "players", "text", "materials",
  "containers", "navigation", "overlays", "data", "charts", "chartsMore", "graph",
  "foundations", "controls", "ranges", "fields", "bindings",
  "gear", "cards", "rewards", "gacha", "saga", "board", "generators", "store", "pass",
  "dialogue", "vn", "quests",
  "shipStatus", "turnOrder", "abilityCards", "sectorMap", "hangar",
  "lobby", "select", "versus", "draft", "worldMap", "songs",
  "garage", "ledger", "cityMap", "chat", "rts",
  "photo", "loadout", "deck", "cgGallery", "routes", "settingsTree",
  ...SHOWCASE_SCREENS,
] as const;
export type Screen = (typeof SCREENS)[number];

/** The engine's built-in UI styles (templates/styles, in the bundle as
 * `@styles/<name>/` because assets/looks/kit-<name>.ogt names each): the
 * atlas packs, then the seed styles og style new generated;
 * "default" is the engine's default style (the design language's, for prototypes). */
export const STYLES = styleNames();
const STYLE_PATHS = stylePaths();
const WIPES = ["none", "fade", "wipe", "iris", "dissolve", "slide"] as const;
const UI_SOUNDS = ["hover", "press", "open", "close", "success", "error"] as const;

const BLURBS: Record<Screen, string> = {
  gallery: "",
  widgets: "Every widget: tabs, select, stepper, radio, meters, rings, touch controls",
  toasts: "The toast queue in every tone",
  players: "Local multiplayer: four pads, four rings",
  text: "Kerning, ellipses, scripts, fields and text areas",
  materials: "Icons, edge light, pictures, two-finger scroll and pinch",
  containers: "Cards, accordion, scroll area, split view",
  navigation: "Tabs with bumpers, carousel, breadcrumbs, pagination, action bar",
  overlays: "Menus, context menu, popover, dialog, confirm, sheets, radial wheel",
  data: "Table of 10,000 rows, list, tree, grid, stat rows",
  charts: "Line, area and grouped bar charts",
  chartsMore: "Stacked bars, radar and donut charts",
  graph: "Skill tree graph and calendar",
  foundations: "Text, prose, icons, art slots, shapes, viewport, effects",
  controls: "Buttons, hold to confirm, switches, checkboxes, radios, segmented, cycler",
  ranges: "Sliders, range slider, steppers",
  fields: "Text fields, search, selects, colour picker",
  bindings: "Key bindings and the on-screen keyboard",
  gear: "Paperdoll, backpack and stash: multi-cell items, stacks, rotation",
  cards: "A hand of cards: fan, hover lift, drag to play, a targeting arc",
  rewards: "Pick one of three: reroll, banish, skip, a locked reward",
  gacha: "A gacha banner, its pity and purse, and a ten-pull reveal",
  saga: "A saga map: levels on a painted path, stars, the avatar's pin, locked and boss levels",
  board: "A match-3 level: goal chips, the score toward three stars, moves left",
  generators: "An idle game: generators with their cycles and buy amount, and the prestige",
  store: "A store: the purse, a featured bundle, a daily rotation, limits and sold out",
  pass: "A battle pass: the season, its level and XP, free and premium tracks, claims",
  dialogue: "A conversation: the speaker, the line typing out, choices with checks and odds, a timer, a roll",
  vn: "A visual novel's text box: the name plate, the narrator, Auto, and the backlog",
  quests: "A quest log: tabs, new and updated quests, objectives, rewards, Track; the HUD's tracker",
  shipStatus: "A ship's status: its name and class, its silhouette, hull, shield pips and reactor",
  turnOrder: "A turn order: portrait rows top to bottom, the acting ship first, enemies striped",
  abilityCards: "A hand of six ability cards: number, name, art, effect, cost, hotkey; End turn",
  sectorMap: "A sector map: allied triangles and hostile diamonds, the current system ringed",
  hangar: "A hangar: ship classes, the hero ship's stats and upgrades, its detail and Commission",
  lobby: "A party lobby: the lineup, the mode, Ready up, Find match, the queue and the ready check",
  select: "A character select: the roster, both players' fighters, Lock in and Random",
  versus: "A versus screen: two fighters on a slant, the mark, the stage and the load",
  draft: "A draft with bans: both teams' picks and bans, the roster, the turn's timer",
  worldMap: "A world map: regions under fog, places by kind, the legend, fast travel and waypoints",
  songs: "A song select: the wheel of jackets, the charts, the best score, the preview's waveform",
  garage: "A garage: the categories, a category's parts and tiers, the car on its turntable, performance; the livery's layers",
  ledger: "A management ledger: the treasury's figures, taxes and services with budgets, sortable, the history and breakdown",
  cityMap: "A city's map overlays: the bar of info views, the shown one's legend and its wash over the map",
  chat: "A chat window: channel tabs with unread counts, links and mentions, the input with its channel; the HUD's fading chat",
  rts: "An RTS console: the selection and its control groups, the command card's orders and Build page, hotkeys and targeting",
  settingsTree: "The settings tree as data: graphics, audio, controls by physical key, accessibility, language and the HUD editor; Apply with a countdown",
  cgGallery: "A visual novel's CG gallery: the chapters, the pictures with the unseen veiled and their hints, the viewer with versions",
  routes: "A visual novel's route flowchart: scenes by chapter and route, the choices on the branches, endings, a jump to a scene seen",
  deck: "A deck builder: the collection's filters and grid with the copies owned, the deck's list, its curve and counts, Done",
  loadout: "A loadout: the loadouts' tabs, the slots, the weapon with its attachments called out, its figures, Remove",
  photo: "Photo mode: the camera's and lens's sliders, filters and frames over the paused shot, the grid, Hide UI and the shutter",

  actionTitle: "An action game's title: Continue, New game, the news, over its key art (the main-menu document)",
  actionHud: "A platformer's HUD: health in segments, coins, the stage timer, the combo, a door's prompt, hits' numbers, touch controls",
  actionPause: "The pause menu over the HUD (the pause document): the objective, time played, Resume, Settings, Quit",
  actionTutorial: "A tutorial step's callout over the HUD (the tutorial document)",
  actionLoading: "Loading into a stage: its key art, a tip, real progress, Continue (the loading document)",
  actionResults: "A stage cleared: the grade, stats, XP and rewards (the results document)",
  shooterHud: "A shooter's HUD: compass, minimap, ring timer, killfeed, crosshair, health and shields, abilities, ammo, a ping",
  shooterScore: "The scoreboard held over the match (the scoreboard document)",
  manageHud: "A management HUD: resources and rates, the date and speed, goals, the build palette, a plot's card",
  raceHud: "A racing HUD: the place and lap with sector pips, the relative tower, the lap time and split, a player-up circuit, the tach; the race's moments",
  raceResults: "A race's results: the place, the stats, the credits (the results document)",
  raceBoard: "Time-trial leaderboards: global and friends, a league (the leaderboard document)",
  sportsHud: "A football broadcast HUD: the scorebug, the player's plate, the radar, a substitution",
  matchFacts: "Full time: the scorers by side and the match stats face to face",
  mobaHud: "A MOBA's HUD: the kills and the clock, the hero's frame, four abilities with the ultimate, the lane map, heroes' plates",
  rpgHud: "An action RPG's HUD: health and mana orbs, skills and potions, XP, the quest tracker, an elite's plate and hits",
  mmoHud: "An MMO raid HUD: player and target frames, the boss's cast, the party, two action bars, buffs, the encounter",
  rogueHud: "A roguelike's run HUD: health and the boons taken, the chamber, gold and keys, the four moves",
  tdHud: "A tower defence HUD: gold and lives, the waves to come, the next wave's call, the towers with their costs",
  rhythmHud: "A rhythm game's play screen: four lanes and their notes, the lane keys, the judgement, combo, score and life",
  fightHud: "A fighting game's HUD: health from the centre out, the round clock and rounds won, super gauges, the combo",
  survivalHud: "A survival HUD: health, food and water, statuses, stamina, the compass, the day, the hotbar, a gather prompt",
  horrorHud: "A horror HUD: almost nothing; the light's battery, a hide prompt, the objective, stamina only when spent",
  openHud: "An open world's HUD: the compass with the quest's marker, the tracked quest, the minimap, the time, a waypoint, a talk prompt",
};

/** The demo's state: one entity named "demo" (dump_entity:demo reads it). */
export const Demo = defineComponent("Demo", {
  screen: u32(0),
  paused: bool(),
  // Asset handles exceed 32 bits: f64 keeps them exact.
  font: f64(0),
  // The display and rounded faces and the sample scripts' fonts (assets/fonts, tools/make-faces.py).
  displayFont: f64(0),
  roundedFont: f64(0),
  hebrewFont: f64(0),
  devanagariFont: f64(0),
  icons: f64(0),
  // The iconoir icon set (assets/icons/iconoir.ogt, cooked like a font) and whether it is the atlas now.
  iconSet: f64(0),
  iconoir: bool(),
  // The materials screen: the map's zoom and the quests pressed.
  zoom: f32(1),
  quests: u32(0),
  backdrop: f64(0),
  hp: f32(72),
  mana: f32(40),
  coins: u32(128),
  score: u32(4250),
  lives: u32(2),
  time: f32(95),
  // Settings (GameSettings, as a game keeps them).
  masterVolume: f32(1),
  musicVolume: f32(0.8),
  sfxVolume: f32(1),
  uiVolume: f32(1),
  muted: bool(),
  subtitles: bool(true),
  quality: u32(1),
  showFps: bool(),
  reducedMotion: bool(),
  highContrast: bool(),
  vibration: bool(true),
  invertY: bool(),
  sensitivity: f32(1),
  language: u32(0),
  saves: u32(0),
  // og.platform.loadSettings's request (read on the first tick), 0 once answered.
  settingsRequest: u32(0),
  // Settings read back from og.settings (a reload comes back as the player left it).
  loads: u32(0),
  // Widgets screen.
  tab: u32(0),
  difficulty: u32(1),
  amount: f32(3),
  mode: u32(0),
  optA: bool(true),
  optB: bool(),
  chip: u32(0),
  stickX: f32(),
  stickY: f32(),
  // Story.
  line: u32(0),
  step: u32(0),
  // Shop and inventory.
  balance: f32(1250),
  owned: u32(0),
  bought: u32(0),
  refused: u32(0),
  inv0: u32(0),
  inv1: u32(1),
  inv2: u32(2),
  inv3: u32(3),
  inv4: u32(4),
  inv5: u32(5),
  inv6: u32(6),
  inv7: u32(7),
  inv8: u32(8),
  inv9: u32(9),
  inv10: u32(10),
  inv11: u32(11),
  inv12: u32(12),
  inv13: u32(13),
  inv14: u32(14),
  inv15: u32(16),
  inv16: u32(18),
  inv17: u32(19),
  inv18: u32(21),
  inv19: u32(24),
  moves: u32(0),
  used: u32(0),
  // Idle.
  gold: f32(40),
  gen0: u32(1),
  gen1: u32(0),
  gen2: u32(0),
  taps: u32(0),
  // Leaderboard.
  scope: u32(0),
  scopeTick: u32(0),
  // `offline:on`: the rows fail to load (the error and Retry); Retry loads again.
  offline: bool(),
  // Dialogs.
  confirmed: u32(0),
  level: u32(0),
  // Players: each pad's pick (HEROES index + 1, 0 none) and the hero its focus is on (index + 1).
  pick0: u32(0),
  pick1: u32(0),
  pick2: u32(0),
  pick3: u32(0),
  focus0: u32(0),
  focus1: u32(0),
  focus2: u32(0),
  focus3: u32(0),
  // How many times the lobby hint's "How to join" link was followed.
  rules: u32(0),
  // The last frame's accessibility tree (og.ui.a11y): its nodes and its focus order's length.
  a11yNodes: u32(0),
  a11yFocus: u32(0),
  // Look: the style pack (STYLES index), the screen transition (WIPES index), UI sounds.
  style: u32(0),
  wipe: u32(0),
  sounds: bool(true),
  // `mixed:on`: a label mixing CJK, Latin and Arabic (menu.mixed) under the locale screens.
  mixed: bool(),
});
type DemoState = { -readonly [K in keyof ReturnType<typeof demoDefaults>]: ReturnType<typeof demoDefaults>[K] };
function demoDefaults() {
  return { screen: 0, paused: false, font: 0, displayFont: 0, roundedFont: 0, hebrewFont: 0, devanagariFont: 0, icons: 0, iconSet: 0, iconoir: false, zoom: 1, quests: 0, backdrop: 0, hp: 72, mana: 40, coins: 128, score: 4250, lives: 2, time: 95, masterVolume: 1, musicVolume: 0.8, sfxVolume: 1, uiVolume: 1, muted: false, subtitles: true, quality: 1, showFps: false, reducedMotion: false, highContrast: false, vibration: true, invertY: false, sensitivity: 1, language: 0, saves: 0, settingsRequest: 0, loads: 0, tab: 0, difficulty: 1, amount: 3, mode: 0, optA: true, optB: false, chip: 0, stickX: 0, stickY: 0, line: 0, step: 0, balance: 1250, owned: 0, bought: 0, refused: 0, inv0: 0, inv1: 1, inv2: 2, inv3: 3, inv4: 4, inv5: 5, inv6: 6, inv7: 7, inv8: 8, inv9: 9, inv10: 10, inv11: 11, inv12: 12, inv13: 13, inv14: 14, inv15: 16, inv16: 18, inv17: 19, inv18: 21, inv19: 24, moves: 0, used: 0, gold: 40, gen0: 1, gen1: 0, gen2: 0, taps: 0, scope: 0, scopeTick: 0, confirmed: 0, level: 0, pick0: 0, pick1: 0, pick2: 0, pick3: 0, focus0: 0, focus1: 0, focus2: 0, focus3: 0, rules: 0, a11yNodes: 0, a11yFocus: 0, style: 0, wipe: 0, sounds: true, mixed: false, offline: false };
}

export const ICONS = [
  "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
  "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
  "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
  "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
];

const QUALITIES = ["low", "medium", "high"] as const;
const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "zh-CN", label: "简体中文" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "ar", label: "العربية" },
];

/** The string tables (assets/strings/*.ogt), one per locale; en is the base. */
const LOCALES = ["en", "zh-CN", "ja", "ko", "ar"] as const;

/** A key of the active locale's table, with its arguments (og.i18n). */
const tr = (world: World, key: string, args?: Record<string, string | number>) => og.i18n.t(world, key, args);

/**
 * The kit's painted art (assets/textures/art, CC0; SOURCE.md): item and
 * boon icons, portraits and scenes, by key, loaded in start(). A handle
 * is 0 where the art did not load, and the screens show the line icon.
 * (patterns.ts keeps its own set for the pattern previews, `pat:art:off`.)
 */
const ART_PATHS: Record<string, string> = {
  // Items stand on the slot's own ground: cut out on transparent ground (art/cutouts).
  ...Object.fromEntries(["iron-sword", "oak-shield", "crown", "hunters-bow", "elven-bow", "healing-draught", "mana-vial", "ember-ring", "iron-key", "sapphire-amulet", "knight-helm", "gauntlets", "boots", "ashbringer", "arcane-staff", "mage-robe", "gold", "crystal", "bomb", "apple"].map((k) => [k, `textures/art/cutouts/${k}.png`])),
  ...Object.fromEntries(["brann", "aria"].map((k) => [k, `textures/art/portraits/${k}.jpg`])),
  ...Object.fromEntries(["kawaii", "cartoon", "storybook"].map((k) => [`event-${k}`, `textures/art/events/${k}.jpg`])),
  ...Object.fromEntries(["neutral", "title", "gacha-moonlit-tide", "paperdoll-aria-vell", "jade", "kawaii", "doodle-town", "midcore", "storybook-vale", "manuscript", "golden-valley", "tactical", "wasteland", "gothic-halls", "scifi", "cyber", "foundry", "broadcast"].map((k) => [k, `textures/art/scenes/${k}.jpg`])),
  "pixel-dusk": "textures/art/scenes/pixel-dusk.png",
};
const KIT_ART: Record<string, number> = {};
/** An art handle by key, or null where it did not load. */
const kitArt = (key: string): never | null => (KIT_ART[key] ? (KIT_ART[key] as never) : null);

/**
 * Each style's backdrop: the key art or scene behind every screen but the
 * gallery and the widget sheet (a KIT_ART key; the default style, and a
 * style without one, keeps the neutral gradient: neutral chrome). A game
 * declares its own the same way: the scene that belongs with its UI.
 */
const STYLE_BACKDROPS: Record<string, string> = {
  fantasy: "title", jade: "jade", kawaii: "kawaii", cartoon: "doodle-town", midcore: "midcore", storybook: "storybook-vale",
  craft: "manuscript", pixel: "pixel-dusk", gilded: "golden-valley", tactical: "tactical", wasteland: "wasteland", dread: "gothic-halls",
  scifi: "scifi", cyber: "cyber", foundry: "foundry", broadcast: "broadcast",
};

/** The lobby's heroes, and each player's colour (the pad's focus ring). */
const HEROES = ["Knight", "Ranger", "Mage", "Rogue", "Bard", "Monk", "Smith", "Thief"];
const PLAYER_INKS = ["focus", "info", "success", "warn"] as const;

/**
 * A preview's panel in the room the kit's chrome leaves (under it on a phone, above it on a desktop), centred there at
 * `width` (grow on a phone): its content (`build`, in a scroll area `<id>/Body`) scrolls where it is taller than that
 * room, as at large text, so nothing runs under the chrome or past the screen.
 */
function roomPanel(id: string, width: number, pad: { x: number; y: number }, gap: number, build: () => void): void {
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const t = og.ui.frameTheme();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const room = chromeRoom(t.space.sm);
  const margin = 16;
  const free = vp.height - sa.top - sa.bottom - room.top - room.bottom - 2 * margin;
  og.ui.anchor(id, {
    anchor: "center",
    offsetY: Math.round((room.top - room.bottom) / 2 + (sa.top - sa.bottom) / 2),
    direction: "column",
    width: phone ? { kind: "grow" } : { kind: "fixed", px: width },
    padding: { left: pad.x, top: pad.y, right: pad.x, bottom: pad.y },
    background: { token: "panel" },
    radius: t.radius.panel,
  }, () => {
    og.ui.scrollArea("Body", { width: { kind: "grow" }, height: { kind: "fit", max: Math.max(80, free - 2 * pad.y) }, gap, fadeInto: "panel", padding: { left: 0, top: 0, right: 0, bottom: 0 } }, build);
  });
}

/** A local-multiplayer lobby: up to four pads, each with its own focus ring; A picks a hero for that pad's player. */
function playersScreen(d: DemoState): void {
  og.ui.setPlayers(4);
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  const picks = [d.pick0, d.pick1, d.pick2, d.pick3];
  roomPanel("Lobby", 640, { x: t.space.xl, y: t.space.xl }, t.space.md, () => {
    og.ui.label("Title", "Players", { step: "xl2", font: "display", heading: true });
    // Rich text: the button in bold, a link (a child element Hint/join, focusable by every pad).
    const hint = og.ui.richText("Hint", "Each pad moves its own ring; [b]A[/b] picks a hero for that player. [link=join]How to join[/link]", { color: { token: "inkMuted" }, width: grow });
    if (hint.link === "join") d.rules += 1;
    // The player cards sit on bgSubtle, darker than the panel, so each player's ink and "Press A" read at 4.5:1.
    og.ui.row("Cards", { gap: t.space.sm, width: grow, wrap: phone }, () => {
      for (let p = 0; p < 4; p++) {
        og.ui.column(og.ui.idi("Player", p), { width: grow, gap: t.space.xxs, cross: "center", padding: { left: t.space.sm, top: t.space.sm, right: t.space.sm, bottom: t.space.sm }, background: { token: "well" }, radius: t.radius.panel }, () => {
          og.ui.label("Name", `P${p + 1}`, { step: "lg", font: "display", color: { token: PLAYER_INKS[p]! }, nowrap: true });
          og.ui.label("Pick", picks[p] ? HEROES[picks[p]! - 1]! : "Press A", { color: { token: picks[p] ? "ink" : "inkMuted" }, nowrap: true });
        });
      }
    });
    og.ui.row("Heroes", { gap: t.space.sm, wrap: true, width: grow }, () => {
      HEROES.forEach((h, i) => {
        // At least a column's width, and as wide as its name at large text.
        const r = og.ui.button(og.ui.idi("Hero", i), h, { width: { kind: "fit", min: phone ? 150 : 132 } });
        if (r.clicked) picks[r.pad ?? 0] = i + 1;
      });
    });
  });
  [d.pick0, d.pick1, d.pick2, d.pick3] = picks as [number, number, number, number];
  const focused = (pad: number) => Number(/Hero#(\d+)$/.exec(og.ui.focus(pad))?.[1] ?? -1) + 1;
  [d.focus0, d.focus1, d.focus2, d.focus3] = [focused(0), focused(1), focused(2), focused(3)];
}

function demoEntity(world: World): Entity | null {
  return world.find("demo");
}

function screenIndex(name: string): number {
  return Math.max(0, SCREENS.indexOf(name as Screen));
}

/** The settings the kit keeps in og.settings (the settings tree saves them, genres3.ts): every field optional. */
type KitSettings = {
  masterVolume: number; musicVolume: number; sfxVolume: number; uiVolume: number; muted: boolean; subtitles: boolean;
  quality: string; showFps: boolean; reducedMotion: boolean; highContrast: boolean; vibration: boolean; invertY: boolean;
  sensitivity: number; language: string; display: unknown; rawMouse: boolean;
};

/**
 * Takes settings into the demo's state: the settings screen's each frame,
 * and og.settings as the platform kept it (`loaded`), where a field left
 * out keeps its value and the language is asked for even when the index
 * matches (the host's locale may be another). The language goes to
 * og.i18n, contrast and motion to the theme.
 */
function takeSettings(world: World, d: DemoState, s: Partial<KitSettings>, loaded: boolean): void {
  const num = (v: unknown, now: number) => (typeof v === "number" && Number.isFinite(v) ? v : now);
  const flag = (v: unknown, now: boolean) => (typeof v === "boolean" ? v : now);
  d.masterVolume = num(s.masterVolume, d.masterVolume);
  d.musicVolume = num(s.musicVolume, d.musicVolume);
  d.sfxVolume = num(s.sfxVolume, d.sfxVolume);
  d.uiVolume = num(s.uiVolume, d.uiVolume);
  d.muted = flag(s.muted, d.muted);
  d.subtitles = flag(s.subtitles, d.subtitles);
  const quality = QUALITIES.indexOf((s.quality ?? (loaded ? "" : "medium")) as (typeof QUALITIES)[number]);
  if (quality >= 0 || !loaded) d.quality = Math.max(0, quality);
  d.showFps = flag(s.showFps, d.showFps);
  d.vibration = flag(s.vibration, d.vibration);
  d.invertY = flag(s.invertY, d.invertY);
  d.sensitivity = num(s.sensitivity, d.sensitivity);
  // The screen applied its own changes; a load puts the saved ones back.
  if (loaded && s.display && typeof s.display === "object") {
    const r = og.render.setDisplay(world, s.display as never);
    if (!r.ok) throw new Error(r.reason);
  }
  if (loaded && typeof s.rawMouse === "boolean") {
    const r = og.input.setMouse(world, { raw: s.rawMouse });
    if (!r.ok) throw new Error(r.reason);
  }
  const language = LANGUAGES.findIndex((l) => l.code === s.language);
  if (language >= 0 || !loaded) {
    const at = Math.max(0, language);
    if (at !== d.language || loaded) og.i18n.setLocale(world, LANGUAGES[at]!.code);
    d.language = at;
  }
  const contrast = flag(s.highContrast, d.highContrast);
  const motion = flag(s.reducedMotion, d.reducedMotion);
  const theme = d.highContrast !== contrast || d.reducedMotion !== motion;
  d.highContrast = contrast;
  d.reducedMotion = motion;
  if (theme) applyTheme(world, d);
}

/**
 * og.settings, asked for on the first tick and taken when the platform
 * answers (og-platform-sdk §2, §3): what the settings screen saved comes
 * back after a reload or a relaunch. Defaults show until it answers.
 */
function followSettings(world: World, tick: number, d: DemoState): void {
  if (tick === 0) {
    d.settingsRequest = og.platform.loadSettings(world);
    return;
  }
  if (d.settingsRequest === 0) return;
  const answer = og.platform.result(world, d.settingsRequest);
  if (answer === null) return;
  d.settingsRequest = 0;
  const saved = answer.ok ? (answer.data as { value?: unknown } | null)?.value : null;
  if (!saved || typeof saved !== "object") return;
  takeSettings(world, d, saved as Partial<KitSettings>, true);
  restoreSettingsTree((saved as { tree?: unknown }).tree);
  d.loads += 1;
}

/** Applies the theme-affecting settings (contrast, motion, the style pack,
 * sounds) with the demo's font and icons, or the pack's. */
function applyTheme(world: World, d: DemoState): void {
  const name = STYLES[d.style] ?? "default";
  const loaded = name === "default" ? null : og.load.style(world, STYLE_PATHS[name]!);
  if (loaded && !loaded.ok) throw new Error(loaded.reason);
  const style = loaded && loaded.ok ? loaded.value : null;
  const sounds: Record<string, unknown> = {};
  if (d.sounds) {
    for (const e of UI_SOUNDS) {
      const s = og.load.sound(world, `audio/ui-${e}.wav`);
      if (s.ok) sounds[e] = s.value;
    }
  }
  // Every field, each time: configure keeps a field left out, so a pack's
  // removal or the sounds turned off are said with null.
  og.ui.configure(world, {
    font: d.font && !style ? (d.font as never) : null,
    displayFont: d.displayFont && !style ? (d.displayFont as never) : null,
    roundedFont: d.roundedFont && !style ? (d.roundedFont as never) : null,
    // Scripts the locale tables' fonts do not cover (the text screen); under
    // a style, whose fonts are Latin-1, the demo's CJK body face too.
    fallbackFonts: [style ? d.font : 0, d.hebrewFont, d.devanagariFont].filter((h) => h !== 0) as never[],
    contrast: d.highContrast ? "high" : "standard",
    reducedMotion: d.reducedMotion,
    style,
    // Every screen draws from the iconoir set (distance-field glyphs, named
    // as ICONS names them too); a style pack's own icons on its screens but
    // the materials one; the PNG grid only if the set did not load.
    icons: d.iconSet && (d.iconoir || !style) ? { font: d.iconSet as never } : style ? null : d.icons ? { texture: d.icons as never, columns: 8, rows: 4, names: ICONS } : {},
    // Under a pack, the names its atlas does not paint (a bell, a map pin) from the iconoir set.
    fallbackIcons: d.iconSet && style && !d.iconoir ? { font: d.iconSet as never } : null,
    sounds: d.sounds ? sounds : null,
  });
}

/**
 * The gallery's rows as last measured, for one style, locale, text size and viewport: each row's height (read where it
 * stood whole in the view, since bounds are cut to it), the subtitle's, and the row under the pointer or the focus and
 * the one held.
 */
const rows = { key: "", heights: [] as number[], head: 0, hot: -1, held: -1 };

/**
 * The gallery: the title over one scroll of the subtitle and a row per screen. Each row is as tall as its name and
 * blurb, both wrapped whole at any text size and in any style, with one gap between rows. The panel keeps its inset
 * from the screen's edges and the content scrolls inside it, fading at an edge where more lies beyond. Only the rows
 * the view crosses (and half a view each side) are built; fillers hold the rest of the scroll, a row not yet measured
 * counting as the mean of those that were (og.ui.scrollState gives the offset).
 */
function gallery(d: DemoState): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const ts = Math.max(1, t.textScale ?? 1);
  const grow = { kind: "grow" } as const;
  const n = SHOWCASES.length;
  const count = n + SCREENS.length - 1;
  const gap = t.space.sm;
  const key = `${d.style}|${d.language}|${ts}|${vp.width}|${vp.height}`;
  if (rows.key !== key) Object.assign(rows, { key, heights: [], head: 0, hot: -1, held: -1 });
  // Last tick's layout: what stood whole in the view gives its height.
  const view = "Gallery/Screens/View";
  const vb = og.ui.bounds(view);
  const whole = (b: { y: number; height: number } | null): b is { y: number; height: number } => !!vb && !!b && b.height > 0 && b.y > vb.y + 0.5 && b.y + b.height < vb.y + vb.height - 0.5;
  const sb = og.ui.bounds(`${view}/Content/Subtitle`);
  if (whole(sb)) rows.head = sb.height;
  for (let i = 0; i < count; i++) {
    const b = og.ui.bounds(`${view}/Content/Row#${i}`);
    if (whole(b)) rows.heights[i] = b.height;
  }
  const line = t.typeRamp["base"]?.lineHeight ?? 24 * ts;
  const known = rows.heights.filter((x) => x > 0);
  const est = known.length > 0 ? known.reduce((x, y) => x + y, 0) / known.length : line * 3 + 2 * t.space.sm;
  const h = (i: number): number => rows.heights[i] ?? est;
  // Where each row starts in the content: the subtitle, then the rows, one gap between each.
  const head = rows.head || line * 2;
  const pos: number[] = [];
  let y = head + gap;
  for (let i = 0; i < count; i++) {
    pos[i] = y;
    y += h(i) + gap;
  }
  const sv = og.ui.scrollState(view);
  const span = sv?.viewHeight || vp.height;
  const off = sv?.offsetY ?? 0;
  const near = (i: number): boolean => pos[i]! + h(i) >= off - span / 2 && pos[i]! <= off + span * 1.5;
  let first = 0;
  while (first < count - 1 && !near(first)) first++;
  let last = count - 1;
  while (last > first && !near(last)) last--;
  const filler = (id: string, px: number): void => {
    if (px <= 0) return;
    og.ui.open(id, { width: grow, height: { kind: "fixed", px } });
    og.ui.close();
  };
  const sa = vp.safeArea;
  const room = vp.height - sa.top - sa.bottom - 2 * (phone ? t.space.md : t.space.xl);
  og.ui.anchor("Gallery", {
    anchor: "center",
    direction: "column",
    gap: t.space.md,
    width: phone ? grow : { kind: "fixed", px: 560 },
    height: { kind: "fixed", px: phone ? room : Math.min(room, Math.round(640 * ts)) },
    padding: { left: t.space.xl, top: t.space.xl, right: t.space.xl, bottom: t.space.lg },
    background: { token: "panel" },
    radius: t.radius.panel,
  }, () => {
    og.ui.label("Title", "UI kit", { step: "displayMd", font: "display" });
    og.ui.scrollArea("Screens", { width: grow, height: grow, gap, fade: Math.round(24 * ts), padding: { left: 0, top: 0, right: 0, bottom: 0 } }, () => {
      og.ui.label("Subtitle", "Templates and widgets for game screens, themed by the platform tokens.", { step: "base", color: { token: "inkMuted" }, width: grow });
      filler("Lead", first > 0 ? pos[first]! - head - 2 * gap : 0);
      // The genre showcases first (each opens on its first page), then every screen of the kit.
      for (let i = first; i <= last; i++) {
        const row = i < n ? showcaseRow(SHOWCASES[i]!) : { name: SCREENS[i - n + 1]!, blurb: BLURBS[SCREENS[i - n + 1]!] };
        // A list row's look: clear at rest, control under the pointer and the focus, controlStrong held.
        const fill = rows.held === i ? "controlStrong" : rows.hot === i ? "control" : null;
        og.ui.open(og.ui.idi("Row", i), {
          direction: "column",
          width: grow,
          padding: { left: t.space.md, top: t.space.sm, right: t.space.md, bottom: t.space.sm },
          interactive: true,
          focusable: true,
          radius: t.radius.control,
          background: fill ? { token: fill } : null,
        });
        og.ui.label("Name", row.name, { step: "base", width: grow, font: i < n ? "display" : "body" });
        og.ui.label("Blurb", row.blurb, { step: "base", color: { token: "inkMuted" }, width: grow });
        const r = og.ui.close();
        if (r.hovered || r.focused) rows.hot = i;
        else if (rows.hot === i) rows.hot = -1;
        if (r.pressed) rows.held = i;
        else if (rows.held === i) rows.held = -1;
        if (r.clicked) d.screen = i < n ? screenIndex(SHOWCASES[i]!.pages[0]!.screen) : i - n + 1;
      }
      filler("Rest", last < count - 1 ? pos[count - 1]! + h(count - 1) - pos[last]! - h(last) - gap : 0);
    });
  });
}

/** The most characters the text screen's field and area take. */
const NAME_CHARS = 24;
const VERSE_CHARS = 16;
const NOTES_CHARS = 256;

/**
 * The text screen's field and area values: world state like every other
 * setting (snapshots, replays and `get:Typed` carry them). A str field holds
 * bytes of UTF-8, at most four a character, so maxChars × 4 always fits.
 */
export const Typed = defineResource("Typed", {
  name: str(NAME_CHARS * 4, "", { label: "Name" }),
  notes: str(NOTES_CHARS * 4, "", { label: "Notes" }),
  verse: str(VERSE_CHARS * 4, "曲径通幽处", { label: "Verse" }),
  reply: str(VERSE_CHARS * 4, "", { label: "Reply" }),
});

/** Type and scripts: the display and rounded faces kerned, labels cut with
 * an ellipsis, Arabic, Hebrew and Devanagari (shaped, in bidi order, a run
 * of Arabic inside a Latin span), a text field and a text area. */
function textScreen(world: World): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  roomPanel("Type", 640, { x: t.space.xl, y: t.space.lg }, t.space.sm, () => {
    og.ui.label("Title", "Type and scripts", { step: "displaySm", font: "display", heading: true });
    og.ui.row("Faces", { gap: t.space.lg, width: grow, cross: "center" }, () => {
      og.ui.label("Kerned", "AVATAR · Toyota · WAVE", { step: "xl", font: "display", nowrap: true });
      og.ui.label("Score", "Level 12 · 48 350", { step: "xl", font: "rounded", nowrap: true });
    });
    og.ui.label("Cut", "A sentence far too long for its one line ends in an ellipsis where it meets the edge, instead of running out of its box", { nowrap: true, ellipsis: true, width: grow });
    og.ui.label("Clamp", "A paragraph clamped to two lines keeps what fits on them and ends the second with an ellipsis where the rest is left out: this sentence, and the one after it, go on well past the two lines the label shows, so the cut is easy to see.", { ellipsis: true, maxLines: 2, width: grow, color: { token: "inkMuted" } });
    og.ui.label("Arabic", "Level 1-2: مغامرة صغيرة!", { width: grow });
    og.ui.richText("Mixed", "Press [b]العب[/b] for [color=primary]مغامرة صغيرة 2[/color] now", { width: grow });
    og.ui.row("Scripts", { gap: t.space.lg, width: grow }, () => {
      og.ui.label("Hebrew", "שלום עולם", { nowrap: true });
      og.ui.label("Devanagari", "नमस्ते दुनिया", { nowrap: true });
      og.ui.label("Hindi", "हिन्दी भाषा", { nowrap: true });
    });
    const typed = world.resource(Typed);
    if (!typed.ok) return;
    // The 大观园 page: an inscription is a line in a box the page draws itself, its field tight to the line (no inset,
    // the line's height) and bare (the box is its frame), dressed as Verse.
    for (const [id, key, hint] of [["Verse", "verse", "题一匾额"], ["Reply", "reply", "续一句"]] as const) {
      // The page's own box, not the field's, dressed as a field's frame: a pack's ornament where it dresses
      // Field.frame, else this ground and rule (the default style has no field frame to draw).
      const box = { left: t.space.md, top: t.space.sm, right: t.space.md, bottom: t.space.sm };
      og.ui.row(`${id}Box`, { width: grow, padding: box, part: "Field.frame", partClass: "field" }, () => {
        const r = t.radius.control;
        og.ui.setDraw({ shape: { kind: "rect", radii: [r, r, r, r] }, fills: [{ kind: "solid", color: { token: "well" } }], strokes: [{ width: 1.5, align: "inside", paint: { kind: "solid", color: { token: "lineStrong" } } }] });
        const v = og.ui.textField(id, typed.value[key], { placeholder: hint, width: grow, maxChars: VERSE_CHARS, bare: true, tight: true, component: "Verse" }).value;
        if (v !== typed.value[key]) typed.value[key] = v;
      });
    }
    const name = og.ui.textField("Name", typed.value.name, { placeholder: "Your name", width: grow, maxChars: NAME_CHARS }).value;
    if (name !== typed.value.name) typed.value.name = name;
    const notes = og.ui.textArea("Notes", typed.value.notes, { placeholder: "Notes", rows: 3, width: grow, maxChars: NOTES_CHARS }).value;
    if (notes !== typed.value.notes) typed.value.notes = notes;
  });
}

/** iconoir icons the materials screen shows (assets/icons/iconoir.ogt names 64). */
const ICONOIR = [
  "heart", "star", "coins", "trophy", "medal", "crown", "gift", "shield",
  "flash", "fire-flame", "potion", "magic-wand", "sparks", "flask", "cube", "puzzle",
  "home", "settings", "play", "pause", "refresh", "search", "lock", "key",
  "map", "compass", "clock", "hourglass", "gamepad", "dice-five", "rocket", "cart",
];

/** Materials: the iconoir set (distance-field glyphs, in rich text too),
 * the dark surfaces' edge light, a picture that keeps its aspect ratio, a
 * map two fingers pinch and pan, and a quest list two fingers scroll. */
function materialsScreen(d: DemoState): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  roomPanel("Materials", 900, { x: t.space.xl, y: t.space.lg }, t.space.md, () => {
    og.ui.label("Title", "Icons, pictures and gestures", { step: "xl", font: "display", heading: true });
    og.ui.row("Icons", { gap: t.space.sm, wrap: true, width: grow }, () => {
      ICONOIR.slice(0, phone ? 16 : 32).forEach((name, i) => og.ui.icon(og.ui.idi("Icon", i), name, { size: i % 8 === 0 ? 32 : 24, tint: { token: i % 8 === 0 ? "primary" : "inkMuted" } }));
    });
    og.ui.richText("Rich", "Collect [icon=coins] coins and [icon=heart] hearts, then [b]claim[/b] the [icon=trophy] trophy.", { width: grow });
    // On a phone the two pictures share a line and the list takes the next.
    const inner = t.sizes.gutter;
    const half = Math.floor((vp.width - 2 * inner - 2 * t.space.xl - t.space.md) / 2);
    og.ui.row("Pictures", { gap: t.space.md, width: grow, wrap: phone }, () => {
      og.ui.column("Left", { width: phone ? { kind: "fixed", px: half } : grow, gap: t.space.xs }, () => {
        og.ui.label("PictureLabel", "A 16:9 picture", { step: "meta", color: { token: "inkMuted" } });
        og.ui.image("Picture", { texture: d.backdrop ? (d.backdrop as never) : null, width: grow, aspect: 16 / 9 });
      });
      og.ui.column("Middle", { width: phone ? { kind: "fixed", px: half } : grow, gap: t.space.xs }, () => {
        og.ui.label("MapLabel", `Pinch or wheel: ${Math.round(d.zoom * 100)}%`, { step: "meta", color: { token: "inkMuted" } });
        const z = og.ui.zoomView("Map", { texture: d.backdrop ? (d.backdrop as never) : null, width: grow, aspect: 16 / 9, maxZoom: 4 });
        d.zoom = z.zoom;
      });
      og.ui.column("Right", { width: phone ? { kind: "fixed", px: 2 * half + t.space.md } : { kind: "fixed", px: 200 }, gap: t.space.xs }, () => {
        og.ui.label("QuestsLabel", "Two fingers scroll", { step: "meta", color: { token: "inkMuted" } });
        og.ui.scroll("Quests", { width: grow, height: { kind: "fixed", px: phone ? 220 : 200 }, gap: t.space.xs, padding: { left: 0, top: 0, right: t.space.md, bottom: 0 } }, () => {
          for (let i = 0; i < 12; i++) {
            if (og.ui.button(og.ui.idi("Quest", i), `Quest ${i + 1}`, { width: grow, size: "small" }).clicked) d.quests += 1;
          }
        });
      });
    });
  });
}

/** The genre patterns' preview screens (patterns.ts). */
const PATTERN_SCREENS: ReadonlySet<string> = new Set(["gear", "cards", "rewards", "gacha", "saga", "board", "generators", "store", "pass", "dialogue", "vn", "quests", "lobby", "select", "versus", "draft", "worldMap", "songs", "garage", "ledger", "cityMap", "chat", "rts", "photo", "loadout", "deck", "cgGallery", "routes", "settingsTree"]);
/** The starship screens (starship.ts): over the style's backdrop, the Gallery button in a pattern screen's corner on a phone. */
const STARSHIP_SCREENS: ReadonlySet<string> = new Set(["shipStatus", "turnOrder", "abilityCards", "sectorMap", "hangar"]);

/** The Gallery button over every screen. */
function galleryButton(world: World, d: DemoState): void {
  const bp = og.ui.breakpoint();
  const vp = og.ui.frameViewport();
  // On a phone the bottom corners belong to the touch controls; its top middle is free.
  const phone = bp === "phonePortrait" || bp === "phoneLandscape";
  const top = phone || SCREENS[d.screen] === "rts" || (SCREENS[d.screen] === "cgGallery" && cgViewing());
  // On the RTS console the bottom is the console's and a phone's top end corner its minimap: on a desktop the button
  // stands at the top middle (as on the shooter), on a phone at the start edge above its middle, clear of the map, the
  // targeting prompt and the console.
  const rts = SCREENS[d.screen] === "rts";
  // The genre patterns' screens lay their header round it: on a phone it takes the top end corner, as a shipped game's
  // menu button does (the start corner is the web shell's Back).
  const corner = phone && !rts && (PATTERN_SCREENS.has(SCREENS[d.screen]!) || STARSHIP_SCREENS.has(SCREENS[d.screen]!));
  // On a versus screen the corners carry the fighters' names and the Skip: the button stands at the top middle over the
  // seam on a desktop. A phone's seam carries the mark and the stage's name, so there it takes the top end corner as on
  // the other pattern screens, and the versus stands under it (genres.ts).
  const versus = SCREENS[d.screen] === "versus" && !phone;
  // A tablet's or a desktop's top edge is the HUD's (timers, wallets, scores), so where touch controls took the
  // bottom-left corner last tick, the button stands mid-way down the left edge.
  const taken = !top && ["TouchMove", "Steer", "Story.Bottom"].some((p) => og.ui.bounds(p) !== null);
  // Elsewhere it sits above a docked action bar where the bar reaches its corner (under a finger the bar spans the
  // bottom edge; at large text a desktop's hints wrap and run to the start edge): last tick's places of the bar and of
  // the chrome, when the screen has one.
  const bar = og.ui.bounds("Hints");
  const was = og.ui.bounds("Back");
  const reach = (was ? was.width : 240) + 2 * og.ui.frameTheme().space.lg;
  const lift = !top && bar && bar.y + bar.height >= vp.height - vp.safeArea.bottom - og.ui.frameTheme().space.xl && bar.x < reach ? vp.height - bar.y : 0;
  // A showcase's document page takes the chrome in its Kit slot (showcases.ts); a HUD page names its free spot.
  if (chromeInDocument(SCREENS[d.screen]!)) return;
  const own = chromeAnchor(SCREENS[d.screen]!);
  // On an edge's middle the chrome stands as a column, so it reaches no further into the HUD than its widest button (a
  // row on a phone on its side, whose height is the scarce one).
  const column = (own === "left" || own === "right") && bp !== "phoneLandscape";
  // On an edge's middle it slides along the edge into the run the HUD leaves free (chromeSlide); where that edge has
  // none (large text fills it), it takes the opposite edge's, held for the look.
  let edge = own;
  let slide = 0;
  if (own && column && was) {
    const look = `${SCREENS[d.screen]}|${vp.width}x${vp.height}|${og.ui.frameTheme().textScale ?? 1}|${og.ui.frameTheme().style}`;
    const other = own === "left" ? "right" : "left";
    if (chromeFlip !== look) {
      const here = chromeSlide(SCREENS[d.screen]!, was);
      const there = here === null ? chromeSlide(SCREENS[d.screen]!, { ...was, x: vp.width - was.x - was.width }) : null;
      if (there !== null) chromeFlip = look;
      else slide = here ?? 0;
    }
    if (chromeFlip === look) {
      edge = other;
      slide = chromeSlide(SCREENS[d.screen]!, was) ?? 0;
    }
  }
  og.ui.anchor("Back", edge ? { anchor: edge, gap: 8, cross: "center", offsetY: slide, ...(column ? { direction: "column" as const } : {}) } : {
    anchor: versus || (rts && phone) ? (phone ? "left" : "top") : corner ? "topRight" : top ? "top" : taken ? "left" : "bottomLeft",
    offsetY: versus ? 0 : rts && phone ? -Math.round(vp.height * 0.12) : corner ? 0 : bp === "phonePortrait" ? 52 : taken ? 0 : -lift,
    gap: 8,
    cross: "center",
  }, () => chromeButtons(world, d));
}

/** The look whose HUD page took its chrome to the opposite edge (chromeSlide found no run on its own). */
let chromeFlip = "";

/** The Gallery button and, on a showcase's page, its next page beside it. */
function chromeButtons(world: World, d: DemoState): void {
  const label = tr(world, "gallery.button");
  const compact = compactChrome(label, SCREENS[d.screen]!);
  if (compact) {
    // Icon buttons at the touch size, their words their accessible names: a label is never cut. The gallery's grid
    // (never a house: Gallery is not home); og-ui draws it in a style's set without one.
    if (og.ui.iconButton("Gallery", "view-grid", { variant: "secondary", label }).clicked) d.screen = 0;
  } else if (og.ui.pushButton("Gallery", label, { variant: "secondary", width: { kind: "fit" } }).clicked) d.screen = 0;
  // The same button as the next page beside it (showcaseNext), so the two read as one pair: one size, one label.
  showcaseNext((next) => (d.screen = screenIndex(next)), SCREENS[d.screen]!, compact);
}

/** The phone chrome's form for each look (compactChrome). */
let chromeLook = { look: "", compact: false };

/**
 * Whether the phone chrome (the Back anchor) stands as icon buttons: measured from its row as last laid. Its words
 * squeeze to fit only once the row spans the whole room between the safe margins, so a row that reached both margins
 * had its labels cut, and the chrome turns to icons for that look (screen, text size, words, width, button step).
 * Words, never ellipsized, on any other look.
 */
function compactChrome(label: string, screen: string): boolean {
  const vp = og.ui.frameViewport();
  // A HUD page's chrome keeps a small footprint at large text on any surface, and under a finger (off a desktop) at any
  // size: the HUD is the frame, and the words' plate would stand over its widgets. Gallery and the next page are then
  // one pair of square icon buttons, their words their accessible names (showcaseNext's square button beside it).
  if (chromeAnchor(screen) && ((og.ui.frameTheme().textScale ?? 1) >= 1.5 || vp.breakpoint !== "desktop")) return true;
  if (vp.breakpoint !== "phonePortrait" && vp.breakpoint !== "phoneLandscape") return false;
  const t = og.ui.frameTheme();
  // A document's Kit slot stands in the room its page left at its own size (main-menu's 64 px above its logo), which
  // the text scale does not grow: at 150 % the words' plate stood over the logo, so the chrome there is icons too.
  if ((t.textScale ?? 1) >= 1.5 && og.ui.bounds("Kit")) return true;
  const look = `${screen}|${label}|${t.textScale ?? 1}|${Math.round(vp.width)}|${t.typeRamp["button"]?.size ?? 0}`;
  if (chromeLook.look !== look) chromeLook = { look, compact: false };
  else if (!chromeLook.compact) {
    const row = og.ui.bounds("Back");
    // Both its ends at an anchor's margin (the row is laid inside the margins, which are no wider than xl).
    if (row && Math.max(row.x, vp.width - row.x - row.width) <= og.ui.frameTheme().space.xl) chromeLook.compact = true;
    // On a document page the chrome is a row in the document's Kit slot (showcases.ts): a row that fills a slot
    // reaching both margins had its words squeezed (pause's "Next" in pixel on a phone, drawn "[Tütöríál…" under the
    // pseudo-locale). A slot that fits its row (loading's) fills by construction and squeezes nothing.
    const slot = row ? null : og.ui.bounds("Kit");
    const inDoc = row ? null : og.ui.bounds("Kit/Chrome");
    const wide = !!slot && Math.max(slot.x, vp.width - slot.x - slot.width) <= og.ui.frameTheme().space.xl;
    if (slot && inDoc && wide && inDoc.width >= slot.width - 0.5) chromeLook.compact = true;
  }
  return chromeLook.compact;
}

/** The raw widget set on one scrolling panel. */
function widgetsScreen(world: World, d: DemoState): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  // Under a finger the touch controls take the bottom (their stick, d-pad and Hit stand about 260 px tall).
  const touch = phone || vp.breakpoint === "tablet" || vp.coarsePointer;
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  // Under the kit's chrome on a phone, above it on a desktop (its room as laid), and above the touch controls.
  const room = chromeRoom(t.space.sm);
  const free = vp.height - vp.safeArea.top - vp.safeArea.bottom - room.top - room.bottom - 2 * t.space.lg - 32;
  og.ui.anchor("Widgets", {
    anchor: phone ? "top" : "topLeft",
    offsetY: room.top,
    width: phone ? grow : { kind: "fixed", px: 560 },
    direction: "column",
    gap: t.space.md,
    padding: { left: t.space.lg, top: t.space.lg, right: t.space.lg, bottom: t.space.lg },
    background: { token: "panel" },
    radius: t.radius.panel,
  }, () => {
    // A scroll area: the row cut at its bottom fades into the panel, so it reads as more below, not a clip.
    og.ui.scrollArea("Body", { width: grow, height: { kind: "fixed", px: Math.max(120, free - (touch ? 260 : 0)) }, gap: t.space.md, padding: { left: 0, top: 0, right: t.space.md, bottom: 0 }, fadeInto: "panel" }, () => {
      const tabs = og.ui.tabs("Tabs", ["Overview", "Stats", "Gear"], d.tab, { fill: true, width: grow });
      d.tab = tabs.selected;
      og.ui.row("Pickers", { width: grow, gap: t.space.md, cross: "center", wrap: true }, () => {
        d.difficulty = og.ui.select("Difficulty", ["Easy", "Normal", "Hard"], d.difficulty, {}).selected;
        d.amount = og.ui.stepper("Amount", d.amount, { min: 0, max: 9 }).value;
      });
      d.mode = og.ui.radio("Mode", ["Solo", "Co-op", "Versus"], d.mode, { horizontal: true }).selected;
      const checks = og.ui.checkboxList("Options", ["Show hints", "Hard mode"], [d.optA, d.optB]);
      d.optA = checks.values[0] ?? d.optA;
      d.optB = checks.values[1] ?? d.optB;
      og.ui.row("Chips", { gap: t.space.sm, cross: "center", wrap: true, width: grow }, () => {
        ["All", "Weapons", "Armor"].forEach((c, i) => {
          if (og.ui.chip(og.ui.idi("Chip", i), c, { selected: d.chip === i }).clicked) d.chip = i;
        });
        og.ui.badge("New", "New", "primary");
        og.ui.badge("Sale", "-20%", "success");
        og.ui.badge("Rare", "Rare", "info");
      });
      og.ui.divider("Rule", false);
      og.ui.meter("Health", d.hp, 100, { kind: "health", label: `${Math.ceil(d.hp)} / 100` });
      og.ui.meter("Mana", d.mana, 100, { kind: "mana" });
      og.ui.meter("Xp", 620, 1000, { kind: "xp", segments: 5 });
      og.ui.row("Rings", { gap: t.space.lg, cross: "center" }, () => {
        const tick = og.ui.tick();
        og.ui.cooldown("Dash", (tick % 240) / 240, { label: "Q" });
        og.ui.cooldown("Heal", 0, { icon: "potion" });
        og.ui.cooldown("Blast", 0.25, { icon: "bolt" });
        og.ui.minimap("Map", { size: 96, markers: [{ x: 0.5, y: 0.55, player: true }, { x: 0.2, y: 0.3, tone: "danger" }, { x: 0.8, y: 0.7, tone: "success" }], label: "Old mill" });
      });
      og.ui.row("Loading", { width: grow, gap: t.space.md, cross: "center" }, () => {
        og.ui.spinner("Spinner", 0);
        og.ui.skeleton("Skeleton", { lines: 2 });
      });
      og.ui.keyValue("Kv", "Difficulty", ["Easy", "Normal", "Hard"][d.difficulty]!, false);
      og.ui.keyValue("Price", "Price", "1,250", true);
      og.ui.row("Icons", { gap: t.space.sm, wrap: true, width: grow }, () => {
        ICONS.slice(0, 16).forEach((name, i) => og.ui.icon(og.ui.idi("Icon", i), name, { size: 24 }));
      });
    });
  });
  // Touch controls, bottom right.
  og.ui.anchor("Touch", { anchor: "bottomRight", gap: t.space.lg, cross: "end" }, () => {
    const j = og.ui.joystick("Stick", {});
    d.stickX = j.x;
    d.stickY = j.y;
    if (!phone) og.ui.dpad("Dpad", {});
    if (og.ui.actionButton("Hit", "Hit", { primary: true }).clicked) hurt(world, d, 15);
  });
  void world;
}

/** The demo's row as a plain object, from the system's column views. */
function readRow(cols: Record<string, ArrayLike<number>>, i: number): DemoState {
  const d = demoDefaults() as Record<string, number | boolean>;
  for (const k of Object.keys(d)) {
    const v = cols[k]?.[i];
    if (v !== undefined) d[k] = typeof d[k] === "boolean" ? v !== 0 : v;
  }
  return d as unknown as DemoState;
}

function writeRow(cols: Record<string, { [n: number]: number }>, i: number, d: DemoState): void {
  for (const [k, v] of Object.entries(d)) {
    const col = cols[k];
    if (col) col[i] = typeof v === "boolean" ? (v ? 1 : 0) : (v as number);
  }
}

/** A hit this tick, for the Thud system (set and taken within one tick). */
let thud = false;

/**
 * A hit the HUDs show: the health drops and the hit is felt (og-feel §3,
 * §4 and §7; og-audio §1): a short hit stop, a shake, the number floating
 * up from the health bar and a thud on the sfx bus, on the hit's tick.
 */
function hurt(world: World, d: DemoState, n: number): void {
  d.hp = Math.max(0, d.hp - n);
  og.time.hitStop(world, 3);
  og.camera.shake(world, 0.3);
  og.render.floatingText(world, { text: `-${n}`, position: { x: -232, y: 128, z: 5 }, size: 22, color: { r: 1, g: 0.36, b: 0.33 }, rise: 36, spread: 6, lifetime: 0.7, pop: 0.5 });
  thud = true;
}

/** Plays the hit's thud: the only og.AudioSource is the hit voice (spawned at start). */
export const Thud = defineSystem("Thud", { writes: [og.AudioSource], after: ["Kit"] }, (q) => {
  if (!thud) return;
  thud = false;
  for (const a of q.archetypes()) {
    const src = a.column(og.AudioSource);
    for (let i = 0; i < a.count; i++) src.playing[i] = 1;
  }
});

export const Kit = defineSystem("Kit", { reads: [og.Input], writes: [Demo, Typed] }, (q, time, world) => {
  for (const a of q.archetypes()) {
    if (a.count === 0) continue;
    const cols = a.column(Demo) as unknown as Record<string, Float64Array>;
    const d = readRow(cols, 0);
    const before = JSON.stringify(d);
    followSettings(world, time.tick, d);
    kitFrame(world, time.dt, d);
    if (JSON.stringify(d) !== before) writeRow(cols, 0, d);
  }
});

/** One tick of the demo's UI over its state. */
function kitFrame(world: World, dt: number, d: DemoState): void {
  // The last frame's accessibility tree (the a11y playtest compares it with the probe's).
  const tree = og.ui.a11y();
  d.a11yNodes = tree.nodes.length;
  d.a11yFocus = tree.focusOrder.length;
  let screen: Screen = SCREENS[d.screen] ?? "gallery";
  if ((screen === "materials") !== d.iconoir) {
    d.iconoir = screen === "materials";
    applyTheme(world, d);
  }
  const wipe = WIPES[d.wipe] ?? "none";
  if (wipe !== "none") {
    // Screen changes cover the old screen, switch, and reveal the new one.
    screen = og.ui.screenTransition("Screens", screen, { kind: wipe, durationMs: 700 }).shown as Screen;
  }
  // The game behind every screen but the gallery and the widget sheet: the style's backdrop (STYLE_BACKDROPS), else a dusk landscape.
  // The genre patterns' screens paint their own scene (patterns.ts): a backdrop would clash with their art.
  if (packGround()) gameScene(world, "scenes/title");
  else if (PATTERN_SCREENS.has(screen)) patternScene(screen);
  const backdrop = kitArt(STYLE_BACKDROPS[STYLES[d.style] ?? "default"] ?? "neutral") ?? kitArt("neutral");
  // The racing HUD draws over its own engine-rendered circuit (src/raceScene.ts).
  if (!packGround() && screen !== "gallery" && screen !== "widgets" && screen !== "raceHud" && !PATTERN_SCREENS.has(screen) && backdrop) og.render.sprite(world, { texture: backdrop, width: 640, height: 360, layer: -10 });

  // The gacha banner runs its art full-bleed on a phone: the Gallery button floats on it, so it is built after it.
  // So does a conversation's: its caption shade and its press layer span the screen, and the button stands over them.
  const floats = screen === "gacha" || screen === "saga" || screen === "dialogue" || screen === "vn";
  // The showcases' documents this screen does not mount leave the layer stack first.
  syncDocs(world, screen);
  if (screen !== "gallery" && !floats && !(screen === "photo" && photoHidden())) galleryButton(world, d);
  let back = false;
  switch (screen) {
    case "gallery":
      gallery(d);
      break;
    case "widgets":
      widgetsScreen(world, d);
      back = og.ui.back();
      break;
    case "players":
      playersScreen(d);
      back = og.ui.back();
      break;
    case "text":
      textScreen(world);
      back = og.ui.back();
      break;
    case "materials":
      // Its icons are the iconoir set: configured first, drawn from the next tick.
      if (d.iconoir) materialsScreen(d);
      back = og.ui.back();
      break;
    case "containers":
      containersScreen();
      back = og.ui.back();
      break;
    case "navigation":
      navigationScreen();
      back = og.ui.back();
      break;
    case "overlays":
      // Back closes the open overlay first (its focus scope takes it).
      overlaysScreen();
      break;
    case "data":
      dataScreen();
      back = og.ui.back();
      break;
    case "charts":
      chartsScreen();
      back = og.ui.back();
      break;
    case "chartsMore":
      chartsScreen(true);
      back = og.ui.back();
      break;
    case "graph":
      graphScreen();
      back = og.ui.back();
      break;
    case "foundations":
      foundationsScreen(d.backdrop ? (d.backdrop as never) : null);
      back = og.ui.back();
      break;
    case "controls":
      controlsScreen();
      back = og.ui.back();
      break;
    case "ranges":
      rangesScreen();
      back = og.ui.back();
      break;
    case "fields":
      // Back closes an open list or the search's suggestions first (their focus scopes take it).
      fieldsScreen();
      break;
    case "bindings":
      // Back cancels a capture or closes the keyboard first.
      bindingsScreen(world);
      break;
    case "gear":
      // Back puts a carried item back first (the grid takes it).
      gearScreen();
      break;
    case "cards":
      // Back stops aiming first (the hand takes it).
      cardsScreen(world);
      break;
    case "rewards":
      // Back disarms banishing first (the choice takes it).
      rewardsScreen();
      back = og.ui.back();
      break;
    case "saga":
      sagaScreen();
      back = og.ui.back();
      break;
    case "board":
      boardScreen();
      back = og.ui.back();
      break;
    case "generators":
      // Back lets go of a held Ascend first (the hold takes it).
      generatorsScreen();
      back = og.ui.back();
      break;
    case "store":
      // Back closes the purchase's confirm first (the modal takes it).
      storeScreen();
      back = og.ui.back();
      break;
    case "pass":
      passScreen();
      back = og.ui.back();
      break;
    case "dialogue":
      // Back is the conversation's Skip, and closes its backlog first.
      dialogueScreen(world);
      break;
    case "vn":
      vnScreen(world);
      break;
    case "quests":
      // Back leaves an open quest for the list first on a phone (the log takes it).
      questsScreen(world);
      back = og.ui.back();
      break;
    case "lobby":
      // Back declines a ready check first (the lobby takes it).
      lobbyScreen();
      back = og.ui.back();
      break;
    case "select":
      // Back unlocks a locked-in pick first (the select takes it).
      charactersScreen();
      back = og.ui.back();
      break;
    case "versus":
      // Back is the versus screen's skip.
      versusScreen();
      break;
    case "draft":
      draftScreen();
      back = og.ui.back();
      break;
    case "worldMap":
      // Back closes a place's detail first (the map takes it).
      mapScreen();
      back = og.ui.back();
      break;
    case "songs":
      songsScreen();
      back = og.ui.back();
      break;
    case "garage":
      garageScreen();
      back = og.ui.back();
      break;
    case "ledger":
      ledgerScreen();
      back = og.ui.back();
      break;
    case "cityMap":
      cityMapScreen();
      back = og.ui.back();
      break;
    case "chat":
      chatScreen();
      back = og.ui.back();
      break;
    case "settingsTree":
      // Back reverts a display change first (the countdown takes it).
      {
        // Apply is the save: the tree's values go to og.settings with the fields a relaunch reads (followSettings).
        const t = settingsTreeScreen(world);
        if (t) {
          const v = t.values;
          const saved: Partial<KitSettings> & { tree: Record<string, number> } = {
            masterVolume: v.master ?? 1, musicVolume: v.music ?? 0.8, sfxVolume: v.effects ?? 1, muted: (v.master ?? 1) <= 0,
            subtitles: (v.subtitles ?? 1) > 0, highContrast: (v.contrast ?? 0) > 0, reducedMotion: (v.reduceMotion ?? 0) > 0,
            invertY: (v.invert ?? 0) > 0, sensitivity: v.sensitivity ?? 0.5, rawMouse: (v.raw ?? 1) > 0, language: t.language,
            quality: t.display.quality ?? "medium", display: t.display, tree: v,
          };
          takeSettings(world, d, saved, false);
          const req = og.platform.saveSettings(world, saved);
          if (!req.ok) throw new Error(req.reason);
          d.saves += 1;
          og.ui.toast(world, L("set.saved"), { tone: "success" });
        }
      }
      back = og.ui.back();
      break;
    case "cgGallery":
      // Back closes the viewer first (the gallery takes it).
      cgGalleryScreen();
      back = og.ui.back();
      break;
    case "routes":
      routeChartScreen();
      back = og.ui.back();
      break;
    case "deck":
      deckScreen();
      back = og.ui.back();
      break;
    case "loadout":
      loadoutScreen();
      back = og.ui.back();
      break;
    case "photo":
      // Back brings hidden controls back first (photo mode takes it); Exit leaves.
      back = photoScreen() || og.ui.back();
      break;
    case "rts":
      // Back cancels a targeting order or leaves the Build page first (the command layer takes it).
      rtsScreen();
      back = og.ui.back();
      break;
    case "gacha":
      // Back skips or closes a pull first (the sequence takes it).
      gachaScreen(d.backdrop as number);
      back = og.ui.back();
      break;
    case "shipStatus":
      shipStatusScreen();
      back = og.ui.back();
      break;
    case "turnOrder":
      turnOrderScreen();
      back = og.ui.back();
      break;
    case "abilityCards":
      abilityCardsScreen();
      back = og.ui.back();
      break;
    case "sectorMap":
      sectorMapScreen();
      back = og.ui.back();
      break;
    case "hangar":
      hangarScreen(world);
      back = og.ui.back();
      break;
    default:
      showcaseScreen(world, screen, (next) => (d.screen = screenIndex(next)), () => chromeButtons(world, d));
      break;
    case "toasts": {
      const t = og.ui.frameTheme();
      roomPanel("ToastDemo", 360, { x: t.space.lg, y: t.space.lg }, t.space.sm, () => {
        og.ui.label("Title", "Toasts", { step: "xl" });
        // Each message a line of words that wraps at large text, its own button beside it (a button's label never
        // wraps, so at 200 % on a phone the words would not fit one).
        for (const [tone, text] of [["neutral", "Checkpoint reached"], ["success", "Achievement unlocked"], ["warn", "Low battery"], ["danger", "Connection lost"], ["info", "New version available"]] as const) {
          og.ui.row(tone, { width: { kind: "grow" }, gap: t.space.sm, cross: "center" }, () => {
            og.ui.label("Text", text, { width: { kind: "grow" } });
            if (og.ui.iconButton("Show", "play", { variant: "secondary", label: `Show “${text}”` }).clicked) og.ui.toast(world, text, { tone });
          });
        }
      });
      back = og.ui.back();
      break;
    }
  }
  // A label of three scripts at once (`mixed:on`): each run in its own face
  // (the role's Latin, a locale's CJK, the Arabic shaped), nothing missing.
  if (d.mixed && (screen === "actionTitle" || screen === "actionPause")) {
    og.ui.anchor("Mixed", { anchor: "bottom", direction: "column", padding: { left: 0, top: 0, right: 0, bottom: 24 } }, () => {
      og.ui.label("Text", tr(world, "menu.mixed"), { step: "lg", nowrap: true });
    });
  }
  if (floats) galleryButton(world, d);
  if (back) d.screen = 0;
}

/** The viewports `viewport:<name>` previews: CSS size, DPR, safe area, pointer. */
const VIEWPORTS: Record<string, Viewport> = {
  desktop: { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" },
  tablet: { width: 820, height: 1180, dpr: 2, safeArea: { left: 0, top: 24, right: 0, bottom: 20 }, coarsePointer: true, breakpoint: "tablet" },
  phone: { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" },
  landscape: { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" },
};

function update(world: World, patch: Partial<DemoState>): void {
  const e = demoEntity(world);
  if (e !== null) world.set(e, Demo, patch);
}

function current(world: World): DemoState | null {
  const e = demoEntity(world);
  const got = e === null ? null : world.get(e, Demo);
  return got && got.ok ? ({ ...got.value } as DemoState) : null;
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Kit, Thud, RaceView],
  resources: [Typed],
  render: { antialias: "msaa4" },
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) {
      camera.value.orthographic = true;
      camera.value.height = 360;
      camera.value.position = { x: 0, y: 0, z: 10 };
      camera.value.quaternion = { x: 0, y: 0, z: 0, w: 1 };
      camera.value.background = { r: 0.0, g: 0.0, b: 0.0 };
      camera.value.exposure = 1;
    }
    const settings = world.resource(og.RenderSettings);
    if (settings.ok) {
      settings.value.toneMapped = false;
      settings.value.environmentIntensity = 0;
      settings.value.sky = false;
    }
    // The racing showcase's circuit, parked until its HUD shows (src/raceScene.ts).
    buildRaceScene(world);
    const e = world.spawn({ name: "demo" });
    if (!e.ok) throw new Error(e.reason);
    // A hit's thud (og sfx's hit preset), on the sfx bus.
    const sound = og.load.sound(world, "audio/hit.ogt");
    const voice = world.spawn({ name: "hit" });
    if (sound.ok && voice.ok) world.insert(voice.value, og.AudioSource, { sound: sound.value, bus: "sfx", playing: false, volume: 0.5 });
    // Every locale's strings; the active one is the host's (the browser's language, og run --locale).
    for (const l of LOCALES) {
      const loaded = og.i18n.load(world, `strings/${l}.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
    }
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    const icons = og.load.texture(world, "textures/icons.png");
    const iconSet = og.load.font(world, "icons/iconoir.ogt");
    const backdrop = og.load.texture(world, "textures/backdrop.png");
    loadPatternArt(world);
    loadStarshipArt(world);
    loadStory(world);
    defineGenres2Actions(world);
    defineGenres3Actions(world);
    defineHudActions(world);
    loadShowcases(world);
    for (const [key, path] of Object.entries(ART_PATHS)) {
      const tex = og.load.texture(world, path);
      KIT_ART[key] = tex.ok ? (tex.value as unknown as number) : 0;
    }
    const handle = (r: { ok: boolean; value?: unknown }): number => (r.ok ? (r.value as number) : 0);
    const face = (path: string) => handle(og.load.font(world, path));
    world.insert(e.value, Demo, {
      font: handle(font),
      displayFont: face("fonts/Inter-Display.ttf"),
      roundedFont: face("fonts/Nunito-Rounded.ttf"),
      hebrewFont: face("fonts/NotoSansHebrew-subset.ttf"),
      devanagariFont: face("fonts/NotoSansDevanagari-subset.ttf"),
      icons: handle(icons),
      iconSet: handle(iconSet),
      backdrop: handle(backdrop),
    });
    const d = current(world);
    if (d) applyTheme(world, d);
  },
  commands: {
    show: (world, args) => update(world, { screen: screenIndex(args[0] ?? "gallery"), paused: false, line: 0, step: 0 }),
    viewport: (world, args) => og.ui.previewViewport(world, VIEWPORTS[args[0] ?? "desktop"] ?? null),
    pad: (world, args) => og.ui.navigate(world, (args[0] ?? "activate") as never),
    toast: (world, args) => {
      og.ui.toast(world, args.join(":") || "Hello", {});
    },
    damage: (world, args) => {
      const d = current(world);
      if (!d) return;
      hurt(world, d, Number(args[0] ?? 10));
      update(world, { hp: d.hp });
    },
    style: (world, args) => {
      update(world, { style: Math.max(0, STYLES.indexOf((args[0] ?? "default") as (typeof STYLES)[number])) });
      const d = current(world);
      if (d) applyTheme(world, d);
    },
    transition: (world, args) => update(world, { wipe: Math.max(0, WIPES.indexOf((args[0] ?? "none") as (typeof WIPES)[number])) }),
    sounds: (world, args) => {
      update(world, { sounds: args[0] !== "off" });
      const d = current(world);
      if (d) applyTheme(world, d);
    },
    heal: (world, args) => {
      const d = current(world);
      if (d) update(world, { hp: Math.min(100, d.hp + Number(args[0] ?? 10)) });
    },
    mixed: (world, args) => update(world, { mixed: args[0] !== "off" }),
    offline: (world, args) => update(world, { offline: args[0] !== "off" }),
    demo: (world, args) => {
      if (!controlsDemo(world, args)) demoCommand(world, args);
    },
    sc: (_world, args) => {
      showcaseCommand(args);
    },
    pat: (world, args) => {
      if (args[0] !== "ship" || !starshipCommand(args[1] ?? "", args[2] ?? "")) patternsCommand(world, args);
    },
  },
  commandHelp: {
    show: { usage: `<${SCREENS.join("|")}>`, doc: "Open a screen of the kit (its usage lists them all: og ui check reads it)." },
    viewport: { usage: "<desktop|tablet|phone|landscape>", doc: "Lay the UI out for that surface (og.ui.previewViewport)." },
    pad: { usage: "<up|down|left|right|next|prev|activate|back>", doc: "A gamepad intent through og.ui.navigate." },
    toast: { usage: "<text>", doc: "Queue a toast." },
    damage: { usage: "<hp>", doc: "Take hit points off the HUDs' health." },
    heal: { usage: "<hp>", doc: "Give hit points back." },
    style: { usage: `<${STYLES.join("|")}>`, doc: "Dress every screen in one of the kit's look documents." },
    transition: { usage: "<none|fade|wipe|iris|dissolve|slide>", doc: "Send screen changes through og.ui.screenTransition." },
    sounds: { usage: "<on|off>", doc: "Hand the UI's event sounds their blips (on by default)." },
    mixed: { usage: "<on|off>", doc: "Show a label mixing CJK, Latin and Arabic (menu.mixed) under the menu, pause and confirm screens." },
    demo: { usage: "<overlay|data|aim|text|device|reset|keyboard|controls>:<value>", doc: "A component preview's state: demo:overlay:menu (menu, submenu, context, popover, dialog, confirm, sheet, sideSheet, radial, none), demo:data:loading (rows, loading, empty, error), demo:aim:0.7,-0.7 (the radial wheel's stick), demo:text:150 (the player's text size in percent: 100 to 150 % is the bar, docs/ui.md §15.2), demo:device:playstation (the device prompts are drawn for, and the key-binding rows': keyboardMouse, xbox, playstation, switch), demo:keyboard:kana (letters, symbols, numeric, kana, none: the on-screen keyboard on the bindings screen), demo:controls:reset (the controls previews' state)." },
    pat: { usage: "<pick|highlight|select|inspect|turn|energy|draw|picks|rselect|balance|banner|pull|ship|reset>:<value>", doc: "A genre pattern preview's state: pat:pick:helm (an item in the hand), pat:highlight:potion (a tag lit), pat:select:<item>, pat:inspect:bash (a card shown large), pat:turn:off (not the player's turn), pat:energy:1 (energy left), pat:draw:strike (a card drawn), pat:picks:2 (a two-pick reward), pat:rselect:thunder,vigor (rewards selected), pat:balance:100 (the gacha purse), pat:banner:weapon, pat:pull:10 (a pull resolved, its reveal started), pat:lobby:found (the lobby's ready check; ready, search, reset), pat:chars:hover:kael (the select's cursor; lock), pat:vs:progress:0.9, pat:draft:phase:ban (hover, lock), pat:map:select:sela (a place's detail; hide:shop,poi), pat:song:select:phoenix (chart:master, playing:off), pat:garage:category:livery (select:engine.race, layer:base, balance:90000), pat:ledger:tab:year (sort:amount, fund:fire=60, overlay:power), pat:chat:channel:party (draft:hi_@Ri, hud:on, typing:on), pat:rts:page:build (target:patrol, gold:100), pat:photo:tab:filters (set:fov=35, filter:noir, frame:cinema, grid:on, hidden:on), pat:loadout:slot:secondary (tab:2, attachment:optic, remove:muzzle), pat:deck:cost:3 (rarity:rare, kind:unit, search:fire, view:deck, full), pat:vn:view:riverboat (chapter:two, variant:1, route:tower), pat:set:page:controls (pending:scale=0.75, confirm:9), pat:reset; the starship screens' pat:ship:card:<n> (the card selected, -1 none), pat:ship:class:<n> (the hangar's class), pat:ship:reset." },
    sc: { usage: "<reset|tutorial|hud>:<key>:<value>", doc: "A genre showcase's preview state: sc:tutorial:3 (the tutorial's step), sc:hud:hp:20 (hp, coins, combo, ammo, reload:on, hit:crit, speed:2), sc:reset." },
    offline: { usage: "<on|off>", doc: "The leaderboard's rows fail to load: its error message and Retry (which loads again)." },
  },
  facts: (world) => {
    const d = current(world);
    return d ? { screen: SCREENS[d.screen] ?? "gallery", style: STYLES[d.style] ?? "default", hp: d.hp, balance: d.balance, saves: d.saves, loads: d.loads, language: LANGUAGES[d.language]?.code ?? "en", picks: [d.pick0, d.pick1, d.pick2, d.pick3], focus: [d.focus0, d.focus1, d.focus2, d.focus3] } : {};
  },
});
