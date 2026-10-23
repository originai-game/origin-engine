// Every piece of Spark Orb's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The run and the save (Game), the knobs (Tuning) and the player's
// settings (Prefs) are resources. The drawn things are rows spawned in
// `start`: the orb, its glow, the motes that circle it (one pool per
// generator) and one voice per sound.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Nova: 2 } as const;

/** The generators, in order: what each is called in the string tables and drawn with. */
export const GENERATORS = ["ember", "storm", "crystal", "star"] as const;

/** Motes drawn per generator at most (one per generator owned). */
export const MOTES = 12;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Tap: 1, Buy: 2, Milestone: 4, Nova: 8, Welcome: 16, Move: 32 } as const;

/** The run, the save and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  leftTitle: bool(false, { doc: "The player has left the title the game opened on: from then on the title enters with the style's entrance (the opening one is built settled, og.ui.mainMenu's instant: the web draws it behind the play gate)." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused, in the settings or a dialog)." }),
  confirmNova: bool(false, { doc: "The Supernova dialog is open." }),
  welcome: bool(false, { doc: "The welcome-back dialog is open." }),
  sparks: f64(0, { doc: "The currency." }),
  runEarned: f64(0, { doc: "Sparks made this run (since the last supernova): the supernova's stardust grows with it." }),
  lifetime: f64(0, { doc: "Sparks made on this save, every run." }),
  owned0: u32(0, { doc: "Embers owned." }),
  owned1: u32(0, { doc: "Storms owned." }),
  owned2: u32(0, { doc: "Crystals owned." }),
  owned3: u32(0, { doc: "Stars owned." }),
  dust: u32(0, { doc: "Stardust: +10 % to everything each (Tuning.dustBonus)." }),
  novas: u32(0, { doc: "Supernovas on this save." }),
  lastGain: u32(0, { doc: "Stardust the last supernova gave." }),
  lastRun: f64(0, { doc: "Sparks the last supernova's run made." }),
  lastRunTicks: u32(0, { unit: "ticks" }),
  runTicks: u32(0, { unit: "ticks", doc: "Ticks played this run." }),
  rate: f64(0, { unit: "sparks/s", doc: "Sparks a second now (the generators, the milestones, the stardust)." }),
  tapValue: f64(1, { doc: "Sparks a tap makes now." }),
  taps: u32(0),
  streak: u32(0, { doc: "Taps in a quick run (each within 20 ticks of the last): the chime climbs." }),
  lastTapTick: i32(-100, { unit: "ticks" }),
  buys: u32(0),
  boughtSlot: i32(-1, { doc: "The generator bought last (its card pops)." }),
  boughtTick: i32(-1, { unit: "ticks" }),
  refused: u32(0, { doc: "Purchases refused for want of sparks." }),
  refusedSlot: i32(-1, { doc: "The generator whose purchase was refused last (its card shakes)." }),
  refusedTick: i32(-1, { unit: "ticks" }),
  milestoneSlot: i32(-1, { doc: "The generator that reached a milestone last (its card bursts)." }),
  milestoneTick: i32(-1, { unit: "ticks" }),
  selected: u32(0, { doc: "The generator the keys and the pad buy (0-3)." }),
  playTick: i32(-1, { unit: "ticks", doc: "The tick play began (from the title or a new run): the key or button that began it is not also a tap or a purchase." }),
  pendingTap: bool(false, { doc: "A tap asked for by the HUD this tick (the Orb system makes it)." }),
  pendingBuy: i32(-1, { doc: "A purchase asked for by the HUD this tick (the Economy system makes it)." }),
  pendingNova: bool(false, { doc: "The supernova confirmed this tick (the Economy system makes it)." }),
  away: f64(0, { unit: "s", doc: "Seconds the game was closed: the save's answer's time (og.platform's host clock) less the time the save was written; relaunch:<s> sets it for a test." }),
  offline: f64(0, { doc: "Sparks the orb made while the game was closed (shown on the welcome-back dialog)." }),
  saveRequest: u32(0, { doc: "The store request reading `orb`." }),
  saveRevision: u32(0, { doc: "The revision `orb` was read or written at." }),
  saveWrite: u32(0, { doc: "The store request writing `orb`." }),
  saveMerge: bool(false, { doc: "A write was refused as stale: the save is read again, merged and written." }),
  saveConflicts: u32(0),
  saves: u32(0, { doc: "Saves written." }),
  loaded: bool(false, { doc: "The save was read (or found empty) on launch." }),
  sinceSave: u32(0, { unit: "ticks" }),
  saveNow: bool(false, { doc: "Write the save this tick (after a supernova, on pause, on Quit)." }),
  burstKind: u32(0, { doc: "The burst asked for this tick (BURST.* of src/systems.ts)." }),
  burstX: f32(0, { unit: "m" }),
  burstY: f32(0, { unit: "m" }),
  burstTick: i32(-1, { unit: "ticks", doc: "The tick a burst was asked for (the Bursts system fires it the same tick)." }),
  sfx: u32(0),
  viewW: f32(0, { unit: "cssPx" }),
  viewH: f32(0, { unit: "cssPx" }),
  orbX: f32(0, { unit: "m", doc: "The orb's centre (it moves up on a phone)." }),
  orbY: f32(0, { unit: "m" }),
  language: u32(0, { doc: "Prefs.language, mirrored for the facts." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  emberRate: f32(0.5, { min: 0, max: 1000, tunable: true, unit: "sparks/s", label: "Ember makes" }),
  emberCost: f32(15, { min: 1, max: 1e6, tunable: true, label: "Ember costs" }),
  stormRate: f32(4, { min: 0, max: 1e4, tunable: true, unit: "sparks/s", label: "Storm makes" }),
  stormCost: f32(110, { min: 1, max: 1e7, tunable: true, label: "Storm costs" }),
  crystalRate: f32(26, { min: 0, max: 1e5, tunable: true, unit: "sparks/s", label: "Crystal makes" }),
  crystalCost: f32(1200, { min: 1, max: 1e8, tunable: true, label: "Crystal costs" }),
  starRate: f32(160, { min: 0, max: 1e6, tunable: true, unit: "sparks/s", label: "Star makes" }),
  starCost: f32(13000, { min: 1, max: 1e9, tunable: true, label: "Star costs" }),
  costGrowth: f32(1.15, { min: 1, max: 2, step: 0.01, tunable: true, label: "Price growth", doc: "Each one bought multiplies the next one's price." }),
  milestone: u32(10, { min: 1, max: 100, tunable: true, label: "Milestone every", doc: "Every this many of a generator doubles what that generator makes." }),
  tapBase: f32(1, { min: 0, max: 1000, tunable: true, label: "A tap makes" }),
  tapShare: f32(0.05, { min: 0, max: 1, step: 0.01, tunable: true, label: "Tap's share of the rate", doc: "A tap also makes this many seconds' worth of the rate." }),
  dustBonus: f32(0.1, { min: 0, max: 1, step: 0.01, tunable: true, label: "Stardust bonus", doc: "Each stardust adds this to every multiplier." }),
  novaScale: f32(100000, { min: 1, max: 1e12, tunable: true, label: "Supernova scale", doc: "Stardust from a supernova is floor(sqrt(run sparks / this))." }),
  offlineShare: f32(0.5, { min: 0, max: 1, step: 0.05, tunable: true, label: "Offline share", doc: "The share of the rate the orb makes while the game is closed." }),
  offlineCap: f32(28800, { min: 0, max: 604800, tunable: true, unit: "s", label: "Offline cap" }),
  saveEvery: u32(600, { min: 60, max: 36000, tunable: true, unit: "ticks", label: "Save every" }),
});

/** The player's settings: kept in og.platform's settings, restored on the first ticks. */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  loadRequest: u32(0),
  saves: u32(0),
});

/** The orb: a tap squashes and flashes it. */
export const Orb = defineComponent("Orb", {
  taps: u32(0),
});

/** A mote circling the orb: one per generator owned, up to MOTES per generator. */
export const Mote = defineComponent("Mote", {
  tier: u32(0, { doc: "The generator (0-3)." }),
  index: u32(0, { doc: "Its place in the tier's ring." }),
});

/** A pooled particle burst of one kind (BURST.* of src/systems.ts). */
export const Burst = defineComponent("Burst", {
  kind: u32(0),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
