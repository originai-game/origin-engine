// The striders: one per seat (src/seats.ts), each a capsule on
// og.CharacterController carrying a strider body (skinned, run by the graph
// of src/anim.ts), moved by its own og.PlayerInput alone. The move (WASD,
// the arrows, the left stick, the touch joystick) walks relative to its
// view, Run (Shift, the right trigger or a click of the stick; on the touch
// HUD a full stick) runs, Jump (Space, A, Jump) jumps, holding Kindle (F, X,
// Kindle) by a beacon walks up to it and reaches into the bowl with the
// right hand (an og.IkChain on the arm) until it catches. Wading in the lake
// slows a strider and splashes; the sentinels cannot follow there. The look
// (the locked mouse, the right stick, a finger dragged across the touch
// HUD's look zone; Q and E) turns the strider's own view, which swings
// slowly behind it as it walks; the view's heading is world state, so a
// room turns it on every peer.
//
// The View system then puts the cameras behind them: og.Camera behind the
// strider this device plays, and with two seats on one device each its own
// half of the screen (an og.CameraView per seat).

import { og, defineSystem, math, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { heroRig } from "./anim.ts";
import { Beacon, Ending, Game, Hero, Phase, Sound, Tuning } from "./components.ts";
import { held, LOCAL, localSeats, SEATS } from "./seats.ts";
import { BEACON_AT, ground, LAKE, must, START, WATER_Y } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type HeroV = Value<typeof Hero>;

/** The capsule's centre above the feet (radius 0.3, a 0.9 m segment). */
export const FEET = 0.75;
/** The strider's scale (about 1.6 m tall). */
const BODY_SCALE = 0.92;
/** The strider's right hand: its index among the skin's joints (Hips is 0). */
export const RIGHT_HAND = 12;
/** How near a beacon's centre the hero stands to reach into its bowl. */
export const STAND = 1.12;
/** Seat 1's strider: its body's tint (a cooler cloth beside seat 0's), and where it starts beside seat 0's. */
const SECOND_TINT = { r: 0.55, g: 0.78, b: 1 };
const SECOND_START = 1.4;
/** The view's turn by Q and E, radians a second. */
const KEY_TURN = 2.2;
/** Ticks the look stays idle before the view swings behind a walking strider again. */
const LOOK_HOLD = 30;
const DEG = Math.PI / 180;
const UP = { x: 0, y: 1, z: 0 };

/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not). */
export const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

/** Tuning, read on its first use in this call (a resource read encodes every field). */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

export function entity(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`${name} is missing`);
  return e;
}

/** Seat `seat`'s strider (seat 0's is `hero`, seat 1's `hero-1`). */
export function heroOf(g: GameV, seat: number): Entity {
  return (seat === 0 ? g.hero : g.hero1) as unknown as Entity;
}

/** Where seat `seat`'s strider starts. */
function startOf(seat: number): { x: number; z: number } {
  return { x: START.x + seat * SECOND_START, z: START.z };
}

