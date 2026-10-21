// The race: the countdown, the car's progress round the circuit, the
// splits and the laps, the resets, the ghost and the finish.
//
// Progress is the nearest centre-line sample (src/track.ts), searched near
// the last one each tick. A split counts when the car passes its line
// going forward; a lap when it crosses the start line having passed both
// splits in order. Times are whole ticks (1/60 s) from GO.
//
// The ghost is the best lap's car: its poses, recorded every second tick of
// a lap (Tape.cur), become the best (Tape.best) when the lap beats the
// record, and are drawn back from the start of every lap (src/ghost.ts).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { COUNTDOWN, Phase, Prefs, Race, Sound, TapeChunk, TAPE_TICKS } from "./components.ts";
import { POSE_BYTES } from "./tape.ts";
import { readTape, writeTape } from "./tapes.ts";
import { hideGhost } from "./ghost.ts";
import { submitLap, writeRecord } from "./save.ts";
import { across, fromStart, JUMP, LAPS, nearest, SAMPLES, SPLITS, START, WIDTH } from "./track.ts";
import { entity, GRID_BACK, must, placeCar } from "./world.ts";

type RaceV = Value<typeof Race>;

const N = SAMPLES.length;

/** Where a reset puts the car: past the jump when it fell in the gap, else where it was. */
function resetSample(k: number): number {
  // The ramp's foot to the landing ramp's top, in samples from the line.
  const foot = SAMPLES.findIndex((s) => s.x > 180 && s.z <= JUMP.rampZ);
  const land = SAMPLES.findIndex((s, i) => i > foot && !s.gap && SAMPLES[(i + N - 1) % N]!.gap);
  const kf = fromStart(foot);
  const kl = fromStart(land);
  if (k >= kf && k <= kl + 2) return kl + 3;
  return k;
}

/** Starts a race from the grid: the countdown runs, the car held on the line. */
export function startRace(world: World, r: RaceV): void {
  r.phase = Phase.Countdown;
  r.countdown = COUNTDOWN;
  r.clock = 0;
  r.lap = 1;
  r.lapStart = 0;
  r.split1 = 0;
  r.split2 = 0;
  r.lap1 = 0;
  r.lap2 = 0;
  r.lap3 = 0;
  r.runBest = 0;
  r.bestS1 = 0;
  r.bestS2 = 0;
  r.bestS3 = 0;
  r.delta = 0;
  r.deltaAt = 0;
  r.deltaShow = 0;
  r.valid = true;
  r.newRecord = false;
  r.total = 0;
  r.finishedFor = 0;
  r.at = -1;
  r.wrongWay = 0;
  r.offTicks = 0;
  r.paused = false;
  r.ghostActive = false;
  r.ghostTick = 0;
  r.ghostAt = -1;
  r.rank = -1;
  r.sfx |= Sound.Beep;
  placeCar(world, -GRID_BACK);
  hideGhost(world);
  r.curLen = 0;
  r.runLen = 0;
}

/** A new lap from this tick: the ghost from the line, the poses afresh. */
function startLap(world: World, r: RaceV, prefs: Value<typeof Prefs>): void {
  r.lapStart = r.clock;
  r.split1 = 0;
  r.split2 = 0;
  r.valid = true;
  r.curLen = 0;
  r.ghostTick = 0;
  r.ghostAt = -1;
  r.ghostActive = prefs.ghost && r.record > 0 && r.bestLen > 0;
  if (!r.ghostActive) hideGhost(world);
}

function showDelta(r: RaceV, which: number, mine: number, theirs: number): void {
  r.deltaAt = which;
  r.delta = theirs > 0 ? mine - theirs : 0;
  r.deltaShow = theirs > 0 ? 180 : 0;
}

/** The lap just crossed the line. */
function endLap(world: World, r: RaceV, prefs: Value<typeof Prefs>): void {
  const ticks = r.clock - r.lapStart;
  const s1 = r.split1;
  const s2 = r.split2 - r.split1;
  const s3 = ticks - r.split2;
  r.laps += 1;
  if (r.lap === 1) r.lap1 = ticks;
  else if (r.lap === 2) r.lap2 = ticks;
  else r.lap3 = ticks;
  showDelta(r, 3, ticks, r.record);
  if (r.valid && (r.runBest === 0 || ticks < r.runBest)) {
    r.runBest = ticks;
    r.bestS1 = s1;
    r.bestS2 = s2;
    r.bestS3 = s3;
  }
  if (r.valid && ticks < TAPE_TICKS && (r.record === 0 || ticks < r.record)) {
    r.record = ticks;
    r.recordS1 = s1;
    r.recordS2 = s2;
    r.recordS3 = s3;
    r.newRecord = true;
    const poses = readTape(world, "cur", r.curLen * POSE_BYTES);
    writeTape(world, "best", poses);
    r.bestLen = r.curLen;
    r.sfx |= Sound.Best;
    og.ui.toast(world, og.i18n.t(world, "toast.record", { time: lapText(ticks) }), { tone: "success" });
    writeRecord(world, r, poses, r.curLen);
  } else {
    r.sfx |= r.valid && r.record > 0 && ticks > r.record ? Sound.Slower : Sound.Lap;
    if (!r.valid) og.ui.toast(world, og.i18n.t(world, "toast.invalid"), { tone: "warn" });
  }
  r.lap += 1;
  if (r.lap > LAPS) {
    r.phase = Phase.Finished;
    r.total = r.clock;
    r.ghostActive = false;
    if (r.runBest > 0) submitLap(world, r, r.runBest);
    return;
  }
  startLap(world, r, prefs);
}

