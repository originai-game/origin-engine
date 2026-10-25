// The critters: Drive moves each seated critter by what its player asks
// (a push towards the speed asked, a dash on the action, turning to face the
// way it goes), and Bumps answers critters running into each other (a shove
// for both by how fast they met, more for one a dash hit, a squash, a thud, a
// shake, a few ticks of hit stop by how hard they met).

import { og, defineSystem } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { Critter, Fx, Game, Phase, Sfx, Sound, Tuning } from "./components.ts";
import { intentOf } from "./input.ts";
import { put, seat } from "./seats.ts";
import { must } from "./util.ts";

/** Whether the players drive their critters now (the round runs, nothing open over it). */
function driving(phase: number, paused: boolean, settings: boolean): boolean {
  return (phase === Phase.Play || phase === Phase.Podium) && !paused && !settings;
}

const wrap = (a: number) => Math.atan2(Math.sin(a), Math.cos(a));

export const Drive = defineSystem(
  "Drive",
  { reads: [og.Input, og.Gamepad, Tuning], writes: [Critter, og.Velocity, og.Transform, Game, Fx, Sfx] },
  (q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const input = must(world.resource(og.Input));
    const pads = must(world.resource(og.Gamepad));
    const fx = must(world.resource(Fx));
    const sfx = must(world.resource(Sfx));
    const dt = time.scaledDt;
    const live = driving(g.phase, g.paused, g.settingsOpen);
    for (const a of q.archetypes()) {
      const c = a.column(Critter);
      const v = a.column(og.Velocity);
      const tr = a.column(og.Transform);
      for (let i = 0; i < a.count; i++) {
        const k = c.seat[i]!;
        const s = seat(g, k);
        const px = tr.position[i * 3]!, py = tr.position[i * 3 + 1]!, pz = tr.position[i * 3 + 2]!;
        put(g, k, { x: px, y: py, z: pz });
        if (!s.joined || dt === 0) continue;
        // Its yaw, from its rotation (locked but about +y).
        const qy = tr.quaternion[i * 4 + 1]!, qw = tr.quaternion[i * 4 + 3]!;
        const yaw = 2 * Math.atan2(qy, qw);
        c.yaw[i] = yaw;
        if (c.cooldown[i]! > 0) c.cooldown[i] = c.cooldown[i]! - 1;
        if (c.stun[i]! > 0) c.stun[i] = c.stun[i]! - 1;
        const control = live && s.alive && c.stun[i] === 0 && py > -0.3;
        const want = control ? intentOf(s.source, input, pads) : { x: 0, z: 0, act: false };
        let vx = v.linear[i * 3]!, vz = v.linear[i * 3 + 2]!;
        if (c.dash[i]! > 0) {
          c.dash[i] = c.dash[i]! - 1;
        } else if (control || py > -0.3) {
          // A push towards the speed asked: quick to start, quick to stop, but a shove still carries it.
          const k2 = Math.min(1, t.grip * dt * (control ? 1 : 0.35));
          vx += (want.x * t.speed - vx) * k2;
          vz += (want.z * t.speed - vz) * k2;
        }
        if (control && want.act && c.cooldown[i] === 0) {
          // A dash the way it asks, or the way it faces.
          const l = Math.hypot(want.x, want.z);
          const dx = l > 0.1 ? want.x / l : Math.sin(yaw), dz = l > 0.1 ? want.z / l : Math.cos(yaw);
          vx = dx * t.dashSpeed;
          vz = dz * t.dashSpeed;
          c.dash[i] = t.dashTicks;
          c.cooldown[i] = t.dashCooldown;
          g.dashes += 1;
          sfx.bits |= Sound.Dash;
          fx.dust = true;
          fx.dustX = px;
          fx.dustZ = pz;
        }
        v.linear[i * 3] = vx;
        v.linear[i * 3 + 2] = vz;
        // Turn to face the way it goes (a spin about +y, eased).
        const sp = Math.hypot(vx, vz);
        const target = sp > 0.6 ? Math.atan2(vx, vz) : yaw;
        v.angular[i * 3 + 1] = control || c.dash[i]! > 0 ? wrap(target - yaw) * 14 : 0;
      }
    }
  },
);

function isCritter(world: World, e: Entity): boolean {
  return world.get(e, Critter).ok;
}

export const Bumps = defineSystem(
  "Bumps",
  { reads: [og.CollisionStarted, Tuning, og.Transform], writes: [Critter, og.Velocity, Game, Fx, Sfx] },
  (_q, _time, world) => {
    const g = must(world.resource(Game));
    if (g.phase !== Phase.Play && g.phase !== Phase.Podium) return;
    const t = must(world.resource(Tuning));
    const fx = must(world.resource(Fx));
    const sfx = must(world.resource(Sfx));
    for (const e of world.events(og.CollisionStarted)) {
      if (!isCritter(world, e.a) || !isCritter(world, e.b)) continue;
      const ca = must(world.get(e.a, Critter)), cb = must(world.get(e.b, Critter));
      // The contact's normal from a to b, flat (two spheres of one size meet level; a lands on b's
      // head only in a fall, and then the line between them serves).
      let nx = e.normal.x, nz = e.normal.z;
      let l = Math.hypot(nx, nz);
      if (l < 0.1) {
        const pa = must(world.get(e.a, og.Transform)).position, pb = must(world.get(e.b, og.Transform)).position;
        nx = pb.x - pa.x;
        nz = pb.z - pa.z;
        l = Math.hypot(nx, nz) || 1;
      }
      nx /= l;
      nz /= l;
      // How fast they closed: the event's velocity of a relative to b at the contact as the step
      // began, along the flat line (og.Velocity by this tick holds the solver's answer: FINDINGS.md #9).
      const rv = e.relativeVelocity;
      const closing = Math.max(0, rv.x * nx + rv.z * nz);
      const shove = Math.max(t.bumpMin, closing * t.bumpShove);
      const aDash = ca.dash > 0, bDash = cb.dash > 0;
      // The one a dash hit takes most of it, and is stunned; the dasher recoils a little.
      const toB = aDash && !bDash ? shove * 1.6 : bDash && !aDash ? shove * 0.35 : shove * 0.8;
      const toA = bDash && !aDash ? shove * 1.6 : aDash && !bDash ? shove * 0.35 : shove * 0.8;
      og.physics.applyImpulse(world, e.b, { x: nx * toB, y: 0.6, z: nz * toB });
      og.physics.applyImpulse(world, e.a, { x: -nx * toA, y: 0.6, z: -nz * toA });
      if (aDash && !bDash) must(world.set(e.b, Critter, { stun: t.stunTicks }));
      if (bDash && !aDash) must(world.set(e.a, Critter, { stun: t.stunTicks }));
      for (const m of [ca.mesh, cb.mesh]) og.tween.squash(world, { entity: m, impact: { x: 1.28, y: 0.72, z: 1.28 }, rest: { x: 1, y: 0.92, z: 1 }, duration: 0.28 });
      g.bumps += 1;
      sfx.bits |= Sound.Bump;
      fx.shake = Math.max(fx.shake, aDash || bDash ? 0.45 : 0.22);
      // Hit stop by impact weight (og-feel §3): three ticks for a dash, two for a hard bump (closing
      // faster than half the top speed), none for a jostle. The engine never shortens one running.
      const stop = aDash || bDash ? 3 : closing >= t.speed * 0.5 ? 2 : 0;
      if (stop > 0) og.time.hitStop(world, stop);
    }
  },
);
