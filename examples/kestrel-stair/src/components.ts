// Every piece of The Kestrel Stair's state lives in these reflected types:
// dumps, the probe, Tune, `get` and `set`, snapshots (the save slot is
// world.snapshot()) and the world hash all read them by name.
//
// The run (Game), the knobs (Tuning) and the player's settings (Prefs) are
// resources. The actors are rows spawned in `start`: the courier (Hero),
// the foes (Foe), the sigils and herbs (Pickup), the waystones (Waystone)
// and one voice per sound (Sfx).

import { bool, defineComponent, defineResource, f32, f64, i32, str, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Ending: 2, Over: 3 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Rung: 1, Night: 2, Fallen: 3 } as const;

/** Which cutscene plays (Game.scene). */
export const Scene = { None: 0, Intro: 1, End: 2 } as const;

/** What is in the courier's reach (Game.nearKind). */
export const Near = { None: 0, Sigil: 1, Herb: 2, Waystone: 3, Gate: 4, Bell: 5 } as const;

/** Foe kinds (Foe.kind). */
export const Kind = { Imp: 0, Warden: 1 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = {
  Step: 1 << 0, Swing: 1 << 1, Hit: 1 << 2, Hurt: 1 << 3, Death: 1 << 4, Jump: 1 << 5, Land: 1 << 6,
  Sigil: 1 << 7, Herb: 1 << 8, Heal: 1 << 9, Gate: 1 << 10, Waystone: 1 << 11, Slam: 1 << 12,
  Alert: 1 << 13, Bite: 1 << 14, Win: 1 << 15, Lose: 1 << 16, Bell: 1 << 17, Error: 1 << 18,
} as const;

/** The courier's strength. */
export const MAX_HP = 100;
/** The run's clock: the hours og.DayNight is held at (late afternoon to night). */
export const HOURS = { start: 16.5, night: 19.5, title: 17.1, end: 19.2 } as const;

/** The run: progress, the clock, the screens and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  chapter: u32(0, { doc: "The chapter the run started at (layout CHAPTERS)." }),
  paused: bool(false),
  settingsOpen: bool(false),
  bagOpen: bool(false),
  chaptersOpen: bool(false),
  recordsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (a menu, the bag, a cutscene)." }),
  scene: u32(Scene.None, { doc: "The cutscene playing (Scene.*)." }),
  line: u32(0, { doc: "The cutscene's line." }),
  tutorial: bool(false, { doc: "The tutorial overlay is open." }),
  tutorialStep: u32(0),
  hp: f32(MAX_HP, { min: 0, max: MAX_HP }),
  sigils: u32(0, { doc: "Bit i: sigil i is carried (0 crypt, 1 ledge, 2 warden)." }),
  herbs: u32(0, { doc: "Hillwort in the bag." }),
  herbsUsed: u32(0),
  gathered: u32(0, { doc: "Bit i: herb i was gathered this run." }),
  foesDown: u32(0, { doc: "Bit i: foe i is down." }),
  kills: u32(0),
  hits: u32(0, { doc: "Strikes that landed." }),
  hurts: u32(0, { doc: "Times the courier was hurt." }),
  swings: u32(0),
  jumps: u32(0),
  landings: u32(0),
  steps: u32(0),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  hour: f32(HOURS.title, { doc: "The time of day og.DayNight is held at." }),
  reached: u32(1, { doc: "Bit i: chapter i was reached this run." }),
  rested: u32(0, { doc: "Bit i: waystone i was rested at this run." }),
  nearKind: u32(Near.None, { doc: "Near.*: what F would use now." }),
  nearIndex: i32(-1),
  refusedAt: u32(0, { unit: "ticks", doc: "The tick F was refused (the gate without three sigils): the prompt shakes." }),
  invulnerable: f32(0, { unit: "s" }),
  knock: f32(0, { unit: "s", doc: "Seconds the courier is thrown back (no control)." }),
  knockX: f32(0),
  knockZ: f32(0),
  attack: f32(0, { unit: "s", doc: "Seconds into the courier's swing (0: not swinging)." }),
  struck: bool(false, { doc: "This swing has landed its hit." }),
  facing: f32(Math.PI, { unit: "rad", doc: "The courier's heading (0 faces +z)." }),
  yaw: f32(0, { unit: "rad", doc: "The view's heading about the courier (0 looks north, -z)." }),
  grounded: bool(false),
  speed: f32(0, { unit: "m/s" }),
  heroX: f32(0),
  heroY: f32(0),
  heroZ: f32(0),
  gateOpen: bool(false),
  gateLift: f32(0, { unit: "m" }),
  inCrypt: bool(false, { doc: "The courier is inside the crypt: the reverb zone." }),
  reverb: f32(0.12, { doc: "The effects bus's reverb send, eased towards the zone's." }),
  overTicks: u32(0, { unit: "ticks" }),
  start: bool(false, { doc: "Every system resets its part of the stair this tick (a new run or the title)." }),
  sfx: u32(0),
  hero: f64(0, { doc: "The courier's entity." }),
  nav: f64(0, { doc: "The navmesh (og.nav.load of nav/stair.ogt)." }),
  look: u32(0, { doc: "The look applied: 0 the stair (looks/kestrel.ogt), 1 the summit (looks/summit.ogt)." }),
  lookStair: f64(0),
  lookSummit: f64(0),
  music: f64(0, { doc: "The music's sound handle." }),
  musicChunks: u32(0),
  // The save slot and the records (og.platform), and what the title shows of them.
  saves: u32(0),
  loads: u32(0),
  hasSave: bool(false),
  savePlace: u32(0, { doc: "The slot's waystone." }),
  saveClock: f32(0, { doc: "The slot's hour." }),
  saveRequest: u32(0),
  slotRequest: u32(0),
  saveParts: u32(0, { doc: "How many store keys the slot's world text takes (slot.0, slot.1, ...)." }),
  restoring: bool(false),
  partRequest0: u32(0),
  partRequest1: u32(0),
  partRequest2: u32(0),
  partRequest3: u32(0),
  fromSave: bool(false, { doc: "Set only inside a saved world: the tick after a restore sees it (src/save.ts)." }),
  saveBytes: u32(0, { unit: "bytes", doc: "The last save's world.snapshot() text length." }),
  conflicts: u32(0, { doc: "Progress writes refused as stale, read again and merged." }),
  progressAgain: bool(false),
  progressRequest: u32(0),
  progressWrite: u32(0),
  progressRevision: u32(0),
  chapterStars: u32(0, { doc: "Four chapters, two bits of stars each (the best this player earned)." }),
  chaptersReached: u32(1, { doc: "Bit i: chapter i was ever reached (progress in the store)." }),
  tutorialDone: bool(false),
  best: f32(0, { unit: "s", doc: "The fastest ascent (s), 0 none." }),
  newBest: bool(false),
  bestScore: u32(0, { doc: "The best score (the results' Best)." }),
  score: u32(0),
  submitRequest: u32(0),
  submitted: u32(0),
  rank: i32(-1),
  recordsScope: u32(0),
  recordsRequest: u32(0),
  recordsState: u32(0, { doc: "0 idle, 1 loading, 2 loaded, 3 failed." }),
  records: str(1536, "", { doc: "The rows read, as JSON: [[rank, name, seconds, me], ...]." }),
  bag: str(160, "", { doc: "The bag's slots: item ids by slot, as JSON (og.ui.inventoryScreen moves them)." }),
  bagMoves: u32(0),
  welcomed: u32(0, { doc: "The loads already welcomed with a toast." }),
  a11yUnnamed: u32(0),
  a11yNodes: u32(0),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  walkSpeed: f32(3.2, { min: 1, max: 8, step: 0.1, unit: "m/s", tunable: true }),
  runSpeed: f32(6.2, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true }),
  jumpSpeed: f32(6.4, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, doc: "1.9 m high: the ledges rise 1.0 to 1.2 m." }),
  swing: f32(0.42, { min: 0.2, max: 1, step: 0.01, unit: "s", tunable: true, doc: "A swing's length; the hit lands on the clip's marker." }),
  reach: f32(1.5, { min: 0.5, max: 4, step: 0.1, unit: "m", tunable: true, doc: "The strike's reach in front of the courier." }),
  biteDamage: f32(12, { min: 0, max: 100, tunable: true }),
  slamDamage: f32(34, { min: 0, max: 100, tunable: true }),
  herbHeal: f32(40, { min: 0, max: 100, tunable: true }),
  hitStop: u32(4, { min: 0, max: 20, unit: "ticks", tunable: true, doc: "A strike on an imp." }),
  heavyStop: u32(7, { min: 0, max: 30, unit: "ticks", tunable: true, doc: "A strike on a warden, and a slam taken." }),
  duskSeconds: f32(300, { min: 30, max: 1200, step: 5, unit: "s", tunable: true, doc: "Seconds of play from 16:30 to night at 19:30." }),
  impPatrol: f32(1.8, { min: 0.2, max: 8, step: 0.1, unit: "m/s", tunable: true }),
  impChase: f32(4.4, { min: 0.5, max: 10, step: 0.1, unit: "m/s", tunable: true }),
  wardenPatrol: f32(1.1, { min: 0.2, max: 6, step: 0.1, unit: "m/s", tunable: true }),
  wardenChase: f32(2.4, { min: 0.5, max: 8, step: 0.1, unit: "m/s", tunable: true }),
  sight: f32(11, { min: 0, max: 40, step: 0.5, unit: "m", tunable: true }),
  cameraDistance: f32(5.6, { min: 2, max: 20, step: 0.1, unit: "m", tunable: true }),
  cameraPitch: f32(0.34, { min: 0, max: 1.4, step: 0.01, unit: "rad", tunable: true }),
});

/** The player's settings, kept in og.platform's settings (loadSettings and saveSettings). */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  muted: bool(false),
  language: u32(0, { doc: "0 English, 1 日本語 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  highContrast: bool(false),
  sensitivity: f32(1, { min: 0.1, max: 3 }),
  invertY: bool(false),
  loadRequest: u32(0),
  saves: u32(0),
});

/** The courier, on its capsule (the skinned body and the cape are children). */
export const Hero = defineComponent("Hero", {
  airTime: f32(0, { unit: "s" }),
  stepClock: f32(0, { unit: "m", doc: "Ground covered since the last footstep." }),
  fall: f32(0, { unit: "m/s", doc: "The fastest fall since leaving the ground." }),
  clipAttack: f64(0, { doc: "The courier's clips (models/courier.glb's, retargeted at cook time)." }),
  clipHurt: f64(0),
  clipDie: f64(0),
});

/** A foe: a thorn imp or a stone warden on the navmesh, minded by og.ai. */
export const Foe = defineComponent("Foe", {
  kind: u32(0),
  index: u32(0),
  hp: f32(3, { doc: "Thorns or stones left; the behaviours read it (`check Foe.hp below 2`)." }),
  cooldown: f32(0, { unit: "s", doc: "Seconds before it may bite again; the imps' behaviour reads it." }),
  stun: f32(0, { unit: "s", doc: "Seconds it reels from a strike (its NavAgent leaves the motion to the game)." }),
  stunX: f32(0),
  stunZ: f32(0),
  slam: f32(0, { unit: "s", doc: "Seconds into a warden's slam (0: none)." }),
  slamHit: bool(false),
  dying: f32(0, { unit: "s", doc: "Seconds since it fell (0: alive)." }),
  hitsTaken: u32(0),
  bites: u32(0),
  alerts: u32(0),
  carries: bool(false, { doc: "It carries the warden sigil (drops it when it falls)." }),
  warm: u32(0, { doc: "Its NavAgent's plans when it was (re)placed: its mind wakes after the next plan (FINDINGS #4)." }),
  homeX: f32(0),
  homeY: f32(0),
  homeZ: f32(0),
  graph: f64(0, { doc: "Its blend tree (og.anim.graph)." }),
  clipAttack: f64(0, { doc: "Its attack clip (retargeted): bite or slam." }),
  clipHurt: f64(0),
  clipDie: f64(0),
  mind: f64(0),
  spare: f64(0, { doc: "A second handle of its behaviour: swapping starts its machine afresh (a new run)." }),
});

/** A sigil or a herb lying on the stair. */
export const Pickup = defineComponent("Pickup", {
  kind: u32(0, { doc: "1 sigil, 2 herb (Near.*)." }),
  index: u32(0),
  taken: bool(false),
  x: f32(0),
  y: f32(0),
  z: f32(0),
});

/** A waystone: a chapter's start and a save point. */
export const Waystone = defineComponent("Waystone", {
  index: u32(0),
  glow: f32(0),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});

/**
 * No entity has it. A system that works on named entities one by one
 * (world.get / world.set: the courier, the camera's listener, a lamp)
 * lists it among its reads, so its query matches nothing: a component a
 * system writes is borrowed by every archetype its query matches, and
 * world.get / world.set of it then refuse, even if the system never takes
 * a view (FINDINGS #3).
 */
export const OneByOne = defineComponent("OneByOne", { unused: u32(0) });
