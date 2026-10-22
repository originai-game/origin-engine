// Rumpus Isle's state: the match (Game), the player's settings (Prefs), the
// tuning (Tuning), the requests for sounds and effects (Sfx, Fx), and the
// components of the critters, tiles and berries. Everything a tick reads is
// here, in the world, so snapshots, replays and the world hash carry it.

import { bool, defineComponent, defineResource, Entity, f32, f64, i32, Material, u32 } from "@origingame/engine";

/** The screens and what the match is doing. */
export const Phase = {
  Title: 0,
  Lobby: 1,
  /** The round's card and the 3, 2, 1 before it. */
  Intro: 2,
  Play: 3,
  /** The round is decided: a beat to see it, then the scoreboard. */
  Over: 4,
  Scores: 5,
  Podium: 6,
} as const;

/** What a screen asks Rules to do (Game.ask). */
export const Ask = { None: 0, Match: 1, Next: 2, Lobby: 3, Title: 4, Again: 5 } as const;

/** The three minigames, played in this order. */
export const Round = { Bump: 0, Berry: 1, Crumble: 2 } as const;
export const ROUNDS = 3;
export const ROUND_KEYS = ["round.bump", "round.berry", "round.crumble"] as const;

/** Who drives a player: a pad (0 to 3), or a half of the keyboard. */
export const Source = { None: -1, Pad0: 0, Pad1: 1, Pad2: 2, Pad3: 3, KeysLeft: 4, KeysRight: 5 } as const;
export const SOURCES = 6;

export const MAX_PLAYERS = 4;

/**
 * The critters' colours: each with its linear RGB for the body, its sRGB for the
 * UI, the critter's name (a name, not a word: the same in every language) and
 * the string key of the colour's name.
 */
export const COLOURS = [
  { name: "Pip", key: "colour.coral", rgb: "#ff6f5e" },
  { name: "Juno", key: "colour.sun", rgb: "#ffc93c" },
  { name: "Moss", key: "colour.mint", rgb: "#54c98a" },
  { name: "Bix", key: "colour.sky", rgb: "#4aa8ff" },
  { name: "Plum", key: "colour.grape", rgb: "#a77bff" },
  { name: "Tofu", key: "colour.cream", rgb: "#f2ece0" },
] as const;

/** Collision layers, as bits (og.Collider.layers and mask). */
export const Bits = { Ground: 1, Critter: 2, Berry: 4 } as const;

/** og.platform's keys. */
export const Keys = { records: "records" } as const;

