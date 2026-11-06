// The cars: who drives each (a seat's og.PlayerInput, or the racing line)
// and the arcade handling that moves them, the same fixed-tick arithmetic
// for every car.
//
//   Drivers  a seat's car from its og.PlayerInput: the stick steers,
//            Throttle, Brake and Boost; on the title and once it has
//            finished, the autopilot drives it (the title's attract laps).
//   Rivals   the other cars from the racing line (src/track.ts): pure
//            pursuit of a point ahead on the line, the line's speed at
//            the car's pace, a lane to pass a slower car, boost on the
//            straights.
//   Drive    every car: grip or drift, the boost meter, the barriers and
//            the other cars, the laps and checkpoints.
//
// The handling: the car's velocity is kept in the world; steering turns
// the heading, and the tyres catch the sideways part of the velocity at
// Tuning.grip a second (most of it turned into forward speed). Brake held
// with Throttle and a turn at speed starts a drift: grip falls to
// Tuning.driftGrip, the turn sharpens by Tuning.driftTurn and the boost
// meter fills. Boost spends the meter for more push and a higher top
// speed. A barrier pushes the car back onto the road, takes the speed
// into it and turns the car along the road; two cars push apart as
// capsules and trade the speed between them.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Car, COUNTDOWN, Game, Phase, Sound, Tuning } from "./components.ts";
import { held } from "./seats.ts";
import { COUNT, LAPS, LENGTH, locate, pointAt, SAMPLES, SECTORS } from "./track.ts";
import { LINE, must, REACH } from "./world.ts";

/** The fixed tick, seconds (og.game's tick is 60). */
export const DT = 1 / 60;

/** Every car's entity, by grid slot (spawnCars fills it once in start). */
export const CARS: Entity[] = [];

type Tune = { [K in keyof typeof TUNE_DEFAULTS]: number };
const TUNE_DEFAULTS = { topSpeed: 42, accel: 15, brake: 28, steer: 2.1, grip: 9, driftGrip: 1.6, driftTurn: 1.45, boostSpeed: 52, boostAccel: 12, boostDrain: 0.42, driftCharge: 0.3, closeAfter: 30, rivalPace: 0.93 };

/** The knobs, or their defaults before the resource exists. */
export function tuning(world: World): Tune {
  const t = world.resource(Tuning);
  return t.ok ? (t.value as unknown as Tune) : TUNE_DEFAULTS;
}

const clamp = (v: number, lo: number, hi: number) => (v < lo ? lo : v > hi ? hi : v);
const wrap = (a: number) => a - Math.floor((a + Math.PI) / (2 * Math.PI)) * 2 * Math.PI;

/** The racing line's offset and speed at a distance along the lap (interpolated). */
function lineAt(along: number): { offset: number; speed: number } {
  const u = (((along / LENGTH) % 1) + 1) % 1 * COUNT;
  const i = Math.floor(u) % COUNT;
  const j = (i + 1) % COUNT;
  const w = u - Math.floor(u);
  return { offset: LINE.offset[i]! + (LINE.offset[j]! - LINE.offset[i]!) * w, speed: LINE.speed[i]! + (LINE.speed[j]! - LINE.speed[i]!) * w };
}

/** What a driver asks of a car this tick. */
interface Intent {
  steer: number;
  throttle: number;
  brake: number;
  boost: boolean;
}

/** The autopilot: pure pursuit of the racing line (plus a lane), at a share of its speed. */
export function autopilot(x: number, z: number, heading: number, speed: number, along: number, lane: number, pace: number, boost: number, top: number, cap = Infinity): Intent {
  const look = 7 + 0.42 * Math.max(0, speed);
  const ahead = along + look;
  const target = pointAt(ahead, lineAt(ahead).offset + lane);
  const fx = -math.sin(heading), fz = -math.cos(heading);
  const rx = math.cos(heading), rz = -math.sin(heading);
  const dx = target.x - x, dz = target.z - z;
  const angle = math.atan2(dx * rx + dz * rz, dx * fx + dz * fz);
  // The speed the line allows a little ahead (where the braking for a bend begins).
  const want = Math.min(cap, lineAt(along + 4 + Math.max(0, speed) * 0.35).speed * pace);
  return {
    steer: clamp(angle * 2.6, -1, 1),
    throttle: speed < want - 0.2 ? clamp((want - speed) / 2.5, 0.35, 1) : 0,
    brake: speed > want + 2.5 ? 1 : 0,
    boost: boost > 0.55 && want >= top * 0.97 && Math.abs(angle) < 0.06,
  };
}

