// Every piece of Saltwind Isle's world state lives in these reflected
// types: dumps, the probe, Tune and the command channel's `get` and `set`
// read and write them by name.
//
// The run (Game) and the knobs (Tuning) are resources. The rows are spawned
// in `start`: an explorer per seat (Hero), the five landmarks (Site), the
// trees and boulders to gather from (Node) and one voice per sound (Sfx).
// The player's settings and record are this device's own (src/device.ts),
// never in the world.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Lit: 1, Night: 2 } as const;

/** What a node gives. */
export const Kind = { Tree: 0, Rock: 1 } as const;

/** What a seat's Gather reaches (Hero.target). */
export const Target = { None: 0, Node: 1, Bridge: 2 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Step: 1, Jump: 2, Splash: 4, Chop: 8, Stone: 16, Pickup: 32, Claim: 64, Discover: 128, Build: 256, Dawn: 512, Night: 1024, Lost: 2048, Tap: 4096 } as const;

/** The landmarks a run claims. */
export const SITE_COUNT = 5;
/** What the bridge costs. */
export const BRIDGE_COST = { wood: 6, stone: 4 } as const;
/** What a felled tree and a broken boulder give. */
export const YIELD = { wood: 2, stone: 2 } as const;
/** The days a run lasts: it ends at the third nightfall. */
export const DAYS = 3;

/** The run: the explorers' progress, the clock, the bag and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  day: u32(1, { doc: "The day of the run, 1 to DAYS." }),
  hour: f32(7, { doc: "The time of day (og.DayNight's hour, 0 to 24)." }),
  night: bool(false, { doc: "The sun is down." }),
  claimed: u32(0, { doc: "Landmarks claimed this run." }),
  claimMask: u32(0, { doc: "Bit i: site i is claimed." }),
  seenMask: u32(0, { doc: "Bit i: site i has been discovered (a seat came within Tuning.sight)." }),
  wood: u32(0, { doc: "Wood in the shared bag." }),
  stone: u32(0, { doc: "Stone in the shared bag." }),
  woodTotal: u32(0, { doc: "Wood gathered this run." }),
  stoneTotal: u32(0, { doc: "Stone gathered this run." }),
  felled: u32(0, { doc: "Trees and boulders gathered this run." }),
  bridge: bool(false, { doc: "The bridge across the strait stands." }),
  building: f32(0, { unit: "s", doc: "How long the bridge has been worked on (every seat at it holding Gather adds its time)." }),
  builtAt: f32(0, { unit: "s", doc: "Game.time when the bridge was finished." }),
  washes: u32(0, { doc: "Times the tide carried an explorer back to the camp." }),
  jumps: u32(0, { doc: "Jumps of every explorer this run." }),
  near: i32(-1, { doc: "Seat 0's target: a node's index, -2 the bridge, -1 none (for the facts)." }),
  gathering: bool(false, { doc: "Seat 0 holds Gather at its target." }),
  wading: bool(false, { doc: "Seat 0's feet are in the sea." }),
  grounded: bool(false),
  speed: f32(0, { unit: "m/s", doc: "Seat 0's speed across the ground." }),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the run ended; the summary comes after a beat." }),
  start: bool(false, { doc: "Every system resets its part of the isle this tick (a new run or the title)." }),
  dawn: bool(false, { doc: "A new day broke this tick: the felled trees and boulders grow back." }),
  sfx: u32(0),
  hero: f64(0, { doc: "Seat 0's explorer's entity." }),
  hero1: f64(0, { doc: "Seat 1's explorer's entity (0 alone)." }),
  terrain: f64(0, { doc: "The isle's og.terrain data." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  walkSpeed: f32(3.4, { min: 1, max: 8, step: 0.1, unit: "m/s", tunable: true, label: "Walk speed" }),
  runSpeed: f32(6.8, { min: 2, max: 14, step: 0.1, unit: "m/s", tunable: true, label: "Run speed" }),
  wade: f32(0.5, { min: 0.1, max: 1, step: 0.05, tunable: true, label: "Wading slows to", doc: "The speed in the shallows, as a fraction." }),
  jumpSpeed: f32(5.4, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Jump speed" }),
  daySeconds: f32(110, { min: 20, max: 900, step: 5, unit: "s", tunable: true, label: "Daylight lasts", doc: "Seconds of play from 7:00 to nightfall at 19:30." }),
  nightSeconds: f32(22, { min: 5, max: 300, step: 1, unit: "s", tunable: true, label: "Night lasts", doc: "Seconds of play from nightfall to the next 7:00." }),
  reach: f32(2.6, { min: 1, max: 6, step: 0.1, unit: "m", tunable: true, label: "Gather reach", doc: "How near a tree or a boulder's edge an explorer gathers it." }),
  chopSeconds: f32(1.1, { min: 0.1, max: 6, step: 0.1, unit: "s", tunable: true, label: "Felling a tree takes" }),
  breakSeconds: f32(1.4, { min: 0.1, max: 6, step: 0.1, unit: "s", tunable: true, label: "Breaking a boulder takes" }),
  buildSeconds: f32(2.4, { min: 0.2, max: 10, step: 0.1, unit: "s", tunable: true, label: "Building the bridge takes" }),
  claimRadius: f32(8, { min: 2, max: 20, step: 0.5, unit: "m", tunable: true, label: "Claim radius", doc: "How near a landmark's centre an explorer claims it." }),
  sight: f32(42, { min: 5, max: 120, step: 1, unit: "m", tunable: true, label: "Discovery distance", doc: "How near a landmark an explorer discovers it (its name on the compass and the map)." }),
});

/**
 * An explorer, on the capsule (its body is a child): one per seat, moved by
 * its og.PlayerInput alone. Its view (yaw, pitch) is its SeatLook
 * (src/view.ts), turned by its own input.
 */
