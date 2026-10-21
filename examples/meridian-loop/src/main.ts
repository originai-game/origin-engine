// Meridian Loop (3D profile): a night time trial round a neon circuit.
// Three laps from a standing start on a 956 m loop with two banked bends, a
// banked hairpin, an S and a jump over a gap on the back straight; a lap
// timer with two splits, the ghost of the best lap, the record kept by the
// platform and the best laps on a leaderboard.
//
// The look (GameDesc.look, assets/looks/meridian.ogt, extending the
// neon-vector preset) is applied before `start`; `start` loads the strings,
// builds the circuit (src/world.ts), spawns the car, the ghost, the effects
// and the voices. Systems in tick order: Ui (screens, the hold, the
// platform), Race (countdown, progress, splits, laps, resets), Driver (the
// intents), Tapes (the inputs and poses taped, the ghost drawn), Chassis
// (the car's inputs and assists), Fx (wheels, smoke, sparks, dust), Marks
// (the skid marks), View (the camera), Voices (the engine, the tyres, the
// cues).

import { og, SHAPE_BOX } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Chassis, Driver } from "./car.ts";
import { Phase, Prefs, Race, Racer } from "./components.ts";
import { Fx, Marks } from "./fx.ts";
import { Tapes } from "./ghost.ts";
import { lapText, RaceSystem, resetCar, startRace } from "./race.ts";
import { spawnVoices, Voices } from "./sound.ts";
import { fromStart, LAPS, SAMPLES } from "./track.ts";
import { LANGUAGES, setLanguage, takeSettings, Ui } from "./ui.ts";
import { View } from "./view.ts";
import { buildCircuit, entity, LAYER, must, placeCar, spawnCars, spawnEffects } from "./world.ts";

function start(world: World): void {
  const r = must(world.resource(Race));
  for (const l of LANGUAGES) must(og.i18n.load(world, `strings/${l.code}.ogt`));
  r.minimap = must(og.load.texture(world, "textures/minimap.png")) as unknown as number;
  buildCircuit(world);
  spawnCars(world);
  spawnEffects(world);
  spawnVoices(world);
  placeCar(world, -5);
  r.phase = Phase.Title;
}

function race(world: World) {
  return must(world.resource(Race));
}

function num(s: string | undefined, what: string): number {
  const n = Number(s);
  if (s === undefined || s === "" || !Number.isFinite(n)) throw new Error(`${what} is a number`);
  return n;
}

/** `race[:go]`: a race from the grid now, as Time trial does; `go` skips the countdown. */
function raceCmd(world: World, args: readonly string[]): void {
  const r = race(world);
  startRace(world, r);
  if (args[0] === "go") r.countdown = 1;
}

/** `drive:<auto|off>`: the autopilot drives (or stops driving) the car. */
function drive(world: World, args: readonly string[]): void {
  if (args[0] !== "auto" && args[0] !== "off") throw new Error("write drive:auto or drive:off");
  race(world).autopilot = args[0] === "auto";
}

/** `warp:<k>[:<speed m/s>]`: the car k samples past the line (2 m a sample), facing along the road, at that speed. */
function warp(world: World, args: readonly string[]): void {
  const r = race(world);
  const k = num(args[0], "k");
  const speed = args[1] === undefined ? 0 : num(args[1], "speed");
  placeCar(world, k);
  const car = entity(world, "car");
  const q = must(world.get(car, og.Transform)).quaternion;
  const fx = 2 * (q.x * q.z + q.w * q.y);
  const fz = 1 - 2 * (q.x * q.x + q.y * q.y);
  must(world.set(car, og.Velocity, { linear: { x: fx * speed, y: 0, z: fz * speed }, angular: { x: 0, y: 0, z: 0 } }));
  must(world.set(car, og.Vehicle, { brake: 0 }));
  const n = SAMPLES.length;
  r.at = -1;
  void n;
}

/** `language:<en|de>`: switches the language as the settings do. */
function language(world: World, args: readonly string[]): void {
  const at = LANGUAGES.findIndex((l) => l.code === args[0]);
  if (at < 0) throw new Error("write language:<en|de>");
  setLanguage(world, must(world.resource(Prefs)), at);
}

/** `open:<settings|board|pause>`: opens a screen, as its button does. */
function open(world: World, args: readonly string[]): void {
  const r = race(world);
  if (args[0] === "settings") r.settingsOpen = true;
  else if (args[0] === "board") {
    r.boardOpen = true;
    r.boardState = 1;
  } else if (args[0] === "pause") r.paused = true;
  else throw new Error("write open:<settings|board|pause>");
}

