// Every piece of Holdout's state lives in these reflected types: dumps, the
// probe, Tune and the command channel's `get` and `set` read and write them
// by name.
//
// The run (Game) and the knobs (Tuning) are resources. The actors are rows
// spawned in `start`: a trooper per seat (Hero), a pool of walkers and
// drones the waves draw from (Foe), the shots' tracers, flashes and marks
// (Fx), and one voice per sound (Sfx). The player's settings are this
// device's own (src/device.ts), never the world's.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** What a run is: the co-op holdout against the waves, or a duel between the two seats. */
export const Mode = { Holdout: 0, Duel: 1 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Held: 1, Overrun: 2, Duel: 3 } as const;

/** Where a holdout is between its waves (Game.waveState). */
export const WaveState = { Breather: 0, Fighting: 1 } as const;

/** A foe's kind (Foe.kind). */
export const Kind = { Walker: 0, Drone: 1 } as const;

/** A foe's state (Foe.state). */
export const FoeState = { Pooled: 0, Advance: 1, Windup: 2, Recover: 3, Dying: 4 } as const;

/**
 * Sound requests: bits of Game.sfx that the Voices system plays once each.
 * `Tap` is the touch HUD answering on the ui bus, as og.ui's own buttons do.
 */
export const Sound = {
  Shot: 1, Reload: 2, Empty: 4, Hit: 8, Pop: 16, Hurt: 32, Down: 64, Swipe: 128, Charge: 256, Bolt: 512,
  Horn: 1024, Clear: 2048, Win: 4096, Lose: 8192, Step: 16384, Jump: 32768, Land: 65536, Tap: 131072,
} as const;

/** The waves of a holdout. */
export const WAVES = 5;
/** The pools the waves draw from: at most this many of each at once. */
export const WALKERS = 10;
export const DRONES = 5;
/** Tracers each trooper's rifle keeps (a shot takes the oldest), and the impact marks shared by all. */
export const TRACERS = 4;
export const MARKS = 24;
/** Frags that win a duel. */
export const DUEL_FRAGS = 5;

/** The run: its phase, its waves, the score and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  mode: u32(Mode.Holdout, { doc: "Mode.*: the co-op holdout or the duel." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  winner: i32(-1, { doc: "The seat that won a duel (-1 none)." }),
  paused: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused)." }),
  start: bool(false, { doc: "Every system resets its part of the arena this tick (a new run or the title)." }),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the run ended; the summary comes after a beat." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  wave: u32(0, { doc: "The wave fought or next (1 to WAVES; 0 before the first)." }),
  waveState: u32(WaveState.Breather, { doc: "WaveState.*: the breather before a wave, or the fight." }),
  waveTimer: f32(0, { unit: "s", doc: "The breather's seconds left." }),
  toSpawn: u32(0, { doc: "Foes of this wave still to come through the gates." }),
  spawnTimer: f32(0, { unit: "s", doc: "Seconds to the next foe through a gate." }),
  alive: u32(0, { doc: "Foes standing now." }),
  score: u32(0, { doc: "The holdout's score (shared by the seats)." }),
  kills: u32(0, { doc: "Foes downed this run (every seat)." }),
  headshots: u32(0),
  shots: u32(0, { doc: "Rounds fired this run (every seat)." }),
  hits: u32(0, { doc: "Rounds that hit a foe or the other duellist." }),
  downs: u32(0, { doc: "Times a trooper went down this run." }),
  runs: u32(0, { doc: "Runs started on this device (the store's `record`)." }),
  wins: u32(0, { doc: "Holdouts that held every wave (the store's `record`)." }),
  best: u32(0, { doc: "The best holdout score (the store's `record`)." }),
  newBest: bool(false),
  recordRequest: u32(0, { doc: "The store request reading `record`." }),
  recordRevision: u32(0, { doc: "The revision `record` was read or written at." }),
  recordWrite: u32(0, { doc: "The store request writing `record`." }),
  recordMerge: bool(false, { doc: "A write was refused as stale: the record is read again and the merge written." }),
  recordRuns: u32(0, { doc: "Runs in the record as last read or written." }),
  recordWins: u32(0, { doc: "Wins in the record as last read or written." }),
  recordDirty: bool(false, { doc: "The record changed while a read or write was in flight." }),
  recordSentRuns: u32(0),
  recordSentWins: u32(0),
  sfx: u32(0, { doc: "Sound.* bits the Voices system plays this tick." }),
  markNext: u32(0, { doc: "The bullet mark the next round into the arena takes (the oldest)." }),
  sparkNext: u32(0, { doc: "The impact sparks the next round into the arena restarts." }),
  sprayNext: u32(0, { doc: "The coolant spray the next round into a foe restarts." }),
  popNext: u32(0, { doc: "The burst the next downed foe restarts." }),
  hero: f64(0, { doc: "Seat 0's trooper's entity." }),
  hero1: f64(0, { doc: "Seat 1's trooper's entity (0 alone)." }),
  nav: f64(0, { doc: "The foes' navmesh." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  moveSpeed: f32(5.2, { min: 1, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Run speed" }),
  jumpSpeed: f32(5.4, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Jump speed" }),
  fireRate: f32(8, { min: 1, max: 20, step: 0.5, unit: "per s", tunable: true, label: "Rounds a second" }),
  damage: f32(17, { min: 1, max: 100, step: 1, tunable: true, label: "Damage a round", doc: "A headshot doubles it." }),
  magazine: u32(24, { min: 1, max: 60, tunable: true, label: "Magazine" }),
  reserve: u32(120, { min: 0, max: 600, tunable: true, label: "Reserve rounds", doc: "A run starts with it; each wave cleared refills it." }),
  reloadSeconds: f32(1.35, { min: 0.2, max: 4, step: 0.05, unit: "s", tunable: true, label: "Reload" }),
  recoil: f32(0.6, { min: 0, max: 4, step: 0.1, unit: "deg", tunable: true, label: "Recoil a round", doc: "How far a round kicks the view up." }),
  spread: f32(0.5, { min: 0, max: 4, step: 0.1, unit: "deg", tunable: true, label: "Spread at rest" }),
  bloom: f32(0.35, { min: 0, max: 2, step: 0.05, unit: "deg", tunable: true, label: "Bloom a round", doc: "Spread each round adds; it settles back at rest." }),
  health: f32(100, { min: 1, max: 500, step: 5, tunable: true, label: "Health" }),
  walkerHealth: f32(70, { min: 1, max: 500, step: 5, tunable: true, label: "Walker health" }),
  walkerSpeed: f32(2.8, { min: 0.5, max: 8, step: 0.1, unit: "m/s", tunable: true, label: "Walker speed", doc: "Each wave adds 0.2 m/s." }),
  walkerDamage: f32(14, { min: 0, max: 100, step: 1, tunable: true, label: "Walker strike" }),
  droneHealth: f32(45, { min: 1, max: 500, step: 5, tunable: true, label: "Drone health" }),
  droneDamage: f32(11, { min: 0, max: 100, step: 1, tunable: true, label: "Drone bolt" }),
  droneRange: f32(13, { min: 4, max: 30, step: 0.5, unit: "m", tunable: true, label: "Drone range" }),
  breather: f32(4, { min: 0.5, max: 20, step: 0.5, unit: "s", tunable: true, label: "Breather between waves" }),
  hitStop: u32(3, { min: 0, max: 12, unit: "ticks", tunable: true, label: "Hit stop on a kill" }),
});

/**
 * A trooper on og.CharacterController: one per seat, moved by its
 * og.PlayerInput alone. Its view (yaw, pitch) is src/view.ts's SeatLook on
 * the same entity; its rifle, recoil and health are here.
 */
