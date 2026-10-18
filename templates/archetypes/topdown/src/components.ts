// Every piece of Tomb Room's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `set` read and write them by
// name.
//
// The run's state (Game) and the knobs (Tuning) are resources, so every
// system reads them without carrying them on its rows. The actors are
// components: one Hero, one Slime, the Pickups and the Sfx voices.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** Which way the hero faces: the clip row of src/art.ts and the attack's direction. */
export const Facing = { Down: 0, Up: 1, Left: 2, Right: 3 } as const;

/** What a pickup is. */
export const PickupKind = { Coin: 1, Key: 2 } as const;

/** Sound requests: bits of Game.sfx that the Sfx system plays once each. */
export const Sound = { Hit: 1, Hurt: 2, Coin: 4, Key: 8, Door: 16 } as const;

/** The run: the hero's health and purse, the door, the room and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  hp: u32(3, { doc: "The hero's hearts left." }),
  hpMax: u32(3, { doc: "Hearts at the start." }),
  coins: u32(0, { doc: "Coins picked up." }),
  coinsTotal: u32(0, { doc: "Coins the map placed (worked out in start)." }),
  hasKey: bool(false, { doc: "The key is picked up." }),
  doorOpen: bool(false, { doc: "The doorway between the rooms is open." }),
  room: u32(1, { doc: "The room the hero stands in: 1 west, 2 east." }),
  enemyHp: u32(0, { doc: "The slime's health (0 once it is dead)." }),
  enemyHits: u32(0, { doc: "Hits the slime has taken; the third is a crit." }),
  heroX: f32(0, { unit: "m", doc: "The hero's feet, written by Control for the other systems." }),
  heroY: f32(0, { unit: "m" }),
  facing: u32(Facing.Down, { doc: "Facing.* of src/components.ts." }),
  swing: bool(false, { doc: "The sword's strike frame fired this tick (the hero's 'hit' flipbook event)." }),
  swingX: f32(0, { unit: "m", doc: "The strike's centre." }),
  swingY: f32(0, { unit: "m" }),
  swings: u32(0, { doc: "Strikes so far." }),
  steps: u32(0, { doc: "Footsteps (the walk clips' 'step' events)." }),
  iframes: u32(0, { unit: "ticks", doc: "Ticks the hero is invulnerable for after a hurt." }),
  hurts: u32(0, { doc: "Times the hero was hurt." }),
  downTicks: u32(0, { unit: "ticks", doc: "Ticks since hp reached 0; the hero stands up at the spawn after a while." }),
  spawnX: f32(0, { unit: "m", doc: "The map's spawn point." }),
  spawnY: f32(0, { unit: "m" }),
  knockX: f32(0, { unit: "m/s", doc: "The hero's knockback velocity; it decays every tick." }),
  knockY: f32(0, { unit: "m/s" }),
  doorX: f32(0, { unit: "m", doc: "The doorway's centre (from the map's door object)." }),
  doorY: f32(0, { unit: "m" }),
  doorCellX: u32(0, { doc: "The doorway's first cell (map columns and rows)." }),
  doorCellY: u32(0),
  doorCellsW: u32(0),
  doorCellsH: u32(0),
  roomSplit: f32(20, { unit: "m", doc: "The x where the east room starts (from the room objects)." }),
  lockedFor: u32(0, { unit: "ticks", doc: "Ticks before the locked door speaks again." }),
  sfx: u32(0, { doc: "Sound.* bits waiting to play." }),
  font: f64(0, { doc: "The UI Font handle." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  speed: f32(4.5, { min: 1, max: 10, step: 0.25, unit: "m/s", tunable: true, label: "Walk speed" }),
  dashSpeed: f32(13, { min: 4, max: 24, step: 0.5, unit: "m/s", tunable: true, label: "Dash speed" }),
  dashTicks: u32(9, { min: 2, max: 30, step: 1, unit: "ticks", tunable: true, label: "Dash length" }),
  dashCooldown: u32(36, { min: 0, max: 120, step: 1, unit: "ticks", tunable: true, label: "Dash cooldown" }),
  reach: f32(0.75, { min: 0.2, max: 2, step: 0.05, unit: "m", tunable: true, label: "Sword reach", doc: "The strike's radius around a point one reach ahead of the hero's chest." }),
  hitStop: u32(4, { min: 0, max: 12, step: 1, unit: "ticks", tunable: true, label: "Hit stop" }),
  shake: f32(0.25, { min: 0, max: 1, step: 0.05, tunable: true, label: "Hit shake", doc: "Camera trauma per hit." }),
  slimeSpeed: f32(1.6, { min: 0, max: 6, step: 0.1, unit: "m/s", tunable: true, label: "Slime speed" }),
  slimeSight: f32(5, { min: 0, max: 12, step: 0.5, unit: "m", tunable: true, label: "Slime sight", doc: "The slime chases a hero closer than this, and wanders otherwise." }),
  knockback: f32(9, { min: 0, max: 20, step: 0.5, unit: "m/s", tunable: true, label: "Knockback" }),
  iframes: u32(50, { min: 0, max: 180, step: 5, unit: "ticks", tunable: true, label: "Invulnerable ticks" }),
});

/** The hero: the attack and dash timers and the clip on show. */
export const Hero = defineComponent("Hero", {
  attack: u32(0, { unit: "ticks", doc: "Ticks left in the current swing; input waits until 0." }),
  dash: u32(0, { unit: "ticks", doc: "Ticks left in the current dash." }),
  dashCooldown: u32(0, { unit: "ticks" }),
  dashX: f32(0, { doc: "The dash's direction." }),
  dashY: f32(0),
  dashes: u32(0, { doc: "Dashes so far." }),
  moving: bool(false, { doc: "Walk input was held this tick." }),
});

