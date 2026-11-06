// The game's state, declared once (the inspector, Tune, the probe and the
// wiki read these schemas): the hello's Health, Velocity and Spin, the
// title's Run, the guard's Alarm, and the run's Mission on the dropship
// with the heroes, cells and voices of src/play.ts.

import { defineComponent, defineResource, bool, f32, u32, Vector3 } from "@origingame/engine";

export const Health = defineComponent("Health", {
  hp: f32(100, { min: 0, max: 1000, tunable: true, label: "Health" }),
  max: f32(100),
});

export const Velocity = defineComponent("Velocity", { v: Vector3() });

/** Turns an entity about +Y; 0 until the player starts (the dropship, lifting off). */
export const Spin = defineComponent("Spin", {
  speed: f32(0, { min: -10, max: 10, step: 0.1, unit: "rad", tunable: true, label: "Spin speed" }),
  target: f32(1.5, { min: -10, max: 10, tunable: true, label: "Spin when started" }),
});

/** Whether the player has pressed Start. */
export const Run = defineResource("Run", { started: bool(), chimed: bool() });

/** The guard's alarms, raised by its machine's `alarm` command. */
export const Alarm = defineComponent("Alarm", { raised: f32(0) });

export const Phase = { Title: 0, Play: 1, Over: 2 } as const;
export const Ending = { None: 0, Escaped: 1, Caught: 2 } as const;
/** Mission.sfx bits: each a voice played once on the tick it is set (Voices). */
export const Sound = { Cell: 1, Jump: 2, Land: 4, Step: 8, Hurt: 16, Win: 32, Lose: 64, Ready: 128 } as const;

/** The run, on the dropship: the yard's objective. */
export const Mission = defineComponent("Mission", {
  phase: u32(0, { doc: "Phase: 0 the title, 1 a run, 2 its summary." }),
  ending: u32(0, { doc: "Ending: 0 none, 1 escaped, 2 caught." }),
  cells: u32(0, { doc: "Power cells brought in this run." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run." }),
  paused: bool(false),
  overTicks: u32(0, { doc: "Ticks the summary has shown: it takes Play again from SUMMARY_READY." }),
  catches: u32(0, { doc: "The guard's grabs this run." }),
  jumps: u32(0),
  runs: u32(0),
  wins: u32(0),
  best: f32(0, { unit: "s", doc: "The fastest escape." }),
  sfx: u32(0, { doc: "Sound bits to play this tick." }),
});

/** The run's numbers: Tune them in the inspector or with `set:dropship.Tuning.<field>=`. */
export const Tuning = defineComponent("Tuning", {
  walkSpeed: f32(2.6, { min: 0.5, max: 8, unit: "mps", tunable: true, label: "Walk speed" }),
  runSpeed: f32(5.2, { min: 1, max: 12, unit: "mps", tunable: true, label: "Run speed" }),
  jumpSpeed: f32(5.2, { min: 1, max: 12, unit: "mps", tunable: true, label: "Jump speed" }),
  catchReach: f32(1.7, { min: 0.5, max: 4, unit: "m", tunable: true, label: "Guard's reach" }),
  grabDamage: f32(34, { min: 1, max: 100, tunable: true, label: "Health a grab costs" }),
  padRadius: f32(1.8, { min: 0.5, max: 4, unit: "m", tunable: true, label: "Pad radius" }),
});

/** A hero: one per seat, moved by its og.PlayerInput alone. */
export const Hero = defineComponent("Hero", {
  seat: u32(0),
  before: u32(0, { doc: "Its buttons last tick (press edges)." }),
  facing: f32(0, { unit: "rad", doc: "The body's heading: 0 faces +Z." }),
  speed: f32(0, { unit: "mps" }),
  grounded: bool(true),
  airTime: f32(0, { unit: "s" }),
  stepClock: f32(0),
  invulnerable: f32(0, { unit: "s", doc: "After a grab: the guard cannot grab it again yet." }),
  clip: u32(0, { doc: "0 idle, 1 walk, 2 run." }),
  startX: f32(0),
  startZ: f32(0),
  fresh: bool(false, { doc: "A run starts this tick: HeroMove turns its look towards the pad." }),
});

/** A power cell: picked up by a hero within its reach. */
export const Cell = defineComponent("Cell", { taken: bool(false), x: f32(0), z: f32(0), pop: f32(0, { doc: "Its pop-up, 0 to 1." }) });

/** A rise: eases its entity's height from `from` to `to` over `seconds` (the fence, the dropship's lift). */
export const Rise = defineComponent("Rise", { from: f32(0), to: f32(0), seconds: f32(1), t: f32(0) });

/** A sound voice: the Sound bit it plays. */
export const Voice = defineComponent("Voice", { bit: u32(0), plays: u32(0) });