/** Puts the car back on the road (R, upside down, or in the jump's gap): the lap no longer counts. */
export function resetCar(world: World, r: RaceV): void {
  const k = r.at < 0 ? -GRID_BACK : fromStart(r.at);
  const to = resetSample(k);
  placeCar(world, to);
  r.at = (((START + to) % N) + N) % N;
  r.offTicks = 0;
  r.resets += 1;
  if (r.phase === Phase.Racing) r.valid = false;
  must(world.set(entity(world, "car"), og.Vehicle, { brake: 0 }));
}

/** Whole ticks as m:ss.mmm. */
export function lapText(ticks: number): string {
  const ms = Math.round((ticks * 1000) / 60);
  const m = Math.floor(ms / 60000);
  const s = Math.floor((ms % 60000) / 1000);
  const f = ms % 1000;
  return `${m}:${s < 10 ? "0" : ""}${s}.${f < 10 ? "00" : f < 100 ? "0" : ""}${f}`;
}

/** A delta in ticks as ±s.mmm. */
export function deltaText(ticks: number): string {
  const ms = Math.round((Math.abs(ticks) * 1000) / 60);
  return `${ticks < 0 ? "-" : "+"}${Math.floor(ms / 1000)}.${String(ms % 1000).padStart(3, "0")}`;
}

/** The race's tick: countdown, progress, splits, laps, resets, the poses, the ghost. */
export const RaceSystem = defineSystem(
  "Race",
  { reads: [Prefs, og.Input, og.Gamepad], writes: [Race, TapeChunk, og.Transform, og.Velocity, og.Vehicle] },
  (_q, time, world) => {
    const r = must(world.resource(Race));
    if (r.held || time.scaledDt === 0) return;
    const prefs = must(world.resource(Prefs));
    if (r.deltaShow > 0) r.deltaShow -= 1;
    if (r.phase === Phase.Countdown) {
      r.countdown -= 1;
      if (r.countdown > 0 && r.countdown % 60 === 0) r.sfx |= Sound.Beep;
      if (r.countdown === 0) {
        r.phase = Phase.Racing;
        r.clock = 0;
        r.sfx |= Sound.Go;
        startLap(world, r, prefs);
      }
      return;
    }
    if (r.phase !== Phase.Racing) return;

    const car = entity(world, "car");
    const tr = must(world.get(car, og.Transform));
    const p = tr.position;
    const prev = r.at;
    const i = nearest(p.x, p.z, prev, prev < 0 ? 0 : 14);
    const pk = prev < 0 ? fromStart(i) : fromStart(prev);
    const k = fromStart(i);
    let d = k - pk;
    if (d > N / 2) d -= N;
    if (d < -N / 2) d += N;
    r.at = i;

    // Splits and the line, forward only.
    if (d > 0 && d < 40) {
      if (r.split1 === 0 && pk < SPLITS[0]! && k >= SPLITS[0]!) {
        r.split1 = Math.max(1, r.clock - r.lapStart);
        r.splits += 1;
        r.sfx |= Sound.Sector;
        showDelta(r, 1, r.split1, r.recordS1);
      } else if (r.split1 !== 0 && r.split2 === 0 && pk < SPLITS[1]! && k >= SPLITS[1]!) {
        r.split2 = Math.max(r.split1 + 1, r.clock - r.lapStart);
        r.splits += 1;
        r.sfx |= Sound.Sector;
        showDelta(r, 2, r.split2, r.recordS1 + r.recordS2);
      }
    }
    if (d > 0 && pk > N - 40 && k < 40 && r.split2 !== 0) endLap(world, r, prefs);
    if (d < -2) r.wrongWay += 1;
    else if (d > 0) r.wrongWay = 0;

    // Off the road: below it (the jump's gap) or upside down, for a second; or R, or the pad's Y.
    const s = SAMPLES[i]!;
    const q4 = tr.quaternion;
    const upY = 1 - 2 * (q4.x * q4.x + q4.z * q4.z);
    const off = p.y < s.y - 1.6 || upY < 0.25 || Math.abs(across(i, p.x, p.z)) > WIDTH / 2 + 4;
    r.offTicks = off ? r.offTicks + 1 : 0;
    const inp = must(world.resource(og.Input));
    const pad = must(world.resource(og.Gamepad));
    if (r.offTicks > 60 || inp.pressed("KeyR") || pad.pressed("y")) resetCar(world, r);

    if (r.phase === Phase.Racing) r.clock += 1;
  },
);
