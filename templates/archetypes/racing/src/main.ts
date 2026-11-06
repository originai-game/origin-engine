// Redline Ring, the racing archetype (3D profile): six cars, three laps of
// an 800 m circuit, one line. Hold Throttle, steer with the stick, brake
// into the bends; Brake held with Throttle through a bend drifts, and a
// drift fills the boost meter that Boost spends. Pass the two checkpoints
// in order and cross the line three times: first across the line wins,
// every other place is a result with its points, and a seat still running
// 30 s after the winner is out (DNF). Then the results, and again.
//
// Every seat's car is driven by its og.PlayerInput alone (src/seats.ts),
// so the race plays alone, two on one device (the screen split, a car
// each) or two over a room in lockstep (each peer's camera chasing its own
// car), as src/creation.ts says; the rivals drive the racing line. The
// view (src/view.ts) chases the car (third-person-3d), sits in it
// (first-person-3d) or watches from a fixed angle above (isometric-2.5d).
//
// `start` loads the strings, the style and the screen documents
// (assets/ui/), builds the circuit (src/track.ts, src/mesh.ts,
// src/world.ts), spawns the field and its voices (src/sound.ts) and puts
// each seat's view on its car. The cars are in src/car.ts, the race's
// standing in src/race.ts, the screens and the HUD in src/ui.ts, the
// effects in src/fx.ts and the state in src/components.ts.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Car, COUNTDOWN, FIELD, Game, Phase, Tuning } from "./components.ts";
import { CARS, Drive, Drivers, Rivals } from "./car.ts";
import { prefs, record, screens } from "./device.ts";
import { CarView, Scene, setupFeel } from "./fx.ts";
import { Race } from "./race.ts";
import { SEATS, Seats, setBot, setupSeats } from "./seats.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { LAPS, LENGTH, nearest, pointAt } from "./track.ts";
import { Device, LANGUAGES, startRace, toTitle, Ui } from "./ui.ts";
import { setupView, ViewRig } from "./view.ts";
import { buildCircuit, headingOf, must, spawnCars, yawQuat } from "./world.ts";

