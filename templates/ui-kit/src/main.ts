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
// damage:<hp>, heal:<hp>, skin:<tokens|pixel|cartoon|fantasy|scifi|paper|neon>
// (a style pack from assets/skins), transition:<none|fade|wipe|iris|dissolve|slide>
// (screen changes go through og.ui.screenTransition), sounds:<on|off> (on by default),
// mixed:<on|off> (a label of CJK, Latin and Arabic under the locale screens),
// offline:<on|off> (the leaderboard's rows fail to load: its error and Retry).

import { og, defineComponent, defineResource, defineSystem, bool, f32, f64, str, u32 } from "@origingame/engine";
import type { Entity, GameSettings, Viewport, World } from "@origingame/engine";

export const SCREENS = [
  "gallery", "widgets", "mainMenu", "pause", "settings", "platformer", "shooter", "puzzle", "idle", "racing",
  "cutscene", "shop", "inventory", "levels", "results", "leaderboard", "tutorial", "loading", "confirm", "alert", "toasts",
  "players", "text", "materials",
] as const;
export type Screen = (typeof SCREENS)[number];

/** The style packs under assets/skins; "tokens" is the platform look. */
export const SKINS = ["tokens", "pixel", "cartoon", "fantasy", "scifi", "paper", "neon"] as const;
const WIPES = ["none", "fade", "wipe", "iris", "dissolve", "slide"] as const;
const UI_SOUNDS = ["hover", "press", "open", "close", "success", "error"] as const;

