// Ember Vale, the third-person archetype (3D profile): dusk is falling on
// a lake vale and four beacons stand dark on its knolls. Walk, run, wade
// and jump the strider there (og.CharacterController on the terrain), hold
// F by a beacon to kindle it (the hand reaches into the bowl through an
// og.IkChain) and light all four before night. Three stone sentinels walk
// their rounds on the navmesh (og.nav) and think with behaviour trees
// (og.ai): one that sees you calls the alarm and runs you down, and its
// grab costs a heart. Three hearts gone, or night, ends the run; every
// beacon lit wins it. Then the summary, and again.
//
// Every strider is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, so the vale plays alone, two on one device (a second
// strider, the screen split in halves) or two over a room in lockstep (each
// peer's camera behind its own), as src/creation.ts says. The two share the
// beacons (kindled together they catch twice as fast), the hearts and the
// dusk; the sentinels hunt the nearest.
//
// `start` loads the strings, the style, the sounds and the screen
// documents (assets/ui/), builds the vale (src/world.ts) and spawns the
// striders (src/hero.ts), the sentinels (src/sentinels.ts) and the voices
// (src/sound.ts). The beacons are in
// src/beacons.ts, the screens and the run's clock in src/ui.ts and the
// state in src/components.ts.

import { og, math } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { BEACONS, Game, Hero, Phase, Prefs, Tuning } from "./components.ts";
import { Beacons, catchBeacon } from "./beacons.ts";
import { FEET, heroOf, HeroMove, hypot, spawnHeroes, spawnViews, STAND, View } from "./hero.ts";
import { SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { alarm, grab, Sentinels, SENTINELS, spawnSentinels } from "./sentinels.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { LANGUAGES, startRun, toTitle, Ui } from "./ui.ts";
import { BEACON_AT, buildVale, ground, HOURS, must, setHour } from "./world.ts";

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/ember/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));
  buildVale(world);
  spawnHeroes(world);
  spawnViews(world);
  spawnSentinels(world);
  spawnVoices(world);
  // Behind the title the vale is reset and its beacons burn.
  toTitle(world, gameOf(world));
}

