// The troopers: one per seat (src/seats.ts), each a capsule on
// og.CharacterController carrying a strider body (skinned, its walk and run
// blended by speed, src/anim.ts) and the pulse rifle, moved by its own
// og.PlayerInput alone. The view is src/view.ts's, the same in every 3D
// starter: its SeatLook (yaw, pitch) is world state, turned by the seat's
// aim, and the creation's view picks the rig:
//   first-person-3d  the eye at the trooper's head, the rifle in front of
//                    it (the hands, drawn only in that seat's own view);
//   third-person-3d  over the right shoulder (Shoulder below moves the
//                    view's follow camera aside and pulls it in front of a
//                    wall), the rifle held at the shoulder by two IK arms;
//   isometric-2.5d   twin-stick: the move is relative to the screen and
//                    the look turns the trooper, who fires where it faces.
// The body always faces the look, so the others see where a trooper aims.

import { og, defineSystem, math, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Quaternion, Value, Vector3, World } from "@origingame/engine";
import { heroRig } from "./anim.ts";
import { creation } from "./creation.ts";
import { Ending, Game, Hero, Phase, Sound, Tuning } from "./components.ts";
import { held, LOCAL, localSeats, SEATS } from "./seats.ts";
import { bodyLayer, FIRST_PERSON, ISOMETRIC, SeatLook, setupView, walkDirection } from "./view.ts";
import { entity, must, point, qmul, yawQ } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type HeroV = Value<typeof Hero>;

export const THIRD_PERSON = !FIRST_PERSON && !ISOMETRIC;
/** The capsule's centre above the feet (radius 0.32, a 1 m segment: 1.64 m tall). */
export const FEET = 0.82;
/** The eye above the capsule's centre (1.6 m above the floor). */
export const EYE = 0.78;
/** Where the rounds leave in isometric (the rifle's height above the capsule's centre). */
const HIP = 0.3;
/** The third-person camera: behind, above, and how far right of the trooper's head it sits. */
const BEHIND = 3.1;
const ABOVE = 1.2;
export const SHOULDER = 0.62;
/** The isometric view's heading (the look's ArtCamera yaw): the screen's up on the floor. */
export const ISO_YAW = ((creation.camera as { yaw?: number }).yaw ?? 45) * (Math.PI / 180);
const BODY_SCALE = 0.94;
/** The strider's hands among its skin's joints (Hips is 0). */
const RIGHT_HAND = 12;
const LEFT_HAND = 8;
/** The first-person rifle's size in the view (the held rifle is full size). */
const HANDS_SCALE = 0.72;
/** Seat 1's trooper: its body's tint beside seat 0's. */
const TINTS = [{ r: 1, g: 0.86, b: 0.72 }, { r: 0.6, g: 0.8, b: 1 }];

/**
 * Layers (og.Mesh/Model.layers, og.Camera/CameraView.layers): the arena on
 * 1, each seat's body on its own bit, and each seat's first-person hands on
 * another, so a seat's own view leaves out its own body and draws its hands.
 */
export const LAYER = { world: 1, body: bodyLayer, hands: (seat: number) => 8 << seat } as const;

/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not). */
export const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

/** Tuning, read on its first use in this call (a resource read encodes every field). */
export function lazyTuning(world: World): TuneV {
  let v: TuneV | null = null;
  return new Proxy({} as TuneV, { get: (_, k) => ((v ??= must(world.resource(Tuning))) as unknown as Record<string | symbol, unknown>)[k] });
}

/** Seat `seat`'s trooper (seat 0's is `hero`, seat 1's `hero-1`). */
export function heroOf(g: GameV, seat: number): Entity {
  return (seat === 0 ? g.hero : g.hero1) as unknown as Entity;
}

/** Where seat `seat` starts a holdout (the scene's spawn-<seat>). */
export function spawnOf(world: World, seat: number): Vector3 {
  const p = point(world, `spawn-${seat}`);
  return { x: p.x, y: p.y + FEET + 0.05, z: p.z };
}

/** The trooper's capsule centre, view and look. */
export interface Aim {
  readonly at: Vector3;
  readonly yaw: number;
  readonly pitch: number;
}