/** Eases the wheel toward the asked lock (a key is all or nothing; a car's wheel is not). */
function easeSteer(now: number, want: number): number {
  const step = 7 * DT;
  return now + clamp(want - now, -step, step);
}

/**
 * Each seat's car from its og.PlayerInput (src/seats.ts's bits: throttle,
 * brake, boost; x the stick's steer). On the title and once finished the
 * autopilot drives it.
 */
export const Drivers = defineSystem("Drivers", { reads: [og.PlayerInput, og.Transform, Game, Tuning], writes: [Car] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  const t = tuning(world);
  for (const a of q.archetypes()) {
    const input = a.column(og.PlayerInput);
    const car = a.column(Car);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      const buttons = input.buttons[i]!;
      let want: Intent;
      if (g.phase !== Phase.Race || car.finish[i]! > 0) {
        want = autopilot(p[i * 3]!, p[i * 3 + 2]!, car.heading[i]!, car.speed[i]!, car.along[i]!, 0, 0.9, car.boost[i]!, t.topSpeed);
        if (g.phase === Phase.Race) want.boost = false;
      } else {
        want = { steer: clamp(input.x[i]!, -1, 1), throttle: held(buttons, "throttle") ? 1 : 0, brake: held(buttons, "brake") ? 1 : 0, boost: held(buttons, "boost") };
      }
      car.steer[i] = easeSteer(car.steer[i]!, want.steer);
      car.throttle[i] = want.throttle;
      car.brake[i] = want.brake;
      car.boostHeld[i] = want.boost ? 1 : 0;
      car.before[i] = buttons;
    }
  }
});

/** The rivals from the racing line: their pace, a lane round a slower car ahead, boost on the straights. */
export const Rivals = defineSystem("Rivals", { reads: [og.Transform, Game, Tuning], writes: [Car], without: [og.PlayerInput], after: ["Drivers"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  const t = tuning(world);
  // The field as it stands: each car's place on the lap and speed.
  const field = CARS.map((e) => must(world.get(e, Car)));
  const seats = field.filter((c) => c.seat >= 0);
  const lead = seats.reduce((m, c) => Math.max(m, c.progress), -Infinity);
  for (const a of q.archetypes()) {
    const car = a.column(Car);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      const me = car.index[i]!;
      const along = car.along[i]!;
      const across = car.across[i]!;
      // A slower car just ahead on its line: move a lane over, to the side with more road.
      // A car close ahead in its lane: follow at its speed until the lane is clear.
      let lane = 0;
      let cap = Infinity;
      for (const o of field) {
        if (o.index === me) continue;
        let gap = o.along - along;
        if (gap < -LENGTH / 2) gap += LENGTH;
        if (gap > LENGTH / 2) gap -= LENGTH;
        if (gap <= 0 || gap > 18 || Math.abs(o.across - across) > 2.3) continue;
        if (o.speed < car.speed[i]! + 1) lane = o.across > lineAt(along).offset ? -3.2 : 3.2;
        if (gap < 9) cap = Math.min(cap, Math.max(0, o.speed) + (gap - 5) * 0.8);
      }
      car.lane[i] = car.lane[i]! + clamp(lane - car.lane[i]!, -2 * DT, 2 * DT);
      // A gentle elastic: a rival far ahead of every seat eases, one far behind presses.
      let pace = t.rivalPace * car.pace[i]!;
      if (g.phase === Phase.Race && seats.length > 0) {
        const gap = car.progress[i]! - lead;
        pace *= gap > 60 ? 0.96 : gap < -80 ? 1.04 : 1;
      }
      const want = autopilot(p[i * 3]!, p[i * 3 + 2]!, car.heading[i]!, car.speed[i]!, along, car.lane[i]!, pace, car.boost[i]!, t.topSpeed, cap);
      if (g.phase !== Phase.Race) want.boost = false;
      car.steer[i] = easeSteer(car.steer[i]!, want.steer);
      car.throttle[i] = want.throttle;
      car.brake[i] = want.brake;
      car.boostHeld[i] = want.boost ? 1 : 0;
    }
  }
});

/** A car's capsule: its spine's half-length and its radius (the body is 4.4 by 1.9 m). */
const SPINE = 1.25;
const RADIUS = 0.95;