const BLURBS: Record<Screen, string> = {
  gallery: "",
  widgets: "Every widget: tabs, select, stepper, radio, meters, rings, touch controls",
  mainMenu: "Title screen",
  pause: "Pause over a HUD",
  settings: "Audio, graphics, controls, language",
  platformer: "Lives, coins, score, timer, d-pad and Jump",
  shooter: "Health, armor, ammo, abilities, crosshair",
  puzzle: "Score toward a target, stars, moves",
  idle: "Currencies, tap target, generators",
  racing: "Place, laps, times, speed, boost",
  cutscene: "Letterbox and typed dialogue",
  shop: "Balance, categories, items, Buy",
  inventory: "Slots with drag and drop",
  levels: "Level cards with stars and locks",
  results: "Stars, score count-up, share",
  leaderboard: "Scopes, ranks, loading and empty states",
  tutorial: "Spotlight and callout steps",
  loading: "Real progress and tips",
  confirm: "A destructive confirm",
  alert: "One-button alert",
  toasts: "The toast queue in every tone",
  players: "Local multiplayer: four pads, four rings",
  text: "Kerning, ellipses, scripts, fields and text areas",
  materials: "Icons, edge light, pictures, two-finger scroll and pinch",
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
  muted: bool(),
  quality: u32(1),
  showFps: bool(),
  reducedMotion: bool(),
  highContrast: bool(),
  vibration: bool(true),
  invertY: bool(),
  sensitivity: f32(1),
  language: u32(0),
  saves: u32(0),
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
  inv3: u32(5),
  inv4: u32(9),
  inv5: u32(13),
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
  // Look: the style pack (SKINS index), the screen transition (WIPES index), UI sounds.
  skin: u32(0),
  wipe: u32(0),
  sounds: bool(true),
  // `mixed:on`: a label mixing CJK, Latin and Arabic (menu.mixed) under the locale screens.
  mixed: bool(),
});
type DemoState = { -readonly [K in keyof ReturnType<typeof demoDefaults>]: ReturnType<typeof demoDefaults>[K] };
function demoDefaults() {
  return { screen: 0, paused: false, font: 0, displayFont: 0, roundedFont: 0, hebrewFont: 0, devanagariFont: 0, icons: 0, iconSet: 0, iconoir: false, zoom: 1, quests: 0, backdrop: 0, hp: 72, mana: 40, coins: 128, score: 4250, lives: 2, time: 95, masterVolume: 1, musicVolume: 0.8, sfxVolume: 1, muted: false, quality: 1, showFps: false, reducedMotion: false, highContrast: false, vibration: true, invertY: false, sensitivity: 1, language: 0, saves: 0, tab: 0, difficulty: 1, amount: 3, mode: 0, optA: true, optB: false, chip: 0, stickX: 0, stickY: 0, line: 0, step: 0, balance: 1250, owned: 0, bought: 0, refused: 0, inv0: 0, inv1: 1, inv2: 2, inv3: 5, inv4: 9, inv5: 13, moves: 0, used: 0, gold: 40, gen0: 1, gen1: 0, gen2: 0, taps: 0, scope: 0, scopeTick: 0, confirmed: 0, level: 0, pick0: 0, pick1: 0, pick2: 0, pick3: 0, focus0: 0, focus1: 0, focus2: 0, focus3: 0, rules: 0, a11yNodes: 0, a11yFocus: 0, skin: 0, wipe: 0, sounds: true, mixed: false, offline: false };
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

const SHOP = [
  { id: "sword", name: "Iron sword", price: 300, icon: "sword", category: "Gear", description: "A dependable blade. +4 attack." },
  { id: "shield", name: "Oak shield", price: 220, icon: "shield", category: "Gear", description: "Blocks the first hit of every fight." },
  { id: "crown", name: "Crown", price: 2400, icon: "crown", category: "Gear", description: "Pure vanity, sold once." },
  { id: "potion", name: "Potion", price: 60, icon: "potion", category: "Items", description: "Restores 50 health." },
  { id: "bomb", name: "Bomb", price: 90, icon: "bomb", category: "Items", description: "Clears a cracked wall." },
  { id: "key", name: "Gold key", price: 500, icon: "key", category: "Items", description: "Opens one locked chest." },
  { id: "gem", name: "Gem pack", price: 1000, icon: "gem", category: "Bundles", description: "Twenty gems for the wheel." },
  { id: "map", name: "Treasure map", price: 180, icon: "map", category: "Bundles", description: "Marks every chest of a level." },
];

const ITEMS = [
  { id: "potion", name: "Potion", icon: "potion", count: 3, description: "Restores 50 health." },
  { id: "sword", name: "Iron sword", icon: "sword", count: 1, description: "A dependable blade." },
  { id: "key", name: "Gold key", icon: "key", count: 2, description: "Opens one locked chest." },
  { id: "apple", name: "Apple", icon: "apple", count: 12, description: "A snack. Restores 5 health." },
  { id: "bomb", name: "Bomb", icon: "bomb", count: 4, description: "Clears a cracked wall." },
  { id: "gem", name: "Gem", icon: "gem", count: 20, description: "Spent at the wheel." },
];

const GENERATORS = [
  { id: "farm", name: "Farm", icon: "leaf", base: 15, rate: 1 },
  { id: "mill", name: "Mill", icon: "gear", base: 120, rate: 8 },
  { id: "forge", name: "Forge", icon: "fire", base: 900, rate: 47 },
];

const CUTSCENE = [
  { speaker: "Mira", text: "The gate opens at dawn. Are you ready to cross?", portrait: "person" },
  { speaker: "You", text: "Ready as I will ever be.", portrait: "" },
  { speaker: "Mira", text: "Then take this key. It opens the old vault under the mill.", portrait: "person" },
  { speaker: "Mira", text: "And whatever you hear down there, do not answer it.", portrait: "skull" },
];

const TUTORIAL = [
  { target: "Items/play", title: "Start here", text: "Play starts the first level." },
  { target: "Items/settings", title: "Make it yours", text: "Sound, graphics and controls live in Settings." },
  { target: "Gallery", title: "Back to the gallery", text: "This button, Escape or gamepad B returns." },
];

/** Level cards: 12 levels, stars from a fixed pattern, the last three locked. */
const LEVELS = Array.from({ length: 12 }, (_, i) => ({ name: `${i + 1}`, stars: i < 9 ? [3, 2, 3, 1, 3, 2, 0, 1, 2][i]! : 0, locked: i >= 9 }));

/** The lobby's heroes, and each player's colour (the pad's focus ring). */
const HEROES = ["Knight", "Ranger", "Mage", "Rogue", "Bard", "Monk", "Smith", "Thief"];
const PLAYER_INKS = ["focus", "info", "success", "warn"] as const;

/** A local-multiplayer lobby: up to four pads, each with its own focus ring; A picks a hero for that pad's player. */
function playersScreen(d: DemoState): void {
  og.ui.setPlayers(4);
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  const picks = [d.pick0, d.pick1, d.pick2, d.pick3];
  og.ui.anchor("Lobby", {
    anchor: "center",
    direction: "column",
    gap: t.space.md,
    width: phone ? grow : { kind: "fixed", px: 640 },
    padding: { left: t.space.xl, top: t.space.xl, right: t.space.xl, bottom: t.space.xl },
    background: { token: "surface" },
    radius: t.radius.panel,
  }, () => {
    og.ui.label("Title", "Players", { step: "xl2", font: "display", heading: true });
    // Rich text: the button in bold, a link (a child element Hint/join, focusable by every pad).
    const hint = og.ui.richText("Hint", "Each pad moves its own ring; [b]A[/b] picks a hero for that player. [link=join]How to join[/link]", { color: { token: "fgMuted" }, width: grow });
    if (hint.link === "join") d.rules += 1;
    // The player cards sit on bgSubtle, darker than the panel, so each player's ink and "Press A" read at 4.5:1.
    og.ui.row("Cards", { gap: t.space.sm, width: grow, wrap: phone }, () => {
      for (let p = 0; p < 4; p++) {
        og.ui.column(og.ui.idi("Player", p), { width: grow, gap: t.space.xxs, cross: "center", padding: { left: t.space.sm, top: t.space.sm, right: t.space.sm, bottom: t.space.sm }, background: { token: "bgSubtle" }, radius: t.radius.card }, () => {
          og.ui.label("Name", `P${p + 1}`, { step: "lg", font: "display", color: { token: PLAYER_INKS[p]! }, nowrap: true });
          og.ui.label("Pick", picks[p] ? HEROES[picks[p]! - 1]! : "Press A", { color: { token: picks[p] ? "fg" : "fgMuted" }, nowrap: true });
        });
      }
    });
    og.ui.row("Heroes", { gap: t.space.sm, wrap: true, width: grow }, () => {
      HEROES.forEach((h, i) => {
        const r = og.ui.button(og.ui.idi("Hero", i), h, { width: { kind: "fixed", px: phone ? 150 : 132 } });
        if (r.clicked) picks[r.pad ?? 0] = i + 1;
      });
    });
  });
  [d.pick0, d.pick1, d.pick2, d.pick3] = picks as [number, number, number, number];
  const focused = (pad: number) => Number(/Hero#(\d+)$/.exec(og.ui.focus(pad))?.[1] ?? -1) + 1;
  [d.focus0, d.focus1, d.focus2, d.focus3] = [focused(0), focused(1), focused(2), focused(3)];
}

/** 64 leaderboard rows from a fixed formula; the player is rank 17. */
const NAMES = ["Ash", "Bea", "Cyd", "Dov", "Eli", "Fen", "Gus", "Hal", "Ivy", "Jun", "Kit", "Lux", "Moe", "Nia", "Oz", "Pip"];
function leaderboard(scope: number) {
  return Array.from({ length: 64 }, (_, i) => ({
    rank: i + 1,
    name: i === 16 ? "You" : `${NAMES[(i * 7 + scope * 3) % 16]}${(i * 13 + scope) % 90 + 10}`,
    score: (98_000 - i * 1310 - scope * 4000).toLocaleString("en-US"),
    player: i === 16,
  }));
}

function demoEntity(world: World): Entity | null {
  return world.find("demo");
}

function screenIndex(name: string): number {
  return Math.max(0, SCREENS.indexOf(name as Screen));
}

function settingsOf(d: DemoState): GameSettings {
  return {
    masterVolume: d.masterVolume,
    musicVolume: d.musicVolume,
    sfxVolume: d.sfxVolume,
    muted: d.muted,
    quality: QUALITIES[d.quality] ?? "medium",
    showFps: d.showFps,
    reducedMotion: d.reducedMotion,
    highContrast: d.highContrast,
    vibration: d.vibration,
    invertY: d.invertY,
    sensitivity: d.sensitivity,
    language: LANGUAGES[d.language]?.code ?? "en",
    bindings: [
      { action: "move", label: "Move", key: "A D / ← →" },
      { action: "jump", label: "Jump", key: "Space" },
      { action: "pause", label: "Pause", key: "Escape" },
      { action: "interact", label: "Interact", key: "E" },
    ],
  };
}

/** Applies the theme-affecting settings (contrast, motion, the style pack,
 * sounds) with the demo's font and icons, or the pack's. */
function applyTheme(world: World, d: DemoState): void {
  const name = SKINS[d.skin] ?? "tokens";
  const loaded = name === "tokens" ? null : og.load.skin(world, `skins/${name}/skin.ogt`);
  if (loaded && !loaded.ok) throw new Error(loaded.reason);
  const skin = loaded && loaded.ok ? loaded.value : null;
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
    font: d.font && !skin ? (d.font as never) : null,
    displayFont: d.displayFont && !skin ? (d.displayFont as never) : null,
    roundedFont: d.roundedFont && !skin ? (d.roundedFont as never) : null,
    // Scripts the locale tables' fonts do not cover (the text screen).
    fallbackFonts: [d.hebrewFont, d.devanagariFont].filter((h) => h !== 0) as never[],
    contrast: d.highContrast ? "high" : "standard",
    reducedMotion: d.reducedMotion,
    skin,
    // The materials screen draws from the iconoir set; the others from the kit's grid (or the pack's).
    icons: d.iconoir && d.iconSet ? { font: d.iconSet as never } : skin ? null : d.icons ? { texture: d.icons as never, columns: 8, rows: 4, names: ICONS } : {},
    sounds: d.sounds ? sounds : null,
  });
}