export const Game = defineResource("Game", {
  phase: u32(Phase.Title, { label: "Phase (0 title, 1 lobby, 2 intro, 3 play, 4 over, 5 scores, 6 podium)" }),
  round: u32(0, { min: 0, max: 2 }),
  paused: bool(),
  settingsOpen: bool(),
  /** Ticks left in the phase's own clock: the intro's count-in, the round, the beat after it. */
  ticks: u32(0),
  /** Seconds left in the round (shown), from ticks. */
  left: f32(0),
  /** Ticks since the round's go: crumbling starts after a grace. */
  played: u32(0),
  /** How many critters are still in this round (bump and crumble). */
  standing: u32(0),
  /** Places handed out so far this round (from the last place up). */
  outs: u32(0),
  /** What the screens asked for this tick, done by Rules (Ask.*): the screens build UI, Rules moves the world. */
  ask: u32(0),
  /** The title was left once: from then it enters as every other screen (og-platform-sdk §3). */
  leftTitle: bool(),
  /** The lobby: all joined players ready, counting down to the first round (0: not). */
  launch: u32(0),
  /** A match was started at least once (the lobby's Back goes to the title). */
  matches: u32(0),
  /** The berries still to come up this round (the ring of berry entities). */
  berrySeq: u32(0),
  /** Records, as og.platform's `records` key keeps them: most berries in a round, longest on the tiles (s). */
  bestBerries: u32(0),
  bestCrumble: f32(0),
  /** This round set a record. */
  newRecord: bool(),
  /** The records were read (or failed): the title stops waiting. */
  recordsRead: bool(),
  recordsRequest: f64(0),
  recordsRevision: i32(-1),
  saveRequest: f64(0),
  /** A records write is wanted (a round set one). */
  writeRecords: bool(),
  settingsRequest: f64(0),
  /** The settings closed: save them. */
  saveSettings: bool(),
  saves: u32(0),
  /**
   * The seats, 0 to 3 (FIELD<k>; src/seats.ts reads and writes them by seat): who sits
   * there (joined, the source driving it, the colour, ready in the lobby), and how
   * the match goes for them (in the round still, the place, the points, the
   * berries, the seconds on the tiles), and where its critter is (x, z), for the
   * systems that loop over tiles and berries.
   */
  joined0: bool(),
  joined1: bool(),
  joined2: bool(),
  joined3: bool(),
  source0: i32(-1, { min: -1, max: 5 }),
  source1: i32(-1, { min: -1, max: 5 }),
  source2: i32(-1, { min: -1, max: 5 }),
  source3: i32(-1, { min: -1, max: 5 }),
  colour0: u32(0, { min: 0, max: 5 }),
  colour1: u32(0, { min: 0, max: 5 }),
  colour2: u32(0, { min: 0, max: 5 }),
  colour3: u32(0, { min: 0, max: 5 }),
  ready0: bool(),
  ready1: bool(),
  ready2: bool(),
  ready3: bool(),
  alive0: bool(),
  alive1: bool(),
  alive2: bool(),
  alive3: bool(),
  place0: u32(0),
  place1: u32(0),
  place2: u32(0),
  place3: u32(0),
  gained0: u32(0),
  gained1: u32(0),
  gained2: u32(0),
  gained3: u32(0),
  points0: u32(0),
  points1: u32(0),
  points2: u32(0),
  points3: u32(0),
  berries0: u32(0),
  berries1: u32(0),
  berries2: u32(0),
  berries3: u32(0),
  lasted0: f32(0),
  lasted1: f32(0),
  lasted2: f32(0),
  lasted3: f32(0),
  x0: f32(0),
  x1: f32(0),
  x2: f32(0),
  x3: f32(0),
  z0: f32(0),
  z1: f32(0),
  z2: f32(0),
  z3: f32(0),
  y0: f32(0),
  y1: f32(0),
  y2: f32(0),
  y3: f32(0),
  /** Counters the playtests read. */
  bumps: u32(0),
  dashes: u32(0),
  splashes: u32(0),
  berriesTaken: u32(0),
  cracks: u32(0),
  joins: u32(0),
  leaves: u32(0),
  /** A focusable control without a name (accessibility). */
  a11yUnnamed: u32(0),
  a11yNodes: u32(0),
  /**
   * The display font's handle for the world's name tags (og.load.font). A handle wants a field of its
   * own kind (og-core §4), but there is no Font() factory: an f64 holds it (FINDINGS.md #2).
   */
  font: f64(0),
  /** The title's camera drift and the podium's, eased in seconds. */
  shown: f32(0),
});

/**
 * A critter's body: one per seat, spawned in start ("p0" to "p3"), off the
 * island and asleep until someone sits there. Its rigid body is the sphere that
 * bumps (rotation locked but about +y, so it turns to face its way); its look is
 * its children (body, eyes, feet). Who sits there and how the match goes for
 * them is the seat's, in Game (`seat`).
 */
export const Critter = defineComponent("Critter", {
  seat: u32(0, { min: 0, max: 3 }),
  /** The way it faces (radians about +y, 0 is +z). */
  yaw: f32(0),
  /** Ticks left of a dash, and before the next. */
  dash: u32(0),
  cooldown: u32(0),
  /** Ticks left of being stunned after a dash hit it (no control). */
  stun: u32(0),
  /** Ticks since it went into the sea (berry round: back on the island after a while). */
  sunk: u32(0),
  /**
   * The velocity Drive handed the step (m/s). og.CollisionStarted is read the tick after the step,
   * when og.Velocity already holds the contact's answer, so this is how hard a bump closed (FINDINGS.md #9).
   */
  vx: f32(0),
  vz: f32(0),
  /** The material of its body, made in start in its seat's colour. */
  body: Material(),
  /** The body's mesh entity (squashed on bumps). */
  mesh: Entity(),
});

/** A tile of the crumble round: kinematic, it shakes once stood on and then drops. */
export const Tile = defineComponent("Tile", {
  /** Ticks since someone first stood on it (0: whole). */
  crack: u32(0),
  /** Dropping: its fall speed. */
  fall: f32(0),
  /** Where it rests. */
  homeX: f32(0),
  homeZ: f32(0),
  down: bool(),
});

/** A berry of the berry round (a ring of entities, shown when up). */
export const Berry = defineComponent("Berry", {
  up: bool(),
  /** A golden berry counts three. */
  gold: bool(),
  /** Ticks it has been up (it pops in, bobs, and a gold one leaves after a while). */
  age: u32(0),
});

export const Prefs = defineResource("Prefs", {
  master: f32(0.9, { min: 0, max: 1 }),
  music: f32(0.7, { min: 0, max: 1 }),
  effects: f32(0.9, { min: 0, max: 1 }),
  muted: bool(),
  reducedMotion: bool(),
  highContrast: bool(),
  vibration: bool(true),
  /** Index into LANGUAGES. */
  language: u32(0, { min: 0, max: 1 }),
});

