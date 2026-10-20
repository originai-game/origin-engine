// Every piece of Hollowbell's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name, and snapshots and the world hash carry them.
//
// The run (Game), the knobs (Tuning) and the player's settings (Prefs) are
// resources. The actors are rows spawned in `start` from the map's objects:
// Wren (Hero), the enemies (Enemy), the pickups (Pickup), the checkpoint
// lanterns (Lantern), the lift (Lift), the stones (Stone), the torches'
// and crystals' lights (Flicker), the bell (Bell), the mole (Mole) and one
// voice per sound (Sfx).

import { bool, defineComponent, defineResource, f32, f64, i32, str, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** Enemy.kind. */
export const Kind = { Beetle: 0, Moth: 1, Gloam: 2 } as const;

/** Enemy.state. */
export const State = { Walk: 0, Wind: 1, Dash: 2, Hurt: 3, Dead: 4, Dive: 5, Rise: 6 } as const;

/** Pickup.kind. */
export const Item = { Glimmer: 0, Chime: 1, Heart: 2 } as const;

/** The layers of sprites/items.ogt (a texture array). */
export const Layer = { Glimmer: 0, Chime: 4, Heart: 8, LanternDark: 9, LanternLit: 10 } as const;

/** Collision layers (Collider.layers bits). */
export const Bits = { World: 1, Hero: 2, Enemy: 4, Stone: 8, Plank: 16 } as const;

/** Sprite layers, back to front (the map's layers start at Draw.map). */
export const Draw = { sky: -40, map: -20, pools: -14, props: -5, items: 2, enemies: 3, hero: 5, fx: 12, text: 100 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = {
  Jump: 1, Land: 2, Step: 4, Swing: 8, Hit: 16, Hurt: 32, Glimmer: 64, Chime: 128,
  Buy: 256, Checkpoint: 512, Boss: 1024, Win: 2048, Bell: 4096,
} as const;

/** Effect requests: bits of Game.fx, and the Fx row's kind. */
export const FxKind = { Dust: 1, Hit: 2, Chime: 4, Burst: 8 } as const;

/** The hearts a run starts with, the chimes the bell needs. */
export const HEARTS = 3;
export const CHIMES = 3;
/** The player's name: at most this many characters (Prefs.name holds four bytes of UTF-8 for each). */
export const NAME_CHARS = 12;

/** The run: progress, the hero's state for the other systems, the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  naming: bool(false, { doc: "The name dialog is open over the title (a new game)." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused, in the settings, talking or shopping)." }),
  start: bool(false, { doc: "Every system resets its part of the hollow this tick (a new run, a continue, the title)." }),
  // Progress (what a save keeps).
  hearts: i32(HEARTS),
  glimmers: u32(0),
  chimes: u32(0, { doc: "Chimes carried (bits of chimeMask counted)." }),
  chimeMask: u32(0, { doc: "Bit i: chime-i taken." }),
  pickedLo: u32(0, { doc: "Bits 0-31 of the glimmers taken (by their map index)." }),
  pickedHi: u32(0, { doc: "Bits 32-63." }),
  heartMask: u32(0, { doc: "Bit i: heart-i taken." }),
  killMask: u32(0, { doc: "Bit i: enemy i beaten (by Enemy.index)." }),
  bossDown: bool(false),
  wick: bool(false, { doc: "The brighter wick is bought: the hero's light reaches Tuning.wickRadius." }),
  cloak: bool(false, { doc: "The moth-wing cloak is bought: one air jump." }),
  checkpoint: i32(-1, { doc: "The lantern last lit (-1: none, the run starts at the spawn)." }),
  litMask: u32(0, { doc: "Bit i: lantern i lit." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  // The run's counts (the facts' hits and kills).
  hurts: u32(0, { doc: "Hearts lost." }),
  kills: u32(0, { doc: "Enemies beaten." }),
  hits: u32(0, { doc: "Strikes that landed." }),
  swings: u32(0),
  falls: u32(0, { doc: "Times every heart was lost." }),
  steps: u32(0),
  jumps: u32(0),
  landings: u32(0),
  // The hero, for the other systems (written by Control each tick).
  hero: f64(0, { doc: "Wren's entity." }),
  map: f64(0, { doc: "The tilemap's entity." }),
  heroX: f32(0, { unit: "m" }),
  heroY: f32(0, { unit: "m", doc: "The feet." }),
  facing: i32(1, { doc: "1 right, -1 left." }),
  grounded: bool(false),
  invulnerable: u32(0, { unit: "ticks", doc: "Ticks before the hero can be hurt again." }),
  knock: u32(0, { unit: "ticks", doc: "Ticks the hero is thrown back, without control." }),
  knockDir: f32(0, { doc: "-1 or 1: the way the hero is thrown." }),
  respawn: u32(0, { unit: "ticks", doc: "Ticks before a fallen hero wakes by the lantern." }),
  spawnX: f32(0, { unit: "m", doc: "Where a new run starts (the map's spawn)." }),
  spawnY: f32(0, { unit: "m" }),
  checkX: f32(0, { unit: "m", doc: "The last lantern lit: where a fallen or continued run wakes." }),
  checkY: f32(0, { unit: "m" }),
  warp: bool(false, { doc: "Control puts the hero at warpX, warpY this tick (a respawn, the pit, a command)." }),
  warpX: f32(0, { unit: "m" }),
  warpY: f32(0, { unit: "m" }),
  // This tick's strike: Control writes it on the attack clip's `hit` event (og.physics.overlap of the mallet's box); Enemies and Stones take it.
  strike: bool(false),
  strikeX: f32(0, { unit: "m" }),
  strikeY: f32(0, { unit: "m" }),
  struck0: f64(0, { doc: "Entities under the strike (0: none)." }),
  struck1: f64(0),
  struck2: f64(0),
  struck3: f64(0),
  // Talking and trading.
  nearMole: bool(false),
  nearBell: bool(false),
  talking: bool(false),
  talkIndex: u32(0),
  shopping: bool(false),
  talks: u32(0),
  bought: u32(0),
  // The gloam.
  arena: bool(false, { doc: "The hero is in the gloam's hall." }),
  bossAwake: bool(false),
  bossHp: i32(0),
  bossX: f32(0, { unit: "m" }),
  bossY: f32(0, { unit: "m" }),
  slowTicks: u32(0, { unit: "ticks", doc: "Real ticks of slow motion left (the gloam's fall)." }),
  // The end.
  ringing: u32(0, { unit: "ticks", doc: "Ticks since the bell was rung; the results come after it." }),
  score: u32(0),
  best: u32(0, { doc: "The best score, from og.platform's store." }),
  newBest: bool(false),
  // The world around the hero.
  area: i32(-1, { doc: "The map's area object holding the hero (its index among the areas)." }),
  ambient: f32(1, { doc: "The 2D ambient light now (og.Lighting2D.ambient's level)." }),
  light: f32(0, { unit: "m", doc: "The hero's lantern's radius now." }),
  // The look on screen: 0 looks/hollowbell.ogt, 1 looks/dawn.ogt (the results).
  look: u32(0),
  mainLook: f64(0),
  dawnLook: f64(0),
  // og.platform's store: the save (progress) and the best.
  saveNow: bool(false, { doc: "Write the run to the store this tick (a lantern lit, Save and quit)." }),
  saveRequest: u32(0),
  loadRequest: u32(0),
  bestRequest: u32(0),
  bestSaveRequest: u32(0),
  saveRevision: f64(0, { doc: "The save's revision as last read or written: a write passes it, so a stale one is refused." }),
  hasSave: bool(false, { doc: "A save was read from the store: Continue is offered." }),
  saves: u32(0, { doc: "Saves the store accepted." }),
  conflicts: u32(0, { doc: "Saves the store refused as stale (read again and merged)." }),
  loads: u32(0, { doc: "Runs continued from a save." }),
  saveJson: str(1024, "", { doc: "The save last read from the store, as JSON: what Continue restores." }),
  // Effects asked for this tick (Fx.* bits) and where: the Fx system moves an emitter there and restarts it.
  fx: u32(0),
  dustX: f32(0), dustY: f32(0),
  hitX: f32(0), hitY: f32(0),
  chimeX: f32(0), chimeY: f32(0),
  burstX: f32(0), burstY: f32(0),
  // Bookkeeping.
  sfx: u32(0),
  musicChunks: u32(0),
  music: f64(0, { doc: "The music's sound handle." }),
  items: f64(0, { doc: "The items' texture array (sprites/items.ogt), for the HUD's screen sprites." }),
  a11yUnnamed: u32(0, { doc: "Focusable nodes of og.ui.a11y() without a name, last checked." }),
  a11yNodes: u32(0),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  maxSpeed: f32(6.5, { min: 2, max: 14, step: 0.1, unit: "m/s", tunable: true, label: "Run speed" }),
  jumpHeight: f32(3.6, { min: 1, max: 8, step: 0.1, unit: "m", tunable: true, label: "Jump height" }),
  timeToApex: f32(0.42, { min: 0.2, max: 1, step: 0.01, unit: "s", tunable: true, label: "Time to apex" }),
  hitStop: u32(5, { min: 0, max: 20, unit: "ticks", tunable: true, label: "Hit stop" }),
  shake: f32(0.22, { min: 0, max: 1, step: 0.01, tunable: true, label: "Shake per hit" }),
  reach: f32(1.2, { min: 0.4, max: 3, step: 0.05, unit: "m", tunable: true, label: "Mallet reach" }),
  iframes: u32(60, { min: 0, max: 180, unit: "ticks", tunable: true, label: "Blinking after a hurt" }),
  knockTicks: u32(12, { min: 0, max: 40, unit: "ticks", tunable: true, label: "Knock-back" }),
  beetleSpeed: f32(1.3, { min: 0.2, max: 6, step: 0.1, unit: "m/s", tunable: true, label: "Beetle pace" }),
  chargeSpeed: f32(8, { min: 2, max: 16, step: 0.5, unit: "m/s", tunable: true, label: "Gloam charge" }),
  mothSpeed: f32(3.2, { min: 0.5, max: 8, step: 0.1, unit: "m/s", tunable: true, label: "Moth dive" }),
  sight: f32(6, { min: 1, max: 16, step: 0.5, unit: "m", tunable: true, label: "Enemy sight" }),
  gloamHp: i32(8, { min: 1, max: 30, tunable: true, label: "Gloam's hits" }),
  lightRadius: f32(4.5, { min: 1, max: 12, step: 0.25, unit: "m", tunable: true, label: "Lantern radius" }),
  wickRadius: f32(7, { min: 1, max: 14, step: 0.25, unit: "m", tunable: true, label: "Brighter wick radius" }),
});

/** The player's settings, kept in og.platform's settings. */
export const Prefs = defineResource("Prefs", {
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  master: f32(1, { min: 0, max: 1 }),
  muted: bool(false),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  highContrast: bool(false),
  name: str(NAME_CHARS * 4, "", { doc: "The bell-keeper's name, at most NAME_CHARS characters: the save slot's name too." }),
  loadRequest: u32(0),
  saves: u32(0),
});

/** Wren. */
export const Hero = defineComponent("Hero", {
  attack: u32(0, { unit: "ticks", doc: "Ticks into the swing; 0 is none." }),
  clip: u32(0, { doc: "The flipbook clip shown (HERO_CLIP)." }),
  wasGrounded: bool(true),
  airJumped: bool(false),
});

/** An enemy: a beetle walks its beat, a moth hovers and dives, the gloam charges. */
export const Enemy = defineComponent("Enemy", {
  kind: u32(0, { doc: "Kind.*" }),
  index: u32(0, { doc: "Its bit in Game.killMask." }),
  hp: i32(1),
  state: u32(0, { doc: "State.*" }),
  timer: u32(0, { unit: "ticks", doc: "Ticks in the state." }),
  homeX: f32(0, { unit: "m" }),
  homeY: f32(0, { unit: "m" }),
  range: f32(4, { unit: "m", doc: "A beetle's beat: homeX ± range." }),
  dir: f32(1, { doc: "-1 or 1: the way it walks or faces." }),
  knock: f32(0, { unit: "m/s", doc: "Its own knock-back speed, decaying." }),
  tx: f32(0, { unit: "m", doc: "A moth's dive target, a gloam's charge direction." }),
  ty: f32(0, { unit: "m" }),
  hitAt: u32(0, { doc: "The tick it was last struck (one strike lands once)." }),
});

/** A pickup: a glimmer, a chime or a heart, drawn from a layer of the items' texture array. */
export const Pickup = defineComponent("Pickup", {
  kind: u32(0, { doc: "Item.*" }),
  index: u32(0, { doc: "Its bit among its kind's (Game.pickedLo/Hi, chimeMask, heartMask)." }),
  taken: bool(false),
  hidden: bool(false, { doc: "Not in play yet (the gloam's chime until it falls)." }),
  baseY: f32(0, { unit: "m" }),
  phase: f32(0, { doc: "Its bob's offset in the cycle." }),
});

/** A checkpoint lantern: touching it lights it and saves the run. */
export const Lantern = defineComponent("Lantern", {
  index: u32(0),
});

/** The lift: a kinematic plank riding to and fro over the pit. */
export const Lift = defineComponent("Lift", {
  x0: f32(0, { unit: "m" }),
  travel: f32(6, { unit: "m" }),
  speed: f32(2, { unit: "m/s" }),
});

/** A stone the mallet knocks about (a dynamic body). */
export const Stone = defineComponent("Stone", {
  x0: f32(0, { unit: "m" }),
  y0: f32(0, { unit: "m" }),
  knocks: u32(0),
});

/** A light that flickers: torches, crystals. */
export const Flicker = defineComponent("Flicker", {
  base: f32(1, { doc: "Its intensity at rest." }),
  rate: f32(9, { doc: "How fast it flickers (radians per second)." }),
  phase: f32(0),
});

/** The bell. */
export const Bell = defineComponent("Bell", {
  rung: bool(false),
  tolls: u32(0),
});

/** Old Burrow. */
export const Mole = defineComponent("Mole", {
  talking: bool(false),
});

/** A one-shot particle emitter the Fx system moves and restarts on request. */
export const Fx = defineComponent("Fx", {
  kind: u32(0, { doc: "FxKind.*" }),
  bursts: u32(0),
});

/**
 * A pool of light drawn over the map where a light shines: og.Lighting2D lights
 * sprites but never a tilemap's layers (FINDINGS.md #F-tilelight), so the map is
 * tinted to the ambient light (src/props.ts MapShade) and these add the light back.
 */
export const Pool = defineComponent("Pool", {
  kind: u32(0, { doc: "0 a fixed light (a torch, a crystal), 1 the hero's lantern, 2 a checkpoint lantern (lit by Game.litMask)." }),
  index: u32(0, { doc: "A checkpoint lantern's index." }),
  strength: f32(0.5, { doc: "Its opacity in the dark." }),
  radius: f32(4, { unit: "m" }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
