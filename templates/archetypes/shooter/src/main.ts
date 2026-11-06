// Holdout, the shooter archetype (3D profile): an outpost's landing field
// at dusk, ringed by canyon walls with three gates. Hold it against five
// waves of walkers (stone stouts that close in and strike) and hover
// drones (that keep their range and charge a bolt down a clear line) with
// a hitscan pulse rifle: recoil, a muzzle flash, tracers, sparks and holes
// in the cover, a 24-round magazine and a reload. Health, a score, waves
// with a breather between them; every trooper down loses the run, the
// fifth wave cleared wins it. Then the summary, and again.
//
// Every trooper is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the arena plays alone, two on one device (the screen
// split in halves, each with its own view and HUD) or two over a room in
// lockstep (each peer looking out of its own seat), as src/creation.ts
// says. Two seats hold the line together (co-op: the waves grow by half,
// a downed partner is back at the next wave) or, chosen on the title, duel
// each other to DUEL_FRAGS. The view is src/view.ts's, the same file in
// every 3D starter: first person, third person over the shoulder, or
// isometric twin-stick (the move relative to the screen, the look turning
// the trooper), by creation.view.
//
// `start` loads the strings, the sounds and the screen documents, builds
// the arena (src/world.ts) and spawns the troopers (src/hero.ts), their
// rifles (src/weapon.ts), the foes' pools (src/foes.ts) and the voices
// (src/sound.ts). The screens are in src/ui.ts, the state in
// src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Foe, FoeState, Game, Hero, Kind, Mode, Phase, Tuning, WAVES, WaveState } from "./components.ts";
import { bringAt, foes, Foes, hurtFoe, hurtHero, spawnFoes, Waves } from "./foes.ts";
import { cameraLayers, FEET, heroFacts, heroOf, HeroMove, Shoulder, spawnHeroes } from "./hero.ts";
import { SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { languageCode, LANGUAGES, setLanguage, startRun, toTitle, Ui } from "./ui.ts";
import { SeatLook, ViewRig } from "./view.ts";
import { Weapon, spawnWeapons } from "./weapon.ts";
import { buildArena, must } from "./world.ts";
import { prefs } from "./device.ts";

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const sound = (path: string) => must(og.load.sound(world, path));
  // The style is the look's `ui` line (assets/looks/holdout.ogt); these are its sounds.
  must(og.ui.configure(world, { sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));
  buildArena(world);
  spawnHeroes(world);
  spawnWeapons(world);
  spawnFoes(world);
  spawnVoices(world);
  cameraLayers(world);
  toTitle(gameOf(world));
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

function playing(world: World) {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("this needs a run (write run or duel first)");
  return g;
}

/** `run`: a holdout now, as Hold the line does. */
function run(world: World): void {
  startRun(world, gameOf(world), Mode.Holdout);
}

/** `duel`: a duel now (two seats), as Duel does. */
function duel(world: World): void {
  if (SEATS < 2) throw new Error("a duel needs two seats (players local-multiplayer or online-lockstep)");
  startRun(world, gameOf(world), Mode.Duel);
}

/** `warp:<x>:<z>[:<seat>]`: a trooper on the floor at x, z. */
function warp(world: World, args: readonly string[]): void {
  const seat = args[2] === undefined ? 0 : num(args[2], "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
  must(world.set(heroOf(gameOf(world), seat), og.Transform, { position: { x: num(args[0], "x"), y: FEET + 0.05, z: num(args[1], "z") } }));
}

/** `aim:<yaw°>:<pitch°>[:<seat>]`: turns a trooper's look (yaw 0 looks north, growing to the left). */
function aim(world: World, args: readonly string[]): void {
  const seat = args[2] === undefined ? 0 : num(args[2], "seat");
  const deg = Math.PI / 180;
  must(world.set(heroOf(gameOf(world), seat), SeatLook, { yaw: num(args[0], "yaw") * deg, pitch: num(args[1], "pitch") * deg }));
}

/** `wave:<n>`: the breather before wave n, its foes gone (n past WAVES: the arena held). */
function wave(world: World, args: readonly string[]): void {
  const g = playing(world);
  const n = num(args[0], "wave");
  if (!Number.isInteger(n) || n < 1 || n > WAVES) throw new Error(`waves are 1 to ${WAVES}`);
  clearFoes(world);
  g.wave = n;
  g.waveState = WaveState.Breather;
  g.waveTimer = 0.5;
  g.toSpawn = 0;
}

/** `clear`: every standing foe downed now (by seat 0), the wave's rest gone. */
function clear(world: World): void {
  const g = playing(world);
  g.toSpawn = 0;
  clearFoes(world);
}

function clearFoes(world: World): void {
  const g = gameOf(world);
  for (const e of foes(world)) hurtFoe(world, g, e, 1e6, false);
}

/** `spawn:<walker|drone>:<x>:<z>`: a foe from its pool on the floor at x, z (it hunts from there). */
function spawn(world: World, args: readonly string[]): void {
  const g = playing(world);
  const kind = args[0] === "walker" ? Kind.Walker : args[0] === "drone" ? Kind.Drone : -1;
  if (kind < 0) throw new Error("write spawn:<walker|drone>:<x>:<z>");
  if (bringAt(world, g, must(world.resource(Tuning)), kind, { x: num(args[1], "x"), y: 0, z: num(args[2], "z") }) === null) throw new Error(`every ${args[0]} is out`);
  g.alive += 1;
}

/** `hurt:<amount>[:<seat>]`: a trooper takes damage from straight ahead. */
function hurt(world: World, args: readonly string[]): void {
  const g = playing(world);
  const seat = args[1] === undefined ? 0 : num(args[1], "seat");
  const p = must(world.get(heroOf(g, seat), og.Transform)).position;
  hurtHero(world, g, seat, num(args[0], "amount"), { x: p.x, y: p.y, z: p.z - 5 });
}

/** `ammo:<magazine>[:<reserve>]`: seat 0's rounds. */
function ammo(world: World, args: readonly string[]): void {
  const hero = heroOf(gameOf(world), 0);
  must(world.set(hero, Hero, { mag: num(args[0], "magazine"), ...(args[1] === undefined ? {} : { reserve: num(args[1], "reserve") }) }));
}

/** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back). */
function bot(_world: World, args: readonly string[]): void {
  setBot(num(args[0], "n"));
}

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  setLanguage(world, at);
}

/** `stale_record`: another device writes the record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { runs: 9, wins: 3, best: 4200 } }));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Seats, ViewRig, Shoulder, Ui, HeroMove, Weapon, Foes, Waves, ...og.systems.cameras, Voices],
  start,
  commands: { run, duel, warp, aim, wave, clear, spawn, hurt, ammo, bot, viewport, language, stale_record: staleRecord },
  commandHelp: {
    run: { usage: "", doc: "Start a holdout now, as Hold the line does." },
    duel: { usage: "", doc: "Start a duel between the two seats now, as Duel does (two seats only)." },
    warp: { usage: "<x>:<z>[:<seat>]", doc: "Put a seat's trooper on the floor at x, z (seat 0 by default)." },
    aim: { usage: "<yaw°>:<pitch°>[:<seat>]", doc: "Turn a seat's look: yaw 0 looks north and grows to the left, pitch above the horizon." },
    wave: { usage: "<n>", doc: "The breather before wave n (1 to 5), the foes standing gone." },
    clear: { usage: "", doc: "Down every standing foe now (seat 0 scores them) and drop the rest of the wave." },
    spawn: { usage: "<walker|drone>:<x>:<z>", doc: "A foe from its pool on the floor at x, z, hunting the nearest trooper." },
    hurt: { usage: "<amount>[:<seat>]", doc: "A trooper takes damage from straight ahead (0 health: down)." },
    ammo: { usage: "<magazine>[:<reserve>]", doc: "Set seat 0's rounds in the magazine and in reserve." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots play the same run." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the record, so this one's next write is refused as stale and merged." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    let walkers = 0;
    let drones = 0;
    for (const e of foes(world)) {
      const f = world.get(e, Foe);
      if (!f.ok || f.value.state === FoeState.Pooled || f.value.state === FoeState.Dying) continue;
      if (f.value.kind === Kind.Walker) walkers += 1;
      else drones += 1;
    }
    const seats: Record<string, unknown> = {};
    for (let s = 0; s < SEATS; s++) seats[`seat${s}`] = heroFacts(world, g, s);
    return {
      phase: ["title", "play", "over"][g.phase],
      mode: g.mode === Mode.Duel ? "duel" : "holdout",
      ending: ["none", "held", "overrun", "duel"][g.ending],
      winner: g.winner,
      paused: g.paused,
      wave: g.wave,
      fighting: g.waveState === WaveState.Fighting,
      toSpawn: g.toSpawn,
      walkers,
      drones,
      score: g.score,
      kills: g.kills,
      headshots: g.headshots,
      shots: g.shots,
      hits: g.hits,
      downs: g.downs,
      time: Math.round(g.time * 10) / 10,
      runs: g.runs,
      wins: g.wins,
      best: g.best,
      language: languageCode(),
      saves: prefs.saves,
      seats: SEATS,
      ...seats,
    };
  },
});