/** Spawns a strider per seat at the start, each with its body, its reach and its splash. */
export function spawnHeroes(world: World): void {
  const g = must(world.resource(Game));
  const rig = heroRig(world);
  const model = must(og.load.gltf(world, "models/strider.glb"));
  for (let seat = 0; seat < SEATS; seat++) {
    const name = seat === 0 ? "hero" : `hero-${seat}`;
    const at = startOf(seat);
    const h = ground(world, g, at.x, at.z).height;
    const hero = must(world.spawn({ name, position: { x: at.x, y: h + FEET + 0.05, z: at.z } }));
    must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 0.9 }));
    must(world.insert(hero, og.CharacterController, { maxSlope: 42, stepHeight: 0.35, snapDistance: 0.3, coyoteTime: 0.12 }));
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    // The sentinels see the nearest strider (og.Perception) and hunt it.
    must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
    const body = must(world.instantiate(model, { name: `${name}-body`, parent: hero, position: { x: 0, y: -FEET, z: 0 }, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
    must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
    if (seat > 0) must(world.set(body, og.Model, { tint: SECOND_TINT }));
    const reach = must(world.spawn({ name: `${name}-reach` }));
    must(world.insert(reach, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 3, solver: "fabrik", tolerance: 0.02, space: "world", weight: 0 }));
    const splash = must(world.spawn({ name: `${name}-splash`, position: { x: 0, y: WATER_Y, z: 0 } }));
    must(world.insert(splash, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/splash.ogt")), seed: 5 + seat, visible: false }));
    must(world.insert(hero, Hero, { seat, body: body as unknown as number, reach: reach as unknown as number, splash: splash as unknown as number }));
    if (seat === 0) g.hero = hero as unknown as number;
    else g.hero1 = hero as unknown as number;
  }
}

function wrap(a: number): number {
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

/** Turns `from` towards `to` by at most `step` radians. */
export function turn(from: number, to: number, step: number): number {
  const d = wrap(to - from);
  return wrap(from + Math.max(-step, Math.min(step, d)));
}

function splash(world: World, g: GameV, me: HeroV, at: Vector3): void {
  const e = me.splash as unknown as Entity;
  must(world.set(e, og.Transform, { position: { x: at.x, y: WATER_Y + 0.02, z: at.z } }));
  must(world.set(e, og.ParticleSystem, { visible: true, startTick: world.tick }));
  g.sfx |= Sound.Splash;
  g.splashes += 1;
}

/** Puts a strider back at its start, facing north, its view behind it. */
function reset(world: World, g: GameV, hero: Entity, me: HeroV): void {
  const at = startOf(me.seat);
  const h = ground(world, g, at.x, at.z).height;
  must(world.set(hero, og.Transform, { position: { x: at.x, y: h + FEET + 0.05, z: at.z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  me.airTime = 0;
  me.stepClock = 0;
  me.facing = Math.PI;
  me.yaw = 0;
  me.pitch = 0;
  me.lookIdle = LOOK_HOLD;
  me.knock = 0;
  me.invulnerable = 0;
  me.kindling = false;
}

/**
 * Per tick, for each strider: reads its og.PlayerInput, moves the capsule,
 * turns the body, drives the graph and the reach, makes the steps, the
 * splashes and the jumps heard, and turns its view.
 */
export const HeroMove = defineSystem(
  "Hero",
  { reads: [og.PlayerInput, Tuning, Beacon], writes: [Game, Hero, og.CharacterController, og.Transform, og.AnimationGraph, og.IkChain, og.ParticleSystem, og.PostEffects], after: ["Ui"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = lazyTuning(world);
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    const heroes: { hero: Entity; me: HeroV; buttons: number; x: number; y: number; aimX: number; aimY: number }[] = [];
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      const input = must(world.get(hero, og.PlayerInput));
      if (g.start) reset(world, g, hero, me);
      // Press edges (Ui read them against last tick's buttons first), kept through a hit-stop.
      me.before = input.buttons;
      heroes.push({ hero, me, buttons: input.buttons, x: input.x, y: input.y, aimX: input.aimX, aimY: input.aimY });
    }
    const dt = time.scaledDt;
    if (dt === 0) {
      for (const h of heroes) must(world.set(h.hero, Hero, h.me));
      return;
    }
    const post = must(world.resource(og.PostEffects));
    if (post.flash > 0) post.flash = Math.max(0, post.flash - dt * 2.5);
    for (const h of heroes) {
      step(world, g, t, h.hero, h.me, h, playing, dt, time.dt);
      must(world.set(h.hero, Hero, h.me));
    }
    const first = heroes[0]!.me;
    g.kindling = first.kindling;
    g.wading = first.wading;
    g.grounded = first.grounded;
    g.speed = first.speed;
  },
);

/** One strider's tick. */
function step(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV, input: { buttons: number; x: number; y: number; aimX: number; aimY: number }, playing: boolean, dt: number, viewDt: number): void {
  const body = me.body as unknown as Entity;
  const reach = me.reach as unknown as Entity;
  const cc = must(world.get(hero, og.CharacterController));
  const p = must(world.get(hero, og.Transform)).position;
  const buttons = input.buttons;

  // The move, relative to the view: forward is where its camera looks.
  const ix = input.x;
  const iy = input.y;
  const mag = Math.min(1, hypot(ix, iy));
  const run = held(buttons, "run");
  const fwd = { x: -math.sin(me.yaw), z: -math.cos(me.yaw) };
  const right = { x: math.cos(me.yaw), z: -math.sin(me.yaw) };
  let dx = 0;
  let dz = 0;
  if (mag > 0.01) {
    const n = hypot(ix, iy);
    dx = (right.x * ix + fwd.x * iy) / n;
    dz = (right.z * ix + fwd.z * iy) / n;
  }

  const feet = p.y - FEET;
  me.wading = feet < WATER_Y - 0.08 && hypot(p.x - LAKE.x, p.z - LAKE.z) < 18;
  me.kindling = playing && held(buttons, "kindle") && me.near >= 0;

  let speed = 0;
  let face: number | null = null;
  if (me.knock > 0) {
    const k = me.knock / 0.35;
    speed = 7 * k;
    dx = me.knockX;
    dz = me.knockZ;
    me.knock = Math.max(0, me.knock - dt);
  } else if (me.kindling) {
    // Walk up to the plinth, then stand and reach.
    const b = BEACON_AT[me.near]!;
    const vx = b.x - p.x;
    const vz = b.z - p.z;
    const d = hypot(vx, vz);
    face = math.atan2(vx, vz);
    if (d > STAND) {
      dx = vx / d;
      dz = vz / d;
      speed = t.walkSpeed;
    }
  } else if (playing && mag > 0.01) {
    // A stick part-way walks slower; a full one walks, and Run runs.
    speed = run ? t.runSpeed : t.walkSpeed * Math.min(1, mag / 0.85);
    face = math.atan2(dx, dz);
  }
  if (me.wading) speed *= t.wade;
  // Not on the tick a run starts: the Jump that set out from the title or the summary.
  const jump = playing && !g.start && !me.kindling && me.knock === 0 && held(buttons, "jump");
  must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed }, jump, jumpSpeed: t.jumpSpeed }));
  if (me.invulnerable > 0) me.invulnerable = Math.max(0, me.invulnerable - dt);

  // The body turns towards where it goes (not while thrown back).
  if (face !== null) me.facing = turn(me.facing, face, 11 * dt);
  must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, me.facing) }));

  // The graph: speed across the ground, and whether it is in the air.
  const v = cc.velocity;
  const hs = hypot(v.x, v.z);
  me.speed = hs;
  const airTime = cc.grounded ? 0 : me.airTime + dt;
  must(world.set(body, og.AnimationGraph, { p0: cc.grounded || airTime < 0.12 ? hs : 0, p1: airTime > 0.12 ? 1 : 0 }));

  // The reach: the right hand into the near bowl while kindling.
  const ik = must(world.get(reach, og.IkChain));
  let weight = ik.weight;
  const want = me.kindling ? 1 : 0;
  weight += Math.max(-dt * 5, Math.min(dt * 5, want - weight));
  if (me.near >= 0) {
    const b = BEACON_AT[me.near]!;
    const bowl = world.find(`beacon-${me.near}`);
    const top = bowl === null ? p.y : must(world.get(bowl, Beacon)).top;
    const vx = p.x - b.x;
    const vz = p.z - b.z;
    const d = Math.max(0.01, hypot(vx, vz));
    // IkChain's "world" space is the frame the body's Transform is in: the hero's (unrotated) capsule.
    must(world.set(reach, og.IkChain, { weight, target: { x: b.x + (vx / d) * 0.5 - p.x, y: top + 0.05 - p.y, z: b.z + (vz / d) * 0.5 - p.z } }));
  } else if (weight !== ik.weight) must(world.set(reach, og.IkChain, { weight }));

  // Steps, splashes, jumps and landings.
  if (cc.grounded && hs > 0.5) {
    me.stepClock += hs * dt;
    const stride = hs > 4 ? 1.6 : 1.15;
    if (me.stepClock >= stride) {
      me.stepClock -= stride;
      me.steps += 1;
      if (me.wading) splash(world, g, me, p);
      else g.sfx |= Sound.Step;
    }
  }
  if (!cc.grounded && me.airTime === 0 && v.y > 1) {
    g.jumps += 1;
    g.sfx |= Sound.Jump;
  }
  if (cc.grounded && me.airTime > 0.3) {
    if (feet < WATER_Y) splash(world, g, me, p);
    else g.sfx |= Sound.Step;
  }
  me.grounded = cc.grounded;
  me.airTime = airTime;

  // The view: turned by the look (right turns it right) and Q or E, and
  // swung behind a walking strider (not one walking at the camera) once the
  // look has rested a moment.
  if (g.phase === Phase.Title) return;
  const keys = (held(buttons, "turnLeft") ? 1 : 0) - (held(buttons, "turnRight") ? 1 : 0);
  const looked = input.aimX !== 0 || input.aimY !== 0 || keys !== 0;
  me.lookIdle = looked ? 0 : Math.min(LOOK_HOLD, me.lookIdle + 1);
  let turnBy = keys * KEY_TURN;
  const behind = me.facing + Math.PI;
  if (!looked && me.lookIdle >= LOOK_HOLD && me.speed > 0.5 && Math.abs(wrap(behind - me.yaw)) < 2.2) turnBy = wrap(behind - me.yaw) * Math.min(1, me.speed / 6) * 0.9;
  me.yaw = wrap(me.yaw + turnBy * viewDt - input.aimX * DEG);
  if (input.aimY !== 0) me.pitch = Math.max(PITCH_MIN - PITCH, Math.min(PITCH_MAX - PITCH, me.pitch - input.aimY * DEG));
}