export const Tuning = defineResource("Tuning", {
  /** Top speed a stick or the keys ask for, m/s. */
  speed: f32(5.2, { min: 1, max: 12, tunable: true, unit: "m/s" }),
  /** How hard a critter pushes towards the speed it wants (1/s). */
  grip: f32(9, { min: 1, max: 30, tunable: true }),
  /** A dash: its speed and how long it lasts. */
  dashSpeed: f32(11, { min: 4, max: 20, tunable: true, unit: "m/s" }),
  dashTicks: u32(10, { min: 1, max: 40 }),
  dashCooldown: u32(45, { min: 0, max: 180 }),
  /**
   * The shove a bump gives the critter hit, N·s per m/s of closing speed (a critter weighs 1 kg: a
   * full-speed head-on at 10 m/s shoves each about 2.4 m/s, a landed dash at 16 m/s throws the one hit
   * nearly 8 m/s), and its least (a jostle).
   */
  bumpShove: f32(0.3, { min: 0, max: 5, tunable: true }),
  bumpMin: f32(1.5, { min: 0, max: 20, tunable: true }),
  /** Ticks a dashed-into critter loses control for. */
  stunTicks: u32(14, { min: 0, max: 60 }),
  /** Rounds' lengths, s. */
  bumpSeconds: f32(40, { min: 5, max: 120 }),
  berrySeconds: f32(30, { min: 5, max: 120 }),
  crumbleSeconds: f32(45, { min: 5, max: 120 }),
  /** The crumble round: ticks from standing on a tile to its drop, and the grace before any crack. */
  crackTicks: u32(42, { min: 5, max: 240 }),
  graceTicks: u32(60, { min: 0, max: 600 }),
  /** Below this a critter is in the sea. */
  seaLevel: f32(-0.7, { min: -5, max: 0 }),
  /** Berry round: ticks before a sunk critter is back. */
  respawnTicks: u32(75, { min: 10, max: 300 }),
  /** Berry round: the berries up at once. */
  berriesUp: u32(6, { min: 1, max: 12 }),
});

/** One voice per sound (og-audio §1): Voices plays a voice when its bit is set in Sfx.bits. */
export const Voice = defineComponent("Voice", { bit: u32(0), plays: u32(0) });

export const Sound = {
  Bump: 1 << 0,
  Dash: 1 << 1,
  Splash: 1 << 2,
  Berry: 1 << 3,
  Gold: 1 << 4,
  Crack: 1 << 5,
  Crumble: 1 << 6,
  Beep: 1 << 7,
  Go: 1 << 8,
  Whistle: 1 << 9,
  Win: 1 << 10,
  Join: 1 << 11,
  Leave: 1 << 12,
  Ready: 1 << 13,
} as const;

export const SOUND_FILES: readonly [number, string][] = [
  [Sound.Bump, "audio/bump.ogt"],
  [Sound.Dash, "audio/dash.ogt"],
  [Sound.Splash, "audio/splash.wav"],
  [Sound.Berry, "audio/berry.ogt"],
  [Sound.Gold, "audio/gold.ogt"],
  [Sound.Crack, "audio/crack.ogt"],
  [Sound.Crumble, "audio/crumble.ogt"],
  [Sound.Beep, "audio/beep.ogt"],
  [Sound.Go, "audio/go.ogt"],
  [Sound.Whistle, "audio/whistle.ogt"],
  [Sound.Win, "audio/win.ogt"],
  [Sound.Join, "audio/join.ogt"],
  [Sound.Leave, "audio/leave.ogt"],
  [Sound.Ready, "audio/ready.ogt"],
];

export const Sfx = defineResource("Sfx", { bits: u32(0), plays: u32(0) });

/**
 * One-shot effects asked for this tick, played by the Effects system: where a
 * splash, a dust puff, a sparkle and a tile's crumbs go (the latest of each).
 */
export const Fx = defineResource("Fx", {
  splash: bool(),
  splashX: f32(0),
  splashZ: f32(0),
  dust: bool(),
  dustX: f32(0),
  dustZ: f32(0),
  sparkle: bool(),
  sparkleX: f32(0),
  sparkleZ: f32(0),
  crumbs: bool(),
  crumbsX: f32(0),
  crumbsZ: f32(0),
  /** Camera trauma to add (og-feel §4). */
  shake: f32(0),
});

/** An effect's entity: the emitter it plays. */
export const Effect = defineComponent("Effect", { kind: u32(0) });