/** The gallery: the list of screens. */
function gallery(d: DemoState): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  og.ui.anchor("Gallery", {
    anchor: "center",
    direction: "column",
    gap: t.space.md,
    width: phone ? { kind: "grow" } : { kind: "fixed", px: 560 },
    padding: { left: t.space.xl, top: t.space.xl, right: t.space.xl, bottom: t.space.lg },
    background: { token: "surface" },
    radius: t.radius.panel,
  }, () => {
    og.ui.label("Title", "UI kit", { step: "displayMd", font: "display" });
    og.ui.label("Subtitle", "Templates and widgets for game screens, themed by the platform tokens.", { color: { token: "fgMuted" }, width: { kind: "grow" } });
    const height = Math.min(phone ? vp.height - 260 : 460, vp.height - 200);
    const r = og.ui.list("Screens", { count: SCREENS.length - 1, rowHeight: 52, height: { kind: "fixed", px: height }, width: { kind: "grow" } }, (i) => {
      const s = SCREENS[i + 1]!;
      og.ui.column("Text", { width: { kind: "grow" }, gap: 0 }, () => {
        og.ui.label("Name", s, { nowrap: true });
        og.ui.label("Blurb", BLURBS[s], { step: "meta", color: { token: "fgMuted" }, nowrap: true });
      });
    });
    if (r.clicked !== null) d.screen = r.clicked + 1;
  });
}

/** The most characters the text screen's field and area take. */
const NAME_CHARS = 24;
const NOTES_CHARS = 256;

/**
 * The text screen's field and area values: world state like every other
 * setting (snapshots, replays and `get:Typed` carry them). A str field holds
 * bytes of UTF-8, at most four a character, so maxChars × 4 always fits.
 */