function gameOf(world: World) {
  return must(world.resource(Game));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `run`: start a run now, as Set out on the title does. */
function run(world: World): void {
  startRun(world, gameOf(world));
}

/** `warp:<beacon>` or `warp:<x>:<z>`: put seat 0's strider by a beacon (facing it) or on the ground at x, z. */
function warp(world: World, args: readonly string[]): void {
  warpSeat(world, 0, args);
}

/** `warp_seat:<seat>:<beacon>` or `warp_seat:<seat>:<x>:<z>`: the same for any seat's strider. */
function warpOf(world: World, args: readonly string[]): void {
  const seat = num(args[0], "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
  warpSeat(world, seat, args.slice(1));
}

function warpSeat(world: World, seat: number, args: readonly string[]): void {
  const g = gameOf(world);
  const hero = heroOf(g, seat);
  let x: number;
  let z: number;
  if (args.length === 1) {
    const b = BEACON_AT[num(args[0], "beacon")];
    if (b === undefined) throw new Error(`beacons are 0 to ${BEACONS - 1}`);
    // Where the strider stands to kindle it: STAND short of its centre, on the side of the lake.
    const len = hypot(b.x, b.z + 6);
    x = b.x - (b.x / len) * STAND;
    z = b.z - ((b.z + 6) / len) * STAND;
    must(world.set(hero, Hero, { facing: math.atan2(b.x - x, b.z - z) }));
  } else {
    x = num(args[0], "x");
    z = num(args[1], "z");
  }
  const h = ground(world, g, x, z).height;
  must(world.set(hero, og.Transform, { position: { x, y: h + FEET + 0.05, z } }));
}

/** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back): the room tests. */
function bot(_world: World, args: readonly string[]): void {
  setBot(num(args[0], "n"));
}

/** `light:<beacon>`: the beacon catches now, as kindling it does. */
function light(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("light needs a run (write run first)");
  const i = num(args[0], "beacon");
  if (i < 0 || i >= BEACONS) throw new Error(`beacons are 0 to ${BEACONS - 1}`);
  catchBeacon(world, g, i);
}

/** `hearts:<n>`: set the hearts left (0 ends the run, caught). */
function hearts(world: World, args: readonly string[]): void {
  gameOf(world).hearts = num(args[0], "n");
}

/** `dusk:<seconds>`: jump the run's clock to this many seconds played (night at Tuning.duskSeconds). */
function dusk(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  g.time = num(args[0], "seconds");
  if (g.phase === Phase.Title) setHour(world, HOURS.title);
}

/** `viewport:<desktop|phone>`: lay the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`. */
function language(world: World, args: readonly string[]): void {
  const p = must(world.resource(Prefs));
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  p.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

/** `stale_record`: another device writes the record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { runs: 9, wins: 3, best: 200 } }));
}

/** Seat 1's strider, for the facts. */
function second(world: World, g: ReturnType<typeof gameOf>) {
  const hero = heroOf(g, 1);
  const me = world.get(hero, Hero);
  const at = world.get(hero, og.Transform);
  if (!me.ok || !at.ok) return {};
  const r = (n: number) => Math.round(n * 100) / 100;
  return { near1: me.value.near, kindling1: me.value.kindling, speed1: r(me.value.speed), x1: r(at.value.position.x), z1: r(at.value.position.z) };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning, Prefs],
  systems: [Seats, ...og.systems.cameras, Ui, HeroMove, View, Sentinels, Beacons, Voices],
  start,
  commands: { run, warp, warp_seat: warpOf, bot, light, hearts, dusk, viewport, language, stale_record: staleRecord, alarm, grab },
  commandHelp: {
    run: { usage: "", doc: "Start a run now, as Set out does." },
    warp: { usage: "<beacon> | <x>:<z>", doc: "Put seat 0's strider where it kindles a beacon (0 to 3), or on the ground at x, z." },
    warp_seat: { usage: "<seat>:<beacon> | <seat>:<x>:<z>", doc: "Put a seat's strider where it kindles a beacon (0 to 3), or on the ground at x, z." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots play the same run." },
    light: { usage: "<beacon>", doc: "The beacon catches now, as kindling it does (the fourth wins)." },
    hearts: { usage: "<n>", doc: "Set the hearts left (0: the sentinels caught you)." },
    dusk: { usage: "<seconds>", doc: "Jump the run's clock to this many seconds played (night falls at Tuning.duskSeconds)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the record, so this one's next write is refused as stale and merged." },
    alarm: { usage: "<sentinel>", doc: "A sentinel (0 to 2: west, north, east; og.ai sends its entity) calls the alarm (its behaviour raises it on sight)." },
    grab: { usage: "<sentinel>", doc: "A sentinel's grab (0 to 2: west, north, east; og.ai sends its entity; its behaviour raises it within reach): a heart, unless the hero is out of reach or was just grabbed." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    return {
      phase: ["title", "play", "over"][g.phase],
      ending: ["none", "dawn", "night", "caught"][g.ending],
      hearts: g.hearts,
      lit: g.lit,
      litMask: g.litMask,
      time: Math.round(g.time * 10) / 10,
      hour: Math.round(g.hour * 100) / 100,
      near: g.near,
      kindle: Math.round(g.kindle * 100) / 100,
      wading: g.wading,
      grounded: g.grounded,
      speed: Math.round(g.speed * 100) / 100,
      jumps: g.jumps,
      splashes: g.splashes,
      grabs: g.grabs,
      alarms: g.alarms,
      runs: g.runs,
      wins: g.wins,
      best: Math.round(g.best * 10) / 10,
      sentinels: SENTINELS.length,
      language: LANGUAGES[g.language]!.code,
      seats: SEATS,
      ...(SEATS > 1 ? second(world, g) : {}),
    };
  },
});
