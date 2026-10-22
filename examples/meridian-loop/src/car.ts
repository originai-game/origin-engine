// The car: the driver's intents and the chassis.
//
// The Driver system reads the keyboard (W/↑ throttle, S/↓ brake and
// reverse, A/D/←/→ steer, Space brake), the pad (right trigger throttle,
// left trigger brake, the left stick steers), the racing HUD's touch
// controls, or the autopilot (the `drive:auto` command: a pure-pursuit
// driver round the centre line, for playtests), quantises them as the tape
// keeps them (src/tape.ts); the Tapes system (src/ghost.ts) tapes them for
// the replay.
//
// The Chassis system turns the intents into og.Vehicle's inputs: the wheel
// eased towards the asked lock, the lock shortened with speed, and two
// assists written as small changes of og.Velocity (there is no force or
// torque call, FINDINGS #2): the roll and pitch rates damped, so a banked turn or a
// landing does not tip the car, a little downforce with four wheels
// down, and in the air the nose turned to where the car flies. It also
// reads back the slide, the air time and the landings.

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Phase, Race, Racer, Sound, Tuning } from "./components.ts";
import { encode, quantise } from "./tape.ts";
import type { Intent } from "./tape.ts";
import { at, fromStart, LENGTH, SAMPLES } from "./track.ts";
import { entity, must } from "./world.ts";

const N = SAMPLES.length;

/** The speed the autopilot aims for at sample k: from the tightest bend in the next 60 m. */
function autoSpeed(k: number): number {
  let tight = Infinity;
  for (let d = 2; d < 30; d += 2) {
    const a = at(k + d - 3);
    const b = at(k + d);
    const c = at(k + d + 3);
    const h1 = Math.atan2(b.x - a.x, b.z - a.z);
    const h2 = Math.atan2(c.x - b.x, c.z - b.z);
    let dh = h2 - h1;
    while (dh > Math.PI) dh -= 2 * Math.PI;
    while (dh < -Math.PI) dh += 2 * Math.PI;
    const radius = 12 / Math.max(1e-4, Math.abs(dh));
    // Bends further away allow more speed: there is room to brake.
    tight = Math.min(tight, Math.sqrt(11 * radius) + d * 0.3);
  }
  return Math.min(44, tight);
}

/** The autopilot: steer at a point ahead on the centre line, throttle to the bend's speed. */
function autopilot(k: number, x: number, z: number, heading: number, speed: number): Intent {
  const ahead = at(k + 3 + Math.round(speed * 0.22));
  const want = Math.atan2(ahead.x - x, ahead.z - z);
  let err = want - heading;
  while (err > Math.PI) err -= 2 * Math.PI;
  while (err < -Math.PI) err += 2 * Math.PI;
  // og.Vehicle steers right with positive steer; heading grows turning left (towards +x from +z).
  const steer = Math.max(-1, Math.min(1, -err * 1.6));
  const target = autoSpeed(k);
  const throttle = speed < target - 1.5 ? 1 : speed < target + 1 ? 0.25 : -1;
  return { steer, throttle, brake: 0 };
}

function headingOf(q: { x: number; y: number; z: number; w: number }): number {
  // Where the car's +z points, as an angle about y from +z towards +x.
  return Math.atan2(2 * (q.x * q.z + q.w * q.y), 1 - 2 * (q.x * q.x + q.y * q.y));
}

/** The player's (or the autopilot's) intents this tick. */
function intents(world: World, r: Value<typeof Race>, speed: number): Intent {
  if (r.autopilot) {
    const car = must(world.get(entity(world, "car"), og.Transform));
    const k = r.at < 0 ? 0 : fromStart(r.at);
    return autopilot(k, car.position.x, car.position.z, headingOf(car.quaternion), speed);
  }
  const inp = must(world.resource(og.Input));
  const pad = must(world.resource(og.Gamepad));
  let steer = inp.axis("ArrowLeft", "ArrowRight") + inp.axis("KeyA", "KeyD");
  let throttle = inp.down("KeyW") || inp.down("ArrowUp") ? 1 : 0;
  if (inp.down("KeyS") || inp.down("ArrowDown")) throttle -= 1;
  let brake = inp.down("Space") ? 1 : 0;
  if (pad.connected(0)) {
    const sx = pad.axis("leftX", 0);
    if (Math.abs(sx) > 0.12) steer += (sx - Math.sign(sx) * 0.12) / 0.88;
    throttle += pad.axis("rightTrigger", 0) - pad.axis("leftTrigger", 0);
    if (pad.down("a", 0)) throttle += 1;
    if (pad.down("x", 0)) brake = 1;
  }
  steer += r.touchSteer;
  if (r.touchThrottle !== 0) throttle += r.touchThrottle;
  return { steer: Math.max(-1, Math.min(1, steer)), throttle: Math.max(-1, Math.min(1, throttle)), brake };
}

export const Driver = defineSystem("Driver", { reads: [og.Input, og.Gamepad, og.Transform, og.Vehicle], writes: [Race], after: ["Race"] }, (_q, time, world) => {
  const r = must(world.resource(Race));
  if (r.held || time.scaledDt === 0) return;
  const carE = entity(world, "car");
  const v = must(world.get(carE, og.Vehicle));
  let want: Intent = { steer: 0, throttle: 0, brake: 1 };
  if (r.phase === Phase.Racing) want = quantise(intents(world, r, v.speed));
  else if (r.phase === Phase.Countdown) want = { steer: 0, throttle: 0, brake: 1 };
  else if (r.phase === Phase.Finished) want = { steer: 0, throttle: 0, brake: 0.5 };
  // A replay's intents come off the run's tape instead (the Tapes system, next).
  if (r.replaying && r.phase === Phase.Racing) return;
  r.inSteer = want.steer;
  r.inThrottle = want.throttle;
  r.inBrake = want.brake;
  r.code = encode(want);
});