export const Typed = defineResource("Typed", {
  name: str(NAME_CHARS * 4, "", { label: "Name" }),
  notes: str(NOTES_CHARS * 4, "", { label: "Notes" }),
});

/** Type and scripts: the display and rounded faces kerned, labels cut with
 * an ellipsis, Arabic, Hebrew and Devanagari (shaped, in bidi order, a run
 * of Arabic inside a Latin span), a text field and a text area. */
function textScreen(world: World): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  og.ui.anchor("Type", {
    anchor: "center",
    direction: "column",
    gap: t.space.sm,
    width: phone ? grow : { kind: "fixed", px: 640 },
    padding: { left: t.space.xl, top: t.space.lg, right: t.space.xl, bottom: t.space.lg },
    background: { token: "surface" },
    radius: t.radius.panel,
  }, () => {
    og.ui.label("Title", "Type and scripts", { step: "displaySm", font: "display", heading: true });
    og.ui.row("Faces", { gap: t.space.lg, width: grow, cross: "center" }, () => {
      og.ui.label("Kerned", "AVATAR · Toyota · WAVE", { step: "xl", font: "display", nowrap: true });
      og.ui.label("Score", "Level 12 · 48 350", { step: "xl", font: "rounded", nowrap: true });
    });
    og.ui.label("Cut", "A sentence far too long for its one line ends in an ellipsis where it meets the edge, instead of running out of its box", { nowrap: true, ellipsis: true, width: grow });
    og.ui.label("Clamp", "A paragraph clamped to two lines keeps what fits on them and ends the second with an ellipsis where the rest is left out: this sentence, and the one after it, go on well past the two lines the label shows, so the cut is easy to see.", { ellipsis: true, maxLines: 2, width: grow, color: { token: "fgMuted" } });
    og.ui.label("Arabic", "Level 1-2: مغامرة صغيرة!", { nowrap: true });
    og.ui.richText("Mixed", "Press [b]العب[/b] for [color=accent]مغامرة صغيرة 2[/color] now", { width: grow });
    og.ui.row("Scripts", { gap: t.space.lg, width: grow }, () => {
      og.ui.label("Hebrew", "שלום עולם", { nowrap: true });
      og.ui.label("Devanagari", "नमस्ते दुनिया", { nowrap: true });
      og.ui.label("Hindi", "हिन्दी भाषा", { nowrap: true });
    });
    const typed = world.resource(Typed);
    if (!typed.ok) return;
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
  og.ui.anchor("Materials", {
    anchor: phone ? "top" : "center",
    offsetY: phone ? 100 : 0,
    direction: "column",
    gap: t.space.md,
    width: phone ? grow : { kind: "fixed", px: 900 },
    padding: { left: t.space.xl, top: t.space.lg, right: t.space.xl, bottom: t.space.lg },
    background: { token: "surface" },
    radius: t.radius.panel,
  }, () => {
    og.ui.label("Title", "Icons, pictures and gestures", { step: "xl", font: "display", heading: true });
    og.ui.row("Icons", { gap: t.space.sm, wrap: true, width: grow }, () => {
      ICONOIR.slice(0, phone ? 16 : 32).forEach((name, i) => og.ui.icon(og.ui.idi("Icon", i), name, { size: i % 8 === 0 ? 32 : 24, tint: { token: i % 8 === 0 ? "accent" : "fgMuted" } }));
    });
    og.ui.richText("Rich", "Collect [icon=coins] coins and [icon=heart] hearts, then [b]claim[/b] the [icon=trophy] trophy.", { width: grow });
    // On a phone the two pictures share a line and the list takes the next.
    const inner = t.sizes.gutter;
    const half = Math.floor((vp.width - 2 * inner - 2 * t.space.xl - t.space.md) / 2);
    og.ui.row("Pictures", { gap: t.space.md, width: grow, wrap: phone }, () => {
      og.ui.column("Left", { width: phone ? { kind: "fixed", px: half } : grow, gap: t.space.xs }, () => {
        og.ui.label("PictureLabel", "A 16:9 picture", { step: "meta", color: { token: "fgMuted" } });
        og.ui.image("Picture", { texture: d.backdrop ? (d.backdrop as never) : null, width: grow, aspect: 16 / 9 });
      });
      og.ui.column("Middle", { width: phone ? { kind: "fixed", px: half } : grow, gap: t.space.xs }, () => {
        og.ui.label("MapLabel", `Pinch or wheel: ${Math.round(d.zoom * 100)}%`, { step: "meta", color: { token: "fgMuted" } });
        const z = og.ui.zoomView("Map", { texture: d.backdrop ? (d.backdrop as never) : null, width: grow, aspect: 16 / 9, maxZoom: 4 });
        d.zoom = z.zoom;
      });
      og.ui.column("Right", { width: phone ? { kind: "fixed", px: 2 * half + t.space.md } : { kind: "fixed", px: 200 }, gap: t.space.xs }, () => {
        og.ui.label("QuestsLabel", "Two fingers scroll", { step: "meta", color: { token: "fgMuted" } });
        og.ui.scroll("Quests", { width: grow, height: { kind: "fixed", px: phone ? 220 : 200 }, gap: t.space.xs, padding: { left: 0, top: 0, right: t.space.md, bottom: 0 } }, () => {
          for (let i = 0; i < 12; i++) {
            if (og.ui.button(og.ui.idi("Quest", i), `Quest ${i + 1}`, { width: grow, size: "small" }).clicked) d.quests += 1;
          }
        });
      });
    });
  });
}

/** The Gallery button over every screen. */
function galleryButton(world: World, d: DemoState): void {
  const bp = og.ui.breakpoint();
  // On a phone the bottom corners belong to the touch controls.
  og.ui.anchor("Back", { anchor: bp === "phonePortrait" || bp === "phoneLandscape" ? "top" : "bottomLeft", offsetY: bp === "phonePortrait" ? 52 : 0 }, () => {
    if (og.ui.button("Gallery", tr(world, "gallery.button"), { variant: "secondary", size: "small" }).clicked) d.screen = 0;
  });
}

/** The raw widget set on one scrolling panel. */
function widgetsScreen(world: World, d: DemoState): void {
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  const t = og.ui.frameTheme();
  const grow = { kind: "grow" } as const;
  og.ui.anchor("Widgets", {
    anchor: phone ? "top" : "topLeft",
    width: phone ? grow : { kind: "fixed", px: 560 },
    direction: "column",
    gap: t.space.md,
    padding: { left: t.space.lg, top: t.space.lg, right: t.space.lg, bottom: t.space.lg },
    background: { token: "surface" },
    radius: t.radius.panel,
  }, () => {
    og.ui.scroll("Body", { width: grow, height: { kind: "fixed", px: phone ? vp.height - 380 : vp.height - 120 }, gap: t.space.md, padding: { left: 0, top: 0, right: t.space.md, bottom: 0 } }, () => {
      const tabs = og.ui.tabs("Tabs", ["Overview", "Stats", "Gear"], d.tab, { fill: true, width: grow });
      d.tab = tabs.selected;
      og.ui.row("Pickers", { width: grow, gap: t.space.md, cross: "center" }, () => {
        d.difficulty = og.ui.select("Difficulty", ["Easy", "Normal", "Hard"], d.difficulty, {}).selected;
        d.amount = og.ui.stepper("Amount", d.amount, { min: 0, max: 9 }).value;
      });
      d.mode = og.ui.radio("Mode", ["Solo", "Co-op", "Versus"], d.mode, { horizontal: true }).selected;
      const checks = og.ui.checkboxList("Options", ["Show hints", "Hard mode"], [d.optA, d.optB]);
      d.optA = checks.values[0] ?? d.optA;
      d.optB = checks.values[1] ?? d.optB;
      og.ui.row("Chips", { gap: t.space.sm, cross: "center" }, () => {
        ["All", "Weapons", "Armor"].forEach((c, i) => {
          if (og.ui.chip(og.ui.idi("Chip", i), c, { selected: d.chip === i }).clicked) d.chip = i;
        });
        og.ui.badge("New", "New", "accent");
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
  // The game behind every screen but the gallery and the widget sheet: a dusk landscape.
  if (screen !== "gallery" && screen !== "widgets" && d.backdrop) og.render.sprite(world, { texture: d.backdrop as never, width: 640, height: 360, layer: -10 });

  if (screen !== "gallery" && screen !== "loading") galleryButton(world, d);
  let back = false;
  switch (screen) {
    case "gallery":
      gallery(d);
      break;
    case "widgets":
      widgetsScreen(world, d);
      back = og.ui.back();
      break;
    case "mainMenu":
    case "tutorial": {
      const r = og.ui.mainMenu("Menu", {
        title: tr(world, "menu.title"),
        subtitle: tr(world, "menu.subtitle"),
        items: [
          { id: "play", label: tr(world, "menu.play") },
          { id: "continue", label: tr(world, "menu.continue"), disabled: true },
          { id: "settings", label: tr(world, "menu.settings") },
          { id: "quit", label: tr(world, "menu.quit") },
        ],
        footer: tr(world, "menu.footer", { version: "0.1.0" }),
      });
      // A greeting with characters no locale font cooked (zh-CN and ja): the
      // Chinese font draws them from its outlines at run time (runtime true).
      if (screen === "mainMenu") {
        og.ui.anchor("Greeting", { anchor: "top", direction: "column", padding: { left: 0, top: 24, right: 0, bottom: 0 } }, () => {
          og.ui.label("Text", tr(world, "menu.greeting"), { step: "lg", color: { token: "fgMuted" }, nowrap: true });
        });
      }
      if (r.selected === "play") d.screen = screenIndex("platformer");
      if (r.selected === "settings") d.screen = screenIndex("settings");
      if (r.selected === "quit") d.screen = 0;
      if (screen === "tutorial") {
        const tut = og.ui.tutorialOverlay("Tutorial", {
          open: true,
          steps: TUTORIAL,
          index: d.step,
          // The overlay's own words, from the active table (og-i18n §2).
          skipLabel: tr(world, "tutorial.skip"),
          nextLabel: tr(world, "tutorial.next"),
          doneLabel: tr(world, "tutorial.done"),
          stepLabel: tr(world, "tutorial.step"),
        });
        d.step = tut.index;
        if (tut.finished || tut.skipped) {
          d.step = 0;
          d.screen = 0;
        }
      } else {
        back = og.ui.back();
      }
      break;
    }
    case "pause":
    case "platformer": {
      if (screen === "platformer" && !d.paused) d.time = Math.max(0, d.time - dt);
      const h = og.ui.platformerHud("Hud", { lives: d.lives, maxLives: 3, coins: d.coins, score: d.score, level: tr(world, "hud.level"), timeLeft: d.time, touch: true });
      if (h.paused) d.paused = true;
      if (h.action === "jump") d.coins += 1;
      const p = og.ui.pauseMenu("Pause", {
        open: screen === "pause" || d.paused,
        title: tr(world, "pause.title"),
        items: [
          { id: "resume", label: tr(world, "pause.resume") },
          { id: "settings", label: tr(world, "pause.settings") },
          { id: "quit", label: tr(world, "pause.quit") },
        ],
      });
      if (p.selected === "resume") {
        d.paused = false;
        if (screen === "pause") d.screen = screenIndex("platformer");
      }
      if (p.selected === "settings") d.screen = screenIndex("settings");
      if (p.selected === "quit") {
        d.paused = false;
        d.screen = 0;
      }
      break;
    }
    case "settings": {
      const r = og.ui.settingsScreen("Settings", {
        open: true,
        settings: settingsOf(d),
        languages: LANGUAGES,
        // The screen's own words, from the active table (og-i18n §2).
        title: tr(world, "settings.title"),
        audioLabel: tr(world, "settings.audio"),
        graphicsLabel: tr(world, "settings.graphics"),
        controlsLabel: tr(world, "settings.controls"),
        languageLabel: tr(world, "settings.language"),
        masterLabel: tr(world, "settings.master"),
        musicLabel: tr(world, "settings.music"),
        effectsLabel: tr(world, "settings.effects"),
        muteLabel: tr(world, "settings.mute"),
        qualityLabel: tr(world, "settings.quality"),
        lowLabel: tr(world, "settings.low"),
        mediumLabel: tr(world, "settings.medium"),
        highLabel: tr(world, "settings.high"),
        fpsLabel: tr(world, "settings.fps"),
        motionLabel: tr(world, "settings.motion"),
        contrastLabel: tr(world, "settings.contrast"),
        sensitivityLabel: tr(world, "settings.sensitivity"),
        invertLabel: tr(world, "settings.invert"),
        vibrationLabel: tr(world, "settings.vibration"),
        doneLabel: tr(world, "settings.done"),
        onSave: (saved) => {
          const req = og.platform.saveSettings(world, saved);
          if (!req.ok) throw new Error(req.reason);
          d.saves += 1;
          og.ui.toast(world, "Settings saved", { tone: "success" });
        },
      });
      const s = r.settings;
      d.masterVolume = s.masterVolume ?? d.masterVolume;
      d.musicVolume = s.musicVolume ?? d.musicVolume;
      d.sfxVolume = s.sfxVolume ?? d.sfxVolume;
      d.muted = s.muted ?? d.muted;
      d.quality = Math.max(0, QUALITIES.indexOf(s.quality ?? "medium"));
      d.showFps = s.showFps ?? d.showFps;
      d.vibration = s.vibration ?? d.vibration;
      d.invertY = s.invertY ?? d.invertY;
      d.sensitivity = s.sensitivity ?? d.sensitivity;
      const language = Math.max(0, LANGUAGES.findIndex((l) => l.code === s.language));
      if (language !== d.language) og.i18n.setLocale(world, LANGUAGES[language]!.code);
      d.language = language;
      const theme = d.highContrast !== (s.highContrast ?? false) || d.reducedMotion !== (s.reducedMotion ?? false);
      d.highContrast = s.highContrast ?? d.highContrast;
      d.reducedMotion = s.reducedMotion ?? d.reducedMotion;
      if (theme) applyTheme(world, d);
      if (r.closed) d.screen = 0;
      break;
    }
    case "shooter": {
      const tick = og.ui.tick();
      const h = og.ui.shooterHud("Hud", {
        health: d.hp, maxHealth: 100, armor: 35, maxArmor: 50, ammo: 18, magazine: 30, reserve: 120, weapon: "Carbine", kills: 7,
        abilities: [
          { id: "dash", key: "Q", remaining: (tick % 300) / 300 },
          { id: "heal", icon: "potion", remaining: 0 },
          { id: "blast", icon: "bolt", remaining: 0.6 },
        ],
        map: { markers: [{ x: 0.5, y: 0.5, player: true }, { x: 0.3, y: 0.25, tone: "danger" }, { x: 0.72, y: 0.4, tone: "danger" }], label: "Sector 4" },
        touch: true,
      });
      if (h.paused) d.screen = 0;
      if (h.action === "fire") hurt(world, d, 5);
      break;
    }
    case "puzzle": {
      const h = og.ui.puzzleHud("Hud", { level: "Level 7", score: 680, target: 1000, moves: 12, timeLeft: 48 });
      if (h.paused) d.screen = 0;
      break;
    }
    case "idle": {
      const gens = [d.gen0, d.gen1, d.gen2];
      const rate = GENERATORS.reduce((n, g, i) => n + g.rate * gens[i]!, 0);
      d.gold += rate * dt;
      const cost = (i: number) => Math.round(GENERATORS[i]!.base * Math.pow(1.15, gens[i]!));
      const tick = og.ui.tick();
      const h = og.ui.idleHud("Hud", {
        currencies: [{ id: "gold", name: "Gold", amount: d.gold, perSecond: rate, icon: "coin" }, { id: "gems", name: "Gems", amount: 12, icon: "gem" }],
        generators: GENERATORS.map((g, i) => ({ id: g.id, name: g.name, icon: g.icon, level: gens[i]!, cost: cost(i), affordable: d.gold >= cost(i), progress: gens[i]! > 0 ? ((tick * (i + 1)) % 120) / 120 : 0 })),
        tapLabel: "Tap",
      });
      if (h.action === "tap") {
        d.gold += 1;
        d.taps += 1;
      }
      if (h.action?.startsWith("buy:")) {
        const i = GENERATORS.findIndex((g) => `buy:${g.id}` === h.action);
        if (i >= 0 && d.gold >= cost(i)) {
          d.gold -= cost(i);
          if (i === 0) d.gen0 += 1;
          if (i === 1) d.gen1 += 1;
          if (i === 2) d.gen2 += 1;
        }
      }
      if (h.paused) d.screen = 0;
      break;
    }
    case "racing": {
      const s = og.ui.seconds();
      const h = og.ui.racingHud("Hud", {
        speed: 180 + 60 * Math.sin(s), gear: "4", lap: 2, laps: 3, position: 3, racers: 8, time: 83.417 + s, best: 41.2, boost: 0.62,
        map: { size: 128, markers: [{ x: 0.62, y: 0.3, player: true }, { x: 0.55, y: 0.24 }, { x: 0.7, y: 0.42 }], label: "" },
        touch: true,
      });
      if (h.paused) d.screen = 0;
      break;
    }
    case "cutscene": {
      const r = og.ui.cutscene("Story", { open: true, lines: CUTSCENE, index: d.line, skipLabel: tr(world, "cutscene.skip") });
      d.line = r.index;
      if (r.finished || r.skipped) {
        d.line = 0;
        d.screen = 0;
      }
      break;
    }
    case "shop": {
      const r = og.ui.shopScreen("Shop", {
        title: tr(world, "shop.title"),
        priceLabel: tr(world, "shop.price"),
        buyLabel: tr(world, "shop.buy"),
        shortLabel: tr(world, "shop.short"),
        ownedLabel: tr(world, "shop.owned"),
        emptyLabel: tr(world, "shop.empty"),
        balance: d.balance,
        currency: "coins",
        currencyIcon: "coin",
        categories: ["Gear", "Items", "Bundles"],
        items: SHOP.map((s, i) => ({ ...s, owned: (d.owned & (1 << i)) !== 0 && s.category === "Gear" })),
      });
      if (r.bought !== null) {
        const i = SHOP.findIndex((s) => s.id === r.bought);
        const item = SHOP[i]!;
        d.balance -= item.price;
        d.owned |= 1 << i;
        d.bought += 1;
        og.ui.toast(world, `Bought ${item.name}`, { tone: "success" });
      }
      // Short of the price: the shop's Buy shook with the error sound (og.ui.shake).
      if (r.refused !== null) d.refused += 1;
      if (r.closed) d.screen = 0;
      break;
    }
    case "inventory": {
      const slots = [d.inv0, d.inv1, d.inv2, d.inv3, d.inv4, d.inv5];
      const r = og.ui.inventoryScreen("Inventory", {
        slots: 24,
        items: ITEMS.map((it, i) => ({ ...it, slot: slots[i]! })),
        hintLabel: tr(world, "inventory.hint"),
        countLabel: tr(world, "inventory.count"),
      });
      if (r.moved) {
        const from = slots.indexOf(r.moved.from);
        const to = slots.indexOf(r.moved.to);
        if (from >= 0) slots[from] = r.moved.to;
        if (to >= 0) slots[to] = r.moved.from;
        [d.inv0, d.inv1, d.inv2, d.inv3, d.inv4, d.inv5] = slots as [number, number, number, number, number, number];
        d.moves += 1;
      }
      if (r.used !== null) {
        d.used += 1;
        og.ui.toast(world, `Used ${ITEMS.find((it) => it.id === r.used)?.name ?? r.used}`, {});
      }
      if (r.closed) d.screen = 0;
      break;
    }
    case "levels": {
      const r = og.ui.levelSelect("Levels", { levels: LEVELS });
      if (r.chosen !== null) {
        d.level = r.chosen + 1;
        d.screen = screenIndex("results");
      }
      if (r.closed) d.screen = 0;
      break;
    }
    case "results": {
      const r = og.ui.resultsScreen("Results", {
        open: true, score: 12840, best: 11200, newBest: true, stars: 2,
        stats: [{ label: "Time", value: "1:42" }, { label: "Coins", value: "86 / 100" }, { label: "Enemies", value: "14" }],
        shareText: "I scored 12,840 in Origin Quest",
        onShare: (text) => og.ui.toast(world, `Shared: ${text}`, { tone: "info" }),
      });
      if (r.selected === "retry") d.screen = screenIndex("platformer");
      if (r.selected === "next") d.screen = screenIndex("levels");
      if (r.selected === "menu") d.screen = 0;
      break;
    }
    case "leaderboard": {
      const tick = og.ui.tick();
      const loading = tick - d.scopeTick < 45;
      const r = og.ui.leaderboardScreen("Board", {
        scopes: ["Today", "This week", "All time"],
        entries: d.scope === 2 ? [] : leaderboard(d.scope),
        loading,
        error: !loading && d.offline ? tr(world, "leaderboard.error") : "",
        empty: "No scores for all time yet",
        retryLabel: tr(world, "leaderboard.retry"),
      });
      if (r.changed) {
        d.scope = r.scope;
        d.scopeTick = tick;
      }
      if (r.retry) {
        d.offline = false;
        d.scopeTick = tick;
      }
      if (r.closed) d.screen = 0;
      break;
    }
    case "loading": {
      const s = og.ui.seconds();
      og.ui.loadingScreen("Loading", {
        title: "Loading the old mill",
        progress: Math.min(1, (s % 8) / 6),
        tips: ["Hold Jump to jump higher.", "Coins add up to extra lives.", "Press Escape to pause at any time."],
        tipSeconds: 3,
      });
      if (og.ui.back()) d.screen = 0;
      break;
    }
    case "confirm": {
      const r = og.ui.confirmDialog("Delete", {
        open: true,
        title: tr(world, "confirm.title"),
        message: tr(world, "confirm.message", { slot: 2 }),
        confirmLabel: tr(world, "confirm.delete"),
        cancelLabel: tr(world, "confirm.cancel"),
        danger: true,
      });
      if (r.confirmed) {
        d.confirmed += 1;
        og.ui.toast(world, "Save deleted", { tone: "danger" });
        d.screen = 0;
      }
      if (r.cancelled) d.screen = 0;
      break;
    }
    case "alert": {
      if (og.ui.alertDialog("Notice", { open: true, title: "Connection lost", message: "Your progress is saved on this device and will sync when you are back online." })) d.screen = 0;
      break;
    }
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
    case "toasts": {
      const t = og.ui.frameTheme();
      og.ui.anchor("ToastDemo", { anchor: "center", direction: "column", gap: t.space.sm, width: { kind: "fixed", px: 300 }, padding: { left: t.space.lg, top: t.space.lg, right: t.space.lg, bottom: t.space.lg }, background: { token: "surface" }, radius: t.radius.panel }, () => {
        og.ui.label("Title", "Toasts", { step: "xl" });
        for (const [tone, text] of [["neutral", "Checkpoint reached"], ["success", "Achievement unlocked"], ["warn", "Low battery"], ["danger", "Connection lost"], ["info", "New version available"]] as const) {
          if (og.ui.button(tone, text, { width: { kind: "grow" } }).clicked) og.ui.toast(world, text, { tone });
        }
      });
      back = og.ui.back();
      break;
    }
  }
  // A label of three scripts at once (`mixed:on`): each run in its own face
  // (the role's Latin, a locale's CJK, the Arabic shaped), nothing missing.
  if (d.mixed && (screen === "mainMenu" || screen === "pause" || screen === "confirm")) {
    og.ui.anchor("Mixed", { anchor: "bottom", direction: "column", padding: { left: 0, top: 0, right: 0, bottom: 24 } }, () => {
      og.ui.label("Text", tr(world, "menu.mixed"), { step: "lg", nowrap: true });
    });
  }
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
  systems: [Kit, Thud],
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
    skin: (world, args) => {
      update(world, { skin: Math.max(0, SKINS.indexOf((args[0] ?? "tokens") as (typeof SKINS)[number])) });
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
  },
  commandHelp: {
    show: { usage: "<screen>", doc: "Open a screen of the kit (gallery, widgets, mainMenu, pause, settings, ...)." },
    viewport: { usage: "<desktop|tablet|phone|landscape>", doc: "Lay the UI out for that surface (og.ui.previewViewport)." },
    pad: { usage: "<up|down|left|right|next|prev|activate|back>", doc: "A gamepad intent through og.ui.navigate." },
    toast: { usage: "<text>", doc: "Queue a toast." },
    damage: { usage: "<hp>", doc: "Take hit points off the HUDs' health." },
    heal: { usage: "<hp>", doc: "Give hit points back." },
    skin: { usage: "<tokens|pixel|cartoon|fantasy|scifi|paper|neon>", doc: "Dress every screen in a style pack." },
    transition: { usage: "<none|fade|wipe|iris|dissolve|slide>", doc: "Send screen changes through og.ui.screenTransition." },
    sounds: { usage: "<on|off>", doc: "Hand the UI's event sounds their blips (on by default)." },
    mixed: { usage: "<on|off>", doc: "Show a label mixing CJK, Latin and Arabic (menu.mixed) under the menu, pause and confirm screens." },
    offline: { usage: "<on|off>", doc: "The leaderboard's rows fail to load: its error message and Retry (which loads again)." },
  },
  facts: (world) => {
    const d = current(world);
    return d ? { screen: SCREENS[d.screen] ?? "gallery", skin: SKINS[d.skin] ?? "tokens", hp: d.hp, balance: d.balance, saves: d.saves, picks: [d.pick0, d.pick1, d.pick2, d.pick3], focus: [d.focus0, d.focus1, d.focus2, d.focus3] } : {};
  },
});