/** Spawns a trooper per seat: the capsule, the body, the rifle (held and in the hands), the arms' IK and the muzzle's light. */
export function spawnHeroes(world: World): void {
  const g = must(world.resource(Game));
  const rig = heroRig(world);
  const model = must(og.load.gltf(world, "models/strider.glb"));
  const rifle = must(og.load.gltf(world, "models/blaster.glb"));
  const heroes: Entity[] = [];
  for (let seat = 0; seat < SEATS; seat++) {
    const name = seat === 0 ? "hero" : `hero-${seat}`;
    const hero = must(world.spawn({ name, position: spawnOf(world, seat) }));
    must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.32, height: 1 }));
    must(world.insert(hero, og.CharacterController, { maxSlope: 42, stepHeight: 0.35, snapDistance: 0.3, coyoteTime: 0.1 }));
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    const body = must(world.instantiate(model, { name: `${name}-body`, parent: hero, position: { x: 0, y: -FEET, z: 0 }, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
    must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
    must(world.set(body, og.Model, { tint: TINTS[seat]!, layers: LAYER.body(seat) }));
    const gun = must(world.instantiate(rifle, { name: `${name}-gun`, parent: hero }));
    must(world.set(gun, og.Model, { layers: LAYER.body(seat) }));
    const hands = must(world.instantiate(rifle, { name: `${name}-hands` }));
    must(world.set(hands, og.Model, { layers: LAYER.hands(seat), castShadow: false, visible: FIRST_PERSON }));
    const flash = must(world.spawn({ name: `${name}-muzzle` }));
    must(world.insert(flash, og.PointLight, { color: { r: 1, g: 0.72, b: 0.4 }, intensity: 0, distance: 7, castShadow: false }));
    const armR = must(world.spawn({ name: `${name}-arm-r` }));
    must(world.insert(armR, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 3, solver: "fabrik", tolerance: 0.01, space: "world", weight: 1 }));
    const armL = must(world.spawn({ name: `${name}-arm-l` }));
    must(world.insert(armL, og.IkChain, { skeleton: body, tip: LEFT_HAND, bones: 3, solver: "fabrik", tolerance: 0.01, space: "world", weight: 1 }));
    const t = must(world.resource(Tuning));
    must(world.insert(hero, Hero, {
      seat, body: body as unknown as number, gun: gun as unknown as number, hands: hands as unknown as number, muzzle: flash as unknown as number,
      armR: armR as unknown as number, armL: armL as unknown as number, health: t.health, mag: t.magazine, reserve: t.reserve, facing: Math.PI,
    }));
    if (seat === 0) g.hero = hero as unknown as number;
    else g.hero1 = hero as unknown as number;
    heroes.push(hero);
  }
  setupView(world, heroes, { eye: EYE, behind: BEHIND, above: ABOVE, isoHeight: 17, maxPitch: FIRST_PERSON ? 1.4 : 0.5, minPitch: FIRST_PERSON ? -1.4 : -0.9 });
  // The rounds come from a seat's eye along its look: the rifle's own capsule and body never stop them.
  if (ISOMETRIC) for (const hero of heroes) must(world.set(hero, SeatLook, { yaw: ISO_YAW }));
}

/** Seat `seat`'s capsule centre and look. */
export function aimOf(world: World, hero: Entity): Aim {
  const look = must(world.get(hero, SeatLook));
  return { at: must(world.get(hero, og.Transform)).position, yaw: look.yaw, pitch: look.pitch };
}

/** The look's rotation: yaw about +y, then pitch about the right axis (-z ahead). */
export function lookQ(yaw: number, pitch: number): Quaternion {
  const sp = math.sin(pitch / 2), cp = math.cos(pitch / 2);
  return qmul(yawQ(yaw), { x: sp, y: 0, z: 0, w: cp });
}

/** A vector turned by a quaternion. */
export function rotate(q: Quaternion, v: Vector3): Vector3 {
  const tx = 2 * (q.y * v.z - q.z * v.y), ty = 2 * (q.z * v.x - q.x * v.z), tz = 2 * (q.x * v.y - q.y * v.x);
  return { x: v.x + q.w * tx + (q.y * tz - q.z * ty), y: v.y + q.w * ty + (q.z * tx - q.x * tz), z: v.z + q.w * tz + (q.x * ty - q.y * tx) };
}

/**
 * Where a seat's rounds start and go: from its eye along its look (first
 * person), from its third-person camera through the crosshair (the camera
 * Shoulder places), or level from its hip where it faces (isometric).
 */
