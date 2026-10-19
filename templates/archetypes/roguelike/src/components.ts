// Every piece of Lamp Dive's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The dive (Game), the knobs (Tuning), the player's settings (Prefs) and
// where each foe stands (Foes, so the hero's system can find what it walks
// into) are resources. The actors are pooled rows spawned in `start`: the
// hero, the foes, the loot and one voice per sound.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** What a foe is (Foe.kind). */
export const FoeKind = { Rat: 0, Skeleton: 1, Guardian: 2 } as const;
export const FOES = ["rat", "skeleton", "guardian"] as const;

/** What a find is (Loot.kind). */
export const LootKind = { Potion: 0, Blade: 1, Ward: 2, Oil: 3, Coin: 4, Relic: 5 } as const;
export const LOOT = ["potion", "blade", "ward", "oil", "coin", "relic"] as const;

/** Sound requests: bits of Game.sfx that the Voices system plays once each. */
export const Sound = { Step: 1, Hit: 2, Hurt: 4, Kill: 8, Pickup: 16, Item: 32, Drink: 64, Stairs: 128, Roar: 256, Telegraph: 512, Fall: 1024, Won: 2048, Bump: 4096, Move: 8192 } as const;

/** The most foes and finds a floor holds. */
export const FOE_SLOTS = 10;
export const LOOT_SLOTS = 12;