/** The closest points of two segments a0-a1 and b0-b1 on the ground: their parameters along each. */
function closest(a0x: number, a0z: number, a1x: number, a1z: number, b0x: number, b0z: number, b1x: number, b1z: number): [number, number] {
  const ux = a1x - a0x, uz = a1z - a0z, vx = b1x - b0x, vz = b1z - b0z, wx = a0x - b0x, wz = a0z - b0z;
  const a = ux * ux + uz * uz, b = ux * vx + uz * vz, c = vx * vx + vz * vz, d = ux * wx + uz * wz, e = vx * wx + vz * wz;
  const den = a * c - b * b;
  let s = den > 1e-9 ? clamp((b * e - c * d) / den, 0, 1) : 0;
  let u = (b * s + e) / c;
  if (u < 0) {
    u = 0;
    s = clamp(-d / a, 0, 1);
  } else if (u > 1) {
    u = 1;
    s = clamp((b - d) / a, 0, 1);
  }
  return [s, u];
}

/** A car's columns (its archetype's) and its row in them. */
type Cols = Record<string, { [i: number]: number }>;
interface Row {
  c: Cols;
  p: Float32Array;
  q: Float32Array;
  i: number;
}

/**
 * Moves every car one tick: the handling, the barriers, the contacts
 * between cars, and the lap's bookkeeping (checkpoints in order, a lap
 * at the line, the finish after LAPS). Held while the race is paused,
 * the cars standing through the countdown.
 */
export const Drive = defineSystem("Drive", { reads: [Tuning], writes: [Car, og.Transform, Game], after: ["Drivers", "Rivals"] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.held) return;
  const t = tuning(world);
  const racing = g.phase !== Phase.Title;
  const standing = racing && g.clock < COUNTDOWN;
  const rows: Row[] = [];
  for (const a of q.archetypes()) {
    const c = a.column(Car) as unknown as Cols;
    const tr = a.column(og.Transform);
    for (let i = 0; i < a.count; i++) rows[c.index![i]!] = { c, p: tr.position, q: tr.quaternion, i };
  }
  for (const r of rows) {
    if (r === undefined) continue;
    const c = r.c;
    const i = r.i;
    if (c.hitCool![i]! > 0) c.hitCool![i] = c.hitCool![i]! - 1;
    if (standing) {
      c.vx![i] = 0;
      c.vz![i] = 0;
      c.speed![i] = 0;
      continue;
    }
    let h = c.heading![i]!;
    let vx = c.vx![i]!, vz = c.vz![i]!;
    let fx = -math.sin(h), fz = -math.cos(h), rx = math.cos(h), rz = -math.sin(h);
    let vf = vx * fx + vz * fz;
    const steer = c.steer![i]!, throttle = c.throttle![i]!, brake = c.brake![i]!;
    // Boost: the meter spent for push and top speed.
    let boost = c.boost![i]!;
    const boosting = c.boostHeld![i]! !== 0 && boost > 0.01 && vf > 2;
    if (boosting) boost = Math.max(0, boost - t.boostDrain * DT);
    const top = boosting ? t.boostSpeed : t.topSpeed;
    // Drift: Brake held with Throttle and a turn at speed; it holds while both are held and the car is quick.
    let drifting = c.drifting![i]! !== 0;
    if (!drifting && brake > 0 && throttle > 0 && Math.abs(steer) > 0.3 && vf > 14) drifting = true;
    else if (drifting && (brake === 0 || throttle === 0 || vf < 9)) drifting = false;
    if (throttle > 0 && vf < top) vf += (t.accel * Math.max(0.08, 1 - Math.max(0, vf) / top) + (boosting ? t.boostAccel : 0)) * throttle * DT;
    if (vf > top) vf = Math.max(top, vf - 7 * DT);
    if (brake > 0 && !drifting) {
      if (vf > 0.3) vf = Math.max(0, vf - t.brake * brake * DT);
      else if (throttle === 0) vf = Math.max(-9, vf - 9 * DT);
    }
    if (throttle === 0 && brake === 0) vf -= Math.sign(vf) * Math.min(Math.abs(vf), 3.5 * DT);
    // The driven velocity in the world, then the heading turned under it.
    let vl = vx * rx + vz * rz;
    vx = fx * vf + rx * vl;
    vz = fz * vf + rz * vl;
    // Steering turns the heading; little at walking pace, a little less at the top.
    const turn = t.steer * steer * clamp(Math.abs(vf) / 6, 0, 1) * (1 - 0.3 * clamp(Math.abs(vf) / t.topSpeed, 0, 1)) * (drifting ? t.driftTurn : 1);
    h = wrap(h - Math.sign(vf) * turn * DT);
    fx = -math.sin(h);
    fz = -math.cos(h);
    rx = math.cos(h);
    rz = -math.sin(h);
    // In the new heading's frame the slide is sideways: the tyres catch it.
    vf = vx * fx + vz * fz;
    vl = vx * rx + vz * rz;
    const grip = drifting ? t.driftGrip : t.grip;
    const caught = vl * ((grip * DT) / (1 + grip * DT));
    vl -= caught;
    vf += Math.sign(vf) * Math.abs(caught) * (drifting ? 0.35 : 0.8);
    if (drifting) boost = Math.min(1, boost + t.driftCharge * DT);
    vx = fx * vf + rx * vl;
    vz = fz * vf + rz * vl;
    const p = r.p;
    let x = p[i * 3]! + vx * DT;
    let z = p[i * 3 + 2]! + vz * DT;
    // The barriers: back onto the road, the speed into the barrier taken, the car turned along it.
    const at = locate(x, z, c.sample![i]!);
    if (Math.abs(at.across) > REACH) {
      const s = SAMPLES[at.index]!;
      const side = Math.sign(at.across);
      const nx = s.rx * side, nz = s.rz * side;
      const depth = Math.abs(at.across) - REACH;
      x -= nx * depth;
      z -= nz * depth;
      const into = vx * nx + vz * nz;
      if (into > 0) {
        vx -= nx * into * 1.25;
        vz -= nz * into * 1.25;
        const scrub = 1 - 0.3 * clamp(into / 15, 0, 1);
        vx *= scrub;
        vz *= scrub;
        // Turned a fifth of the way along the road (whichever way it faces).
        const along = s.tx * fx + s.tz * fz >= 0 ? math.atan2(-s.tx, -s.tz) : math.atan2(s.tx, s.tz);
        h = wrap(h + wrap(along - h) * 0.2);
        hit(g, c, i, into, true);
      }
    }
    c.heading![i] = h;
    c.vx![i] = vx;
    c.vz![i] = vz;
    c.speed![i] = vx * -math.sin(h) + vz * -math.cos(h);
    c.slip![i] = vx * math.cos(h) + vz * -math.sin(h);
    c.drifting![i] = drifting ? 1 : 0;
    c.boosting![i] = boosting ? 1 : 0;
    c.boost![i] = boost;
    c.spin![i] = wrap(c.spin![i]! - (c.speed![i]! / 0.36) * DT);
    p[i * 3] = x;
    p[i * 3 + 2] = z;
  }
  contacts(g, rows);
  for (const r of rows) {
    if (r === undefined) continue;
    if (!standing) laps(g, r, racing);
    const h = r.c.heading![r.i]!;
    r.q[r.i * 4] = 0;
    r.q[r.i * 4 + 1] = math.sin(h / 2);
    r.q[r.i * 4 + 2] = 0;
    r.q[r.i * 4 + 3] = math.cos(h / 2);
  }
});

