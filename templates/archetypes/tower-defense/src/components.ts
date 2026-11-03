// Every piece of Path Guard's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The round (Game), the knobs (Tuning) and the player's settings (Prefs)
// are resources. The actors are pooled rows spawned in `start`: one Plant
// per buildable cell (placing a plant switches its row on), the Visitors
// (the creeps) and the Shots, plus the Cursor and one Sfx row per sound.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** What a plant is: the tray's order (keys 1, 2, 3). */
export const Plant = { Thorn: 0, Frost: 1, Hedge: 2 } as const;
export const PLANTS = ["thorn", "frost", "hedge"] as const;

/** What a visitor is. */
export const Kind = { Beetle: 0, Moth: 1, Snail: 2 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Plant: 1, Thorn: 2, Frost: 4, Pop: 8, Leak: 16, Wave: 32, Clear: 64, Won: 128, Lost: 256, Move: 512 } as const;

/** Why a placement was refused (Game.refused counts them; the HUD shakes and says why). */
export const Refusal = { None: 0, Poor: 1, Blocked: 2, Taken: 3 } as const;

/** The round: the lantern's light, the purse, the waves and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  paused: bool(false, { doc: "The pause menu is open (the garden holds)." }),
  settingsOpen: bool(false, { doc: "The settings dialog is open." }),
  won: bool(false, { doc: "The last round held every wave." }),
  rounds: u32(0, { doc: "Rounds started." }),
  lives: u32(0, { doc: "The lantern's light: a visitor that reaches the doorstep takes its share (Tuning.startLives at Play)." }),
  gold: u32(0, { doc: "Coins to plant with." }),
  wave: u32(0, { doc: "The wave under way or last spent (1-based; 0 before the first)." }),
  inWave: bool(false, { doc: "Visitors of the wave are still coming or walking." }),
  toSpawn: u32(0, { doc: "Visitors of this wave not yet through the gate." }),
  spawnTicks: u32(0, { unit: "ticks", doc: "Ticks to the next visitor." }),
  spawned: u32(0, { doc: "Visitors of this wave through the gate so far." }),
  breakTicks: u32(0, { unit: "ticks", doc: "Ticks before the next wave starts by itself." }),
  callNext: bool(false, { doc: "The player asked for the next wave now (an early call pays its bonus)." }),
  walking: u32(0, { doc: "Visitors on the field now." }),
  kills: u32(0, { doc: "Visitors turned away." }),
  leaks: u32(0, { doc: "Visitors that reached the doorstep." }),
  placed: u32(0, { doc: "Plants set this round (each stays to the round's end)." }),
  refused: u32(0, { doc: "Placements refused (too poor, the path closed, the cell taken)." }),
  refusal: u32(0, { doc: "Refusal.* of the last refused placement." }),
  refusedTick: i32(-1, { unit: "ticks", doc: "The tick of the last refusal (the HUD shakes the gold on it)." }),
  shots: u32(0, { doc: "Shots fired." }),
  selected: u32(Plant.Thorn, { doc: "The plant the next placement sets (Plant.*)." }),
  cursorX: i32(4, { doc: "The cursor's cell, a map column." }),
  cursorY: i32(9, { doc: "The cursor's cell, a map row (0 the top)." }),
  score: u32(0, { doc: "The last round's score: kills × 10 + light × 50 + coins left." }),
  best: u32(0, { doc: "The best score, from the store." }),
  newBest: bool(false),
  bestRequest: u32(0, { doc: "The store request reading `best` (og.platform)." }),
  bestRevision: u32(0, { doc: "The revision `best` was read or written at." }),
  bestWrite: u32(0, { doc: "The store request writing `best`." }),
  overTick: u32(0, { unit: "ticks", doc: "The tick the round ended." }),
  lastStop: i32(-100, { unit: "ticks", doc: "The tick of the last kill's hit stop (at most one per 10 ticks, og-feel §3)." }),
  occ0: u32(0, { doc: "Cells holding a visitor (bit per cell, row-major, 4 words): no plant goes there." }),
  occ1: u32(0),
  occ2: u32(0),
  occ3: u32(0),
  pendingPlants: u32(0, { doc: "Plants set on the pads layer that the Plants system has yet to switch on (the pads tile says which)." }),
  reset: bool(false, { doc: "A new round asked the Plants and Visitors systems to clear their rows." }),
  sfx: u32(0, { doc: "Sound.* bits waiting to play." }),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  viewW: f32(0, { unit: "cssPx", doc: "The surface the camera was last framed for." }),
  viewH: f32(0, { unit: "cssPx" }),
  viewTop: f32(0, { unit: "cssPx" }),
  language: u32(0, { doc: "Prefs.language, mirrored for the facts." }),
  grid: f64(0, { doc: "The og.nav grid of the field." }),
  field: f64(0, { doc: "The og.nav flow field to the doorstep." }),
  map: f64(0, { doc: "The field's og.Tilemap entity." }),
  sheet: f64(0, { doc: "The sprite sheet's texture (cursor and range ring are drawn from it)." }),
  ring: f64(0, { doc: "The range ring's texture." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  startGold: u32(150, { min: 0, max: 1000, step: 10, tunable: true, label: "Starting coins" }),
  startLives: u32(10, { min: 1, max: 50, step: 1, tunable: true, label: "Lantern light" }),
  waveBreak: f32(8, { min: 0, max: 30, step: 0.5, unit: "s", tunable: true, label: "Break between waves" }),
  spawnGap: f32(1, { min: 0.2, max: 4, step: 0.1, unit: "s", tunable: true, label: "Gap between visitors" }),
  hpGrowth: f32(0.2, { min: 0, max: 1, step: 0.02, tunable: true, label: "Visitor toughness per wave", doc: "Each wave's visitors have 1 + growth × (wave − 1) times their base health." }),
  thornCost: u32(50, { min: 0, max: 500, step: 5, tunable: true, label: "Thorn cost" }),
  thornDamage: f32(14, { min: 1, max: 100, step: 1, tunable: true, label: "Thorn damage" }),
  thornReload: u32(30, { min: 6, max: 240, step: 2, unit: "ticks", tunable: true, label: "Thorn reload" }),
  thornRange: f32(2.8, { min: 1, max: 6, step: 0.1, unit: "m", tunable: true, label: "Thorn range" }),
  frostCost: u32(70, { min: 0, max: 500, step: 5, tunable: true, label: "Frost lily cost" }),
  frostDamage: f32(5, { min: 0, max: 100, step: 1, tunable: true, label: "Frost damage" }),
  frostReload: u32(60, { min: 6, max: 240, step: 2, unit: "ticks", tunable: true, label: "Frost reload" }),
  frostRange: f32(2.2, { min: 1, max: 6, step: 0.1, unit: "m", tunable: true, label: "Frost range" }),
  frostSlow: f32(0.5, { min: 0.1, max: 1, step: 0.05, tunable: true, label: "Frost slow", doc: "A frosted visitor walks at this fraction of its speed." }),
  frostTicks: u32(120, { min: 0, max: 600, step: 10, unit: "ticks", tunable: true, label: "Frost length" }),
  hedgeCost: u32(10, { min: 0, max: 200, step: 5, tunable: true, label: "Hedge cost" }),
  shotSpeed: f32(11, { min: 2, max: 40, step: 0.5, unit: "m/s", tunable: true, label: "Shot speed" }),
  killStop: u32(2, { min: 0, max: 8, step: 1, unit: "ticks", tunable: true, label: "Kill hit stop" }),
  leakStop: u32(6, { min: 0, max: 16, step: 1, unit: "ticks", tunable: true, label: "Leak hit stop" }),
  leakShake: f32(0.4, { min: 0, max: 1, step: 0.05, tunable: true, label: "Leak shake" }),
});

/** The player's settings: kept in og.platform's settings, restored on the first ticks. */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1, doc: "The master volume." }),
  music: f32(0.7, { min: 0, max: 1, doc: "The music bus." }),
  effects: f32(1, { min: 0, max: 1, doc: "The effects and UI buses." }),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false, { doc: "Screens fade instead of irising; shakes are a third." }),
  loadRequest: u32(0, { doc: "The og.platform.loadSettings request." }),
  loaded: bool(false, { doc: "The saved settings were applied (or there were none)." }),
  saves: u32(0, { doc: "Times the settings were saved." }),
});

