// The explorers: one per seat (src/seats.ts), each a capsule on
// og.CharacterController carrying a strider body (skinned, run by the graph
// of src/anim.ts), moved by its own og.PlayerInput alone. The move (WASD,
// the arrows, the left stick, the touch stick) walks relative to its view
// (src/view.ts: behind it in third person, through its eyes in first
// person, the fixed angle in isometric), Run runs, Jump jumps, and holding
// Gather by a tree, a boulder or the bridge's landing stands still and
// works it, the right hand reaching out (an og.IkChain on the arm). The
// shallows slow it and splash; deep water carries it back to the camp
// with half the bag washed away. At night its lantern glows.
//
// The Eye system then keeps each third-person camera above the ground and
// the sea, and in first person hides the body whose eyes the camera is
// behind.

import { og, defineSystem, math, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { heroRig } from "./anim.ts";
import { Ending, Game, Hero, Kind, Node, Phase, Sound, Target, Tuning } from "./components.ts";
import { CAMP, ROCKS, SEA, TREES } from "./layout.ts";
import { held, LOCAL, SEATS } from "./seats.ts";
import { creation } from "./creation.ts";
import { bodyLayer, FIRST_PERSON, ISOMETRIC, SeatLook, setupView, walkDirection } from "./view.ts";
import { BRIDGE_SITE, ground, must, nodeOf } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type HeroV = Value<typeof Hero>;

/** The capsule's centre above the feet (radius 0.3, a 0.9 m segment). */
export const FEET = 0.75;
/** The strider's scale (about 1.6 m tall). */
const BODY_SCALE = 0.92;
/** The strider's right hand: its index among the skin's joints (Hips is 0). */
const RIGHT_HAND = 12;
/** Seat 1's body tint (a cooler cloth beside seat 0's), and where it stands beside seat 0's. */
const SECOND_TINT = { r: 0.55, g: 0.78, b: 1 };
const SECOND_START = 1.6;
/** How deep the water under the feet carries an explorer away, and how deep it wades. */
const DEEP = 1.0;
const SHALLOW = 0.15;
/** How near the bridge's landing a seat builds it. */
export const BUILD_REACH = 4.5;
/** Ticks the look stays idle before a third-person view swings behind a walking explorer. */
const LOOK_HOLD = 40;
/** The isometric view's heading (the look's ArtCamera): the stick walks relative to it. */
export const ISO_YAW = (((creation.camera as { yaw?: number }).yaw ?? 45) * Math.PI) / 180;
const UP = { x: 0, y: 1, z: 0 };
const DEG = Math.PI / 180;

/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not). */
export const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

/** Tuning, read on its first use in this call (a resource read encodes every field). */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

/** Seat `seat`'s explorer (seat 0's is `hero`, seat 1's `hero-1`). */
export function heroOf(g: GameV, seat: number): Entity {
  return (seat === 0 ? g.hero : g.hero1) as unknown as Entity;
}

/** Where seat `seat`'s explorer starts and wakes: beside the camp's fire. */
export function startOf(seat: number): { x: number; z: number } {
  return { x: CAMP.x + 1.5 + seat * SECOND_START, z: CAMP.z - 3 };
}

/** The trees' and boulders' places (src/layout.ts, in Node.index order) and their kinds. */
export const NODES: readonly { x: number; z: number; kind: number }[] = [
  ...TREES.map(([x, z]) => ({ x, z, kind: Kind.Tree })),
  ...ROCKS.map(([x, z]) => ({ x, z, kind: Kind.Rock })),
];