/** Ticks after a contact before another sounds (src/fx.ts reads a contact made this tick as its cool-down at full). */
export const HIT_COOL = 12;

/** A contact: counted, sounded (not too often) and, hard enough, sparks and a shake (src/fx.ts reads hits). */
function hit(g: { bumps: number; sfx: number }, c: Cols, i: number, speed: number, wall: boolean): void {
  if (speed < 1.5 || c.hitCool![i]! > 0) return;
  c.hits![i] = c.hits![i]! + 1;
  c.hitCool![i] = HIT_COOL;
  g.bumps += 1;
  g.sfx |= wall && speed > 9 ? Sound.Crash : Sound.Bump;
}

/** The cars push apart as capsules along their headings, trading the speed between them. */
function contacts(g: { bumps: number; sfx: number }, rows: Row[]): void {
  for (let a = 0; a < rows.length; a++) {
    const A = rows[a];
    if (A === undefined) continue;
    for (let b = a + 1; b < rows.length; b++) {
      const B = rows[b];
      if (B === undefined) continue;
      const ca = A.c;
      const cb = B.c;
      const ax = A.p[A.i * 3]!, az = A.p[A.i * 3 + 2]!, bx = B.p[B.i * 3]!, bz = B.p[B.i * 3 + 2]!;
      if ((ax - bx) ** 2 + (az - bz) ** 2 > 36) continue;
      const ha = ca.heading![A.i]!, hb = cb.heading![B.i]!;
      const afx = -math.sin(ha) * SPINE, afz = -math.cos(ha) * SPINE, bfx = -math.sin(hb) * SPINE, bfz = -math.cos(hb) * SPINE;
      const [s, u] = closest(ax - afx, az - afz, ax + afx, az + afz, bx - bfx, bz - bfz, bx + bfx, bz + bfz);
      const pax = ax - afx + 2 * afx * s, paz = az - afz + 2 * afz * s;
      const pbx = bx - bfx + 2 * bfx * u, pbz = bz - bfz + 2 * bfz * u;
      let nx = pbx - pax, nz = pbz - paz;
      const d = Math.sqrt(nx * nx + nz * nz);
      if (d >= 2 * RADIUS) continue;
      if (d < 1e-6) {
        nx = 1;
        nz = 0;
      } else {
        nx /= d;
        nz /= d;
      }
      const push = (2 * RADIUS - d) / 2;
      A.p[A.i * 3] = ax - nx * push;
      A.p[A.i * 3 + 2] = az - nz * push;
      B.p[B.i * 3] = bx + nx * push;
      B.p[B.i * 3 + 2] = bz + nz * push;
      const rel = (cb.vx![B.i]! - ca.vx![A.i]!) * nx + (cb.vz![B.i]! - ca.vz![A.i]!) * nz;
      if (rel >= 0) continue;
      const j = (1.3 * rel) / 2;
      ca.vx![A.i] = ca.vx![A.i]! + j * nx;
      ca.vz![A.i] = ca.vz![A.i]! + j * nz;
      cb.vx![B.i] = cb.vx![B.i]! - j * nx;
      cb.vz![B.i] = cb.vz![B.i]! - j * nz;
      hit(g, ca, A.i, -rel, false);
      hit(g, cb, B.i, -rel, false);
    }
  }
}

