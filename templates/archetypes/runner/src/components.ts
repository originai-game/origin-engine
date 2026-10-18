// Every piece of game state lives in these reflected components: dumps, the
// probe, Tune and the command channel's `set` read and write them by name.
//
// Why every actor row carries Run and Tuning: a system sees exactly one
// query (the AND of the components it declares), and inside a system the
// engine reaches no other entity and no game resource yet. So the systems
// that move obstacles, collect coins and end the run must see the run's
// state on the same rows as the actors. Every actor is spawned with
// og.Transform, Actor, Run and Tuning (plus og.Sprite, og.Flipbook or
// og.Tilemap for what it draws), and the row whose Actor.kind is
// Kind.Runner, the entity named "runner", is the authority: the other rows'
// Run and Tuning keep their defaults and nothing reads them. Tune and
// `set:runner.Tuning.<field>=<value>` write that row.

import { bool, defineComponent, f32, f64, u32 } from "@origingame/engine";

/** What an actor row is. */
export const Kind = {
  Sky: 1,
  Moon: 2,
  Cloud: 3,
  City: 4,
  Dust: 5,
  Runner: 6,
  Crate: 7,
  Fence: 8,
  Coin: 9,
} as const;

/** Run phases. */
export const Phase = { Title: 0, Running: 1, Over: 2 } as const;

/** One thing in the world: its kind, hit box and scroll behaviour. */
export const Actor = defineComponent("Actor", {
  kind: u32(0, { doc: "Kind.* of src/components.ts." }),
  active: bool(false, { doc: "Pooled obstacles and coins collide while true; a collected coin is inactive while its pop plays, parked rows wait at x = -1000." }),
  w: f32(1, { min: 0, unit: "m", doc: "Hit box width, centred on the position (the runner's: on its feet plus RUNNER_SIZE.y / 2)." }),
  h: f32(1, { min: 0, unit: "m", doc: "Hit box height, centred like the width." }),
  parallax: f32(1, { min: 0, max: 1, doc: "The fraction of the run speed this row scrolls left at (0 never moves)." }),
  wrap: f32(0, { min: 0, unit: "m", doc: "Scenery: the span a row jumps right by once it has scrolled that far (a tilemap's width); 0 for pooled rows." }),
  vy: f32(0, { unit: "m/s", doc: "Vertical velocity (the runner)." }),
});

/** The run: phase, pace, score. Authoritative on the runner row. */
export const Run = defineComponent("Run", {
  phase: u32(Phase.Title, { doc: "0 title, 1 running, 2 over." }),
  phaseTicks: u32(0, { unit: "ticks", doc: "Ticks since the phase began." }),
  speed: f32(0, { unit: "m/s", doc: "The current run speed." }),
  distance: f64(0, { unit: "m", doc: "Metres run this round." }),
  coins: u32(0, { doc: "Coins collected this round." }),
  score: u32(0, { doc: "floor(distance) + coins * Tuning.coinValue." }),
  best: u32(0, { doc: "The best score: this session's, or the player's store's (og.platform) when higher." }),
  bestRequest: u32(0, { doc: "The og.platform request reading the stored best (0 once answered)." }),
  rounds: u32(0, { doc: "Rounds started." }),
  spawnIn: f32(0, { unit: "s", doc: "Seconds until the next obstacle pattern." }),
  grounded: bool(true, { doc: "The runner stands on the roof." }),
  buffer: u32(0, { unit: "ticks", doc: "Ticks left in which an early press still jumps on landing." }),
  hold: u32(0, { unit: "ticks", doc: "Ticks the current jump has been held." }),
  rising: bool(false, { doc: "The runner is in the held part of a jump." }),
});

/** Tune's parameters. Authoritative on the runner row. */
export const Tuning = defineComponent("Tuning", {
  speed: f32(6, { min: 3, max: 12, step: 0.5, unit: "m/s", tunable: true, label: "Start speed", doc: "The run speed when a round starts." }),
  speedRamp: f32(0.12, { min: 0, max: 1, step: 0.02, unit: "m/s²", tunable: true, label: "Speed ramp", doc: "How fast the run speeds up." }),
  speedMax: f32(12, { min: 4, max: 20, step: 0.5, unit: "m/s", tunable: true, label: "Top speed" }),
  gravity: f32(36, { min: 10, max: 80, step: 1, unit: "m/s²", tunable: true, label: "Gravity" }),
  jumpSpeed: f32(12, { min: 6, max: 20, step: 0.5, unit: "m/s", tunable: true, label: "Jump speed", doc: "Take-off speed; a full jump peaks at jumpSpeed² / (2 gravity)." }),
  jumpCut: f32(0.5, { min: 0.1, max: 1, step: 0.05, tunable: true, label: "Jump cut", doc: "The share of upward speed a release keeps: a tap is a hop, a hold a full jump." }),
  spawnInterval: f32(1.3, { min: 0.5, max: 60, step: 0.1, unit: "s", tunable: true, label: "Spawn interval", doc: "Seconds between obstacle patterns, before the dice (×0.85–1.25)." }),
  coinValue: u32(10, { min: 0, max: 100, step: 1, tunable: true, label: "Coin value" }),
});

/** Where the HUD draws: the font and the visible area, worked out in `start`. On the runner row only. */
export const Hud = defineComponent("Hud", {
  font: f64(0, { doc: "The Font handle." }),
  halfWidth: f32(8.9, { unit: "m" }),
  top: f32(8.5, { unit: "m" }),
  bottom: f32(-1.5, { unit: "m" }),
});
