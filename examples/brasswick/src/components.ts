// Brasswick's state: the run (Game), the player's settings (Prefs), the
// tuning (Tuning), the requests for effects and sounds (Fx, Sfx), and the
// components of every thing in the three levels. Everything a tick reads is
// here, in the world, so snapshots, replays and the save carry it.

import { bool, defineComponent, defineResource, f32, f64, i32, str, u32 } from "@origingame/engine";

/** The screens and what the run is doing. */
export const Phase = { Title: 0, Play: 1, Results: 2 } as const;

export const LEVELS = 3;
/** Gears in each level (the maps' `gear` objects; src/level.ts checks the count). */
export const GEARS = [6, 8, 8] as const;

/** Collision layers, as bits (og.Collider.layers and mask). */
export const Bits = {
  World: 1,
  Grate: 2,
  Wick: 4,
  Crate: 8,
  Door: 16,
  Lift: 32,
} as const;

/** Sprite layers by role (og-feel §10): the map's layers sit at -10 to -5. */
export const Draw = {
  map: -10,
  steam: 3,
  plate: -2,
  door: -1,
  crate: 0,
  prop: 1,
  tick: 2,
  wick: 4,
  gear: 5,
  hatch: -3,
  fx: 12,
} as const;

/** og.platform's keys. */
export const Keys = { progress: "progress", slot: "vault" } as const;

export const NAME_CHARS = 16;

export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { label: "Phase (0 title, 1 play, 2 results)" }),
  level: u32(0, { min: 0, max: 2 }),
  levelsOpen: bool(),
  paused: bool(),
  settingsOpen: bool(),
  /** Seconds in the level (scaled time: a pause stops it). */
  time: f64(0),
  gears: u32(0),
  gearMask: u32(0),
  deaths: u32(0),
  stomps: u32(0),
  /** The lamp last lit (-1: the spawn), and where Wick comes back. */
  checkpoint: i32(-1),
  checkX: f32(0),
  checkY: f32(0),
  spawnX: f32(0),
  spawnY: f32(0),
  /** A warp asked for: Wick's feet put here on the next tick. */
  warp: bool(),
  warpX: f32(0),
  warpY: f32(0),
  /** The HUD's touch controls this tick (a phone's d-pad and Jump). */
  touchX: f32(0),
  touchJump: bool(),
  /** Wick, as the Control system last left him. */
  heroX: f32(0),
  heroY: f32(0),
  heroVY: f32(0),
  facing: f32(1),
  grounded: bool(),
  onWall: f32(0),
  pushing: bool(),
  /** Something hurt Wick this tick (Ticks, Hazards): Control answers it on the next. */
  hurtAsk: bool(),
  /** Ticks of the hurt before the respawn (0: none). */
  hurt: u32(0),
  /** Ticks of invulnerability after a respawn. */
  safe: u32(0),
  /** A stomp landed: bounce Wick on the next tick. */
  bounce: bool(),
  /** Ticks since the exit opened (0: not yet). */
  exiting: u32(0),
  /** A level's things put back this tick: 0 none, 1 to the last lamp (crates where it saw them), 2 as the level starts. */
  reset: u32(0),
  resetLevel: u32(0),
  /** A lamp was lit: the crates take where they stand as their checkpoint (then the save is written). */
  snap: bool(),
  /** Plates pressed, as bits by switch id (Plate.target, Door.id, Gate.id). */
  switches: u32(0),
  /** Lighting2D's ambient for this level, eased to. */
  ambient: f32(1),
  /** Counters the playtests read. */
  jumps: u32(0),
  wallJumps: u32(0),
  landings: u32(0),
  steps: u32(0),
  shoves: u32(0),
  presses: u32(0),
  opens: u32(0),
  lampsLit: u32(0),
  /** The results of the level just finished. */
  score: u32(0),
  stars: u32(0),
  newBest: bool(),
  /** Progress, as og.platform's `progress` key keeps it. */
  unlocked: u32(1, { min: 1, max: 3 }),
  best0: u32(0),
  best1: u32(0),
  best2: u32(0),
  stars0: u32(0),
  stars1: u32(0),
  stars2: u32(0),
  time0: f32(0),
  time1: f32(0),
  time2: f32(0),
  /** The progress key's revision as last read (0: never written). */
  revision: f64(0),
  /** og.platform requests in flight (0: none). Reset after a restore (src/save.ts). */
  progressRequest: f64(0),
  writeRequest: f64(0),
  settingsRequest: f64(0),
  slotRequest: f64(0),
  loadRequest: f64(0),
  saveRequest: f64(0),
  /** The progress record to write (a level finished), and the settings (the settings closed). */
  writeNow: bool(),
  /** A progress write was refused as stale: the read under way is merged, then written again. */
  rewrite: bool(),
  settingsNow: bool(),
  /** A save asked for (a lamp lit): written by the Saves system after the tick's systems. */
  saveNow: bool(),
  /** The vault slot's manifest answered, and what it holds. */
  hasSlot: bool(),
  slotLevel: u32(0),
  slotGears: u32(0),
  saves: u32(0),
  loads: u32(0),
  conflicts: u32(0),
  /** True in every saved world (set just before world.snapshot): the tick after a restore sees it. */
  restored: bool(),
  /** Handles loaded in start, kept as numbers (asset handles do not fit a u32: f64). */
  dawnLook: f64(0),
  /** Accessibility: unnamed focusable controls and nodes, counted for the playtests. */
  a11yUnnamed: u32(0),
  a11yNodes: u32(0),
});

/** The player's settings (og.platform's og.settings key, og-platform-sdk §3). */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  muted: bool(),
  reducedMotion: bool(),
  highContrast: bool(),
  /** LANGUAGES index (src/ui.ts). */
  language: u32(0),
});

