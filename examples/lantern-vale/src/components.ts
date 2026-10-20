// Every piece of Lantern Vale's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The run (Game), the knobs (Tuning) and the player's settings (Prefs) are
// resources. The actors are rows spawned in `start`: the hero (Hero), the
// four lanterns (Lantern), the three slimes (Slime), the rocks of the pile
// (Rock) and one voice per sound (Sfx).

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Lit: 1, Night: 2, Caught: 3 } as const;

/** The slimes' expression bubbles: the layers of sprites/faces.ogt. */
export const Face = { Calm: 0, Alert: 1, Lost: 2, Sleepy: 3 } as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Step: 1, Light: 2, Hurt: 4, Hit: 8, Win: 16, Lose: 32, Kindle: 64, Alert: 128 } as const;

/** The lanterns a run lights, the hearts it starts with. */
export const LANTERN_COUNT = 4;
export const HEARTS = 3;
/** The player's name: at most this many characters, kept as code points in Prefs (no string fields: FINDINGS.md). */
export const NAME_CHARS = 12;

/** The run: the hero's progress, the clock and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  hearts: i32(HEARTS),
  best: u32(0, { doc: "The best score of a won run this session." }),
  newBest: bool(false, { doc: "This run set the best." }),
  hurts: u32(0, { doc: "Hearts lost this run (the facts' `hits`)." }),
  lit: u32(0, { doc: "Lanterns lit this run." }),
  litMask: u32(0, { doc: "Bit i: lantern i glows." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  hour: f32(15.5, { doc: "The time of day og.DayNight is held at: a run goes from 15:30 to night at 19:30 (HOURS in src/world.ts)." }),
  near: i32(-1, { doc: "The unlit lantern within reach (-1 none), from og.physics.overlap." }),
  kindle: f32(0, { unit: "s", doc: "How long the near lantern has been kindled." }),
  kindling: bool(false),
  grounded: bool(false),
  speed: f32(0, { unit: "m/s", doc: "The hero's speed across the ground." }),
  steps: u32(0),
  facing: f32(Math.PI, { unit: "rad", doc: "The hero's heading (0 faces +z)." }),
  invulnerable: f32(0, { unit: "s", doc: "Seconds before a slime can hurt the hero again." }),
  knock: f32(0, { unit: "s", doc: "Seconds the hero is thrown back (no control)." }),
  knockX: f32(0),
  knockZ: f32(0),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the run ended; the results come after a beat." }),
  start: bool(false, { doc: "Every system resets its part of the vale this tick (a new run or the title)." }),
  sfx: u32(0),
  hero: f64(0, { doc: "The hero's entity." }),
  terrain: f64(0, { doc: "The vale's og.terrain data." }),
  look: u32(0, { doc: "The look applied: 0 dusk (looks/lantern-vale.ogt), 1 dawn (looks/dawn.ogt, the results)." }),
  dusk: f64(0, { doc: "og.load.look of looks/lantern-vale.ogt." }),
  dawn: f64(0, { doc: "og.load.look of looks/dawn.ogt." }),
  far: u32(0, { doc: "The far-side streaming group's state (StreamState's order: 0 unloaded … 3 resident, 4 failed), from og.stream.status." }),
  farLoads: u32(0, { doc: "Times the far side became resident." }),
  farUsed: f64(0, { unit: "bytes", doc: "og.stream.budget().used, copied each tick for scripts." }),
  musicChunks: u32(0, { doc: "og.load.soundInfo(music).chunks: the track was cooked in chunks." }),
  music: f64(0, { doc: "The music's sound handle." }),
  rockMoved: f32(0, { unit: "m", doc: "The farthest any rock of the pile has moved from where it was set." }),
  a11yUnnamed: u32(0, { doc: "Focusable nodes of og.ui.a11y() without a name, last frame." }),
  a11yNodes: u32(0, { doc: "Nodes of og.ui.a11y(), last frame." }),
  styled: bool(false, { doc: "The models' materials are styled (src/world.ts styleModels): toon voxels, the lanterns' clear coat." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  walkSpeed: f32(3.4, { min: 1, max: 8, step: 0.1, unit: "m/s", tunable: true, label: "Walk speed" }),
  runSpeed: f32(6, { min: 2, max: 12, step: 0.1, unit: "m/s", tunable: true, label: "Run speed" }),
  kindleSeconds: f32(1.2, { min: 0.2, max: 6, step: 0.1, unit: "s", tunable: true, label: "Lighting takes" }),
  reach: f32(2.2, { min: 1, max: 5, step: 0.1, unit: "m", tunable: true, label: "Lighting reach", doc: "The radius of the overlap query round the hero." }),
  duskSeconds: f32(300, { min: 20, max: 900, step: 5, unit: "s", tunable: true, label: "Dusk lasts", doc: "Seconds of play before night ends the run." }),
  patrolSpeed: f32(1.4, { min: 0.2, max: 6, step: 0.1, unit: "m/s", tunable: true, label: "Slime patrol speed" }),
  chaseSpeed: f32(3.6, { min: 0.5, max: 9, step: 0.1, unit: "m/s", tunable: true, label: "Slime chase speed" }),
  sight: f32(7, { min: 1, max: 20, step: 0.5, unit: "m", tunable: true, label: "Slime sight" }),
  hitStop: u32(6, { min: 0, max: 20, unit: "ticks", tunable: true, label: "Hit stop" }),
  cameraHeight: f32(11, { min: 4, max: 30, step: 0.5, unit: "m", tunable: true, label: "Camera height" }),
  cameraBack: f32(8, { min: 2, max: 30, step: 0.5, unit: "m", tunable: true, label: "Camera distance behind" }),
});

/** The player's settings, kept in og.platform's settings. */
export const Prefs = defineResource("Prefs", {
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  nameLength: u32(0, { doc: "The player's name: its length, its code points in n0 … n11." }),
  n0: u32(0), n1: u32(0), n2: u32(0), n3: u32(0), n4: u32(0), n5: u32(0),
  n6: u32(0), n7: u32(0), n8: u32(0), n9: u32(0), n10: u32(0), n11: u32(0),
  loadRequest: u32(0),
  saves: u32(0),
});

