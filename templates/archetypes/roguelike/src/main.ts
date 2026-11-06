// Lamp Dive, the roguelike archetype (2D profile): a lamp-bearer goes two
// floors down a tomb, one step a turn. Every dive is new: the rooms, the
// corridors, the foes and the finds come from og.Rng streams forked with
// the dive's seed. The lamp lights only so far and only what a straight
// line reaches; what it lit before stays dimly remembered. Rats and
// skeletons think with behaviour trees (og.ai) and walk og.nav paths; the
// moss-faced guardian waits by the relic on floor 2, and its every third
// strike is charged and telegraphed. The lamp out is the end of the dive
// (no second chances): the summary, then a new dive.
//
// Every delver is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the dive plays alone, two in co-op on one device (a
// second delver beside the first, in a cooler ink, the camera framing
// both) or two over a room in lockstep, as src/creation.ts says. Each
// seat takes its own step; the world's turn (the foes, the finds) comes
// once every delver on its feet has. A delver struck down is out until the
// floor ends; both down, the lamp is out; the relic wins for both.
//
// `start` loads the floor (assets/maps/dive.ogt, carved at run time), the
// sheet, the minds, the sounds and the screen documents (assets/ui/), and
// spawns the pools: the lamp-bearer, ten foes, twelve finds and a voice
// per sound. The systems are in src/systems.ts, the foes' commands in
// src/foes.ts, the floor's plan in src/dive.ts, the screens in src/ui.ts
// and the state in src/components.ts.