/** How far the camera's eye stays above the ground and the water. */
const CLEARANCE = 0.6;
/** The view's resting pitch, down from level, and how far the look tips it. */
const PITCH = 0.36;
const PITCH_MIN = -0.05;
const PITCH_MAX = 1.1;

/** Where a strider's camera wants to be: behind it along its view, above the ground and the lake. */
function eyeOf(world: World, g: GameV, t: TuneV, hero: Entity): { eye: Vector3; focus: Vector3 } {
  const me = must(world.get(hero, Hero));
  const p = must(world.get(hero, og.Transform)).position;
  const d = t.cameraDistance;
  const pitch = PITCH + me.pitch;
  const focus = { x: p.x, y: p.y + 0.9, z: p.z };
  const eye = {
    x: focus.x + math.sin(me.yaw) * d * math.cos(pitch),
    y: focus.y + d * math.sin(pitch),
    z: focus.z + math.cos(me.yaw) * d * math.cos(pitch),
  };
  const floor = Math.max(ground(world, g, eye.x, eye.z).height, WATER_Y) + CLEARANCE;
  if (eye.y < floor) eye.y = floor;
  return { eye, focus };
}

/** Eases a camera at `at` towards `eye` (snapping when far or on a new run). */
function ease(at: Vector3, eye: Vector3, dt: number, snap: boolean): Vector3 {
  const far = Math.sqrt((eye.x - at.x) ** 2 + (eye.y - at.y) ** 2 + (eye.z - at.z) ** 2) > 12 || snap;
  const k = far ? 1 : 1 - math.exp(-dt * 9);
  return { x: at.x + (eye.x - at.x) * k, y: at.y + (eye.y - at.y) * k, z: at.z + (eye.z - at.z) * k };
}