export function rayOf(world: World, hero: Entity): { origin: Vector3; dir: Vector3 } {
  const { at, yaw, pitch } = aimOf(world, hero);
  if (ISOMETRIC) return { origin: { x: at.x, y: at.y + HIP, z: at.z }, dir: { x: -math.sin(yaw), y: 0, z: -math.cos(yaw) } };
  if (FIRST_PERSON) {
    const cp = math.cos(pitch);
    return { origin: { x: at.x, y: at.y + EYE, z: at.z }, dir: { x: -math.sin(yaw) * cp, y: math.sin(pitch), z: -math.cos(yaw) * cp } };
  }
  const cam = must(world.get(hero, og.FollowCamera));
  const eye = { x: at.x + cam.offset.x, y: at.y + cam.offset.y, z: at.z + cam.offset.z };
  const to = { x: at.x + cam.lookOffset.x, y: at.y + cam.lookOffset.y, z: at.z + cam.lookOffset.z };
  const d = { x: to.x - eye.x, y: to.y - eye.y, z: to.z - eye.z };
  const n = Math.sqrt(d.x * d.x + d.y * d.y + d.z * d.z) || 1;
  // Start the ray at the trooper's head, not the camera, so cover behind it never stops a round.
  const dir = { x: d.x / n, y: d.y / n, z: d.z / n };
  const back = (to.x - eye.x) * dir.x + (to.y - eye.y) * dir.y + (to.z - eye.z) * dir.z;
  return { origin: { x: eye.x + dir.x * back, y: eye.y + dir.y * back, z: eye.z + dir.z * back }, dir };
}

/** The rifle's pose relative to the capsule: at the right shoulder, along the look. */
function gunPose(yaw: number, pitch: number, kick: number, reload: number): { position: Vector3; quaternion: Quaternion } {
  const p = ISOMETRIC ? 0 : pitch;
  const q = lookQ(yaw, p + kick * 0.6 - reload * 0.5);
  const shoulder = rotate(yawQ(yaw), { x: 0.16, y: 0.42, z: -0.12 });
  const along = rotate(q, { x: 0, y: 0, z: 0.05 * kick });
  return { position: { x: shoulder.x + along.x, y: shoulder.y + along.y, z: shoulder.z + along.z }, quaternion: q };
}

/** The rifle's muzzle in world space, as the others see it (the flash's light, the tracer's start). */
export function muzzleOf(world: World, hero: Entity): Vector3 {
  const me = must(world.get(hero, Hero));
  const { at, yaw, pitch } = aimOf(world, hero);
  const pose = gunPose(yaw, pitch, 0, 0);
  const tip = rotate(pose.quaternion, { x: 0, y: 0.04, z: -0.64 });
  if (FIRST_PERSON && !me.down) {
    // The first-person rifle's muzzle, a little right of and below the eye.
    const q = lookQ(yaw, pitch);
    const m = rotate(q, { x: 0.16, y: -0.17, z: -0.86 });
    return { x: at.x + m.x, y: at.y + EYE + m.y, z: at.z + m.z };
  }
  return { x: at.x + pose.position.x + tip.x, y: at.y + pose.position.y + tip.y, z: at.z + pose.position.z + tip.z };
}