export const Hero = defineComponent("Hero", {
  seat: u32(0, { doc: "The seat it plays for (its og.PlayerInput.player)." }),
  body: f64(0, { doc: "Its skinned body's entity." }),
  reach: f64(0, { doc: "Its arm's og.IkChain entity." }),
  lantern: f64(0, { doc: "Its lantern's og.PointLight entity." }),
  splash: f64(0, { doc: "Its splash's og.ParticleSystem entity." }),
  before: u32(0, { doc: "Its og.PlayerInput.buttons last tick (press edges)." }),
  facing: f32(0, { unit: "rad", doc: "Its body's heading." }),
  lookIdle: u32(40, { unit: "ticks", doc: "Ticks since its input last turned the view (a third-person view swings behind it once idle)." }),
  target: u32(0, { doc: "Target.*: what its Gather reaches." }),
  node: i32(-1, { doc: "The node within its reach (-1 none)." }),
  gathering: bool(false, { doc: "It holds Gather at its target." }),
  workClock: f32(0, { unit: "s", doc: "Time since its last stroke of the axe or pick." }),
  wading: bool(false, { doc: "Its feet are in the sea." }),
  grounded: bool(false),
  speed: f32(0, { unit: "m/s", doc: "Its speed across the ground." }),
  airTime: f32(0, { unit: "s", doc: "Seconds since the capsule last stood on the ground." }),
  stepClock: f32(0, { unit: "m", doc: "Ground covered since the last footstep." }),
  steps: u32(0, { doc: "Footsteps: one per stride of ground covered." }),
});

/** A landmark: its building, the standard that rises and the beacon that lights when it is claimed. */
export const Site = defineComponent("Site", {
  index: u32(0),
  claimed: bool(false),
  claimedAt: f32(0, { unit: "s", doc: "Game.time when it was claimed." }),
  seen: bool(false, { doc: "Discovered: an explorer came within Tuning.sight." }),
});

/** A tree to fell or a boulder to break: its model, and what stands in its place once gathered. */
export const Node = defineComponent("Node", {
  index: u32(0),
  kind: u32(Kind.Tree, { doc: "Kind.*: a tree gives wood, a boulder stone." }),
  felled: bool(false, { doc: "Gathered: hidden until the next dawn." }),
  progress: f32(0, { unit: "s", doc: "How long it has been worked: every explorer at it holding Gather adds its time; it is lost once none does for a tick." }),
  workedTick: u32(0, { unit: "ticks", doc: "The tick it was last worked." }),
  radius: f32(0.6, { unit: "m", doc: "Its trunk's or its side's distance from its centre." }),
  model: f64(0, { doc: "Its standing model's entity." }),
  stump: f64(0, { doc: "What is left once gathered (a stump, rubble)." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