/** The chassis: og.Vehicle's inputs from the intents, the assists, and what it reads back. */
export const Chassis = defineSystem("Chassis", { reads: [Tuning], writes: [Race, Racer, og.Vehicle, og.Velocity, og.Transform], after: ["Tapes"] }, (_q, time, world) => {
  const r = must(world.resource(Race));
  if (r.held || time.scaledDt === 0) return;
  const t = must(world.resource(Tuning));
  const carE = entity(world, "car");
  const racer = must(world.get(carE, Racer));
  const veh = must(world.get(carE, og.Vehicle));
  const tr = must(world.get(carE, og.Transform));
  const vel = must(world.get(carE, og.Velocity));
  const dt = time.dt;
  const speed = veh.speed;

  // The wheel eases to the asked lock; the lock shortens with speed.
  const step = t.steerRate * dt;
  const steer = racer.steer + Math.max(-step, Math.min(step, r.inSteer - racer.steer));
  const lock = 1 / (1 + Math.abs(speed) / t.steerFalloff);
  must(world.set(carE, og.Vehicle, {
    steer: steer * lock,
    throttle: r.inThrottle,
    brake: r.inBrake,
    grip: t.grip,
    engineForce: t.engineForce,
    brakeForce: t.brakeForce,
    maxSpeed: t.maxSpeed,
    maxSteer: t.maxSteer,
  }));

  // The car's axes.
  const q = tr.quaternion;
  const right = { x: 1 - 2 * (q.y * q.y + q.z * q.z), y: 2 * (q.x * q.y + q.w * q.z), z: 2 * (q.x * q.z - q.w * q.y) };
  const up = { x: 2 * (q.x * q.y - q.w * q.z), y: 1 - 2 * (q.x * q.x + q.z * q.z), z: 2 * (q.y * q.z + q.w * q.x) };
  const fwd = { x: 2 * (q.x * q.z + q.w * q.y), y: 2 * (q.y * q.z - q.w * q.x), z: 1 - 2 * (q.x * q.x + q.y * q.y) };
  const lin = vel.linear;
  const ang = vel.angular;
  // The assists: the roll (about forward) and pitch (about right) rates damped; downforce on the ground.
  const rollRate = ang.x * fwd.x + ang.y * fwd.y + ang.z * fwd.z;
  const pitchRate = ang.x * right.x + ang.y * right.y + ang.z * right.z;
  const k = t.rollDamp;
  const contacts = veh.contacts;
  const down = contacts === 4 ? t.downforce * speed * speed * dt : 0;
  const newAng = {
    x: ang.x - k * (rollRate * fwd.x + pitchRate * right.x),
    y: ang.y - k * (rollRate * fwd.y + pitchRate * right.y),
    z: ang.z - k * (rollRate * fwd.z + pitchRate * right.z),
  };
  // In the air the nose is turned towards where the car flies: a twist off the
  // kicker's lip (a few degrees at 40 m/s) would otherwise land it sideways and
  // spin it on the landing ramp.
  if (contacts === 0 && speed > 5) {
    let err = Math.atan2(lin.x, lin.z) - Math.atan2(fwd.x, fwd.z);
    while (err > Math.PI) err -= 2 * Math.PI;
    while (err < -Math.PI) err += 2 * Math.PI;
    newAng.y += (err * t.airAlign - newAng.y) * 0.25;
  }
  must(world.set(carE, og.Velocity, { linear: { x: lin.x - up.x * down, y: lin.y - up.y * down, z: lin.z - up.z * down }, angular: newAng }));

  // The slide: sideways speed at the rear axle (the yaw rate swings it out).
  const yawRate = ang.x * up.x + ang.y * up.y + ang.z * up.z;
  const side = lin.x * right.x + lin.y * right.y + lin.z * right.z;
  const slip = side + yawRate * 1.3;
  const air = contacts === 0 ? racer.air + 1 : 0;
  if (contacts > 0 && racer.air > 18 && r.phase === Phase.Racing) {
    r.landings += 1;
    r.sfx |= Sound.Land;
    r.landedAt = world.tick;
    r.landedAir = racer.air;
    if (racer.air > 30) r.jumps += 1;
  }
  if (air > r.longestAir) r.longestAir = air;
  if (air > 0 && r.phase === Phase.Racing) r.airTicks += 1;
  must(world.set(carE, Racer, { steer, contacts, air, slip }));

  // HUD readouts: km/h, a gear and the engine's revs from the speed.
  const kmh = Math.abs(speed) * 3.6;
  r.kmh = kmh;
  const shifts = [0, 11, 19, 27, 35, 60];
  let gear = 1;
  while (gear < 5 && Math.abs(speed) > shifts[gear]!) gear += 1;
  const lo = shifts[gear - 1]!;
  const hi = shifts[gear]!;
  r.gear = speed < -0.5 ? 0 : gear;
  r.rpm = Math.max(0.18, Math.min(1, (Math.abs(speed) - lo) / (hi - lo))) * (contacts === 0 ? 1 : 1);
  r.slip = slip;
  r.airborne = contacts === 0;
  const skidding = contacts >= 2 && Math.abs(slip) > 3.2 && Math.abs(speed) > 6;
  r.skidding = skidding;
  if (skidding && r.phase === Phase.Racing) r.skidTicks += 1;
  void LENGTH;
  void N;
});