/** Spawns an explorer per seat at the camp, each with its body, reach, lantern and splash, and gives each its view. */
export function spawnHeroes(world: World): void {
  const g = must(world.resource(Game));
  const rig = heroRig(world);
  const model = must(og.load.gltf(world, "models/strider.glb"));
  const splashFx = must(og.load.emitter(world, "fx/splash.ogt"));
  const heroes: Entity[] = [];
  for (let seat = 0; seat < SEATS; seat++) {
    const name = seat === 0 ? "hero" : `hero-${seat}`;
    const at = startOf(seat);
    const h = ground(world, g, at.x, at.z).height;
    const hero = must(world.spawn({ name, position: { x: at.x, y: h + FEET + 0.05, z: at.z } }));
    must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 0.9 }));
    must(world.insert(hero, og.CharacterController, { maxSlope: 44, stepHeight: 0.4, snapDistance: 0.35, coyoteTime: 0.12 }));
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    const body = must(world.instantiate(model, { name: `${name}-body`, parent: hero, position: { x: 0, y: -FEET, z: 0 }, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
    must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
    // In first person each seat's body is on its own layer, which its own eye skips (src/view.ts).
    must(world.set(body, og.Model, { layers: FIRST_PERSON ? bodyLayer(seat) : 1, ...(seat > 0 ? { tint: SECOND_TINT } : {}) }));
    const reach = must(world.spawn({ name: `${name}-reach` }));
    must(world.insert(reach, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 3, solver: "fabrik", tolerance: 0.02, space: "world", weight: 0 }));
    const lantern = must(world.spawn({ name: `${name}-lantern`, parent: hero, position: { x: 0, y: 3.2, z: 0 } }));
    must(world.insert(lantern, og.PointLight, { color: { r: 1, g: 0.72, b: 0.42 }, intensity: 0, distance: 16, castShadow: false }));
    const splash = must(world.spawn({ name: `${name}-splash`, position: { x: 0, y: SEA, z: 0 } }));
    must(world.insert(splash, og.ParticleSystem, { emitter: splashFx, seed: 5 + seat, visible: false }));
    must(world.insert(hero, Hero, { seat, body: body as unknown as number, reach: reach as unknown as number, lantern: lantern as unknown as number, splash: splash as unknown as number }));
    if (seat === 0) g.hero = hero as unknown as number;
    else g.hero1 = hero as unknown as number;
    heroes.push(hero);
  }
  setupView(world, heroes, { eye: 0.85, behind: 5.6, above: 1.4, isoHeight: 26 });
  // The isle is wide: every view sees to its far shore.
  if (LOCAL) {
    for (let seat = 0; seat < SEATS; seat++) {
      const v = world.find(`view-${seat}`);
      if (v !== null) must(world.set(v, og.CameraView, { far: 600 }));
    }
  } else must(world.resource(og.Camera)).far = 600;
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

/** How dark it is: 0 by day, 1 at night (an hour of dusk and of dawn between). */
export function darkness(hour: number): number {
  if (hour >= 18.5) return Math.min(1, (hour - 18.5) / 1.5);
  if (hour < 7.5) return Math.min(1, (7.5 - hour) / 1.2);
  return 0;
}

function splash(world: World, g: GameV, me: HeroV, at: Vector3): void {
  const e = me.splash as unknown as Entity;
  must(world.set(e, og.Transform, { position: { x: at.x, y: SEA + 0.02, z: at.z } }));
  must(world.set(e, og.ParticleSystem, { visible: true, startTick: world.tick }));
  g.sfx |= Sound.Splash;
}

/** Puts an explorer at its place by the camp, facing north, its view behind it. */
function reset(world: World, g: GameV, hero: Entity, me: HeroV): void {
  const at = startOf(me.seat);
  const h = ground(world, g, at.x, at.z).height;
  must(world.set(hero, og.Transform, { position: { x: at.x, y: h + FEET + 0.05, z: at.z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  me.airTime = 0;
  me.stepClock = 0;
  me.facing = Math.PI;
  me.lookIdle = LOOK_HOLD;
  me.gathering = false;
  me.target = Target.None;
  me.node = -1;
  const look = must(world.get(hero, SeatLook));
  must(world.set(hero, SeatLook, { yaw: ISOMETRIC ? ISO_YAW : 0, pitch: ISOMETRIC || FIRST_PERSON ? 0 : -0.25, seat: look.seat }));
}

/** The nearest standing tree or boulder within an explorer's reach, or -1. */
function nearestNode(world: World, t: TuneV, x: number, z: number): number {
  let best = -1;
  let bestD = 1e9;
  for (let i = 0; i < NODES.length; i++) {
    const n = NODES[i]!;
    const dx = n.x - x;
    const dz = n.z - z;
    if (Math.abs(dx) > 5 || Math.abs(dz) > 5) continue;
    const d = hypot(dx, dz);
    if (d < bestD) {
      const row = must(world.get(nodeOf(world, i), Node));
      if (row.felled || d > t.reach + row.radius) continue;
      best = i;
      bestD = d;
    }
  }
  return best;
}

/**
 * Per tick, for each explorer: reads its og.PlayerInput and its view,
 * moves the capsule, turns the body, finds and works what its Gather
 * reaches, drives the graph, the reach and the lantern, makes the steps,
 * the splashes and the jumps heard, and carries it back from deep water.
 */
export const HeroMove = defineSystem(
  "Hero",
  { reads: [og.PlayerInput, Tuning, Node], writes: [Game, Hero, SeatLook, og.CharacterController, og.Transform, og.AnimationGraph, og.IkChain, og.ParticleSystem, og.PointLight, og.PostEffects], after: ["Ui"] },
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
    if (post.flash > 0) post.flash = Math.max(0, post.flash - dt * 2);
    const dark = darkness(g.hour);
    for (const h of heroes) {
      step(world, g, t, h.hero, h.me, h, playing, dt);
      must(world.set((h.me.lantern as unknown as Entity), og.PointLight, { intensity: 12 * dark }));
      must(world.set(h.hero, Hero, h.me));
    }
    const first = heroes[0]!.me;
    g.near = first.target === Target.Node ? first.node : first.target === Target.Bridge ? -2 : -1;
    g.gathering = first.gathering;
    g.wading = first.wading;
    g.grounded = first.grounded;
    g.speed = first.speed;
  },
);

/** One explorer's tick. */
function step(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV, input: { buttons: number; x: number; y: number; aimX: number; aimY: number }, playing: boolean, dt: number): void {
  const body = me.body as unknown as Entity;
  const reach = me.reach as unknown as Entity;
  const cc = must(world.get(hero, og.CharacterController));
  const p = must(world.get(hero, og.Transform)).position;
  const look = must(world.get(hero, SeatLook));
  const buttons = input.buttons;

  // What Gather reaches: the bridge's landing (until it stands), else the nearest tree or boulder.
  me.node = -1;
  me.target = Target.None;
  if (playing) {
    if (!g.bridge && hypot(p.x - BRIDGE_SITE.x, p.z - BRIDGE_SITE.z) < BUILD_REACH) me.target = Target.Bridge;
    else {
      me.node = nearestNode(world, t, p.x, p.z);
      if (me.node >= 0) me.target = Target.Node;
    }
  }
  me.gathering = playing && me.target !== Target.None && held(buttons, "gather");

  // The move, relative to the view: forward is where its camera looks (the fixed angle in isometric).
  const mag = Math.min(1, hypot(input.x, input.y));
  const dir = walkDirection(ISOMETRIC ? ISO_YAW : look.yaw, input.x, input.y);
  let dx = 0;
  let dz = 0;
  if (mag > 0.01) {
    const n = hypot(dir.x, dir.z);
    dx = dir.x / n;
    dz = dir.z / n;
  }
  const feet = p.y - FEET;
  const depth = SEA - feet;
  me.wading = depth > SHALLOW;

  let speed = 0;
  let face: number | null = null;
  if (me.gathering) {
    // Stand and face the work.
    const at = me.target === Target.Bridge ? BRIDGE_SITE : NODES[me.node]!;
    face = math.atan2(at.x - p.x, at.z - p.z);
    dx = 0;
    dz = 0;
  } else if (playing && mag > 0.01) {
    // A stick part-way walks slower; a full one walks, and Run runs.
    speed = held(buttons, "run") ? t.runSpeed : t.walkSpeed * Math.min(1, mag / 0.85);
    face = math.atan2(dx, dz);
  }
  if (me.wading) speed *= t.wade;
  // In first person the body turns with the look; in isometric an explorer standing still turns with the aim.
  if (FIRST_PERSON) face = math.atan2(-math.sin(look.yaw), -math.cos(look.yaw));
  else if (ISOMETRIC && face === null && input.aimX !== 0) face = me.facing - input.aimX * DEG * 2;
  // Not on the tick a run starts: the Jump that set out from the title or the summary.
  const jump = playing && !g.start && !me.gathering && held(buttons, "jump");
  must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed }, jump, jumpSpeed: t.jumpSpeed }));

  if (face !== null) me.facing = turn(me.facing, face, (FIRST_PERSON ? 30 : 11) * dt);
  must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, me.facing) }));

  // The graph: speed across the ground, and whether it is in the air.
  const v = cc.velocity;
  const hs = hypot(v.x, v.z);
  me.speed = hs;
  const airTime = cc.grounded ? 0 : me.airTime + dt;
  must(world.set(body, og.AnimationGraph, { p0: cc.grounded || airTime < 0.12 ? hs : 0, p1: airTime > 0.12 ? 1 : 0 }));

  // The reach: the right hand out to the work while gathering.
  const ik = must(world.get(reach, og.IkChain));
  let weight = ik.weight;
  weight += Math.max(-dt * 5, Math.min(dt * 5, (me.gathering ? 1 : 0) - weight));
  if (me.target !== Target.None) {
    const at = me.target === Target.Bridge ? BRIDGE_SITE : NODES[me.node]!;
    const vx = at.x - p.x;
    const vz = at.z - p.z;
    const d = Math.max(0.01, hypot(vx, vz));
    const r = Math.min(0.7, d);
    // IkChain's "world" space is the frame the body's Transform is in: the explorer's (unrotated) capsule.
    must(world.set(reach, og.IkChain, { weight, target: { x: (vx / d) * r, y: 0.35, z: (vz / d) * r } }));
  } else if (weight !== ik.weight) must(world.set(reach, og.IkChain, { weight }));

  // Steps, splashes, jumps and landings.
  if (cc.grounded && hs > 0.5) {
    me.stepClock += hs * dt;
    const stride = hs > 5 ? 1.7 : 1.2;
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
    if (me.wading) splash(world, g, me, p);
    else g.sfx |= Sound.Step;
  }
  me.grounded = cc.grounded;
  me.airTime = airTime;

  // Deep water: the tide carries the explorer back to the camp, and half the bag with it.
  if (playing && depth > DEEP) {
    splash(world, g, me, p);
    const at = startOf(me.seat);
    const h = ground(world, g, at.x, at.z).height;
    must(world.set(hero, og.Transform, { position: { x: at.x, y: h + FEET + 0.05, z: at.z } }));
    must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
    g.wood -= Math.floor(g.wood / 2);
    g.stone -= Math.floor(g.stone / 2);
    g.washes += 1;
    g.sfx |= Sound.Lost;
    must(world.resource(og.PostEffects)).flash = 0.6;
  }

  // A third-person view swings slowly behind a walking explorer (not one
  // walking at the camera) once its look has rested a moment.
  if (FIRST_PERSON || ISOMETRIC || g.phase === Phase.Title) return;
  const looked = input.aimX !== 0 || input.aimY !== 0;
  me.lookIdle = looked ? 0 : Math.min(LOOK_HOLD, me.lookIdle + 1);
  const behind = wrap(me.facing + Math.PI);
  if (!looked && me.lookIdle >= LOOK_HOLD && me.speed > 0.5 && Math.abs(wrap(behind - look.yaw)) < 2.2) {
    must(world.set(hero, SeatLook, { yaw: turn(look.yaw, behind, Math.min(1, me.speed / 6) * 1.1 * dt) }));
  }
}