/** The hero, on its capsule (the voxel body is a child). */
export const Hero = defineComponent("Hero", {
  airTime: f32(0, { unit: "s" }),
  stepClock: f32(0, { unit: "m", doc: "Ground covered since the last footstep." }),
});

/** A lantern: its glass glow, light and burst hang off it. */
export const Lantern = defineComponent("Lantern", {
  index: u32(0),
  lit: bool(false),
  litAt: f32(0, { unit: "s", doc: "Game.time when it caught." }),
  glow: f32(0, { doc: "Its light swelling in, 0 to 1." }),
});

/** A slime: patrols between two points, chases the hero on sight, hurts on contact. */
export const Slime = defineComponent("Slime", {
  index: u32(0),
  ax: f32(0), az: f32(0), bx: f32(0), bz: f32(0),
  leg: u32(0, { doc: "0 heading to a, 1 heading to b." }),
  chasing: bool(false),
  face: u32(Face.Calm, { doc: "Face.*: the bubble's layer." }),
  faceTime: f32(0, { unit: "s", doc: "Seconds the face has shown." }),
  heading: f32(0, { unit: "rad" }),
  hits: u32(0),
  bounce: f32(0, { unit: "s", doc: "Seconds it reels back after a hit." }),
  idleClip: f64(0, { doc: "The `idle` clip of models/slime.glb (og.load.animation's)." }),
  chaseClip: f64(0, { doc: "The `chase` clip." }),
});

/** A rock of the pile: where it was set, to measure how far the hero pushed it. */
export const Rock = defineComponent("Rock", {
  x0: f32(0), z0: f32(0),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