function start(world: World): void {
  setupSeats(world);
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  must(og.ui.configure(world, { style: must(og.load.style(world, "styles/redline/style.ogt")) }));
  for (const doc of ["ui/main-menu.ogt", "ui/pause.ogt", "ui/racing/race-results.ogt"]) must(og.load.document(world, doc));
  buildCircuit(world);
  CARS.push(...spawnCars(world, SEATS));
  setupView(world, CARS.slice(0, SEATS), { eye: 1.3, behind: 6.6, above: 1.3, isoHeight: 34 });
  setupFeel(world);
  spawnVoices(world);
  // Behind the title the field laps on the autopilot.
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

function seatOf(s: string | undefined): number {
  const seat = num(s, "seat");
  if (!Number.isInteger(seat) || seat < 0 || seat >= SEATS) throw new Error(`seats are 0 to ${SEATS - 1}`);
  return seat;
}

/** `race` or `race:go`: a race from the grid now, as Race does; `go` skips the countdown. */
function race(world: World, args: readonly string[]): void {
  const g = gameOf(world);
  startRace(world, g);
  if (args[0] === "go") g.clock = COUNTDOWN;
}

/** `warp:<along>[:<across>[:<speed>]]`: seat 0's car `along` metres past the line, `across` right of the centre, at a speed (m/s). */
function warp(world: World, args: readonly string[]): void {
  warpCar(world, 0, args);
}

/** `warp_seat:<seat>:<along>[:<across>[:<speed>]]`: the same for any seat's car. */
function warpSeat(world: World, args: readonly string[]): void {
  warpCar(world, seatOf(args[0]), args.slice(1));
}

/** `warp_car:<slot>:<along>[:<across>[:<speed>]]`: the same for any car of the field (0 to 5, the seats' first). */
function warpAny(world: World, args: readonly string[]): void {
  const slot = num(args[0], "slot");
  if (!Number.isInteger(slot) || slot < 0 || slot >= FIELD) throw new Error(`cars are 0 to ${FIELD - 1}`);
  warpCar(world, slot, args.slice(1));
}

function warpCar(world: World, slot: number, args: readonly string[]): void {
  const along = num(args[0], "along");
  const across = args[1] === undefined ? 0 : num(args[1], "across");
  const speed = args[2] === undefined ? 0 : num(args[2], "speed");
  const at = pointAt(along, across);
  const heading = headingOf(at.tx, at.tz);
  const car = CARS[slot]!;
  must(world.set(car, og.Transform, { position: { x: at.x, y: 0, z: at.z }, quaternion: yawQuat(heading) }));
  const a = ((along % LENGTH) + LENGTH) % LENGTH;
  must(world.set(car, Car, { heading, camYaw: heading, vx: at.tx * speed, vz: at.tz * speed, speed, slip: 0, sample: nearest(at.x, at.z), along: a, across }));
}

/** `lap:<slot>:<n>`: a car of the field (the seats' first) has done n laps: it crosses the line next for lap n + 1, or the finish after the last. */
function lap(world: World, args: readonly string[]): void {
  const slot = num(args[0], "slot");
  if (!Number.isInteger(slot) || slot < 0 || slot >= FIELD) throw new Error(`cars are 0 to ${FIELD - 1}`);
  const n = num(args[1], "n");
  if (!Number.isInteger(n) || n < 0 || n >= LAPS) throw new Error(`laps done are 0 to ${LAPS - 1}`);
  const g = gameOf(world);
  must(world.set(CARS[slot]!, Car, { laps: n, checkpoint: 2, lapStart: Math.max(COUNTDOWN, g.clock - 1500) }));
}

/** `bot:<n>`: a scripted player drives this device's seats from the tick (0 hands the input back): the room tests. */
function bot(_world: World, args: readonly string[]): void {
  setBot(num(args[0], "n"));
}

/** `viewport:<desktop|phone|landscape>`: lay the UI out for a phone in portrait, or held sideways. */
function viewport(world: World, args: readonly string[]): void {
  const phone = { width: 390, height: 844, dpr: 3, safeArea: { left: 0, top: 47, right: 0, bottom: 34 }, coarsePointer: true, breakpoint: "phonePortrait" } as const;
  const landscape = { width: 844, height: 390, dpr: 3, safeArea: { left: 47, top: 0, right: 47, bottom: 21 }, coarsePointer: true, breakpoint: "phoneLandscape" } as const;
  og.ui.previewViewport(world, args[0] === "phone" ? phone : args[0] === "landscape" ? landscape : null);
}

/** `language:<en|zh-CN>`: this device's language. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|zh-CN>");
  prefs.language = at;
  og.i18n.setLocale(world, LANGUAGES[at]!.code);
}

/** `stale_record`: another device writes the record (the next write here is refused as stale and merged). */
function staleRecord(world: World): void {
  must(og.platform.storeSet(world, { key: "record", value: { races: 9, wins: 3, bestLap: 1500 } }));
}

/** `settings`: open the settings, as the title's Settings does. */
function openSettings(): void {
  screens.settingsOpen = true;
}

const round = (n: number) => Math.round(n * 100) / 100;

/** A seat's car, for the facts. */
function carFacts(world: World, seat: number) {
  const c = must(world.get(CARS[seat]!, Car));
  const at = must(world.get(CARS[seat]!, og.Transform)).position;
  return { place: c.place, laps: c.laps, checkpoint: c.checkpoint, along: round(c.along), across: round(c.across), speed: round(c.speed), kmh: Math.round(c.speed * 3.6), boost: round(c.boost), drifting: c.drifting, boosting: c.boosting, bestLap: c.bestLap, lastLap: c.lastLap, finish: c.finish, out: c.out, hits: c.hits, x: round(at.x), z: round(at.z) };
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Game, Tuning],
  systems: [Seats, Device, Ui, Drivers, Rivals, Drive, Race, CarView, ViewRig, ...og.systems.cameras, Scene, Voices],
  start,
  commands: { race, warp, warp_seat: warpSeat, warp_car: warpAny, lap, bot, viewport, language, stale_record: staleRecord, settings: openSettings },
  commandHelp: {
    race: { usage: "[go]", doc: "A race from the grid now, as Race does; `go` skips the countdown." },
    warp: { usage: "<along>[:<across>[:<speed>]]", doc: "Put seat 0's car `along` metres past the line, `across` right of the centre line, at a speed in m/s, facing down the road." },
    warp_seat: { usage: "<seat>:<along>[:<across>[:<speed>]]", doc: "The same for a seat's car." },
    warp_car: { usage: "<slot>:<along>[:<across>[:<speed>]]", doc: "The same for any car of the field (0 to 5, the seats' first)." },
    lap: { usage: "<slot>:<n>", doc: "A car of the field (0 to 5, the seats' first) has done n laps: crossing the line next begins lap n + 1 (or finishes after the last)." },
    bot: { usage: "<n>", doc: "A scripted player drives this device's seats from the tick (0 hands the input back): two peers with bots race the same race." },
    viewport: { usage: "<desktop|phone|landscape>", doc: "Lay the UI out for a phone in portrait (390×844 at 3×) or held sideways (844×390)." },
    language: { usage: "<en|zh-CN>", doc: "Switch this device's language as the settings do." },
    stale_record: { usage: "", doc: "Another device writes the record, so this one's next write is refused as stale and merged." },
    settings: { usage: "", doc: "Open the settings dialog, as Settings does." },
  },
  facts: (world) => {
    const game = world.resource(Game);
    if (!game.ok || CARS.length === 0) return null;
    const g = game.value;
    return {
      phase: ["title", "race", "results"][g.phase],
      clock: g.clock,
      countdown: g.phase === Phase.Race && g.clock < COUNTDOWN,
      paused: g.paused,
      closed: g.closed,
      bumps: g.bumps,
      seats: SEATS,
      car0: carFacts(world, 0),
      ...(SEATS > 1 ? { car1: carFacts(world, 1) } : {}),
      races: record.races,
      wins: record.wins,
      bestLap: record.bestLap,
      recordConflicts: record.conflicts,
      language: LANGUAGES[prefs.language]!.code,
      settingsOpen: screens.settingsOpen,
    };
  },
});