/** How far a third-person camera's eye stays above the ground and the sea. */
const CLEARANCE = 0.7;
/** Behind the title, the third-person camera stands back over the isle: this far out and up. */
const TITLE_BACK = 6;
const TITLE_UP = 24;

/**
 * Keeps each third-person camera above the ground and the sea (the
 * controllers' offsets, and with two seats here each half's view), and on
 * the title stands it back and up so the orbit shows the isle and the sea.
 * List it after ViewRig and before og.systems.cameras.
 */
export const Eye = defineSystem("Eye", { reads: [Game, Hero], writes: [og.FollowCamera, og.Transform], after: ["ViewRig"] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  if (FIRST_PERSON || ISOMETRIC) return;
  const title = g.phase === Phase.Title;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const p = must(world.get(hero, og.Transform)).position;
    if (LOCAL) {
      const v = world.find(`view-${seat}`);
      if (v === null) continue;
      let eye = must(world.get(v, og.Transform)).position;
      if (title) eye = { x: p.x + (eye.x - p.x) * TITLE_BACK, y: p.y + TITLE_UP, z: p.z + (eye.z - p.z) * TITLE_BACK };
      const floor = Math.max(ground(world, g, eye.x, eye.z).height, SEA) + CLEARANCE;
      if (eye.y >= floor && !title) continue;
      const raised = { x: eye.x, y: Math.max(eye.y, floor), z: eye.z };
      must(world.set(v, og.Transform, { position: raised, quaternion: og.transform.lookAt(raised, { x: p.x, y: p.y + 0.85, z: p.z }) }));
    } else {
      const fc = must(world.get(hero, og.FollowCamera));
      const off = title ? { x: fc.offset.x * TITLE_BACK, y: TITLE_UP, z: fc.offset.z * TITLE_BACK } : fc.offset;
      const eye = { x: p.x + off.x, y: p.y + off.y, z: p.z + off.z };
      const floor = Math.max(ground(world, g, eye.x, eye.z).height, SEA) + CLEARANCE;
      if (eye.y < floor || title) must(world.set(hero, og.FollowCamera, { offset: { x: off.x, y: off.y + Math.max(0, floor - eye.y), z: off.z } }));
    }
  }
});
