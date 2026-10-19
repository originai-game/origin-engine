// The hero: a capsule on og.CharacterController carrying the strider
// (skinned, run by the graph of src/anim.ts), and the camera that follows
// it. WASD or the arrows (the left stick, the touch joystick) walk
// relative to the view, Shift (a full stick) runs, Space (A, Jump) jumps,
// holding F (X, Kindle) by a beacon walks up to it and reaches into the
// bowl with the right hand (an og.IkChain on the arm) until it catches.
// Wading in the lake slows the hero and splashes; the sentinels cannot
// follow there. Q and E (the right stick, a finger dragged across the
// touch HUD's look zone) turn the view; walking swings it slowly behind
// the hero.

import { og, defineSystem, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { heroRig } from "./anim.ts";
import { Beacon, Ending, Game, Hero, Phase, Sound, Tuning } from "./components.ts";
import { BEACON_AT, ground, LAKE, must, START, WATER_Y } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

/** The capsule's centre above the feet (radius 0.3, a 0.9 m segment). */
export const FEET = 0.75;
/** The strider's scale (about 1.6 m tall). */
const BODY_SCALE = 0.92;
/** The strider's right hand: its index among the skin's joints (Hips is 0). */
export const RIGHT_HAND = 12;
/** How near a beacon's centre the hero stands to reach into its bowl. */
export const STAND = 1.12;
const UP = { x: 0, y: 1, z: 0 };

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

/** Spawns the hero at the start, its body, its reach and its splash. */
export function spawnHero(world: World): void {
  const g = must(world.resource(Game));
  const rig = heroRig(world);
  const h = ground(world, g, START.x, START.z).height;
  const hero = must(world.spawn({ name: "hero", position: { x: START.x, y: h + FEET + 0.05, z: START.z } }));
  must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 0.9 }));
  must(world.insert(hero, og.CharacterController, { maxSlope: 42, stepHeight: 0.35, snapDistance: 0.3, coyoteTime: 0.12 }));
  must(world.insert(hero, Hero));
  must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
  g.hero = hero as unknown as number;
  const body = must(world.instantiate(must(og.load.gltf(world, "models/strider.glb")), { name: "hero-body", parent: hero, position: { x: 0, y: -FEET, z: 0 }, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
  must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
  const reach = must(world.spawn({ name: "hero-reach" }));
  must(world.insert(reach, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 3, solver: "fabrik", tolerance: 0.02, space: "world", weight: 0 }));
  const splash = must(world.spawn({ name: "hero-splash", position: { x: 0, y: WATER_Y, z: 0 } }));
  must(world.insert(splash, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/splash.ogt")), seed: 5, visible: false }));
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

function splash(world: World, g: GameV, at: Vector3): void {
  must(world.set(entity(world, "hero-splash"), og.Transform, { position: { x: at.x, y: WATER_Y + 0.02, z: at.z } }));
  must(world.set(entity(world, "hero-splash"), og.ParticleSystem, { visible: true, startTick: world.tick }));
  g.sfx |= Sound.Splash;
  g.splashes += 1;
}

/** Puts the hero back at the start, facing north, the view behind it. */
function reset(world: World, g: GameV, hero: Entity): void {
  const h = ground(world, g, START.x, START.z).height;
  must(world.set(hero, og.Transform, { position: { x: START.x, y: h + FEET + 0.05, z: START.z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  must(world.set(hero, Hero, { airTime: 0, stepClock: 0 }));
  g.facing = Math.PI;
  g.yaw = 0;
  g.knock = 0;
  g.invulnerable = 0;
  g.kindling = false;
}

/**
 * Per tick: reads the controls, moves the capsule, turns the body, drives
 * the graph and the reach, and makes the steps, the splashes and the jumps
 * heard.
 */
export const HeroMove = defineSystem(
  "Hero",
  { reads: [og.Input, og.Gamepad, Tuning, Beacon], writes: [Game, Hero, og.CharacterController, og.Transform, og.AnimationGraph, og.IkChain, og.ParticleSystem, og.PostEffects], after: ["Ui"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = lazyTuning(world);
    const hero = g.hero as unknown as Entity;
    if (g.start) reset(world, g, hero);
    const inp = must(world.resource(og.Input));
    const gp = must(world.resource(og.Gamepad));
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    // Pause before the time gate: a press during a hit-stop (scaledDt 0)
    // still pauses. Not while held: the press that resumed from the menu
    // this tick (Ui runs first) would open it again.
    if (playing && !g.held && (inp.pressed("Escape") || inp.pressed("KeyP") || gp.pressed("start"))) g.paused = true;
    const dt = time.scaledDt;
    if (dt === 0) return;
    const body = entity(world, "hero-body");
    const reach = entity(world, "hero-reach");
    const cc = must(world.get(hero, og.CharacterController));
    const me = must(world.get(hero, Hero));
    const p = must(world.get(hero, og.Transform)).position;

    // The controls, relative to the view: forward is where the camera looks.
    let ix = inp.axis("KeyA", "KeyD") + inp.axis("ArrowLeft", "ArrowRight");
    let iy = inp.axis("KeyS", "KeyW") + inp.axis("ArrowDown", "ArrowUp");
    let analog = false;
    const sx = gp.axis("leftX");
    const sy = -gp.axis("leftY");
    if (Math.hypot(sx, sy) > 0.2) {
      ix += sx;
      iy += sy;
      analog = true;
    }
    if (g.stickX !== 0 || g.stickY !== 0) {
      ix += g.stickX;
      iy -= g.stickY;
      analog = true;
    }
    const mag = Math.min(1, Math.hypot(ix, iy));
    const run = analog ? mag > 0.85 : inp.down("ShiftLeft") || inp.down("ShiftRight");
    const fwd = { x: -Math.sin(g.yaw), z: -Math.cos(g.yaw) };
    const right = { x: Math.cos(g.yaw), z: -Math.sin(g.yaw) };
    let dx = 0;
    let dz = 0;
    if (mag > 0.01) {
      const n = Math.hypot(ix, iy);
      dx = (right.x * ix + fwd.x * iy) / n;
      dz = (right.z * ix + fwd.z * iy) / n;
    }

    const feet = p.y - FEET;
    g.wading = feet < WATER_Y - 0.08 && Math.hypot(p.x - LAKE.x, p.z - LAKE.z) < 18;
    const kindleHeld = playing && (inp.down("KeyF") || gp.down("x") || g.padKindle);
    g.kindling = kindleHeld && g.near >= 0;

    let speed = 0;
    let face: number | null = null;
    if (g.knock > 0) {
      const k = g.knock / 0.35;
      speed = 7 * k;
      dx = g.knockX;
      dz = g.knockZ;
      g.knock = Math.max(0, g.knock - dt);
    } else if (g.kindling) {
      // Walk up to the plinth, then stand and reach.
      const b = BEACON_AT[g.near]!;
      const vx = b.x - p.x;
      const vz = b.z - p.z;
      const d = Math.hypot(vx, vz);
      face = Math.atan2(vx, vz);
      if (d > STAND) {
        dx = vx / d;
        dz = vz / d;
        speed = t.walkSpeed;
      }
    } else if (playing && mag > 0.01) {
      speed = (run ? t.runSpeed : t.walkSpeed) * (analog && !run ? mag / 0.85 : 1);
      face = Math.atan2(dx, dz);
    }
    if (g.wading) speed *= t.wade;
    const jump = playing && !g.kindling && g.knock === 0 && (inp.down("Space") || gp.down("a") || g.padJump);
    must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed }, jump, jumpSpeed: t.jumpSpeed }));
    g.padJump = false;
    if (g.invulnerable > 0) g.invulnerable = Math.max(0, g.invulnerable - dt);
    const post = must(world.resource(og.PostEffects));
    if (post.flash > 0) post.flash = Math.max(0, post.flash - dt * 2.5);

    // The body turns towards where it goes (not while thrown back).
    if (face !== null) g.facing = turn(g.facing, face, 11 * dt);
    must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, g.facing) }));

    // The graph: speed across the ground, and whether it is in the air.
    const v = cc.velocity;
    const hs = Math.hypot(v.x, v.z);
    g.speed = hs;
    const airTime = cc.grounded ? 0 : me.airTime + dt;
    must(world.set(body, og.AnimationGraph, { p0: cc.grounded || airTime < 0.12 ? hs : 0, p1: airTime > 0.12 ? 1 : 0 }));

    // The reach: the right hand into the near bowl while kindling.
    const ik = must(world.get(reach, og.IkChain));
    let weight = ik.weight;
    const want = g.kindling ? 1 : 0;
    weight += Math.max(-dt * 5, Math.min(dt * 5, want - weight));
    if (g.near >= 0) {
      const b = BEACON_AT[g.near]!;
      const bowl = world.find(`beacon-${g.near}`);
      const top = bowl === null ? p.y : must(world.get(bowl, Beacon)).top;
      const vx = p.x - b.x;
      const vz = p.z - b.z;
      const d = Math.max(0.01, Math.hypot(vx, vz));
      // IkChain's "world" space is the frame the body's Transform is in: the hero's (unrotated) capsule.
      must(world.set(reach, og.IkChain, { weight, target: { x: b.x + (vx / d) * 0.5 - p.x, y: top + 0.05 - p.y, z: b.z + (vz / d) * 0.5 - p.z } }));
    } else if (weight !== ik.weight) must(world.set(reach, og.IkChain, { weight }));

    // Steps, splashes, jumps and landings.
    let steps = me.steps;
    let stepClock = me.stepClock;
    if (cc.grounded && hs > 0.5) {
      stepClock += hs * dt;
      const stride = hs > 4 ? 1.6 : 1.15;
      if (stepClock >= stride) {
        stepClock -= stride;
        steps += 1;
        if (g.wading) splash(world, g, p);
        else g.sfx |= Sound.Step;
      }
    }
    if (!cc.grounded && me.airTime === 0 && v.y > 1) {
      g.jumps += 1;
      g.sfx |= Sound.Jump;
    }
    if (cc.grounded && me.airTime > 0.3) {
      if (feet < WATER_Y) splash(world, g, p);
      else g.sfx |= Sound.Step;
    }
    g.grounded = cc.grounded;
    must(world.set(hero, Hero, { airTime, steps, stepClock }));
  },
);

/** How far the camera's eye stays above the ground and the water. */
const CLEARANCE = 0.6;
/** The view's pitch, down from level. */
const PITCH = 0.36;

/**
 * The camera: behind the title an orbit of the vale; in a run it follows
 * the hero, turned by Q and E (the right stick) and swung slowly behind it
 * as it walks, kept above the ground and the lake.
 */
export const View = defineSystem("View", { reads: [og.Input, og.Gamepad, og.Transform, Tuning], writes: [Game, og.Camera], after: ["Hero"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const cam = must(world.resource(og.Camera));
  const t = lazyTuning(world);
  const inp = must(world.resource(og.Input));
  const gp = must(world.resource(og.Gamepad));
  const dt = time.dt;
  let eye: Vector3;
  let focus: Vector3;
  if (g.phase === Phase.Title) {
    const a = time.elapsed * 0.035 + 0.6;
    eye = { x: LAKE.x + Math.sin(a) * 42, y: 17, z: LAKE.z + Math.cos(a) * 42 };
    focus = { x: LAKE.x, y: 3, z: LAKE.z };
  } else {
    const p = must(world.get(g.hero as unknown as Entity, og.Transform)).position;
    if (time.scaledDt > 0) {
      let turnBy = inp.axis("KeyQ", "KeyE") * 2.2 + (Math.abs(gp.axis("rightX")) > 0.2 ? gp.axis("rightX") * 2.6 : 0);
      // Swing behind a walking hero (not one walking at the camera), unless a finger holds the view.
      const behind = g.facing + Math.PI;
      if (turnBy === 0 && g.lookId < 0 && g.speed > 0.5 && Math.abs(wrap(behind - g.yaw)) < 2.2) turnBy = (wrap(behind - g.yaw) * Math.min(1, g.speed / 6)) * 0.9;
      // A finger dragging across the look zone (Ui's touch turn): dragging right turns the view right.
      g.yaw = wrap(g.yaw + turnBy * dt - g.lookTurn);
      if (g.lookTurn !== 0) g.lookTurn = 0;
    }
    const d = t.cameraDistance;
    focus = { x: p.x, y: p.y + 0.9, z: p.z };
    eye = {
      x: focus.x + Math.sin(g.yaw) * d * Math.cos(PITCH),
      y: focus.y + d * Math.sin(PITCH),
      z: focus.z + Math.cos(g.yaw) * d * Math.cos(PITCH),
    };
    const floor = Math.max(ground(world, g, eye.x, eye.z).height, WATER_Y) + CLEARANCE;
    if (eye.y < floor) eye.y = floor;
  }
  const at = cam.position;
  const far = Math.hypot(eye.x - at.x, eye.y - at.y, eye.z - at.z) > 12 || g.start;
  const k = far ? 1 : 1 - Math.exp(-dt * 9);
  const pos = { x: at.x + (eye.x - at.x) * k, y: at.y + (eye.y - at.y) * k, z: at.z + (eye.z - at.z) * k };
  cam.position = pos;
  cam.quaternion = og.transform.lookAt(pos, focus);
  cam.fov = 55;
  cam.far = 400;
});
