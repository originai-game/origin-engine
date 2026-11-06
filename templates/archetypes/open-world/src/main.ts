// Saltwind Isle, the open-world archetype (3D profile): two islands on a
// terrain in an endless sea, three days and two nights. Walk, run, wade
// and jump the explorer (og.CharacterController) from the camp on the
// south beach, discover the island's five landmarks and claim each by
// reaching it (its flag rises, its brazier catches). Two stand on the north
// isle across a deep strait: fell trees for wood and break boulders for
// stone (hold Gather), carry them in the bag and build the bridge at the
// strait's landing. Every landmark claimed before the third nightfall wins
// the run; the third nightfall ends it. Deep water is a soft fail: the
// tide carries the explorer back to the camp and half the bag with it.
//
// Every explorer is a seat (src/seats.ts): its og.PlayerInput is all the
// systems read, and its view is src/view.ts's (third person, first person
// or isometric, as src/creation.ts says), so the isle plays alone, two on
// one device (the screen split in halves) or two over a room in lockstep
// (each peer's camera with its own explorer). The two share the bag, the
// landmarks, the bridge and the clock.
//
// `start` loads the strings, the style, the sounds, the quests and the
// screen documents, builds the isle (src/world.ts) and spawns the explorers
// (src/hero.ts) and the voices (src/sound.ts). Gathering, the bridge and
// the landmarks are src/work.ts, the screens and the clock src/ui.ts, the
// state src/components.ts.