/** One cell's plant: every buildable cell has a row, off until a plant is set there. */
export const Bed = defineComponent("Bed", {
  cellX: i32(0, { doc: "The map column." }),
  cellY: i32(0, { doc: "The map row (0 the top)." }),
  planted: bool(false, { doc: "A plant stands here." }),
  kind: u32(Plant.Thorn, { doc: "Plant.* of src/components.ts." }),
  cool: u32(0, { unit: "ticks", doc: "Ticks to the next shot; the behaviour fires at 0 (assets/ai/*.ogt reads Bed.cool)." }),
  anim: u32(0, { unit: "ticks", doc: "Ticks the shooting clip has left." }),
  shots: u32(0, { doc: "Shots this plant fired." }),
});

/** A visitor (a creep): health, its walk and what turning it away pays. */
export const Visitor = defineComponent("Visitor", {
  slot: u32(0, { doc: "Its pool index." }),
  active: bool(false, { doc: "Walking the field." }),
  dying: u32(0, { unit: "ticks", doc: "Ticks left of its pop before the row is parked." }),
  kind: u32(Kind.Beetle, { doc: "Kind.* of src/components.ts." }),
  hp: f32(0),
  maxHp: f32(0),
  speed: f32(0, { unit: "m/s", doc: "Its walking speed before frost." }),
  slow: u32(0, { unit: "ticks", doc: "Ticks of frost left." }),
});

/** A shot in flight: a tween carries it, its tag says whom it hits. */
export const Shot = defineComponent("Shot", {
  tween: f64(0, { doc: "Its flight's og.tween handle; the shot is free once that tween is done." }),
});

/** The cell cursor the keys and the pad move. */
export const Cursor = defineComponent("Cursor", {
  moves: u32(0, { doc: "Times it moved." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0, { doc: "Sound.* of src/components.ts." }),
  plays: u32(0, { doc: "Times it started." }),
});
