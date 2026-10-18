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
// damage:<hp>, heal:<hp>.

import { og, defineComponent, defineSystem, bool, f32, f64, u32 } from "@origingame/engine";
import type { Entity, GameSettings, Viewport, World } from "@origingame/engine";

export const SCREENS = [
  "gallery", "widgets", "mainMenu", "pause", "settings", "platformer", "shooter", "puzzle", "idle", "racing",
  "cutscene", "shop", "inventory", "levels", "results", "leaderboard", "tutorial", "loading", "confirm", "alert", "toasts",
] as const;
export type Screen = (typeof SCREENS)[number];

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
};

/** The demo's state: one entity named "demo" (dump_entity:demo reads it). */
export const Demo = defineComponent("Demo", {
  screen: u32(0),
  paused: bool(),
  // Asset handles exceed 32 bits: f64 keeps them exact.
  font: f64(0),
  icons: f64(0),
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
  // Dialogs.
  confirmed: u32(0),
  level: u32(0),
});
type DemoState = { -readonly [K in keyof ReturnType<typeof demoDefaults>]: ReturnType<typeof demoDefaults>[K] };
function demoDefaults() {
  return { screen: 0, paused: false, font: 0, icons: 0, backdrop: 0, hp: 72, mana: 40, coins: 128, score: 4250, lives: 2, time: 95, masterVolume: 1, musicVolume: 0.8, sfxVolume: 1, muted: false, quality: 1, showFps: false, reducedMotion: false, highContrast: false, vibration: true, invertY: false, sensitivity: 1, language: 0, saves: 0, tab: 0, difficulty: 1, amount: 3, mode: 0, optA: true, optB: false, chip: 0, stickX: 0, stickY: 0, line: 0, step: 0, balance: 1250, owned: 0, bought: 0, inv0: 0, inv1: 1, inv2: 2, inv3: 5, inv4: 9, inv5: 13, moves: 0, used: 0, gold: 40, gen0: 1, gen1: 0, gen2: 0, taps: 0, scope: 0, scopeTick: 0, confirmed: 0, level: 0 };
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
  { code: "es", label: "Español" },
  { code: "fr", label: "Français" },
];

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

/** Applies the theme-affecting settings (contrast, motion) with the demo's font and icons. */
function applyTheme(world: World, d: DemoState): void {
  og.ui.configure(world, {
    ...(d.font ? { font: d.font as never } : {}),
    contrast: d.highContrast ? "high" : "standard",
    reducedMotion: d.reducedMotion,
    icons: d.icons ? { texture: d.icons as never, columns: 8, rows: 4, names: ICONS } : {},
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

/** The Gallery button over every screen. */
function galleryButton(d: DemoState): void {
  const bp = og.ui.breakpoint();
  // On a phone the bottom corners belong to the touch controls.
  og.ui.anchor("Back", { anchor: bp === "phonePortrait" || bp === "phoneLandscape" ? "top" : "bottomLeft", offsetY: bp === "phonePortrait" ? 52 : 0 }, () => {
    if (og.ui.button("Gallery", "Gallery", { variant: "secondary", size: "small" }).clicked) d.screen = 0;
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
    if (og.ui.actionButton("Hit", "Hit", { primary: true }).clicked) d.hp = Math.max(0, d.hp - 15);
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

export const Kit = defineSystem("Kit", { reads: [og.Input], writes: [Demo] }, (q, time, world) => {
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
  const screen = SCREENS[d.screen] ?? "gallery";
  // The game behind every screen but the gallery and the widget sheet: a dusk landscape.
  if (screen !== "gallery" && screen !== "widgets" && d.backdrop) og.render.sprite(world, { texture: d.backdrop as never, width: 640, height: 360, layer: -10 });

  if (screen !== "gallery" && screen !== "loading") galleryButton(d);
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
        title: "Origin Quest",
        subtitle: "A small adventure",
        items: [
          { id: "play", label: "Play" },
          { id: "continue", label: "Continue", disabled: true },
          { id: "settings", label: "Settings" },
          { id: "quit", label: "Quit" },
        ],
        footer: "v0.1.0",
      });
      if (r.selected === "play") d.screen = screenIndex("platformer");
      if (r.selected === "settings") d.screen = screenIndex("settings");
      if (r.selected === "quit") d.screen = 0;
      if (screen === "tutorial") {
        const tut = og.ui.tutorialOverlay("Tutorial", { open: true, steps: TUTORIAL, index: d.step });
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
      const h = og.ui.platformerHud("Hud", { lives: d.lives, maxLives: 3, coins: d.coins, score: d.score, level: "1-2 Old mill", timeLeft: d.time, touch: true });
      if (h.paused) d.paused = true;
      if (h.action === "jump") d.coins += 1;
      const p = og.ui.pauseMenu("Pause", { open: screen === "pause" || d.paused });
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
        onSave: () => {
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
      d.language = Math.max(0, LANGUAGES.findIndex((l) => l.code === s.language));
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
      if (h.action === "fire") d.hp = Math.max(0, d.hp - 5);
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
      const r = og.ui.cutscene("Story", { open: true, lines: CUTSCENE, index: d.line });
      d.line = r.index;
      if (r.finished || r.skipped) {
        d.line = 0;
        d.screen = 0;
      }
      break;
    }
    case "shop": {
      const r = og.ui.shopScreen("Shop", {
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
      if (r.closed) d.screen = 0;
      break;
    }
    case "inventory": {
      const slots = [d.inv0, d.inv1, d.inv2, d.inv3, d.inv4, d.inv5];
      const r = og.ui.inventoryScreen("Inventory", { slots: 24, items: ITEMS.map((it, i) => ({ ...it, slot: slots[i]! })) });
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
        empty: "No scores for all time yet",
      });
      if (r.changed) {
        d.scope = r.scope;
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
      const r = og.ui.confirmDialog("Delete", { open: true, title: "Delete this save?", message: "Your progress in slot 2 will be lost. This cannot be undone.", confirmLabel: "Delete", danger: true });
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
  systems: [Kit],
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
    const font = og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf");
    const icons = og.load.texture(world, "textures/icons.png");
    const backdrop = og.load.texture(world, "textures/backdrop.png");
    const handle = (r: { ok: boolean; value?: unknown }): number => (r.ok ? (r.value as number) : 0);
    world.insert(e.value, Demo, { font: handle(font), icons: handle(icons), backdrop: handle(backdrop) });
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
      if (d) update(world, { hp: Math.max(0, d.hp - Number(args[0] ?? 10)) });
    },
    heal: (world, args) => {
      const d = current(world);
      if (d) update(world, { hp: Math.min(100, d.hp + Number(args[0] ?? 10)) });
    },
  },
  facts: (world) => {
    const d = current(world);
    return d ? { screen: SCREENS[d.screen] ?? "gallery", hp: d.hp, balance: d.balance, saves: d.saves } : {};
  },
});