import { og, math } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { BRIDGE_COST, Game, Hero, Kind, Phase, SITE_COUNT, Tuning } from "./components.ts";
import { prefs, record } from "./device.ts";
import { FEET, heroOf, HeroMove, Eye, hypot, NODES, spawnHeroes, startOf } from "./hero.ts";
import { CAMP, SITES } from "./layout.ts";
import { SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { begin, Device, LANGUAGES, loadHudArt, runSeconds, toTitle, Ui } from "./ui.ts";
import { SeatLook, ViewRig } from "./view.ts";
import { claim, spawnChips, Work } from "./work.ts";
import { BRIDGE_SITE, buildIsle, ground, must } from "./world.ts";

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  const style = must(og.load.style(world, "styles/saltwind/style.ogt"));
  const sound = (path: string) => must(og.load.sound(world, path));
  must(og.ui.configure(world, { style, sounds: { press: sound("audio/ui-press.ogt"), open: sound("audio/ui-open.ogt"), close: sound("audio/ui-close.ogt"), error: sound("audio/ui-error.ogt"), success: sound("audio/ui-success.ogt") } }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt"]) must(og.load.document(world, doc));
  // The quests (the first og.story load registers og.Story, which a system cannot).
  for (const q of ["story/lights.ogt", "story/bridge.ogt"]) must(og.story.load(world, q));
  loadHudArt(world);
  buildIsle(world);
  spawnHeroes(world);
  spawnChips(world);
  spawnVoices(world);
  // Behind the title the isle is reset and the camp's fire burns.
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
  begin(world, gameOf(world));
}

/** Puts a seat's explorer on the ground at x, z, facing (and its view looking) the given heading. */
function put(world: World, seat: number, x: number, z: number, yaw?: number): void {
  const g = gameOf(world);
  const hero = heroOf(g, seat);
  const h = ground(world, g, x, z).height;
  must(world.set(hero, og.Transform, { position: { x, y: h + FEET + 0.05, z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 } }));
  if (yaw !== undefined) {
    must(world.set(hero, Hero, { facing: yaw + Math.PI }));
    must(world.set(hero, SeatLook, { yaw }));
  }
}

/** The tree or boulder of a kind nearest the camp. */
function nearestToCamp(kind: number): number {
  let best = -1;
  let bestD = 1e9;
  NODES.forEach((n, i) => {
    const d = hypot(n.x - CAMP.x, n.z - CAMP.z);
    if (n.kind === kind && d < bestD) {
      bestD = d;
      best = i;
    }
  });
  return best;
}

/** A seat's warp: `<site>` (just short of a landmark, facing it), `tree`, `rock`, `bridge`, `camp`, or `<x>:<z>`. */
function warpSeat(world: World, seat: number, args: readonly string[]): void {
  let to: { x: number; z: number };
  if (args.length === 2) {
    put(world, seat, num(args[0], "x"), num(args[1], "z"));
    return;
  }
  const what = args[0];
  if (what === "tree" || what === "rock") to = NODES[nearestToCamp(what === "tree" ? Kind.Tree : Kind.Rock)]!;
  else if (what === "bridge") to = BRIDGE_SITE;
  else if (what === "camp") {
    const s = startOf(seat);
    put(world, seat, s.x, s.z, 0);
    return;
  } else {
    const i = num(what, "site");
    if (!Number.isInteger(i) || i < 0 || i >= SITE_COUNT) throw new Error(`sites are 0 to ${SITE_COUNT - 1}, or write tree, rock, bridge, camp or <x>:<z>`);
    to = SITES[i]!;
  }
  // Short of it on the camp's side (a landmark: outside its claim radius; work: within reach), facing it.
  const away = what === "tree" || what === "rock" ? 1.6 + seat * 0.9 : what === "bridge" ? 1.5 + seat * 1.2 : 13;
  const dx = CAMP.x - to.x;
  const dz = CAMP.z - to.z;
  const d = hypot(dx, dz) || 1;
  const x = to.x + (dx / d) * away + (seat === 0 ? 0 : (dz / d) * 0.8);
  const z = to.z + (dz / d) * away - (seat === 0 ? 0 : (dx / d) * 0.8);
  // The heading that looks from (x, z) at the target: look direction (-sin yaw, -cos yaw).
  put(world, seat, x, z, math.atan2(-(to.x - x), -(to.z - z)));
}

/** `warp:<site|tree|rock|bridge|camp>` or `warp:<x>:<z>`: puts seat 0's explorer there. */
function warp(world: World, args: readonly string[]): void {
  warpSeat(world, 0, args);
}

/** `warp_seat:<seat>:...`: the same for any seat's explorer. */
function warpOf(world: World, args: readonly string[]): void {
  const seat = num(args[0], "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
  warpSeat(world, seat, args.slice(1));
}

/** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back): the room tests. */
function bot(_world: World, args: readonly string[]): void {
  setBot(num(args[0], "n"));
}

/** `give:<wood>:<stone>`: puts wood and stone in the bag. */
function give(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  g.wood += num(args[0], "wood");
  g.stone += num(args[1] ?? "0", "stone");
}

/** `claim:<site>`: claims a landmark now, as reaching it does (the fifth wins). */
function claimSite(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("claim needs a run (write run first)");
  const i = num(args[0], "site");
  if (!Number.isInteger(i) || i < 0 || i >= SITE_COUNT) throw new Error(`sites are 0 to ${SITE_COUNT - 1}`);
  if ((g.claimMask & (1 << i)) === 0) claim(world, g, i);
}

/** `clock:<seconds>`: jumps the run's clock to this many seconds played (the third nightfall at the run's length). */
function clockTo(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  if (g.phase !== Phase.Play) throw new Error("clock needs a run (write run first)");
  g.time = Math.max(0, num(args[0], "seconds"));
}

/** `viewport:<desktop|phone>`: lays the UI out for a phone in portrait. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : null);
}

/** `language:<en|zh-CN>`. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  prefs.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

/** `stale_record`: another device writes the record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { runs: 9, wins: 3, best: 300 } }));
}

/** Seat 1's explorer, for the facts. */
function second(world: World) {
  const g = gameOf(world);
  const hero = heroOf(g, 1);
  const me = world.get(hero, Hero);
  const at = world.get(hero, og.Transform);
  if (!me.ok || !at.ok) return {};
  const r = (n: number) => Math.round(n * 100) / 100;
  return { target1: me.value.target, gathering1: me.value.gathering, speed1: r(me.value.speed), x1: r(at.value.position.x), z1: r(at.value.position.z) };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Seats, Device, ViewRig, Eye, ...og.systems.cameras, Ui, HeroMove, Work, Voices],
  start,
  commands: { run, warp, warp_seat: warpOf, bot, give, claim: claimSite, clock: clockTo, viewport, language, stale_record: staleRecord },
  commandHelp: {
    run: { usage: "", doc: "Start a run now, as Set out does." },
    warp: { usage: "<site> | tree | rock | bridge | camp | <x>:<z>", doc: "Put seat 0's explorer short of a landmark (0 to 4, outside its claim radius), by the tree or boulder nearest the camp, at the bridge's landing, at the camp, or on the ground at x, z, facing it." },
    warp_seat: { usage: "<seat>:<where>", doc: "The same for any seat's explorer." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots play the same run." },
    give: { usage: "<wood>:<stone>", doc: `Put wood and stone in the bag (the bridge takes ${BRIDGE_COST.wood} and ${BRIDGE_COST.stone}).` },
    claim: { usage: "<site>", doc: "Claim a landmark (0 to 4) now, as reaching it does (the fifth wins)." },
    clock: { usage: "<seconds>", doc: "Jump the run's clock to this many seconds played (the third nightfall ends it)." },
    viewport: { usage: "<desktop|phone>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×)." },
    language: { usage: "<en|zh-CN>", doc: "Switch the language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the record, so this one's next write is refused as stale and merged." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok) return null;
    const g = game.value;
    const t = must(world.resource(Tuning));
    return {
      phase: ["title", "play", "over"][g.phase],
      ending: ["none", "lit", "night"][g.ending],
      day: g.day,
      hour: Math.round(g.hour * 100) / 100,
      night: g.night,
      time: Math.round(g.time * 10) / 10,
      runSeconds: runSeconds(t),
      claimed: g.claimed,
      claimMask: g.claimMask,
      seenMask: g.seenMask,
      wood: g.wood,
      stone: g.stone,
      bridge: g.bridge,
      washes: g.washes,
      near: g.near,
      gathering: g.gathering,
      wading: g.wading,
      grounded: g.grounded,
      speed: Math.round(g.speed * 100) / 100,
      jumps: g.jumps,
      runs: record.runs,
      wins: record.wins,
      best: Math.round(record.best * 10) / 10,
      language: LANGUAGES[prefs.language]!.code,
      seats: SEATS,
      ...(SEATS > 1 ? second(world) : {}),
    };
  },
});
