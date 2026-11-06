// Every piece of Redline Ring's state lives in these reflected types:
// dumps, the probe, Tune and the command channel's `get` and `set` read
// and write them by name.
//
// The race (Game) and the knobs (Tuning) are resources; the player's
// settings and the record are this device's (src/device.ts), never the
// world's. The actors are rows spawned in `start`: a car per grid
// slot (Car: the seats' cars carry og.PlayerInput too, the rivals drive
// from the racing line) and one voice per sound (Sfx).

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Race: 1, Results: 2 } as const;

/**
 * Sound requests: bits of Game.sfx that the Voices system plays once each.
 * `Tap` is the touch HUD answering on the ui bus, as og.ui's own buttons do.
 */
export const Sound = { Beep: 1, Go: 2, Bump: 4, Lap: 8, Best: 16, Boost: 32, Finish: 64, Sector: 128, Tap: 256, Crash: 512 } as const;

/** Cars on the grid: the seats' and the rivals'. */
export const FIELD = 6;
/** Ticks of the countdown: three seconds of lights, then GO. */
export const COUNTDOWN = 180;
/** Ticks after the last seat finishes (or the race closes) before the results. */
export const RESULTS_DELAY = 150;

/** The race: its clock, its close and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  paused: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused, or this device's settings open outside a room)." }),
  clock: u32(0, { unit: "ticks", doc: "Ticks of this race played (paused ticks excluded), from the countdown's first." }),
  firstFinish: u32(0, { unit: "ticks", doc: "Game.clock when the first car crossed the line for the last time (0 none)." }),
  lastSeatFinish: u32(0, { unit: "ticks", doc: "Game.clock when the race was over for every seat (0 not yet)." }),
  closed: bool(false, { doc: "The race is over for every seat: finished, or out when it closed." }),
  bumps: u32(0, { doc: "Car contacts with the barriers and each other this race." }),
  start: bool(false, { doc: "Every system resets its part of the race this tick (a new race or the title)." }),
  sfx: u32(0),
  car0: f64(0, { doc: "Seat 0's car." }),
  car1: f64(0, { doc: "Seat 1's car (0 alone)." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  topSpeed: f32(42, { min: 10, max: 90, step: 0.5, unit: "m/s", tunable: true, label: "Top speed" }),
  accel: f32(15, { min: 2, max: 40, step: 0.5, unit: "m/s²", tunable: true, label: "Acceleration", doc: "From a standing start; it falls to nothing at the top speed." }),
  brake: f32(28, { min: 4, max: 60, step: 0.5, unit: "m/s²", tunable: true, label: "Braking" }),
  steer: f32(2.1, { min: 0.5, max: 5, step: 0.05, unit: "rad/s", tunable: true, label: "Steering", doc: "The yaw rate at full lock, at speed." }),
  grip: f32(9, { min: 1, max: 30, step: 0.5, unit: "1/s", tunable: true, label: "Grip", doc: "How fast a sideways slide is caught." }),
  driftGrip: f32(1.6, { min: 0.2, max: 10, step: 0.1, unit: "1/s", tunable: true, label: "Drift grip", doc: "The same while drifting (Brake held with Throttle and a turn)." }),
  driftTurn: f32(1.45, { min: 1, max: 3, step: 0.05, tunable: true, label: "Drift turn", doc: "The yaw rate while drifting, as a multiple of Steering." }),
  boostSpeed: f32(52, { min: 10, max: 100, step: 0.5, unit: "m/s", tunable: true, label: "Boost top speed" }),
  boostAccel: f32(12, { min: 0, max: 40, step: 0.5, unit: "m/s²", tunable: true, label: "Boost push" }),
  boostDrain: f32(0.42, { min: 0.05, max: 2, step: 0.01, unit: "1/s", tunable: true, label: "Boost drain", doc: "The meter's share spent each second of boost." }),
  driftCharge: f32(0.3, { min: 0, max: 2, step: 0.01, unit: "1/s", tunable: true, label: "Drift charge", doc: "The meter's share a second of drifting fills." }),
  closeAfter: f32(30, { min: 1, max: 120, step: 1, unit: "s", tunable: true, label: "Classified after", doc: "Seconds after the winner crosses the line before the race closes: a seat still running then is out (DNF)." }),
  rivalPace: f32(0.93, { min: 0.5, max: 1.2, step: 0.01, tunable: true, label: "Rival pace", doc: "The rivals' speed on the racing line, as a share of what it allows (each rival a little apart)." }),
});

/**
 * A car on the circuit: a seat's (seat ≥ 0, with og.PlayerInput) or a
 * rival's (seat -1, driven from the racing line). Its intent (steer,
 * throttle, brake, boost) is written by the Drivers or the Rivals system
 * and turned into motion by Drive; its standing by Race.
 */