/** The split screen's views: one og.CameraView per seat, seat 0 the left half and seat 1 the right. */
export function spawnViews(world: World): void {
  if (!LOCAL) return;
  for (let seat = 0; seat < SEATS; seat++) {
    const v = must(world.spawn({ name: `view-${seat}` }));
    must(world.insert(v, og.CameraView, { viewportX: seat / SEATS, viewportWidth: 1 / SEATS, viewportHeight: 1, fov: 55, far: 400, order: seat, active: false }));
  }
}

/**
 * The cameras: behind the title an orbit of the vale; in a run og.Camera
 * follows the strider this device plays (og.Lockstep.localPlayer in a
 * room), and with two seats on one device each seat's og.CameraView
 * follows its own strider in its half of the screen.
 */
export const View = defineSystem("View", { reads: [Game, og.Transform, Hero, Tuning, og.Lockstep], writes: [og.Camera, og.CameraView, og.Transform], after: ["Hero"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const cam = must(world.resource(og.Camera));
  const t = lazyTuning(world);
  const dt = time.dt;
  let eye: Vector3;
  let focus: Vector3;
  const title = g.phase === Phase.Title;
  if (title) {
    const a = time.elapsed * 0.035 + 0.6;
    eye = { x: LAKE.x + math.sin(a) * 42, y: 17, z: LAKE.z + math.cos(a) * 42 };
    focus = { x: LAKE.x, y: 3, z: LAKE.z };
  } else {
    ({ eye, focus } = eyeOf(world, g, t, heroOf(g, localSeats(world)[0] ?? 0)));
  }
  const pos = ease(cam.position, eye, dt, g.start);
  cam.position = pos;
  cam.quaternion = og.transform.lookAt(pos, focus);
  cam.fov = 55;
  cam.far = 400;
  if (!LOCAL) return;
  // Two seats here: the halves draw the run and the main frame under them draws nothing.
  cam.layers = title ? 1 : 0;
  for (let seat = 0; seat < SEATS; seat++) {
    const v = entity(world, `view-${seat}`);
    if (title) {
      must(world.set(v, og.CameraView, { active: false }));
      continue;
    }
    const want = eyeOf(world, g, t, heroOf(g, seat));
    const at = must(world.get(v, og.Transform)).position;
    const p = ease(at, want.eye, dt, g.start || !must(world.get(v, og.CameraView)).active);
    must(world.set(v, og.Transform, { position: p, quaternion: og.transform.lookAt(p, want.focus) }));
    must(world.set(v, og.CameraView, { active: true }));
  }
});