export const Hero = defineComponent("Hero", {
  seat: u32(0, { doc: "The seat it plays for (its og.PlayerInput.player)." }),
  body: f64(0, { doc: "Its skinned body's entity." }),
  gun: f64(0, { doc: "Its rifle as the others see it (held by the body)." }),
  hands: f64(0, { doc: "Its rifle in its own first-person view." }),
  muzzle: f64(0, { doc: "The muzzle flash's light." }),
  armR: f64(0, { doc: "Its right arm's og.IkChain to the grip." }),
  armL: f64(0, { doc: "Its left arm's og.IkChain to the fore-grip." }),
  before: u32(0, { doc: "Its og.PlayerInput.buttons last tick (press edges)." }),
  health: f32(100),
  down: bool(false, { doc: "Out of the fight: back at the next wave (holdout) or after a beat (duel)." }),
  respawn: f32(0, { unit: "s", doc: "A downed duellist's seconds until it is back." }),
  mag: u32(24, { doc: "Rounds in the magazine." }),
  reserve: u32(120, { doc: "Rounds left to reload from." }),
  reload: f32(0, { unit: "s", doc: "Seconds of the reload left (0: not reloading)." }),
  cooldown: f32(0, { unit: "s", doc: "Seconds until the rifle can fire again." }),
  kick: f32(0, { unit: "rad", doc: "The recoil's climb still on the view; it settles back while not firing." }),
  bloom: f32(0, { unit: "deg", doc: "Spread added by firing; it settles back." }),
  firing: f32(0, { unit: "s", doc: "Seconds since the last round (the flash and the settle read it)." }),
  tracer: u32(0, { doc: "The next tracer of its pool." }),
  hitTick: i32(-1000, { unit: "ticks", doc: "When its last round hit (the crosshair's marker)." }),
  hitKind: u32(0, { doc: "0 a hit, 1 a headshot, 2 a kill." }),
  hurtTick: i32(-1000, { unit: "ticks", doc: "When it was last hurt (the damage arc, the flash)." }),
  hurtX: f32(0, { doc: "Where the hurt came from (x)." }),
  hurtZ: f32(0, { doc: "Where the hurt came from (z)." }),
  kills: u32(0),
  frags: u32(0, { doc: "Duel: the other duellist downed." }),
  score: u32(0, { doc: "Its own points (the holdout's score is their sum)." }),
  facing: f32(0, { unit: "rad", doc: "Its body's heading." }),
  speed: f32(0, { unit: "m/s" }),
  grounded: bool(false),
  airTime: f32(0, { unit: "s" }),
  stepClock: f32(0, { unit: "m" }),
  jumps: u32(0),
});

/** A walker or a drone: pooled, drawn through a gate by a wave and put back when downed. */
export const Foe = defineComponent("Foe", {
  index: u32(0, { doc: "Its place in its kind's pool." }),
  kind: u32(Kind.Walker, { doc: "Kind.*." }),
  state: u32(FoeState.Pooled, { doc: "FoeState.*." }),
  health: f32(0),
  timer: f32(0, { unit: "s", doc: "Seconds left in its windup, recovery or fall." }),
  target: u32(0, { doc: "The seat it hunts (the nearest standing trooper)." }),
  aimX: f32(0, { doc: "A drone's locked aim (x), where its bolt goes." }),
  aimY: f32(0),
  aimZ: f32(0),
  flash: f32(0, { doc: "The hit flash, fading." }),
  body: f64(0, { doc: "Its model's entity." }),
  beam: f64(0, { doc: "A drone's aiming beam." }),
  eye: f64(0, { doc: "Its eye's light." }),
  bob: f32(0, { doc: "A drone's hover phase." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
