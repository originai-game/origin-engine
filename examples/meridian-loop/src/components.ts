// Meridian Loop's state: every number the run, the cars and the screens
// keep is a field here (og-core §4), so snapshots, replays and the probe
// carry it; nothing lives in module variables.

import { bool, defineComponent, defineResource, f32, f64, i32, str, u32 } from "@origingame/engine";

export const Phase = { Title: 0, Countdown: 1, Racing: 2, Finished: 3 } as const;

/** Ticks of the countdown: 3, 2, 1 a second each, then GO. */
export const COUNTDOWN = 180;

/** The longest lap a tape holds (90 s at 60 Hz); a slower lap keeps no ghost. */
export const TAPE_TICKS = 5400;

/** Sounds a system asks for this tick (Race.sfx bits), played by the Voices system. */
export const Sound = {
  Beep: 1,
  Go: 2,
  Sector: 4,
  Lap: 8,
  Best: 16,
  Slower: 32,
  Bump: 64,
  Land: 128,
  Error: 256,
} as const;

/** The run, the screens and what the platform answered. */
export const Race = defineResource("Race", {
  phase: u32(0),
  paused: bool(false),
  settingsOpen: bool(false),
  boardOpen: bool(false),
  settingsClosedAt: u32(0),
  /** Time scale held at 0 under a menu. */
  held: bool(false),
  /** Ticks left in the countdown. */
  countdown: u32(0),
  /** Ticks since GO (0 before). */
  clock: u32(0),
  /** The lap being driven, 1 to LAPS; LAPS + 1 once finished. */
  lap: u32(1),
  /** Race ticks when this lap began, and when its splits were crossed (0: not yet). */
  lapStart: u32(0),
  split1: u32(0),
  split2: u32(0),
  /** The run's laps (ticks; 0 not driven). */
  lap1: u32(0),
  lap2: u32(0),
  lap3: u32(0),
  /** The run's best lap (ticks) and its best sectors. */
  runBest: u32(0),
  bestS1: u32(0),
  bestS2: u32(0),
  bestS3: u32(0),
  /** The best lap ever (ticks; the store's or this session's) and its sectors, which the splits compare with. */
  record: u32(0),
  recordS1: u32(0),
  recordS2: u32(0),
  recordS3: u32(0),
  /** The best run's points (the results' score). */
  recordPoints: u32(0),
  /** The last split's delta against the record (ticks, negative faster) and which split (1, 2, 3 for the line). */
  delta: i32(0),
  deltaAt: u32(0),
  deltaShow: u32(0),
  /** The current lap counts (no reset, no shortcut). */
  valid: bool(true),
  newRecord: bool(false),
  total: u32(0),
  /** Ticks since the finish (the results follow). */
  finishedFor: u32(0),
  /** The car's sample index along the circuit, and its previous one. */
  at: i32(-1),
  wrongWay: u32(0),
  /** Ticks the car has been off the road or upside down. */
  offTicks: u32(0),
  /** Counters the playtests and og polish read. */
  laps: u32(0),
  resets: u32(0),
  bumps: u32(0),
  jumps: u32(0),
  landings: u32(0),
  airTicks: u32(0),
  longestAir: u32(0),
  skidTicks: u32(0),
  splits: u32(0),
  /** The tapes' lengths: the current lap's poses, the best lap's, the run's input ticks. */
  curLen: u32(0),
  bestLen: u32(0),
  runLen: u32(0),
  /** The ghost: shown, replaying (tick of its tape), its own progress, when it crossed the line. */
  ghostActive: bool(false),
  ghostTick: u32(0),
  ghostAt: i32(-1),
  ghostLaps: u32(0),
  ghostLapTicks: u32(0),
  ghostEndError: f32(0),
  /** The test hook: the autopilot drives instead of the player. */
  autopilot: bool(false),
  /** What the driver asks this tick, quantised (src/tape.ts), and its code for the run's tape. */
  inSteer: f32(0),
  inThrottle: f32(0),
  inBrake: f32(0),
  code: u32(0),
  /** The replay: the world restored to the race's countdown (src/replay.ts) and the run's inputs driving the car. */
  replayMark: bool(false),
  replaying: bool(false),
  replayWait: bool(false),
  replayLen: u32(0),
  replayTotal: u32(0),
  /** How the last replay ended: 0 not yet, 1 on the original's tick, 2 elsewhere. */
  replayMatched: u32(0),
  replayReady: bool(false),
  replaySlotOk: bool(false),
  replaySave: u32(0),
  replayTapeWrite: u32(0),
  replayLoad: u32(0),
  replayTapeRead: u32(0),
  replays: u32(0),
  /** The size of the replay's saved world (world.snapshot()'s text). */
  snapBytes: u32(0),
  /** The racing HUD's touch controls this tick. */
  touchSteer: f32(0),
  touchThrottle: f32(0),
  /** The last landing: its tick and the ticks in the air before it. */
  landedAt: u32(0),
  landedAir: u32(0),
  /** Skid marks: metres slid since the last pair was laid, and the next slot of the ring. */
  markTravel: f32(0),
  nextMark: u32(0),
  /** HUD readouts. */
  kmh: f32(0),
  gear: u32(1),
  rpm: f32(0),
  slip: f32(0),
  skidding: bool(false),
  airborne: bool(false),
  /** Sounds asked for this tick (Sound bits). */
  sfx: u32(0),
  /** og.platform requests in flight (0: none) and what they answered. */
  recordRequest: u32(0),
  recordRevision: u32(0),
  recordWrite: u32(0),
  /** A conflicting write: read again, then written over if ours is faster. */
  recordAgain: bool(false),
  submitRequest: u32(0),
  boardRequest: u32(0),
  boardScope: u32(0),
  boardState: u32(0),
  board: str(1024),
  rank: i32(-1),
  saves: u32(0),
  loaded: bool(false),
  conflicts: u32(0),
  /** og.ui's accessibility counts, for the locales playtest. */
  a11yNodes: u32(0),
  a11yUnnamed: u32(0),
  /** Asset handles loaded in `start`, by number (a handle is past 2^32: an f64, FINDINGS #5). */
  minimap: f64(0),
});