/** Where a car is on the lap, and the lap's bookkeeping: the checkpoints in order, the line, the finish. */
function laps(g: { clock: number; sfx: number }, r: Row, racing: boolean): void {
  const c = r.c;
  const i = r.i;
  const before = c.along![i]!;
  const at = locate(r.p[i * 3]!, r.p[i * 3 + 2]!, c.sample![i]!);
  c.sample![i] = at.index;
  c.along![i] = at.along;
  c.across![i] = at.across;
  const s = SAMPLES[at.index]!;
  const backwards = -math.sin(c.heading![i]!) * s.tx + -math.cos(c.heading![i]!) * s.tz < -0.5 && Math.abs(c.speed![i]!) > 3;
  c.wrongWay![i] = backwards ? c.wrongWay![i]! + 1 : 0;
  if (!racing || c.finish![i]! > 0) {
    c.progress![i] = c.laps![i]! * LENGTH + at.along;
    return;
  }
  const seat = c.seat![i]! >= 0;
  const forward = before > LENGTH * 0.75 && at.along < LENGTH * 0.25;
  const back = before < LENGTH * 0.25 && at.along > LENGTH * 0.75;
  const cp = c.checkpoint![i]!;
  if (forward) {
    if (cp === 3) {
      c.laps![i] = c.laps![i]! + 1;
      c.checkpoint![i] = 0;
    } else if (cp === 2) {
      const done = c.laps![i]! + 1;
      c.laps![i] = done;
      c.checkpoint![i] = 0;
      c.lapAt![i] = g.clock;
      if (done >= 1) {
        const lap = g.clock - c.lapStart![i]!;
        c.lastLap![i] = lap;
        const improved = c.bestLap![i]! === 0 || lap < c.bestLap![i]!;
        if (improved) {
          c.bestLap![i] = lap;
          c.best0![i] = c.checkpoint0![i]!;
          c.best1![i] = c.checkpoint1![i]!;
        }
        if (seat) c.sfx![i] = c.sfx![i]! | (improved && done > 1 ? Sound.Best : Sound.Lap);
      }
      // The first lap is timed from GO, the grid being behind the line.
      c.lapStart![i] = done === 0 ? COUNTDOWN : g.clock;
      if (done >= LAPS) {
        c.finish![i] = g.clock;
        if (seat) c.sfx![i] = c.sfx![i]! | Sound.Finish;
      }
    }
  } else if (back) {
    c.laps![i] = c.laps![i]! - 1;
    c.checkpoint![i] = 3;
  } else if (cp < 2) {
    const mark = SECTORS[cp]! * LENGTH;
    // Driven past it this tick (a car moves under a metre a tick; a warp past it passes nothing).
    if (before < mark && at.along >= mark && at.along - before < 20) {
      const split = g.clock - c.lapStart![i]!;
      const best = cp === 0 ? c.best0![i]! : c.best1![i]!;
      if (cp === 0) c.checkpoint0![i] = split;
      else c.checkpoint1![i] = split;
      c.checkpoint![i] = cp + 1;
      c.splitAt![i] = g.clock;
      c.splitDelta![i] = best > 0 ? split - best : 0;
      if (seat) c.sfx![i] = c.sfx![i]! | Sound.Sector;
    }
  }
  c.progress![i] = Math.min(LAPS, c.laps![i]!) * LENGTH + (c.laps![i]! >= LAPS ? 0 : at.along);
}