/** The dive: the lamp-bearer, the floor, the turn, the counts and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  dives: u32(0, { doc: "Dives started on this device (the store's `record`)." }),
  relics: u32(0, { doc: "Dives that took the relic (the store's `record`)." }),
  recordRequest: u32(0, { doc: "The store request reading `record`." }),
  recordRevision: u32(0, { doc: "The revision `record` was read or written at." }),
  recordWrite: u32(0, { doc: "The store request writing `record`." }),
  recordMerge: bool(false, { doc: "A write was refused as stale: the record is read again and the merge written." }),
  recordDives: u32(0, { doc: "Dives in the record as last read or written: Game.dives less this is what this device adds." }),
  recordRelics: u32(0, { doc: "Relics in the record as last read or written." }),
  recordConflicts: u32(0, { doc: "Record writes refused as stale (another device or tab wrote first)." }),
  seed: u32(0, { doc: "This dive's seed: every floor's rooms, foes and finds come from og.Rng streams forked with it." }),
  floor: u32(0, { doc: "The floor the lamp-bearer is on (1 or 2; 0 before the first dive)." }),
  deepest: u32(0),
  build: bool(false, { doc: "The Hero system carves the floor this tick." }),
  place: bool(false, { doc: "The Foes and Loot systems place this floor's rows this tick." }),
  hp: i32(0),
  hpMax: i32(0),
  attack: i32(0, { doc: "Damage a strike deals (before the die's +0 or +1)." }),
  armour: i32(0, { doc: "Damage a foe's strike loses." }),
  lamp: f32(0, { unit: "cells", doc: "How far the lamp lights." }),
  potions: u32(0),
  gold: u32(0),
  kills: u32(0),
  hits: u32(0, { doc: "Strikes that landed on a foe." }),
  hurts: u32(0, { doc: "Strikes that landed on the lamp-bearer." }),
  finds: u32(0, { doc: "Things picked up." }),
  turns: u32(0),
  heroX: i32(0, { doc: "The lamp-bearer's cell, a map column." }),
  heroY: i32(0, { doc: "The lamp-bearer's cell, a map row (0 the top)." }),
  stairsX: i32(-1, { doc: "The stairs' cell on floor 1 (-1 none)." }),
  stairsY: i32(-1),
  foesTurn: bool(false, { doc: "The foes act this turn: their behaviour (assets/ai/*.ogt) checks it." }),
  lock: u32(0, { unit: "ticks", doc: "Ticks before the next action is taken (the step's or the strike's animation)." }),
  strikeSlot: i32(-1, { doc: "The foe the lamp-bearer struck this tick, for the Foes system."}),
  strikeDamage: i32(0),
  queued: u32(0, { doc: "An action asked for while the last played out (0 none, 1-4 a direction, 5 wait, 6 drink)." }),
  repeat: u32(0, { unit: "ticks", doc: "Ticks a direction has been held (it repeats after 14, then every 7)." }),
  lastDir: u32(0, { doc: "The direction held last tick (0 none, 1 up, 2 right, 3 down, 4 left)." }),
  padDir: u32(0, { doc: "The HUD d-pad's direction this tick (written by Ui)." }),
  padWait: bool(false, { doc: "The HUD's Wait was pressed this tick." }),
  padDrink: bool(false, { doc: "The HUD's Drink was pressed this tick." }),
  refusedTick: i32(-1, { unit: "ticks", doc: "The tick a draught was refused (the HUD shakes Drink)." }),
  foesAt: i32(-1, { unit: "ticks", doc: "The tick foesTurn went up; the Foes system takes it down on a later tick, after og.ai has thought with it." }),
  foeMind: f64(0, { doc: "assets/ai/foe.ogt's behaviour handle." }),
  guardianMind: f64(0, { doc: "assets/ai/guardian.ogt's behaviour handle." }),
  won: bool(false),
  killedBy: i32(-1, { doc: "FoeKind.* of the foe that put the lamp out (-1 none)." }),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the lamp went out or the relic was taken; the card comes after a beat." }),
  newRecord: bool(false),
  sfx: u32(0),
  fogDirty: bool(false, { doc: "The lamp-bearer moved: the Fog system lights the cells it sees." }),
  lit: u32(0, { doc: "Cells the lamp lights now." }),
  seen: u32(0, { doc: "Cells of this floor the lamp has ever lit (lit now or remembered)." }),
  grid: f64(0, { doc: "The floor's og.nav grid." }),
  map: f64(0, { doc: "The floor's og.Tilemap entity." }),
  glow: f64(0, { doc: "The lamp's light texture." }),
  viewW: f32(0, { unit: "cssPx" }),
  viewH: f32(0, { unit: "cssPx" }),
  language: u32(0, { doc: "Prefs.language, mirrored for the facts." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  heroHp: i32(20, { min: 1, max: 99, step: 1, tunable: true, label: "Lamp-bearer's health" }),
  heroAttack: i32(3, { min: 1, max: 20, step: 1, tunable: true, label: "Strike" }),
  lamp: f32(4.5, { min: 2, max: 12, step: 0.5, unit: "cells", tunable: true, label: "Lamp reach" }),
  potionHeal: i32(8, { min: 1, max: 40, step: 1, tunable: true, label: "A draught heals" }),
  ratHp: i32(5, { min: 1, max: 40, tunable: true, label: "Rat health" }),
  ratAttack: i32(1, { min: 0, max: 20, tunable: true, label: "Rat strike" }),
  skeletonHp: i32(9, { min: 1, max: 60, tunable: true, label: "Skeleton health" }),
  skeletonAttack: i32(2, { min: 0, max: 20, tunable: true, label: "Skeleton strike" }),
  guardianHp: i32(18, { min: 1, max: 99, tunable: true, label: "Guardian health" }),
  guardianAttack: i32(3, { min: 0, max: 20, tunable: true, label: "Guardian strike", doc: "Its charged strike, every third turn and telegraphed the turn before, deals twice this." }),
  sight: f32(6, { min: 1, max: 15, step: 0.5, unit: "m", tunable: true, label: "Foes' sight", doc: "How far a foe sees (og.Perception.sight); walls hide the lamp-bearer." }),
  stepTime: f32(0.09, { min: 0.02, max: 0.4, step: 0.01, unit: "s", tunable: true, label: "Step time", doc: "How long a step's tween takes; actions wait for it." }),
  hitStop: u32(4, { min: 0, max: 12, unit: "ticks", tunable: true, label: "Hit stop" }),
});

/** The player's settings: kept in og.platform's settings, restored on the first ticks. */
export const Prefs = defineResource("Prefs", {
  master: f32(1, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(1, { min: 0, max: 1 }),
  language: u32(0, { doc: "0 English, 1 简体中文 (LANGUAGES in src/ui.ts)." }),
  reducedMotion: bool(false),
  loadRequest: u32(0),
  saves: u32(0),
});

/** Where each foe stands and whether it is up: the Hero system reads it to strike what it walks into. */
export const Foes = defineResource("Foes", Object.fromEntries(
  Array.from({ length: FOE_SLOTS }, (_, i) => [
    [`x${i}`, i32(-1)],
    [`y${i}`, i32(-1)],
  ]).flat(),
) as Record<string, ReturnType<typeof i32>>);

/** The lamp-bearer. */
export const Hero = defineComponent("Hero", {
  steps: u32(0, { doc: "Cells walked." }),
  bumps: u32(0, { doc: "Steps into stone (refused with a bump)." }),
});

/** A foe: a pooled row a floor switches on. What it decides is assets/ai/<kind>.ogt (og.ai). */
export const Foe = defineComponent("Foe", {
  slot: u32(0),
  active: bool(false),
  kind: u32(FoeKind.Rat),
  hp: i32(0),
  hpMax: i32(0),
  attack: i32(0),
  cx: i32(0, { doc: "Its cell." }),
  cy: i32(0),
  turns: u32(0, { doc: "Its turns on this floor (the guardian charges every third)." }),
  charging: bool(false, { doc: "Its next strike is charged (telegraphed in red)." }),
  dying: u32(0, { unit: "ticks" }),
});

/** A find on the floor. */
export const Loot = defineComponent("Loot", {
  slot: u32(0),
  active: bool(false),
  kind: u32(LootKind.Potion),
  amount: u32(0, { doc: "Gold in a coin pile." }),
  cx: i32(0),
  cy: i32(0),
  taken: u32(0, { unit: "ticks", doc: "Ticks since it was picked up (it rises and fades, then the row parks)." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
