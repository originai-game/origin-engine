// Every piece of Greywatch's state lives in these reflected types: dumps,
// the probe, Tune and the command channel's `get` and `set` read and write
// them by name.
//
// The run (Game) and the knobs (Tuning) are resources (the player's
// settings are this device's own, src/device.ts, never in the world). The actors are rows spawned in `start`: a hero per seat
// (Hero), the risen (Foe), what they drop (Loot) and one voice per sound
// (Sfx). The conversation and the quest are og.story's own state
// (assets/story/), mirrored into Game for the facts.

import { bool, defineComponent, defineResource, f32, f64, i32, u32 } from "@origingame/engine";

/** The screen the game is on. */
export const Phase = { Title: 0, Play: 1, Over: 2 } as const;

/** How a run ended (Game.ending). */
export const Ending = { None: 0, Won: 1, Fallen: 2 } as const;

/** The quest as the yard plays it (Game.quest, mirrored from og.story's `seal` quest): not yet asked, clearing the yard, the chest, back to Maren, done. */
export const Quest = { None: 0, Clear: 1, Fetch: 2, Return: 3, Done: 4 } as const;

/** A foe's kind (Foe.kind). */
export const Kind = { Minion: 0, Captain: 1 } as const;

/** What a hero stands by (Hero.near): Talk acts on it. */
export const Near = { None: 0, Maren: 1, Chest: 2 } as const;

/** What a drop is (Loot.kind). */
export const LootKind = { Gold: 0, Potion: 1 } as const;

/**
 * Sound requests: bits of Game.sfx that the Voices system plays once each.
 * `Tap` is the touch HUD's buttons answering on the ui bus.
 */
export const Sound = {
  Step: 1, Swing: 2, Hit: 4, Clang: 8, Bones: 16, Hurt: 32, Potion: 64, Coin: 128,
  LevelUp: 256, Alarm: 512, Chest: 1024, Win: 2048, Lose: 4096, Tap: 8192, Crush: 16384, Quest: 32768,
} as const;

/** The potions the party sets out with. */
export const POTIONS = 2;
/** The highest level a hero reaches. */
export const MAX_LEVEL = 5;

/** The run: the party's shared bag, the quest, the conversation and the effects' bookkeeping. */
export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { doc: "Phase.* of src/components.ts." }),
  ending: u32(Ending.None, { doc: "Ending.* once a run is over." }),
  paused: bool(false),
  settingsOpen: bool(false),
  held: bool(false, { doc: "The time scale is held at 0 (paused or in the settings)." }),
  start: bool(false, { doc: "Every system resets its part of the keep this tick (a new run or the title)." }),
  runs: u32(0, { doc: "Runs started on this device (the store's `record`)." }),
  wins: u32(0, { doc: "Runs that ended with the quest done (the store's `record`)." }),
  best: f32(0, { unit: "s", doc: "The fastest win (the store's `record`; 0 none)." }),
  newBest: bool(false),
  recordRequest: u32(0, { doc: "The store request reading `record`." }),
  recordRevision: u32(0, { doc: "The revision `record` was read or written at." }),
  recordWrite: u32(0, { doc: "The store request writing `record`." }),
  recordMerge: bool(false, { doc: "A write was refused as stale: the record is read again and the merge written." }),
  recordRuns: u32(0, { doc: "Runs in the record as last read or written." }),
  recordWins: u32(0, { doc: "Wins in the record as last read or written." }),
  recordConflicts: u32(0, { doc: "Record writes refused as stale (another device or tab wrote first)." }),
  recordDirty: bool(false, { doc: "The record changed while a read or write of it was in flight." }),
  recordSentRuns: u32(0, { doc: "Runs in the record write in flight." }),
  recordSentWins: u32(0, { doc: "Wins in the record write in flight." }),
  time: f32(0, { unit: "s", doc: "Seconds played this run (paused time excluded)." }),
  quest: u32(Quest.None, { doc: "Quest.*: the yard quest's stage, mirrored from og.story." }),
  slain: u32(0, { doc: "Risen put down this run (the quest's count)." }),
  gold: u32(0, { doc: "The party's gold (the shared bag)." }),
  potions: u32(POTIONS, { doc: "The party's healing draughts (the shared bag)." }),
  seal: bool(false, { doc: "The warden's seal is in the bag (the quest's item, from the chest)." }),
  chestOpen: bool(false),
  conversation: u32(0, { doc: "The og.story conversation on screen (0: none)." }),
  talker: i32(-1, { doc: "The seat that started it (-1: none): its hero stands still while it runs." }),
  lineSeq: u32(0, { doc: "The conversation's line last answered (DialogueView.seq): one press answers one line." }),
  lineAt: u32(0, { unit: "ticks", doc: "The tick the line on screen first showed: a press answers it only after a beat." }),
  toast: u32(0, { doc: "A quest stage (Quest.*) whose toast waits for the conversation on screen to close (0: none)." }),
  toastUp: u32(0, { doc: "The stage toast on screen, plus one (0: none): a conversation that opens dismisses it." }),
  bagOpen: bool(false, { doc: "The bag (the inventory and the quest log) is open over the pause." }),
  loot: u32(0, { doc: "The next loot slot a fallen risen drops into (loot-0 to loot-7, round)." }),
  target: f64(0, { doc: "The foe the party last struck (its frame shows at the top; 0 none)." }),
  targetTicks: u32(0, { unit: "ticks", doc: "Ticks since it was struck (the frame leaves after a while)." }),
  hits: u32(0, { doc: "Blows that landed this run." }),
  hurts: u32(0, { doc: "Blows the party took this run." }),
  drunk: u32(0, { doc: "Potions drunk this run." }),
  levelUps: u32(0, { doc: "Level-ups this run (either hero)." }),
  overTicks: u32(0, { unit: "ticks", doc: "Ticks since the run ended; the summary comes after a beat." }),
  sfx: u32(0),
  hero: f64(0, { doc: "Seat 0's hero's entity." }),
  hero1: f64(0, { doc: "Seat 1's hero's entity (0 alone)." }),
  maren: f64(0, { doc: "Castellan Maren's entity." }),
  nav: f64(0, { doc: "The yard's navmesh." }),
});

