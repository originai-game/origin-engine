// What the cars look and feel like, from their state (nothing here feeds
// back into the race):
//
//   CarView  each seat's view (src/view.ts's SeatLook): the chase camera's
//            heading easing after its car's, the cockpit's the car's own,
//            and the field of view widening with speed and boost.
//   Scene    each car's body rolling into a slide and squatting under
//            power, its wheels turning and the front ones steering, the
//            boost flame, tyre smoke while it drifts or slides, sparks and
//            a flash on a contact, the camera's shake and punch for this
//            device's cars, and the gantry's lamps through the countdown.

import { og, defineSystem, math } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Car, COUNTDOWN, Game, Phase, Tuning } from "./components.ts";
import { prefs } from "./device.ts";
import { CARS, DT, HIT_COOL, tuning } from "./car.ts";
import { localSeats, LOCAL } from "./seats.ts";
import { FIRST_PERSON, ISOMETRIC, SeatLook } from "./view.ts";
import { LENGTH, pointAt } from "./track.ts";
import { must, partOf, setLamps } from "./world.ts";

const wrap = (a: number) => a - Math.floor((a + Math.PI) / (2 * Math.PI)) * 2 * Math.PI;

/** The chase camera's pitch (looking a little down on the car) and the cockpit's. */
const CHASE_PITCH = -0.11;
const COCKPIT_PITCH = -0.05;

/** Turns each seat's look after its car (src/view.ts's ViewRig then points the camera). */
export const CarView = defineSystem("CarView", { reads: [Game], writes: [Car, SeatLook] }, (q, _time, world) => {
  const g = must(world.resource(Game));
  for (const a of q.archetypes()) {
    const car = a.column(Car);
    const look = a.column(SeatLook);
    for (let i = 0; i < a.count; i++) {
      const h = car.heading[i]!;
      if (g.start) car.camYaw[i] = h;
      // The chase camera eases after the heading (quicker the faster the car turns), so a drift shows the car's flank.
      car.camYaw[i] = wrap(car.camYaw[i]! + wrap(h - car.camYaw[i]!) * Math.min(1, 6 * DT));
      if (!ISOMETRIC) {
        look.yaw[i] = FIRST_PERSON ? h : car.camYaw[i]!;
        look.pitch[i] = FIRST_PERSON ? COCKPIT_PITCH : CHASE_PITCH;
      }
    }
  }
});

/** The field of view for a car's speed: wider as it goes, wider still on boost. */
function fovFor(speed: number, boosting: boolean, top: number): number {
  return (FIRST_PERSON ? 70 : 58) + 12 * Math.min(1, Math.max(0, speed) / top) + (boosting ? 6 : 0);
}

/** Whether each car was boosting when this device last drew it (the camera's punch is this device's). */
const boosted: boolean[] = [];

/** Ry(a) * Rx(b) as a quaternion. */
function yawPitch(a: number, b: number) {
  const sy = math.sin(a / 2), cy = math.cos(a / 2), sp = math.sin(b / 2), cp = math.cos(b / 2);
  return { x: cy * sp, y: sy * cp, z: -sy * sp, w: cy * cp };
}

/** Rz(roll) * Rx(pitch). */
function rollPitch(roll: number, pitch: number) {
  const sr = math.sin(roll / 2), cr = math.cos(roll / 2), sp = math.sin(pitch / 2), cp = math.cos(pitch / 2);
  return { x: cr * sp, y: sr * sp, z: sr * cp, w: cr * cp };
}