import { og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { checkClips, clipOf } from "./art.ts";
import { Foe, FOE_SLOTS, FOES, Foes, Game, Hero, heroName, Loot, LOOT_SLOTS, Phase, setCell, setHp, Sfx, Sound, Tuning } from "./components.ts";
import { H, W } from "./dive.ts";
import { brace, hunt, roam, strike } from "./foes.ts";
import { LOCAL, ONLINE, SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { FoeTurn, Fog, HeroTurn, LootTurn, Voices } from "./systems.ts";
import { prefs } from "./device.ts";
import { FLOORS, LANGUAGES, setLanguage, startDive, Ui } from "./ui.ts";

/** The tomb's dark, linear. */
const DARK = { r: 0.0024, g: 0.0018, b: 0.0034 };
/** Collision layer of the floor's walls: they hide the lamp-bearer from the foes. */
const WALLS = 1;
/** The second seat's delver: a cool tint over the same sheet (linear). */
const SECOND_INK = { r: 0.55, g: 0.8, b: 1 };

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T;
function must(r: { ok: true } | { ok: false; reason: string }): void;
function must<T>(r: { ok: true; value?: T } | { ok: false; reason: string }): T | void {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/lampdive/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));

  const settings = must(world.resource(og.RenderSettings));
  settings.environmentIntensity = 0;
  settings.toneMapped = false;
  const camera = must(world.resource(og.Camera));
  camera.orthographic = true;
  camera.height = 13;
  camera.position = { x: W / 2, y: H / 2, z: 50 };
  camera.background = DARK;
  const physics = must(world.resource(og.Physics));
  physics.gravity = { x: 0, y: 0, z: 0 };

  const sheet = must(og.load.flipbook(world, "sprites/dive.ogt"));
  checkClips(sheet);
  const tilemap = must(og.load.tilemap(world, "maps/dive.ogt"));
  const map = must(world.spawn({ name: "tomb", position: { x: 0, y: H, z: 0 } }));
  must(world.insert(map, og.Tilemap, { map: tilemap.map, colliderLayers: WALLS }));

  const game = must(world.resource(Game));
  game.map = map as unknown as number;
  game.glow = must(og.load.texture(world, "sprites/glow.png")) as unknown as number;
  game.foeMind = must(og.ai.load(world, "ai/foe.ogt")).behaviour as unknown as number;
  game.guardianMind = must(og.ai.load(world, "ai/guardian.ogt")).behaviour as unknown as number;
  // Behind the title, the lamp-bearer waits in the first room of seed 1's floor.
  game.seed = 1;
  game.floor = 1;
  game.lamp = must(world.resource(Tuning)).lamp;
  game.build = true;

  // Beyond the map's edges is the dark too: four black quads over the lamp's glow there.
  const edges: [number, number, number, number][] = [[-50, H / 2, 100, 200], [W + 50, H / 2, 100, 200], [W / 2, H + 50, 200, 100], [W / 2, -50, 200, 100]];
  edges.forEach(([x, y, w, h], i) => {
    const e = must(world.spawn({ name: `edge-${i}`, position: { x, y, z: 0 } }));
    must(world.insert(e, og.Sprite, { width: w, height: h, color: DARK, layer: 25 }));
  });

  const sprite = (e: Entity, clip: string, layer: number, visible: boolean, color = { r: 1, g: 1, b: 1 }) => {
    must(world.insert(e, og.Sprite, { texture: sheet.texture, width: 1, height: 1, layer, visible, sortY: true, color }));
    must(world.insert(e, og.Flipbook, { sheet: sheet.sheet, clip: clipOf(clip) }));
  };
  // A delver per seat. The camera: alone it follows the lamp-bearer; two on
  // one device, the delvers' middle (`view`, moved by the Hero system); in a
  // room each peer's follows its own seat's delver.
  const follow = { smoothTime: 0.15, deadZone: { x: 0.5, y: 0.4 } };
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = must(world.spawn({ name: heroName(seat), position: { x: W / 2 + seat, y: H / 2, z: 0 } }));
    sprite(hero, "hero", 3, true, seat > 0 ? SECOND_INK : undefined);
    must(world.insert(hero, Hero, { seat }));
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    must(world.insert(hero, og.Perceivable, { team: 1 }));
    if (!LOCAL) must(world.insert(hero, og.Follow2D, ONLINE ? { ...follow, seat } : follow));
  }
  if (LOCAL) {
    const view = must(world.spawn({ name: "view", position: { x: W / 2, y: H / 2, z: 0 } }));
    must(world.insert(view, og.Follow2D, { smoothTime: 0.15, deadZone: { x: 1, y: 0.8 } }));
  }
  for (let i = 0; i < FOE_SLOTS; i++) {
    const f = must(world.spawn({ name: `foe-${i}`, position: { x: -60, y: -60, z: 0 } }));
    sprite(f, FOES[i % 2]!, 2, false);
    must(world.insert(f, Foe, { slot: i }));
    must(world.insert(f, og.AiAgent, { behaviour: game.foeMind as unknown as never, active: false }));
    must(world.insert(f, og.Perception, { sight: 6, fov: 360, occlude: true, occluders: WALLS, team: 2, memory: 6 }));
  }
  for (let i = 0; i < LOOT_SLOTS; i++) {
    const l = must(world.spawn({ name: `loot-${i}`, position: { x: -60, y: -60, z: 0 } }));
    sprite(l, "potion", 1, false);
    must(world.insert(l, Loot, { slot: i }));
  }

  const voices: [string, number, "sfx" | "ui", number][] = [
    ["audio/step.ogt", Sound.Step, "sfx", 0.35],
    ["audio/hit.wav", Sound.Hit, "sfx", 0.6],
    ["audio/hurt.wav", Sound.Hurt, "sfx", 0.6],
    ["audio/kill.wav", Sound.Kill, "sfx", 0.6],
    ["audio/pickup.wav", Sound.Pickup, "sfx", 0.5],
    ["audio/item.wav", Sound.Item, "sfx", 0.55],
    ["audio/drink.wav", Sound.Drink, "sfx", 0.6],
    ["audio/stairs.wav", Sound.Stairs, "sfx", 0.7],
    ["audio/roar.wav", Sound.Roar, "sfx", 0.7],
    ["audio/telegraph.wav", Sound.Telegraph, "sfx", 0.5],
    ["audio/fall.wav", Sound.Fall, "sfx", 0.7],
    ["audio/won.wav", Sound.Won, "sfx", 0.7],
    ["audio/bump.wav", Sound.Bump, "sfx", 0.5],
    ["audio/ui-press.ogt", Sound.Move, "ui", 0.3],
  ];
  for (const [path, bit, bus, volume] of voices) {
    const v = must(world.spawn({ name: `sfx-${bit}` }));
    must(world.insert(v, og.AudioSource, { sound: sound(path), playing: false, bus, volume, priority: 160 }));
    must(world.insert(v, Sfx, { bit }));
  }
  const music = must(world.spawn({ name: "music" }));
  must(world.insert(music, og.AudioSource, { sound: sound("audio/tomb.wav"), bus: "music", loop: true, volume: 0.6, priority: 220 }));
  const fx = must(world.spawn({ name: "music-bus" }));
  must(world.insert(fx, og.AudioBusFx, { bus: "music", duck: true, duckBus: "sfx", duckDepth: 6, duckRelease: 0.5 }));
  // The tomb's echo on the effects (og-audio §4).
  const echo = must(world.spawn({ name: "sfx-bus" }));
  must(world.insert(echo, og.AudioBusFx, { bus: "sfx", reverb: 0.22 }));
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `dive[:<seed>]`: start a dive now, as Dive on the title does, with this seed when given. */
function dive(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  startDive(world, g);
  if (args[0] !== undefined) g.seed = num(args[0], "seed");
}

