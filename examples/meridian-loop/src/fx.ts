// What the car throws off, drawn from the physics' last step:
//
// - the four wheel models at og.physics.vehicleWheels' poses;
// - tyre smoke at the rear wheels while the car slides (og.ParticleSystem's
//   startTick and stopTick), and skid marks: a ring of og.Decal streaks
//   laid behind the rear wheels every 1.1 m of a slide, fading as they age;
// - sparks and a knock where the car meets a wall (og.CollisionStarted),
//   the camera shaken by how hard;
// - dust and a shake when the car lands after the jump.

import { og, defineSystem } from "@origingame/engine";
import { Mark, Phase, Prefs, Race, Racer, Sound } from "./components.ts";
import { yaw } from "./world.ts";
import { entity, MARKS, must } from "./world.ts";

/** A mark lasts this many ticks, fading over its last third. */
const MARK_LIFE = 600;

export const Fx = defineSystem(
  "Fx",
  { reads: [Prefs, Racer, og.CollisionStarted], writes: [Race, og.Transform, og.ParticleSystem], after: ["Chassis"] },
  (_q, time, world) => {
    const r = must(world.resource(Race));
    const prefs = must(world.resource(Prefs));
    const carE = entity(world, "car");
    const car = must(world.get(carE, og.Transform));
    const racer = must(world.get(carE, Racer));
    const wheels = og.physics.vehicleWheels(world, carE);
    const poses = wheels.ok ? wheels.value : [];
    poses.forEach((w, i) => must(world.set(entity(world, `wheel-${i}`), og.Transform, { position: w.position, quaternion: w.rotation })));
    if (r.held || time.scaledDt === 0) return;
    const tick = world.tick;
    const shake = prefs.reducedMotion ? 0.3 : 1;

    // Smoke from the rear wheels while sliding.
    const sliding = r.skidding && r.phase === Phase.Racing;
    for (const [side, w] of [["l", poses[2]], ["r", poses[3]]] as const) {
      if (!w) continue;
      const e = entity(world, `smoke-${side}`);
      const ps = must(world.get(e, og.ParticleSystem));
      must(world.set(e, og.Transform, { position: { x: w.position.x, y: w.position.y - 0.2, z: w.position.z } }));
      const on = ps.stopTick === 0;
      if (sliding && !on) must(world.set(e, og.ParticleSystem, { startTick: tick, stopTick: 0 }));
      if (!sliding && on) must(world.set(e, og.ParticleSystem, { stopTick: tick }));
    }

    // Walls struck this tick.
    const walls = entity(world, "walls");
    for (const c of world.events(og.CollisionStarted)) {
      // The car against the walls (its belly grounding on the road is not a knock).
      if (!((c.a === carE && c.b === walls) || (c.b === carE && c.a === walls))) continue;
      if (r.phase !== Phase.Racing) continue;
      const v = Math.abs(racer.slip) + Math.abs(r.kmh / 3.6) * 0.25;
      r.bumps += 1;
      r.sfx |= Sound.Bump;
      const sp = entity(world, "sparks");
      must(world.set(sp, og.Transform, { position: c.point }));
      must(world.set(sp, og.ParticleSystem, { startTick: tick, stopTick: 0 }));
      og.camera.shake(world, Math.min(0.45, 0.12 + v * 0.02) * shake);
      og.camera.kick(world, { x: c.normal.x * (c.a === carE ? -1 : 1), y: 0, z: c.normal.z * (c.a === carE ? -1 : 1) }, 0.15 * shake);
      break;
    }

    // A landing after a long flight.
    if (r.landedAt === tick - 0 && r.landedAir > 18 && r.phase === Phase.Racing) {
      const d = entity(world, "dust");
      must(world.set(d, og.Transform, { position: { x: car.position.x, y: car.position.y - 0.2, z: car.position.z } }));
      must(world.set(d, og.ParticleSystem, { startTick: tick, stopTick: 0 }));
      og.camera.shake(world, Math.min(0.3, 0.08 + r.landedAir * 0.003) * shake);
      og.camera.punch(world, 0.06 * shake);
    }
  },
);

/** The skid marks: a ring of decals over the Mark entities (og.Decal, og.Transform). */
export const Marks = defineSystem("Marks", { writes: [Race, Mark, og.Decal, og.Transform], after: ["Fx"] }, (q, time, world) => {
  const r = must(world.resource(Race));
  if (r.held || time.scaledDt === 0) return;
  const carE = entity(world, "car");
  const car = must(world.get(carE, og.Transform));
  const wheels = og.physics.vehicleWheels(world, carE);
  const poses = wheels.ok ? wheels.value : [];
  const sliding = r.skidding && r.phase === Phase.Racing;

  // Skid marks: a streak behind each rear wheel every 1.1 m of a slide; the oldest slot is reused.
  const travelled = r.markTravel + Math.abs(r.kmh / 3.6) * time.dt;
  const lay = sliding && travelled >= 1.1 && poses.length === 4;
  r.markTravel = lay ? 0 : sliding ? travelled : 0;
  const heading = Math.atan2(2 * (car.quaternion.x * car.quaternion.z + car.quaternion.w * car.quaternion.y), 1 - 2 * (car.quaternion.x * car.quaternion.x + car.quaternion.y * car.quaternion.y));
  // Laid flat: the decal projects along its -z, turned to point down, its length along the car.
  const flat = yaw(heading);
  const down = { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 };
  const rot = {
    x: flat.w * down.x + flat.x * down.w + flat.y * down.z - flat.z * down.y,
    y: flat.w * down.y - flat.x * down.z + flat.y * down.w + flat.z * down.x,
    z: flat.w * down.z + flat.x * down.y - flat.y * down.x + flat.z * down.w,
    w: flat.w * down.w - flat.x * down.x - flat.y * down.y - flat.z * down.z,
  };
  let next = r.nextMark;
  const places: { x: number; y: number; z: number }[] = [];
  if (lay) for (const w of [poses[2]!, poses[3]!]) places.push({ x: w.position.x, y: w.position.y - 0.3, z: w.position.z });
  for (const a of q.archetypes()) {
    const mark = a.column(Mark);
    const decal = a.column(og.Decal);
    const t = a.column(og.Transform);
    for (let i = 0; i < a.count; i++) {
      let age = mark.age[i]! + 1;
      const slot = mark.slot[i]!;
      for (let p = 0; p < places.length; p++) {
        if (slot !== (next + p) % MARKS) continue;
        const at = places[p]!;
        t.position[i * 3] = at.x;
        t.position[i * 3 + 1] = at.y;
        t.position[i * 3 + 2] = at.z;
        t.quaternion[i * 4] = rot.x;
        t.quaternion[i * 4 + 1] = rot.y;
        t.quaternion[i * 4 + 2] = rot.z;
        t.quaternion[i * 4 + 3] = rot.w;
        age = 0;
      }
      mark.age[i] = Math.min(age, MARK_LIFE);
      decal.opacity[i] = age >= MARK_LIFE ? 0 : Math.min(0.85, ((MARK_LIFE - age) / (MARK_LIFE / 3)) * 0.85);
    }
  }
  if (places.length > 0) next = (next + places.length) % MARKS;
  r.nextMark = next;
});