function wrap(a: number): number {
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

/** Puts a trooper back at its start (a new run, the title), whole, its view ahead. */
export function resetHero(world: World, hero: Entity, me: HeroV, t: TuneV, at: Vector3): void {
  must(world.set(hero, og.Transform, { position: at }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  must(world.set(hero, SeatLook, { yaw: ISOMETRIC ? ISO_YAW : 0, pitch: 0 }));
  me.health = t.health;
  me.down = false;
  me.respawn = 0;
  me.mag = t.magazine;
  me.reserve = t.reserve;
  me.reload = 0;
  me.cooldown = 0;
  me.kick = 0;
  me.bloom = 0;
  me.facing = ISOMETRIC ? ISO_YAW + Math.PI : Math.PI;
  me.airTime = 0;
  me.stepClock = 0;
}

/**
 * Per tick, for each trooper: reads its og.PlayerInput, moves the capsule
 * (relative to its look, or to the screen in isometric), jumps, turns the
 * body to the look, drives the graph, poses the rifle and the arms, and
 * makes the steps and the landings heard. Weapon (src/weapon.ts) fires.
 */
export const HeroMove = defineSystem(
  "Hero",
  { reads: [og.PlayerInput, Tuning, SeatLook], writes: [Game, Hero, og.CharacterController, og.Transform, og.AnimationGraph, og.IkChain, og.Model], after: ["Ui"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = lazyTuning(world);
    const dt = time.scaledDt;
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      const input = must(world.get(hero, og.PlayerInput));
      if (dt > 0) step(world, g, t, hero, me, input, playing && !me.down, dt);
      pose(world, hero, me, g.phase === Phase.Play);
      must(world.set(hero, Hero, me));
    }
  },
);

function step(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV, input: { x: number; y: number; buttons: number }, live: boolean, dt: number): void {
  const look = must(world.get(hero, SeatLook));
  const cc = must(world.get(hero, og.CharacterController));
  const mag = Math.min(1, hypot(input.x, input.y));
  const dir = live && mag > 0.01 ? walkDirection(ISOMETRIC ? ISO_YAW : look.yaw, input.x, input.y) : { x: 0, z: 0 };
  // Firing or reloading slows a trooper a little; a part-way stick walks.
  const speed = t.moveSpeed * (me.reload > 0 || me.firing < 0.12 ? 0.82 : 1);
  const jump = live && !g.start && held(input.buttons, "jump");
  must(world.set(hero, og.CharacterController, { move: { x: dir.x * speed, y: 0, z: dir.z * speed }, jump, jumpSpeed: t.jumpSpeed }));
  // The body faces the look (+z is the body's front, the look's ahead is -z).
  me.facing = wrap(look.yaw + Math.PI);
  const v = cc.velocity;
  const hs = hypot(v.x, v.z);
  me.speed = hs;
  const airTime = cc.grounded ? 0 : me.airTime + dt;
  must(world.set(entity_(me.body), og.AnimationGraph, { p0: me.down ? 0 : cc.grounded || airTime < 0.12 ? hs : 0, p1: airTime > 0.12 ? 1 : 0 }));
  if (cc.grounded && hs > 0.5) {
    me.stepClock += hs * dt;
    if (me.stepClock >= 1.5) {
      me.stepClock -= 1.5;
      g.sfx |= Sound.Step;
    }
  }
  if (!cc.grounded && me.airTime === 0 && v.y > 1) {
    me.jumps += 1;
    g.sfx |= Sound.Jump;
  }
  if (cc.grounded && me.airTime > 0.35) g.sfx |= Sound.Land;
  me.grounded = cc.grounded;
  me.airTime = airTime;
}

const entity_ = (n: number): Entity => n as unknown as Entity;

/** The body's heading and fall, the rifle at the shoulder, the arms on its grips, the first-person hands. */
function pose(world: World, hero: Entity, me: HeroV, inRun: boolean): void {
  const look = must(world.get(hero, SeatLook));
  const down = me.down;
  const turn = yawQ(me.facing);
  must(world.set(entity_(me.body), og.Transform, { quaternion: down ? qmul(turn, { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 }) : turn, position: { x: 0, y: down ? -FEET + 0.2 : -FEET, z: 0 } }));
  must(world.set(entity_(me.body), og.Model, { flash: Math.max(0, 1 - (world.tick - me.hurtTick) / 8) * 0.8, flashColor: { r: 1, g: 0.2, b: 0.1 } }));
  const reload = me.reload > 0 ? math.sin(Math.min(1, me.reload / 1.35) * Math.PI) : 0;
  const gun = gunPose(look.yaw, look.pitch, me.kick * 8, reload);
  must(world.set(entity_(me.gun), og.Transform, gun));
  must(world.set(entity_(me.gun), og.Model, { visible: !down }));
  // IkChain's "world" space is the frame the body's Transform is in: the trooper's (unturned) capsule.
  const grip = rotate(gun.quaternion, { x: 0, y: -0.06, z: 0.02 });
  const fore = rotate(gun.quaternion, { x: 0, y: -0.03, z: -0.32 });
  const w = down ? 0 : 1;
  must(world.set(entity_(me.armR), og.IkChain, { weight: w, target: { x: gun.position.x + grip.x, y: gun.position.y + grip.y, z: gun.position.z + grip.z } }));
  must(world.set(entity_(me.armL), og.IkChain, { weight: w, target: { x: gun.position.x + fore.x, y: gun.position.y + fore.y, z: gun.position.z + fore.z } }));
  if (FIRST_PERSON) {
    // The hands: the rifle, at HANDS_SCALE, right of and below the eye, kicked back by the recoil and dipped by a reload (only in a run: the title shows the arena).
    const at = must(world.get(hero, og.Transform)).position;
    const q = lookQ(look.yaw, look.pitch + me.kick * 2 - reload * 0.7);
    const off = rotate(lookQ(look.yaw, look.pitch), { x: 0.16, y: -0.2 - reload * 0.08, z: -0.4 + me.kick * 1.2 });
    must(world.set(entity_(me.hands), og.Transform, { position: { x: at.x + off.x, y: at.y + EYE + off.y, z: at.z + off.z }, quaternion: q, scale: { x: HANDS_SCALE, y: HANDS_SCALE, z: HANDS_SCALE } }));
    must(world.set(entity_(me.hands), og.Model, { visible: !down && inRun }));
  }
}

/**
 * Over the shoulder (third person): moves each seat's follow camera, and
 * with two seats here its half's view, to the right of the head by
 * SHOULDER, and pulls it in front of a wall between it and the head. Runs
 * after src/view.ts's ViewRig, which places it behind the head.
 */
export const Shoulder = defineSystem("Shoulder", { reads: [Game, SeatLook, Hero], writes: [og.FollowCamera, og.Transform], after: ["ViewRig"] }, (_q, _time, world) => {
  if (!THIRD_PERSON) return;
  const g = must(world.resource(Game));
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const { at, yaw } = aimOf(world, hero);
    const cam = must(world.get(hero, og.FollowCamera));
    const right = { x: math.cos(yaw), z: -math.sin(yaw) };
    const head = { x: right.x * SHOULDER, y: EYE, z: right.z * SHOULDER };
    let off = { x: cam.offset.x + head.x, y: cam.offset.y, z: cam.offset.z + head.z };
    // A wall between the head and the camera: the camera comes in to just short of it.
    const from = { x: at.x + head.x, y: at.y + head.y, z: at.z + head.z };
    const d = { x: off.x - head.x, y: off.y - head.y, z: off.z - head.z };
    const len = Math.sqrt(d.x * d.x + d.y * d.y + d.z * d.z);
    const hit = len > 0.01 ? og.physics.raycast(world, { origin: from, direction: d, maxDistance: len + 0.3, filter: { exclude: hero } }) : null;
    if (hit && hit.ok && hit.value !== null) {
      const k = Math.max(0.15, (hit.value.distance - 0.3) / len);
      off = { x: head.x + d.x * k, y: head.y + d.y * k, z: head.z + d.z * k };
    }
    must(world.set(hero, og.FollowCamera, { offset: off, lookOffset: head }));
    if (!LOCAL) continue;
    const view = world.find(`view-${seat}`);
    if (view === null) continue;
    const eye = { x: at.x + off.x, y: at.y + off.y, z: at.z + off.z };
    must(world.set(view, og.Transform, { position: eye, quaternion: og.transform.lookAt(eye, { x: at.x + head.x, y: at.y + head.y, z: at.z + head.z }) }));
  }
});

/**
 * Which layers each camera draws: the arena, every body but its own seat's
 * in first person (every body otherwise), and its own seat's hands. The
 * main camera follows this device's seat; with two seats here each half's
 * view its own.
 */
export function cameraLayers(world: World): void {
  const all = LAYER.world | LAYER.body(0) | LAYER.body(1);
  const of = (seat: number) => (FIRST_PERSON ? (all & ~LAYER.body(seat)) | LAYER.hands(seat) : all);
  const cam = must(world.resource(og.Camera));
  if (LOCAL) {
    for (let seat = 0; seat < SEATS; seat++) {
      const view = world.find(`view-${seat}`);
      if (view !== null) must(world.set(view, og.CameraView, { layers: of(seat) }));
    }
    return;
  }
  cam.layers = of(localSeats(world)[0] ?? 0);
}

/** A seat's trooper as the probe's facts see it. */
export function heroFacts(world: World, g: GameV, seat: number): Record<string, number | boolean> {
  const hero = heroOf(g, seat);
  const me = must(world.get(hero, Hero));
  const p = must(world.get(hero, og.Transform)).position;
  const look = must(world.get(hero, SeatLook));
  const r = (n: number) => Math.round(n * 100) / 100;
  return { health: r(me.health), down: me.down, mag: me.mag, reserve: me.reserve, reloading: me.reload > 0, kills: me.kills, frags: me.frags, x: r(p.x), y: r(p.y), z: r(p.z), yaw: r(look.yaw), pitch: r(look.pitch), speed: r(me.speed), grounded: me.grounded, jumps: me.jumps };
}

export { entity };