/** The player's settings (og.platform.saveSettings) and the ghost toggle. */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(0.9, { min: 0, max: 1 }),
  muted: bool(false),
  reducedMotion: bool(false),
  highContrast: bool(false),
  language: u32(0),
  ghost: bool(true),
  loadRequest: u32(0),
  saves: u32(0),
});

/** The car's handling and the chase camera, tunable through the probe. */
export const Tuning = defineResource("Tuning", {
  engineForce: f32(5200, { min: 0, max: 20000, unit: "N", tunable: true, doc: "Force per driven wheel at full throttle." }),
  brakeForce: f32(2600, { min: 0, max: 20000, tunable: true }),
  maxSpeed: f32(46, { min: 5, max: 90, unit: "mps", tunable: true }),
  grip: f32(0.62, { min: 0.05, max: 4, tunable: true }),
  maxSteer: f32(0.5, { min: 0.05, max: 1.2, unit: "rad", tunable: true }),
  /** Steering at speed: the lock falls to 1 / (1 + speed / steerFalloff). */
  steerFalloff: f32(26, { min: 1, max: 200, unit: "mps", tunable: true }),
  /** How fast the wheel turns to the asked lock, per second (full lock in 1 / steerRate s). */
  steerRate: f32(4.5, { min: 0.5, max: 50, tunable: true }),
  /** Roll and pitch rate removed per tick in the air and on the ground (anti-roll), 0..1. */
  rollDamp: f32(0.12, { min: 0, max: 1, tunable: true }),
  /** Extra pull towards the road with all four wheels down, m/s² per (m/s)². */
  downforce: f32(0.004, { min: 0, max: 0.05, tunable: true }),
  cameraBack: f32(7.5, { min: 2, max: 30, unit: "m", tunable: true }),
  cameraHeight: f32(2.6, { min: 0.5, max: 20, unit: "m", tunable: true }),
  cameraLag: f32(0.12, { min: 0, max: 2, unit: "s", tunable: true }),
});

/** A car on the circuit (the player's or the ghost): its driver's intents and what the chassis reads back. */
export const Racer = defineComponent("Racer", {
  /** The ghost (replays a tape) or the player. */
  ghost: bool(false),
  /** The wheel's lock now, eased towards Race.inSteer. */
  steer: f32(0),
  /** Wheels on the ground, and ticks spent off it. */
  contacts: u32(4),
  air: u32(0),
  /** Sideways speed (m/s) at the rear axle: the slide. */
  slip: f32(0),
});

/** The longest run the replay keeps: three laps of 55 s. */
export const RUN_TICKS = 10000;

/**
 * A str field holds at most 4096 bytes (FINDINGS #4), so a tape is cut in
 * chunks of CHUNK bytes, one entity each (`tape-cur-<i>`, `tape-best-<i>`,
 * `tape-run-<i>`): CHUNK is a whole number of poses (15 bytes) and of input
 * ticks (2 bytes).
 */
export const CHUNK = 3990;
export const TAPES = { cur: 0, best: 1, run: 2 } as const;
export const CHUNKS = { cur: Math.ceil(((TAPE_TICKS / 2) * 15) / CHUNK), best: Math.ceil(((TAPE_TICKS / 2) * 15) / CHUNK), run: Math.ceil((RUN_TICKS * 2) / CHUNK) } as const;

/** One chunk of a tape (src/tape.ts). */
export const TapeChunk = defineComponent("TapeChunk", {
  tape: u32(0),
  index: u32(0),
  bytes: str(CHUNK),
});

/** A one-shot voice for a Sound bit. */
export const Sfx = defineComponent("Sfx", { bit: u32(0), plays: u32(0) });

/** A skid mark's slot in the ring of decals. */
export const Mark = defineComponent("Mark", { slot: u32(0), age: u32(0) });