/** `descend`: go down to the next floor now, as the stairs do. */
function descend(world: World): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play || g.floor >= FLOORS) return;
  g.floor += 1;
  g.build = true;
}

/** The seat a command names after its own arguments (0 when none). */
function seatArg(s: string | undefined): number {
  const seat = s === undefined ? 0 : num(s, "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seat is 0${SEATS > 1 ? ` to ${SEATS - 1}` : ""}`);
  return seat;
}

/** `hp:<n>[:<seat>]`: set a delver's health (the lamp-bearer's when no seat is given). */
function hp(world: World, args: readonly string[]): void {
  setHp(gameOf(world), seatArg(args[1]), num(args[0], "n"));
}

/** `give:<potion|blade|ward|oil>`: a find, as picking it up gives it. */
function give(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (args[0] === "potion") g.potions += 1;
  else if (args[0] === "blade") g.attack += 1;
  else if (args[0] === "ward") g.armour += 1;
  else if (args[0] === "oil") {
    g.lamp += 1;
    g.fogDirty = true;
  } else throw new Error("write give:<potion|blade|ward|oil>");
}

/** `warp:<x>:<y>[:<seat>]`: put a delver on a cell (the lamp-bearer when no seat is given; the fog follows). */
function warp(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  const x = num(args[0], "x");
  const y = num(args[1], "y");
  const seat = seatArg(args[2]);
  setCell(g, seat, x, y);
  g.fogDirty = true;
  const hero = world.find(heroName(seat));
  if (hero !== null) must(world.set(hero, og.Transform, { position: { x: x + 0.5, y: H - y - 0.5, z: 0 } }));
}

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`: this device's language. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  setLanguage(world, at);
}

/** `stale_record`: another device writes the dive record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { dives: 7, relics: 2 } }));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning, Foes],
  systems: [Seats, ...og.systems.cameras, Ui, HeroTurn, FoeTurn, LootTurn, Fog, Voices],
  render: { antialias: "off" },
  start,
  commands: { strike, hunt, roam, brace, dive, descend, hp, give, warp, viewport, language, stale_record: staleRecord, bot: (_world, args) => setBot(Number(args[0])) },
  commandHelp: {
    strike: { usage: "<foe>", doc: "A foe's strike at the lamp-bearer beside it (its behaviour raises it)." },
    hunt: { usage: "<foe>", doc: "A foe's step along og.nav's path to the lamp-bearer (its behaviour raises it)." },
    roam: { usage: "<foe>", doc: "A foe's step in a direction of the dice (its behaviour raises it)." },
    brace: { usage: "<foe>", doc: "The guardian holds its ground (its behaviour raises it)." },
    dive: { usage: "[<seed>]", doc: "Start a dive now, as Dive does; with a seed, that dive." },
    descend: { usage: "", doc: "Go down to the next floor now, as the stairs do." },
    hp: { usage: "<n>[:<seat>]", doc: "Set a delver's health (the lamp-bearer's when no seat is given)." },
    give: { usage: "<potion|blade|ward|oil>", doc: "A find, as picking it up gives it." },
    warp: { usage: "<x>:<y>[:<seat>]", doc: "Put a delver on a cell (column, row from the top; the lamp-bearer when no seat is given)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the dive record, so this one's next write is refused as stale and merged." },
    bot: { usage: "<n>", doc: "This device's seats take scripted input by the tick (1, 2, ...); 0 gives them back." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "over"][g.phase],
      seed: g.seed,
      floor: g.floor,
      deepest: g.deepest,
      hp: g.hp,
      hpMax: g.hpMax,
      attack: g.attack,
      armour: g.armour,
      lamp: g.lamp,
      potions: g.potions,
      gold: g.gold,
      kills: g.kills,
      hits: g.hits,
      hurts: g.hurts,
      finds: g.finds,
      turns: g.turns,
      hero: { x: g.heroX, y: g.heroY },
      seats: SEATS,
      ...(SEATS > 1 ? { hp2: g.hp2, hero2: { x: g.hero2X, y: g.hero2Y } } : {}),
      lit: g.lit,
      seen: g.seen,
      won: g.won,
      killedBy: g.killedBy >= 0 ? FOES[g.killedBy] : null,
      dives: g.dives,
      relics: g.relics,
      language: LANGUAGES[prefs.language]!.code,
    };
  },
});
