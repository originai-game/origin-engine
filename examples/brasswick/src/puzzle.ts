// The puzzle pieces: crates (dynamic boxes Wick shoves), pressure plates
// (an og.physics.overlap above each: a crate or Wick holds it down), doors
// (kinematic bodies moved by their og.Velocity, og-game §13), the gate (a
// slider on a prismatic og.Joint whose position motor opens it) and lifts
// (kinematic, turned round at their ends). Each system resets its own
// things when a level starts or Wick comes back to a lamp (Game.reset).

import { og, defineSystem } from "@origingame/engine";
import { Bits, Crate, Door, Game, Gate, Lift, Member, Phase, Plate, Sfx, Sound, Tuning } from "./components.ts";
import { clip } from "./level.ts";

/** Crates back where the level starts them, or where the last lamp saw them; a lit lamp records where they stand. */
export const Crates = defineSystem("Crates", { reads: [Crate, Member], writes: [Game, Crate, og.Transform, og.Velocity] }, (q, _time, world) => {
  const game = world.resource(Game);
  if (!game.ok) return;
  const g = game.value;
  if (g.reset === 0 && !g.snap) return;
  for (const a of q.archetypes()) {
    const level = a.column(Member).level;
    const c = a.column(Crate);
    const pos = a.column(og.Transform).position;
    const rot = a.column(og.Transform).quaternion;
    const vel = a.column(og.Velocity);
    for (let i = 0; i < a.count; i++) {
      if (level[i] !== g.level && level[i] !== g.resetLevel) continue;
      if (g.snap && level[i] === g.level) {
        c.checkX[i] = pos[i * 3]!;
        c.checkY[i] = pos[i * 3 + 1]!;
      }
      if (g.reset !== 0 && level[i] === g.resetLevel) {
        if (g.reset === 2) {
          c.checkX[i] = c.homeX[i]!;
          c.checkY[i] = c.homeY[i]!;
        }
        pos[i * 3] = c.checkX[i]!;
        pos[i * 3 + 1] = c.checkY[i]!;
        rot.set([0, 0, 0, 1], i * 4);
        vel.linear.fill(0, i * 3, i * 3 + 3);
        vel.angular.fill(0, i * 3, i * 3 + 3);
      }
    }
  }
  g.snap = false;
});

/** A plate is down while a crate or Wick stands on it; the doors and the gate read the switches. */
export const Plates = defineSystem("Plates", { reads: [Plate, Member], writes: [Game, Sfx, Plate, og.Flipbook] }, (q, _time, world) => {
  const g = world.resource(Game);
  const sfx = world.resource(Sfx);
  if (!g.ok || !sfx.ok) return;
  let switches = 0;
  let pressedNow = 0;
  // The overlaps first (engine calls), then the views (og-core §4: per-entity and engine calls before views).
  const down: boolean[] = [];
  for (const a of q.archetypes()) {
    const p = a.column(Plate);
    const level = a.column(Member).level;
    for (let i = 0; i < a.count; i++) {
      let on = false;
      if (g.value.phase !== Phase.Title && level[i] === g.value.level) {
        const hit = og.physics.overlap(world, {
          shape: { shape: 0, size: { x: 0.9, y: 0.4, z: 1 } },
          position: { x: p.x[i]!, y: p.y[i]! + 0.35, z: 0 },
          filter: { mask: Bits.Crate | Bits.Wick },
        });
        on = hit.ok && hit.value.length > 0;
      }
      down.push(on);
    }
  }
  let k = 0;
  for (const a of q.archetypes()) {
    const p = a.column(Plate);
    const fb = a.column(og.Flipbook).clip;
    for (let i = 0; i < a.count; i++) {
      const on = down[k++]!;
      if (on && !p.pressed[i]) pressedNow += 1;
      p.pressed[i] = on ? 1 : 0;
      fb[i] = on ? 1 : 0;
      if (on) switches |= 1 << p.target[i]!;
    }
  }
  g.value.switches = switches;
  if (pressedNow > 0) {
    g.value.presses += pressedNow;
    sfx.value.bits |= Sound.Plate;
  }
});

/** Doors slide up into the ceiling while their switch is held, and back down when it lets go. */
export const Doors = defineSystem("Doors", { reads: [Tuning, Door], writes: [Game, Sfx, Door, og.Transform, og.Velocity] }, (q, time, world) => {
  const game = world.resource(Game);
  const tuning = world.resource(Tuning);
  const sfx = world.resource(Sfx);
  if (!game.ok || !tuning.ok || !sfx.ok) return;
  const g = game.value;
  const speed = tuning.value.doorSpeed;
  for (const a of q.archetypes()) {
    const d = a.column(Door);
    const pos = a.column(og.Transform).position;
    const vel = a.column(og.Velocity).linear;
    for (let i = 0; i < a.count; i++) {
      const open = (g.switches >>> d.id[i]!) & 1;
      if (open !== d.open[i]) {
        d.open[i] = open;
        sfx.value.bits |= Sound.Door;
        if (open) g.opens += 1;
      }
      const target = open ? d.openY[i]! : d.closedY[i]!;
      const y = pos[i * 3 + 1]!;
      const step = speed * time.scaledDt;
      if (Math.abs(target - y) <= step || time.scaledDt === 0) {
        if (time.scaledDt > 0) pos[i * 3 + 1] = target;
        vel[i * 3 + 1] = 0;
      } else vel[i * 3 + 1] = Math.sign(target - y) * speed;
    }
  }
});

/** The gate's prismatic joint: its position motor's target is the travel while the plate is down, 0 otherwise. */
export const Gates = defineSystem("Gates", { reads: [Gate], writes: [Game, Sfx, Gate, og.Joint] }, (q, _time, world) => {
  const game = world.resource(Game);
  const sfx = world.resource(Sfx);
  if (!game.ok || !sfx.ok) return;
  for (const a of q.archetypes()) {
    const gate = a.column(Gate);
    const joint = a.column(og.Joint);
    for (let i = 0; i < a.count; i++) {
      const open = (game.value.switches >>> gate.id[i]!) & 1;
      if (open !== gate.open[i]) {
        gate.open[i] = open;
        sfx.value.bits |= Sound.Gate;
        if (open) game.value.opens += 1;
      }
      joint.motorTarget[i] = open ? gate.travel[i]! : 0;
    }
  }
});

/** Lifts run between their ends; a level's start puts them back. */
export const Lifts = defineSystem("Lifts", { reads: [Tuning, Lift, Game, Member], writes: [og.Velocity, og.Transform] }, (q, _time, world) => {
  const game = world.resource(Game);
  const tuning = world.resource(Tuning);
  if (!game.ok || !tuning.ok) return;
  const speed = tuning.value.liftSpeed;
  for (const a of q.archetypes()) {
    const l = a.column(Lift);
    const level = a.column(Member).level;
    const pos = a.column(og.Transform).position;
    const vel = a.column(og.Velocity).linear;
    for (let i = 0; i < a.count; i++) {
      if (game.value.reset === 2 && level[i] === game.value.resetLevel) {
        pos[i * 3] = l.homeX[i]!;
        vel[i * 3] = speed;
      }
      const x = pos[i * 3]!;
      if (x >= l.maxX[i]! && vel[i * 3]! > 0) vel[i * 3] = -speed;
      else if (x <= l.minX[i]! && vel[i * 3]! < 0) vel[i * 3] = speed;
    }
  }
});

void clip;
