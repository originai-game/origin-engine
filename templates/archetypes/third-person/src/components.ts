// Every piece of Ember Vale's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The run (Game), the knobs (Tuning) and the player's settings (Prefs) are
// resources. The actors are rows spawned in `start`: the hero (Hero), the
// four beacons (Beacon), the three sentinels (Sentinel) and one voice per
// sound (Sfx).

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Dawn: 1, Night: 2, Caught: 3 } as const;

/**
 * Sound requests: bits of Game.sfx that the Voices system plays once each.
 * `Tap` is the touch HUD's stick, Jump and Kindle answering on the ui bus, as
 * og.ui's own buttons do (og-ui's touch controls play no press sound of their
 * own).
 */
export const Sound = { Step: 1, Jump: 2, Splash: 4, Ignite: 8, Alarm: 16, Grab: 32, Dawn: 64, Night: 128, Tap: 256 } as const;

/** The beacons a run lights. */
export const BEACONS = 4;
/** The hearts a run starts with. */
export const HEARTS = 3;

/** The run: the hero's progress, the clock and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  leftTitle: bool(false, { doc: "The player has left the title the game opened on: from then on the title enters with the style's entrance (the opening one is built settled, og.ui.mainMenu's instant: the web draws it behind the play gate)." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  runs: u32(0, { doc: "Runs started on this device (the store's `record`)." }),
  wins: u32(0, { doc: "Runs that lit every beacon (the store's `record`)." }),
  best: f32(0, { unit: "s", doc: "The fastest win (the store's `record`; 0 none)." }),
  recordRequest: u32(0, { doc: "The store request reading `record`." }),
  recordRevision: u32(0, { doc: "The revision `record` was read or written at." }),
  recordWrite: u32(0, { doc: "The store request writing `record`." }),
  recordMerge: bool(false, { doc: "A write was refused as stale: the record is read again and the merge written." }),
  recordRuns: u32(0, { doc: "Runs in the record as last read or written: Game.runs less this is what this device adds." }),
  recordWins: u32(0, { doc: "Wins in the record as last read or written." }),
  recordConflicts: u32(0, { doc: "Record writes refused as stale (another device or tab wrote first)." }),
  recordDirty: bool(false, { doc: "The record changed while a read or write of it was in flight: written when that answers." }),
  recordSentRuns: u32(0, { doc: "Runs in the record write in flight (what the store holds once it answers ok)." }),
  recordSentWins: u32(0, { doc: "Wins in the record write in flight." }),
  newBest: bool(false),
  hearts: i32(HEARTS),
  hurts: u32(0, { doc: "Hearts lost this run." }),
  lit: u32(0, { doc: "Beacons lit this run." }),
  litMask: u32(0, { doc: "Bit i: beacon i burns." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  hour: f32(0, { doc: "The time of day (og.atmosphere.timeOfDay): night falls at Tuning.nightHour." }),
  near: i32(-1, { doc: "The beacon within reach (-1 none)." }),
  kindle: f32(0, { unit: "s", doc: "How long the near beacon has been kindled." }),
  kindling: bool(false, { doc: "The hero is kindling (holding F, X or Kindle by a beacon)." }),
  wading: bool(false, { doc: "The hero's feet are in the lake." }),
  grounded: bool(false),
  speed: f32(0, { unit: "m/s", doc: "The hero's speed across the ground." }),
  jumps: u32(0),
  splashes: u32(0),
  grabs: u32(0, { doc: "Sentinel grabs that cost a heart." }),
  alarms: u32(0, { doc: "Times a sentinel called the alarm." }),
  invulnerable: f32(0, { unit: "s", doc: "Seconds before the hero can be grabbed again." }),
  knock: f32(0, { unit: "s", doc: "Seconds the hero is thrown back (no control)." }),
  knockX: f32(0),
  knockZ: f32(0),
  yaw: f32(0, { unit: "rad", doc: "The view's heading: 0 looks north (-z)." }),
  facing: f32(0, { unit: "rad", doc: "The hero's heading." }),
  padJump: bool(false, { doc: "The HUD's Jump was pressed this tick." }),
  padKindle: bool(false, { doc: "The HUD's Kindle is held." }),
  stickX: f32(0, { doc: "The HUD joystick (written by Ui)." }),
  stickY: f32(0),
  lookId: i32(-1, { doc: "The finger turning the view on a touch screen (an og.Pointers id; -1 none)." }),
  lookX: f32(0, { unit: "cssPx", doc: "Where that finger was last tick." }),
  lookTurn: f32(0, { unit: "rad", doc: "How far the finger turned the view this tick (written by Ui, applied by View)." }),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the run ended; the summary comes after a beat." }),
  start: bool(false, { doc: "Every system resets its part of the vale this tick (a new run or the title)." }),
  sfx: u32(0),
  hero: f64(0, { doc: "The hero's entity." }),
  terrain: f64(0, { doc: "The vale's og.terrain data." }),
  nav: f64(0, { doc: "The sentinels' navmesh." }),
  viewW: f32(0, { unit: "cssPx" }),
  viewH: f32(0, { unit: "cssPx" }),
  language: u32(0, { doc: "Prefs.language, mirrored for the facts." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  walkSpeed: f32(3, { min: 1, max: 8, step: 0.1, unit: "m/s", tunable: true, label: "Walk speed" }),
  runSpeed: f32(6, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Run speed" }),
  wade: f32(0.45, { min: 0.1, max: 1, step: 0.05, tunable: true, label: "Wading slows to", doc: "The hero's speed in the lake, as a fraction." }),
  jumpSpeed: f32(5.2, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Jump speed" }),
  kindleSeconds: f32(1.6, { min: 0.2, max: 6, step: 0.1, unit: "s", tunable: true, label: "Kindling takes" }),
  reach: f32(2.4, { min: 1, max: 5, step: 0.1, unit: "m", tunable: true, label: "Kindling reach", doc: "How near a beacon's centre the hero kindles it." }),
  duskSeconds: f32(150, { min: 20, max: 900, step: 5, unit: "s", tunable: true, label: "Dusk lasts", doc: "Seconds of play from the first light to night." }),
  patrolSpeed: f32(1.6, { min: 0.5, max: 6, step: 0.1, unit: "m/s", tunable: true, label: "Sentinel patrol speed" }),
  chaseSpeed: f32(4.2, { min: 1, max: 9, step: 0.1, unit: "m/s", tunable: true, label: "Sentinel chase speed" }),
  sight: f32(13, { min: 2, max: 40, step: 0.5, unit: "m", tunable: true, label: "Sentinel sight" }),
  hitStop: u32(5, { min: 0, max: 20, unit: "ticks", tunable: true, label: "Hit stop" }),
  cameraDistance: f32(6.2, { min: 2, max: 14, step: 0.1, unit: "m", tunable: true, label: "Camera distance" }),
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

/** The hero, on the capsule (its body, the strider, is a child). */
export const Hero = defineComponent("Hero", {
  steps: u32(0, { doc: "Footsteps: one per stride of ground covered." }),
  airTime: f32(0, { unit: "s", doc: "Seconds since the capsule last stood on the ground." }),
  stepClock: f32(0, { unit: "m", doc: "Ground covered since the last footstep." }),
});

/** A beacon: its plinth, bowl, fire, smoke, light and decals hang off it. */
export const Beacon = defineComponent("Beacon", {
  index: u32(0),
  lit: bool(false),
  litAt: f32(0, { unit: "s", doc: "Game.time when it caught." }),
  top: f32(0, { unit: "m", doc: "The bowl's rim, world y." }),
});

/** A sentinel: a stone stout on the navmesh, thinking with assets/ai/sentinel-<name>.ogt. */
export const Sentinel = defineComponent("Sentinel", {
  index: u32(0),
  grabs: u32(0),
  alarms: u32(0),
  speed: f32(0, { unit: "m/s", doc: "Its speed across the ground (the graph's p0)." }),
  alert: bool(false, { doc: "It is chasing or grabbing (its eye burns red)." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