export const Car = defineComponent("Car", {
  index: u32(0, { doc: "Its grid slot and livery (0 to FIELD - 1)." }),
  seat: i32(-1, { doc: "The seat that drives it (its og.PlayerInput.player), -1 for a rival." }),
  body: f64(0, { doc: "Its body's entity (it rolls and pitches)." }),
  before: u32(0, { doc: "Its seat's og.PlayerInput.buttons last tick (press edges)." }),
  // The intent this tick.
  steer: f32(0, { doc: "-1 full left to 1 full right." }),
  throttle: f32(0, { doc: "0 to 1." }),
  brake: f32(0, { doc: "0 to 1." }),
  boostHeld: bool(false),
  // Motion.
  heading: f32(0, { unit: "rad", doc: "0 along -z, growing to the left (as src/view.ts's yaw)." }),
  vx: f32(0, { unit: "m/s" }),
  vz: f32(0, { unit: "m/s" }),
  speed: f32(0, { unit: "m/s", doc: "Along its heading (negative reversing)." }),
  slip: f32(0, { unit: "m/s", doc: "Its sideways speed." }),
  drifting: bool(false),
  boosting: bool(false),
  boost: f32(0.35, { min: 0, max: 1, doc: "The boost meter, 0 to 1." }),
  spin: f32(0, { unit: "rad", doc: "Its wheels' turn, for the models." }),
  camYaw: f32(0, { unit: "rad", doc: "The chase camera's heading, easing after the car's." }),
  hits: u32(0, { doc: "Contacts this race." }),
  sfx: u32(0, { doc: "Its seat's own sound requests (Sound.Lap, Sector, Finish): played on the device that drives it." }),
  hitCool: u32(0, { unit: "ticks", doc: "Ticks before another contact sounds." }),
  // Standing.
  sample: u32(0, { doc: "The centre line's sample nearest it (src/track.ts)." }),
  along: f32(0, { unit: "m", doc: "Metres from the line along the lap, 0 to the lap's length." }),
  across: f32(0, { unit: "m", doc: "Metres right of the centre line." }),
  laps: i32(-1, { doc: "Laps completed: -1 on the grid behind the line, LAPS once finished." }),
  checkpoint: u32(2, { doc: "Checkpoints passed this lap (src/track.ts SECTORS; 2: only the line is left; 3: it backed over the line, so crossing it again restores the lap without timing one)." }),
  progress: f32(0, { unit: "m", doc: "Race distance: laps × the lap's length + along." }),
  lapStart: u32(0, { unit: "ticks", doc: "Game.clock when this lap began." }),
  lastLap: u32(0, { unit: "ticks" }),
  bestLap: u32(0, { unit: "ticks", doc: "Its best lap this race (0 none)." }),
  checkpoint0: u32(0, { unit: "ticks", doc: "This lap's time at its first checkpoint." }),
  checkpoint1: u32(0, { unit: "ticks", doc: "This lap's time at its second checkpoint." }),
  best0: u32(0, { unit: "ticks", doc: "Its best lap's time at the first checkpoint (0 none)." }),
  best1: u32(0, { unit: "ticks", doc: "Its best lap's time at the second checkpoint (0 none)." }),
  splitAt: u32(0, { unit: "ticks", doc: "Game.clock at its last checkpoint." }),
  splitDelta: i32(0, { unit: "ticks", doc: "That checkpoint against its best lap's (negative: ahead; 0 on its first lap)." }),
  lapAt: u32(0, { unit: "ticks", doc: "Game.clock when it last crossed the line (a lap done, the final lap begun)." }),
  finish: u32(0, { unit: "ticks", doc: "Game.clock when it finished (0 running)." }),
  out: bool(false, { doc: "Still running when the race closed (DNF)." }),
  place: u32(1, { doc: "Its place in the field, from 1." }),
  wrongWay: u32(0, { unit: "ticks", doc: "Ticks it has driven the wrong way." }),
  // A rival's driving.
  pace: f32(1, { doc: "Its share of Tuning.rivalPace." }),
  lane: f32(0, { unit: "m", doc: "Its offset from the racing line, to pass a slower car." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