/** A slime: health, its own knockback, the wander and the death. */
export const Slime = defineComponent("Slime", {
  hp: u32(50, { doc: "Health; the sword deals 12, 12, then a 26 crit." }),
  knockX: f32(0, { unit: "m/s" }),
  knockY: f32(0, { unit: "m/s" }),
  stun: u32(0, { unit: "ticks", doc: "Ticks it neither moves nor hurts after a hit." }),
  wanderX: f32(0, { doc: "The current wander direction." }),
  wanderY: f32(0),
  wanderFor: u32(0, { unit: "ticks" }),
  dead: bool(false),
  deadTicks: u32(0, { unit: "ticks", doc: "Ticks since it died; it hides once the death tween is over." }),
  bob: i32(0, { doc: "The bob clip's index (src/art.ts)." }),
  hurt: i32(0, { doc: "The hurt clip's index." }),
});

/** A coin or the key, taken by overlap. */
export const Pickup = defineComponent("Pickup", {
  kind: u32(PickupKind.Coin, { doc: "PickupKind.* of src/components.ts." }),
  taken: bool(false),
  takenTicks: u32(0, { unit: "ticks" }),
  baseY: f32(0, { unit: "m", doc: "Where it rests; it hovers around this." }),
});

/**
 * The run's facts, copied from Game onto the hero every tick by the Hud
 * system: `dump_entity:hero` shows them, which is how the playtests read the
 * run (the command channel dumps entities, not resources).
 */
export const Report = defineComponent("Report", {
  room: u32(1),
  hp: u32(0),
  coins: u32(0),
  hasKey: bool(false),
  doorOpen: bool(false),
  enemyHp: u32(0),
  hits: u32(0, { doc: "Hits the slime took." }),
  swings: u32(0),
  steps: u32(0),
  hurts: u32(0),
  facing: u32(0),
  doorWall: i32(0, { doc: "og.tilemap.getTile of the walls terrain at the doorway's first cell: 1 filled while closed, 0 once open." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0, { doc: "Sound.* of src/components.ts." }),
});