/** Wick's feel, tuned from og-feel §12's precise column towards the level's cells (one unit a cell). */
export const Tuning = defineResource("Tuning", {
  maxSpeed: f32(6.5, { min: 1, max: 15, unit: "mps", tunable: true }),
  pushSpeed: f32(2.4, { min: 0.5, max: 8, unit: "mps", tunable: true, label: "Run speed while pushing a crate" }),
  jumpHeight: f32(3.4, { min: 1, max: 6, unit: "m", tunable: true }),
  timeToApex: f32(0.38, { min: 0.2, max: 0.8, unit: "s", tunable: true }),
  wallSlideSpeed: f32(2.6, { min: 0.5, max: 10, unit: "mps", tunable: true }),
  wallJumpX: f32(6, { min: 2, max: 14, unit: "mps", tunable: true }),
  wallJumpY: f32(14, { min: 4, max: 20, unit: "mps", tunable: true }),
  coyoteTime: f32(0.1, { min: 0, max: 0.3, unit: "s", tunable: true }),
  jumpBuffer: f32(0.12, { min: 0, max: 0.3, unit: "s", tunable: true }),
  tickSpeed: f32(1.4, { min: 0.2, max: 6, unit: "mps", tunable: true }),
  doorSpeed: f32(5, { min: 0.5, max: 20, unit: "mps", tunable: true }),
  liftSpeed: f32(2.2, { min: 0.5, max: 8, unit: "mps", tunable: true }),
  bounce: f32(11, { min: 2, max: 20, unit: "mps", tunable: true, label: "Bounce off a stomped tick" }),
  hurtTicks: u32(36, { min: 1, max: 240 }),
  safeTicks: u32(60, { min: 0, max: 240 }),
  exitTicks: u32(80, { min: 1, max: 600, label: "Ticks of the cheer before the results" }),
});

/** Effects asked for this tick: each kind's position and the tick asked (src/feel.ts, the Effects system). */
export const Fx = defineResource("Fx", {
  dustX: f32(0),
  dustY: f32(0),
  dustTick: f64(-1),
  gearX: f32(0),
  gearY: f32(0),
  gearTick: f64(-1),
  stompX: f32(0),
  stompY: f32(0),
  stompTick: f64(-1),
});

/** Sounds asked for this tick, as bits of Sound (the Voices system plays them). */
export const Sfx = defineResource("Sfx", { bits: u32(0), plays: u32(0) });

export const Sound = {
  Jump: 1 << 0,
  Land: 1 << 1,
  Step: 1 << 2,
  Gear: 1 << 3,
  Plate: 1 << 4,
  Door: 1 << 5,
  Gate: 1 << 6,
  Stomp: 1 << 7,
  Hurt: 1 << 8,
  Lamp: 1 << 9,
  Exit: 1 << 10,
  Shove: 1 << 11,
} as const;
export const SOUND_FILES: readonly [number, string][] = [
  [Sound.Jump, "jump"],
  [Sound.Land, "land"],
  [Sound.Step, "step"],
  [Sound.Gear, "gear"],
  [Sound.Plate, "plate"],
  [Sound.Door, "door"],
  [Sound.Gate, "gate"],
  [Sound.Stomp, "stomp"],
  [Sound.Hurt, "hurt"],
  [Sound.Lamp, "lamp"],
  [Sound.Exit, "exit"],
  [Sound.Shove, "shove"],
];

/** Which level a thing belongs to (every level's things exist at once, side by side in x). */
export const Member = defineComponent("Member", { level: u32(0) });

/** Wick. */
export const Hero = defineComponent("Hero", { anim: u32(0), squashed: u32(0) });

export const Crate = defineComponent("Crate", {
  homeX: f32(0),
  homeY: f32(0),
  /** Where it stood when the last lamp was lit (a death puts it back there). */
  checkX: f32(0),
  checkY: f32(0),
});

export const Plate = defineComponent("Plate", {
  /** The switch it holds open (Door.id or Gate.id). */
  target: u32(0),
  pressed: bool(),
  x: f32(0),
  y: f32(0),
});

export const Door = defineComponent("Door", {
  id: u32(0),
  closedY: f32(0),
  openY: f32(0),
  open: bool(),
});

/** The jointed gate: its og.Joint's position motor slides it up when its plate is pressed. */
export const Gate = defineComponent("Gate", { id: u32(0), open: bool(), travel: f32(3) });

export const Gear = defineComponent("Gear", { index: u32(0), taken: bool(), baseY: f32(0), phase: f32(0) });

export const Lamp = defineComponent("Lamp", { index: u32(0), lit: bool() });

export const Exit = defineComponent("Exit", { open: bool() });

export const Tick = defineComponent("Tick", {
  minX: f32(0),
  maxX: f32(0),
  homeX: f32(0),
  dir: f32(1),
  dead: bool(),
  deadTicks: u32(0),
});

export const Lift = defineComponent("Lift", { minX: f32(0), maxX: f32(0), homeX: f32(0) });

/** A hurting rectangle (spikes), in world units. */
export const Hazard = defineComponent("Hazard", { x0: f32(0), y0: f32(0), x1: f32(0), y1: f32(0) });

/** A level's bounds and data from its `zone` object. */
export const Zone = defineComponent("Zone", { par: f32(60), ambient: f32(0.5), ambientG: f32(0.5), ambientB: f32(0.5), title: str(24) });

/** The effects' emitters, one per kind (FxKind). */
export const FxSlot = defineComponent("FxSlot", { kind: u32(0) });
export const FxKind = { Dust: 0, Gear: 1, Stomp: 2 } as const;

/** One voice per sound (og-audio §1). */
export const Voice = defineComponent("Voice", { bit: u32(0), plays: u32(0) });