/** The knobs. Tune reads the tunable fields; `set:Tuning.<field>=<value>` writes them. */
export const Tuning = defineResource("Tuning", {
  walkSpeed: f32(4.2, { min: 1, max: 9, step: 0.1, unit: "m/s", tunable: true, label: "Walk speed" }),
  swing: f32(0.5, { min: 0.2, max: 1.5, step: 0.05, unit: "s", tunable: true, label: "Swing lasts" }),
  damage: f32(12, { min: 1, max: 100, step: 1, tunable: true, label: "Blow damage", doc: "A level 1 hero's blow; each level adds Tuning.damagePerLevel." }),
  damagePerLevel: f32(4, { min: 0, max: 30, step: 1, tunable: true, label: "Damage per level" }),
  reach: f32(2.1, { min: 0.8, max: 4, step: 0.1, unit: "m", tunable: true, label: "Blow reach" }),
  arc: f32(75, { min: 10, max: 180, step: 5, unit: "deg", tunable: true, label: "Blow arc", doc: "Half the swing's arc each side of where the hero faces." }),
  heal: f32(55, { min: 5, max: 200, step: 5, tunable: true, label: "Potion heals" }),
  health: f32(100, { min: 10, max: 500, step: 5, tunable: true, label: "Hero health", doc: "At level 1; each level adds Tuning.healthPerLevel." }),
  healthPerLevel: f32(20, { min: 0, max: 100, step: 5, tunable: true, label: "Health per level" }),
  levelXp: f32(60, { min: 10, max: 500, step: 5, tunable: true, label: "XP to level 2", doc: "Each level after asks for this many more." }),
  respawn: f32(6, { min: 1, max: 30, step: 0.5, unit: "s", tunable: true, label: "Respawn after", doc: "Two heroes: a fallen one rises at the gate while the other stands." }),
  hitStop: u32(4, { min: 0, max: 20, unit: "ticks", tunable: true, label: "Hit stop" }),
  minionHealth: f32(34, { min: 1, max: 500, step: 1, tunable: true, label: "Minion health" }),
  captainHealth: f32(120, { min: 1, max: 1000, step: 5, tunable: true, label: "Captain health" }),
  minionDamage: f32(9, { min: 0, max: 100, step: 1, tunable: true, label: "Minion damage" }),
  captainDamage: f32(18, { min: 0, max: 200, step: 1, tunable: true, label: "Captain damage" }),
  minionXp: f32(30, { min: 0, max: 500, step: 5, tunable: true, label: "Minion XP" }),
  captainXp: f32(90, { min: 0, max: 1000, step: 5, tunable: true, label: "Captain XP" }),
  sight: f32(9, { min: 2, max: 30, step: 0.5, unit: "m", tunable: true, label: "Foe sight" }),
  potionChance: f32(0.4, { min: 0, max: 1, step: 0.05, tunable: true, label: "Potion drop chance" }),
});