/** `ghost:<on|off>`: the ghost switch, as pause's item. */
function ghost(world: World, args: readonly string[]): void {
  takeSettings(world, must(world.resource(Prefs)), { ghost: args[0] === "on" });
}

/**
 * `twin`: FINDINGS #3's repro. A second og.Vehicle on the car's spot, on a
 * collision layer of its own that meets only the circuit (and the car's mask
 * leaves it out): no collider of either car interacts with the other's, yet
 * each car's wheel rays land on the other's chassis and the two climb.
 */
function twin(world: World): void {
  const car = entity(world, "car");
  const t = must(world.get(car, og.Transform));
  const e = must(world.spawn({ name: "twin", position: t.position, quaternion: t.quaternion }));
  must(world.insert(e, og.RigidBody, { mass: 1000 }));
  must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 1.7, y: 0.25, z: 4 }, layers: 4, mask: LAYER.track }));
  must(world.insert(e, og.Vehicle, { ...must(world.get(car, og.Vehicle)), throttle: 0, steer: 0, brake: 1 }));
}

/** `reset`: the car back on the road, as R. */
function reset(world: World): void {
  resetCar(world, race(world));
}

function facts(world: World): Record<string, unknown> {
  const r = race(world);
  const car = entity(world, "car");
  const t = must(world.get(car, og.Transform)).position;
  const racer = must(world.get(car, Racer));
  const g = must(world.get(entity(world, "ghost"), og.Transform)).position;
  return {
    phase: r.phase,
    paused: r.paused,
    countdown: r.countdown,
    clock: r.clock,
    lap: r.lap,
    laps: r.laps,
    lapTimes: [r.lap1, r.lap2, r.lap3],
    lapText: [r.lap1, r.lap2, r.lap3].map((x) => (x === 0 ? "" : lapText(x))),
    runBest: r.runBest,
    record: r.record,
    recordSectors: [r.recordS1, r.recordS2, r.recordS3],
    split1: r.split1,
    split2: r.split2,
    splits: r.splits,
    delta: r.delta,
    deltaAt: r.deltaAt,
    valid: r.valid,
    newRecord: r.newRecord,
    total: r.total,
    k: r.at < 0 ? -1 : fromStart(r.at),
    resets: r.resets,
    bumps: r.bumps,
    jumps: r.jumps,
    landings: r.landings,
    longestAir: r.longestAir,
    skidTicks: r.skidTicks,
    kmh: Math.round(r.kmh * 10) / 10,
    gear: r.gear,
    slip: Math.round(r.slip * 100) / 100,
    skidding: r.skidding,
    airborne: r.airborne,
    steer: racer.steer,
    contacts: racer.contacts,
    x: t.x,
    y: t.y,
    z: t.z,
    ghostActive: r.ghostActive,
    ghostTick: r.ghostTick,
    ghost: [g.x, g.y, g.z],
    poses: r.curLen,
    bestPoses: r.bestLen,
    runInputs: r.runLen,
    rank: r.rank,
    boardState: r.boardState,
    board: r.board,
    saves: r.saves,
    loaded: r.loaded,
    conflicts: r.conflicts,
    language: LANGUAGES[must(world.resource(Prefs)).language]!.code,
    ghostOn: must(world.resource(Prefs)).ghost,
    a11yNodes: r.a11yNodes,
    a11yUnnamed: r.a11yUnnamed,
    laps_total: LAPS,
  };
}

export default og.game({
  look: "looks/meridian.ogt",
  start,
  systems: [Ui, RaceSystem, Driver, Tapes, Chassis, Fx, Marks, View, Voices],
  commands: { race: raceCmd, drive, warp, language, open, ghost, reset, twin },
  commandHelp: {
    race: { usage: "[go]", doc: "A race from the grid now; go skips the countdown (and with it the replay's saved start)." },
    drive: { usage: "<auto|off>", doc: "The autopilot drives the car round the centre line, or stops." },
    warp: { usage: "<k>[:<speed>]", doc: "The car k samples (2 m each) past the start line, along the road, at speed m/s." },
    language: { usage: "<en|de>", doc: "Switches the language as the settings do." },
    open: { usage: "<settings|board|pause>", doc: "Opens a screen as its button does." },
    ghost: { usage: "<on|off>", doc: "Shows or hides the best lap's ghost, as pause's item does." },
    reset: { usage: "", doc: "Puts the car back on the road, as R does (the lap no longer counts)." },
    twin: { usage: "", doc: "FINDINGS #3: a second car on the car's spot whose colliders meet only the circuit; the two climb each other." },
  },
  facts,
});