export const Scene = defineSystem("Scene", { reads: [Car, Game, Tuning, og.Lockstep], writes: [og.Transform, og.Mesh, og.ParticleSystem, og.Camera, og.CameraView], after: ["Race"] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  const t = tuning(world);
  const reduced = prefs.reducedMotion;
  const tick = world.tick;
  const mine = localSeats(world);
  for (const e of CARS) {
    const c = must(world.get(e, Car));
    const k = c.index;
    // The body: rolled out of a slide, squatting under power and boost, diving under the brake.
    const roll = Math.max(-0.07, Math.min(0.07, -c.slip * 0.012));
    const pitch = c.brake > 0 && c.speed > 2 && !c.drifting ? -0.035 : c.throttle > 0 && c.speed < 30 ? 0.02 : c.boosting ? 0.025 : 0;
    must(world.set(partOf(world, k, "body"), og.Transform, { quaternion: rollPitch(roll, pitch) }));
    for (let w = 0; w < 4; w++) must(world.set(partOf(world, k, `wheel-${w}`), og.Transform, { quaternion: yawPitch(w < 2 ? -c.steer * 0.42 : 0, c.spin) }));
    must(world.set(partOf(world, k, "flame"), og.Mesh, { visible: c.boosting }));
    if (c.boosting) must(world.set(partOf(world, k, "flame"), og.Transform, { scale: { x: 1, y: 0.8 + 0.4 * ((tick * 7 + k * 3) % 5) / 4, z: 1 } }));
    // Tyre smoke while drifting or sliding hard.
    const smoke = partOf(world, k, "smoke");
    const ps = must(world.get(smoke, og.ParticleSystem));
    const sliding = c.drifting || Math.abs(c.slip) > 4.5;
    const on = ps.stopTick === 0;
    if (sliding && !on) must(world.set(smoke, og.ParticleSystem, { startTick: tick, stopTick: 0 }));
    if (!sliding && on) must(world.set(smoke, og.ParticleSystem, { stopTick: tick }));
    // A contact this tick (Drive just set its cool-down): sparks and a flash for every car; the shake for this device's.
    const own = c.seat >= 0 && mine.includes(c.seat);
    if (c.hitCool === HIT_COOL) {
      must(world.set(partOf(world, k, "sparks"), og.ParticleSystem, { visible: true, startTick: tick }));
      must(og.tween.flash(world, { entity: partOf(world, k, "body"), color: { r: 1, g: 0.9, b: 0.7 }, amount: 0.6, duration: 0.15 }));
      if (own && !LOCAL && !reduced) must(og.camera.shake(world, Math.min(0.45, 0.12 + Math.sqrt(c.vx * c.vx + c.vz * c.vz) * 0.006)));
    }
    // Boost lit: the view kicks out a little for this device's car.
    if (c.boosting && !boosted[k] && own && !LOCAL && !reduced) must(og.camera.punch(world, -0.06));
    boosted[k] = c.boosting;
    // The field of view: this device's car's on the camera, each seat's on its half of a split screen.
    if (own && !ISOMETRIC && g.phase !== Phase.Title) {
      const want = fovFor(c.speed, c.boosting, t.topSpeed);
      if (LOCAL) {
        const v = world.find(`view-${c.seat}`);
        if (v !== null) {
          const now = must(world.get(v, og.CameraView)).fov;
          must(world.set(v, og.CameraView, { fov: now + (want - now) * 0.15 }));
        }
      } else {
        const cam = must(world.resource(og.Camera));
        cam.fov = cam.fov + (want - cam.fov) * 0.15;
      }
    }
  }
  titleCamera(world, g.phase === Phase.Title);
  // The gantry: a lamp a fifth of the countdown, every one green at GO, dark a second later.
  if (g.phase === Phase.Race && g.clock <= COUNTDOWN + 60) {
    if (g.clock < COUNTDOWN) setLamps(world, Math.min(5, Math.floor(g.clock / (COUNTDOWN / 5)) + 1), false);
    else setLamps(world, 0, g.clock < COUNTDOWN + 60);
  } else if (g.start) setLamps(world, 0, false);
});

/** The title's camera posts: one every POST metres of the lap, POST_OUT metres right of the centre line, POST_UP up. */
const POST = 90;
const POST_OUT = 21;
const POST_UP = 12;

/**
 * Behind the title a broadcast camera follows seat 0's car (it leads the
 * field's attract laps) from the nearest of the posts along the circuit,
 * cutting to the next as the car passes; with two seats on one device the
 * split's views rest and the main frame draws it. A race gives the camera
 * back to src/view.ts (the rig keeps following the car meanwhile).
 */
function titleCamera(world: World, title: boolean): void {
  const cam = must(world.resource(og.Camera));
  if (LOCAL) {
    cam.layers = title ? 1 : 0;
    for (let seat = 0; seat < 2; seat++) {
      const v = world.find(`view-${seat}`);
      if (v !== null && must(world.get(v, og.CameraView)).active !== !title) must(world.set(v, og.CameraView, { active: !title }));
    }
  }
  if (ISOMETRIC) cam.orthographic = !title;
  if (!title) return;
  const car = must(world.get(CARS[0]!, Car));
  const at = must(world.get(CARS[0]!, og.Transform)).position;
  const k = Math.round((car.along - POST / 2) / POST);
  const post = pointAt((((k * POST + POST / 2) % LENGTH) + LENGTH) % LENGTH, POST_OUT);
  const eye = { x: post.x, y: POST_UP, z: post.z };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: at.x, y: 0.9, z: at.z });
  cam.fov = 42;
}

/** Called once in start: the camera's feel (a quick decay so a knock is a knock). */
export function setupFeel(world: World): void {
  const cam = must(world.resource(og.Camera));
  cam.traumaDecay = 1.6;
  cam.punchDecay = 5;
}