/** A hero on the capsule (its body is a child): one per seat, moved by its og.PlayerInput alone. */
export const Hero = defineComponent("Hero", {
  seat: u32(0, { doc: "The seat it plays for (its og.PlayerInput.player)." }),
  body: f64(0, { doc: "Its skinned body's entity." }),
  before: u32(0, { doc: "Its og.PlayerInput.buttons last tick (press edges)." }),
  hp: f32(100),
  maxHp: f32(100),
  level: u32(1),
  xp: f32(0, { doc: "Experience towards the next level (spent on each level-up)." }),
  facing: f32(0, { unit: "rad", doc: "Its body's heading: 0 faces +z (south), growing towards +x." }),
  attack: f32(0, { unit: "s", doc: "Seconds into its swing (0: not swinging)." }),
  struck: bool(false, { doc: "This swing's blow has landed (or missed)." }),
  buffer: f32(0, { unit: "s", doc: "An Attack pressed this recently starts the next swing as soon as it may." }),
  hurt: f32(0, { unit: "s", doc: "Seconds of its hurt flash left." }),
  invulnerable: f32(0, { unit: "s", doc: "Seconds before it can be struck again." }),
  knock: f32(0, { unit: "s", doc: "Seconds it is thrown back (no control)." }),
  knockX: f32(0),
  knockZ: f32(0),
  dead: bool(false),
  respawn: f32(0, { unit: "s", doc: "Seconds until a fallen hero rises at the gate (two heroes, one standing)." }),
  near: u32(Near.None, { doc: "Near.*: what Talk acts on." }),
  speed: f32(0, { unit: "m/s", doc: "Its speed across the ground." }),
  stepClock: f32(0, { unit: "m", doc: "Ground covered since the last footstep." }),
  swings: u32(0),
  kills: u32(0),
});

/** A risen: a skeleton on the navmesh, thinking with assets/ai/<kind>.ogt. */
export const Foe = defineComponent("Foe", {
  index: u32(0),
  kind: u32(Kind.Minion, { doc: "Kind.*." }),
  hp: f32(30, { doc: "Its health (the behaviour reads it: `check Foe.hp above 0`)." }),
  maxHp: f32(30),
  walk: f32(1.3, { unit: "m/s", doc: "Its wandering pace (the behaviour's)." }),
  chase: f32(3, { unit: "m/s", doc: "Its chasing pace (the behaviour's)." }),
  reach: f32(1.5, { unit: "m", doc: "How near it closes before it winds up (the behaviour's)." }),
  body: f64(0, { doc: "Its skinned body's entity." }),
  homeX: f32(0),
  homeZ: f32(0),
  flash: f32(0, { unit: "s", doc: "Seconds of its struck flash left." }),
  attack: f32(0, { unit: "s", doc: "Seconds into its strike (0: none): a wind-up, then the clip, its blow on the clip's `hit` marker." }),
  landed: bool(false, { doc: "This strike's blow has landed (or missed)." }),
  oneShot: bool(false, { doc: "A hurt clip plays over the graph." }),
  stagger: f32(0, { unit: "s", doc: "Seconds it reels from a blow (it stands)." }),
  dying: f32(0, { unit: "s", doc: "Seconds since it fell (0: standing)." }),
  speed: f32(0, { unit: "m/s", doc: "Its speed across the ground (the graph's p0)." }),
  alert: bool(false, { doc: "It is chasing or striking." }),
  strikes: u32(0),
});

/** What a fallen risen dropped: gold or a potion, picked up by walking over it. */
export const Loot = defineComponent("Loot", {
  active: bool(false, { doc: "Lying in the yard (a slot of the pool is hidden when not)." }),
  kind: u32(LootKind.Gold, { doc: "LootKind.*." }),
  amount: u32(1),
  baseY: f32(0, { unit: "m" }),
  born: u32(0, { unit: "ticks", doc: "The tick it dropped (it bobs from then)." }),
});

/** One voice: plays its sound once when Game.sfx asks for its bit. */
export const Sfx = defineComponent("Sfx", {
  bit: u32(0),
  plays: u32(0),
});
