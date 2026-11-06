// The foes and the waves. Walkers are the stout, its clips retargeted from
// the library (src/anim.ts), in dark plate with a burning visor; drones are
// the faceted hover drone (assets/models/drone.glb). Both are pooled at
// start (WALKERS, DRONES) and drawn through the gates by the waves, each an
// og.NavAgent on the arena's navmesh (og.Crowd keeps a pack from pushing
// through itself), hunting the nearest standing trooper:
//   a walker closes in, winds up (its visor flares, a swipe sounds) and
//   strikes whoever is still within reach;
//   a drone hovers 1.9 m up, keeps its range, and with a clear line charges
//   its eye (a red aiming beam that locks a beat before it fires) and fires
//   a bolt: step behind cover or aside before it locks.
// A round's hit flashes the foe; downed, it bursts in sparks and smoke and
// goes back to its pool. The Waves system runs the holdout (a breather, the
// wave through the gates, cleared: a bonus, health and ammunition back, a
// downed partner back on its feet; the fifth cleared holds the arena) and
// the duel's respawns.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { walkerRig } from "./anim.ts";
import { DRONES, Ending, Foe, FoeState, Game, Hero, Kind, Mode, Phase, Sound, Tuning, WALKERS, WAVES, WaveState } from "./components.ts";
import { EYE, FEET, heroOf, hypot, lazyTuning, resetHero, spawnOf } from "./hero.ts";
import { SEATS } from "./seats.ts";
import { SeatLook } from "./view.ts";
import { burst, entity, GATES, must, point, POPS, yawQ } from "./world.ts";

type GameV = Value<typeof Game>;
type FoeV = Value<typeof Foe>;
type TuneV = Value<typeof Tuning>;

/** The walker's body: its scale (2 m tall), its capsule for the rounds, its head. */
const WALKER_SCALE = 1.12;
export const WALKER = { radius: 0.42, low: 0.3, high: 1.5, head: 1.78, headRadius: 0.25 } as const;
/** A drone's hover height and its body's radius for the rounds. */
export const DRONE = { hover: 1.9, radius: 0.5 } as const;
/** A walker strikes within this of its trooper, after its windup; a drone's charge, the part of it its aim still follows. */
const REACH = 1.75;
const STRIKE = 2.25;
const WINDUP = 0.5;
const CHARGE = 0.95;
const TRACK = 0.6;
/** Points: a walker, a drone, a headshot's bonus, a wave's bonus a wave number. */
export const POINTS = { walker: 100, drone: 150, headshot: 50, wave: 250 } as const;

/** The foes each wave sends through the gates, in order (W a walker, D a drone); two seats get half again. */
const WAVE_FOES = ["WWWWW", "WWWWWWW", "WWDWWDWW", "WWDWWDWWDW", "WWDWWDWWDWWDW"];

export function foeName(kind: number, i: number): string {
  return `${kind === Kind.Walker ? "walker" : "drone"}-${i}`;
}

const PARKED = { x: 0, y: -40, z: 0 };
const ent = (n: number): Entity => n as unknown as Entity;

/** Spawns every pooled foe, parked out of the arena. */
export function spawnFoes(world: World): void {
  const g = must(world.resource(Game));
  const rig = walkerRig(world);
  const stout = must(og.load.gltf(world, "models/stout.glb"));
  const drone = must(og.load.gltf(world, "models/drone.glb"));
  const visorGeo = must(og.render.boxGeometry(world, { width: 0.36, height: 0.09, depth: 0.08 }));
  const visor = must(og.render.standardMaterial(world, { color: { r: 0.3, g: 0.05, b: 0.02 }, emissive: { r: 1, g: 0.3, b: 0.05 }, emissiveIntensity: 5 }));
  const nav = g.nav as never;
  for (let i = 0; i < WALKERS; i++) {
    const e = must(world.spawn({ name: foeName(Kind.Walker, i), position: PARKED }));
    must(world.insert(e, og.NavAgent, { nav, target: PARKED, seeking: false, drive: false, speed: 2.8, acceleration: 14, radius: 0.45, stopDistance: 1.2, slowRadius: 0.8, faceMovement: true, turnSpeed: 7 }));
    must(world.insert(e, og.Crowd, {}));
    const body = must(world.instantiate(stout, { name: `${foeName(Kind.Walker, i)}-body`, parent: e, scale: { x: WALKER_SCALE, y: WALKER_SCALE, z: WALKER_SCALE } }));
    must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(body, og.AnimationGraph, { asset: rig.graph, speed: 1 }));
    must(world.set(body, og.Model, { tint: { r: 0.34, g: 0.36, b: 0.42 }, visible: false }));
    const v = must(world.spawn({ name: `${foeName(Kind.Walker, i)}-visor`, parent: e, position: { x: 0, y: WALKER.head - 0.02, z: 0.26 } }));
    must(world.insert(v, og.Mesh, { geometry: visorGeo, material: visor, castShadow: false, visible: false }));
    const eye = must(world.spawn({ name: `${foeName(Kind.Walker, i)}-eye`, parent: e, position: { x: 0, y: WALKER.head, z: 0.45 } }));
    must(world.insert(eye, og.PointLight, { color: { r: 1, g: 0.35, b: 0.08 }, intensity: 0, distance: 4, castShadow: false }));
    must(world.insert(e, Foe, { index: i, kind: Kind.Walker, body: body as unknown as number, eye: eye as unknown as number, beam: v as unknown as number }));
  }
  for (let i = 0; i < DRONES; i++) {
    const e = must(world.spawn({ name: foeName(Kind.Drone, i), position: PARKED }));
    must(world.insert(e, og.NavAgent, { nav, target: PARKED, seeking: false, drive: false, speed: 3.6, acceleration: 10, radius: 0.5, stopDistance: 0.6, slowRadius: 1.5, faceMovement: true, turnSpeed: 5 }));
    must(world.insert(e, og.Crowd, {}));
    const body = must(world.instantiate(drone, { name: `${foeName(Kind.Drone, i)}-body`, parent: e, position: { x: 0, y: DRONE.hover, z: 0 } }));
    must(world.set(body, og.Model, { visible: false }));
    const eye = must(world.spawn({ name: `${foeName(Kind.Drone, i)}-eye`, parent: e, position: { x: 0, y: DRONE.hover, z: 0.55 } }));
    must(world.insert(eye, og.PointLight, { color: { r: 1, g: 0.15, b: 0.05 }, intensity: 0, distance: 5, castShadow: false }));
    const beam = must(world.spawn({ name: `${foeName(Kind.Drone, i)}-beam`, position: PARKED }));
    must(world.insert(beam, og.Beam, { end: PARKED, width: 0.025, color: { r: 4, g: 0.3, b: 0.15 }, opacity: 0, blend: "additive" }));
    must(world.insert(e, Foe, { index: i, kind: Kind.Drone, body: body as unknown as number, eye: eye as unknown as number, beam: beam as unknown as number, bob: i * 1.7 }));
  }
}

/** Every foe entity, walkers first. */
export function foes(world: World): Entity[] {
  const out: Entity[] = [];
  for (let i = 0; i < WALKERS; i++) out.push(entity(world, foeName(Kind.Walker, i)));
  for (let i = 0; i < DRONES; i++) out.push(entity(world, foeName(Kind.Drone, i)));
  return out;
}

/** A foe back in its pool: parked, still and unseen. */
function park(world: World, e: Entity, f: FoeV): void {
  f.state = FoeState.Pooled;
  f.health = 0;
  f.timer = 0;
  f.flash = 0;
  must(world.set(e, og.Transform, { position: PARKED }));
  must(world.set(e, og.NavAgent, { target: PARKED, seeking: false, drive: false }));
  must(world.set(ent(f.body), og.Model, { visible: false, flash: 0 }));
  must(world.set(ent(f.body), og.Transform, { quaternion: { x: 0, y: 0, z: 0, w: 1 }, scale: f.kind === Kind.Walker ? { x: WALKER_SCALE, y: WALKER_SCALE, z: WALKER_SCALE } : { x: 1, y: 1, z: 1 }, position: { x: 0, y: f.kind === Kind.Drone ? DRONE.hover : 0, z: 0 } }));
  must(world.set(ent(f.eye), og.PointLight, { intensity: 0 }));
  if (f.kind === Kind.Walker) must(world.set(ent(f.beam), og.Mesh, { visible: false }));
  else must(world.set(ent(f.beam), og.Beam, { opacity: 0 }));
}

/** Parks every foe (a new run, the title). */
export function parkFoes(world: World): void {
  for (const e of foes(world)) {
    const f = must(world.get(e, Foe));
    park(world, e, f);
    must(world.set(e, Foe, f));
  }
}

/** A pooled foe of a kind through gate `gate`, at full health. False when its pool is all out. */
function bring(world: World, g: GameV, t: TuneV, kind: number, gate: number): boolean {
  const e = bringAt(world, g, t, kind, point(world, `gate-${gate}`));
  if (e === null) return false;
  const on = must(world.get(e, og.Transform)).position;
  burst(world, "gatefx", gate, { x: on.x, y: on.y + 0.05, z: on.z });
  return true;
}

/** A pooled foe of a kind on the navmesh at `at`, at full health (the `spawn` command, and a gate's foe). Null when its pool is all out. */
export function bringAt(world: World, g: GameV, t: TuneV, kind: number, at: Vector3): Entity | null {
  const n = kind === Kind.Walker ? WALKERS : DRONES;
  for (let i = 0; i < n; i++) {
    const e = entity(world, foeName(kind, i));
    const f = must(world.get(e, Foe));
    if (f.state !== FoeState.Pooled) continue;
    const on = og.nav.nearest(world, g.nav as never, at) ?? at;
    must(world.set(e, og.Transform, { position: on, quaternion: yawQ(math.atan2(-on.x, -on.z)) }));
    must(world.set(e, og.NavAgent, { target: on, seeking: false, drive: true, speed: kind === Kind.Walker ? t.walkerSpeed + 0.2 * (g.wave - 1) : 3.6 }));
    f.state = FoeState.Advance;
    f.health = kind === Kind.Walker ? t.walkerHealth : t.droneHealth;
    f.timer = 0;
    f.flash = 0;
    must(world.set(ent(f.body), og.Model, { visible: true }));
    if (kind === Kind.Walker) must(world.set(ent(f.beam), og.Mesh, { visible: true }));
    must(world.set(ent(f.eye), og.PointLight, { intensity: kind === Kind.Walker ? 1.5 : 1 }));
    must(world.set(e, Foe, f));
    return e;
  }
  return null;
}

/** The nearest standing trooper to a point, and how far. */
function nearestHero(world: World, g: GameV, at: Vector3): { seat: number; pos: Vector3; dist: number } | null {
  let best: { seat: number; pos: Vector3; dist: number } | null = null;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (must(world.get(hero, Hero)).down) continue;
    const pos = must(world.get(hero, og.Transform)).position;
    const dist = hypot(pos.x - at.x, pos.z - at.z);
    if (best === null || dist < best.dist) best = { seat, pos, dist };
  }
  return best;
}

/**
 * A trooper takes `amount` from a point (a foe's strike or bolt, the other
 * duellist's round): the damage arc and the flash, and at 0 it goes down
 * (answered true: a duel's shooter counts the frag, src/weapon.ts). A
 * holdout is overrun when every trooper is down; a duel's downed trooper is
 * back after a beat.
 */
export function hurtHero(world: World, g: GameV, seat: number, amount: number, from: Vector3): boolean {
  const hero = heroOf(g, seat);
  const me = must(world.get(hero, Hero));
  if (me.down || g.ending !== Ending.None) return false;
  me.health = Math.max(0, me.health - amount);
  me.hurtTick = world.tick;
  me.hurtX = from.x;
  me.hurtZ = from.z;
  g.sfx |= Sound.Hurt;
  if (me.health > 0) {
    must(world.set(hero, Hero, me));
    return false;
  }
  me.down = true;
  me.respawn = 2.5;
  g.downs += 1;
  g.sfx |= Sound.Down;
  must(world.set(hero, Hero, me));
  if (g.mode === Mode.Duel) return true;
  let standing = 0;
  for (let s = 0; s < SEATS; s++) if (!must(world.get(heroOf(g, s), Hero)).down) standing += 1;
  if (standing === 0) {
    g.ending = Ending.Overrun;
    g.sfx |= Sound.Lose;
  }
  return true;
}

/** A round hits a foe: the flash and the damage; at 0 the fall, the points and the burst. Answers the points of a fall (0: it stands), for the shooter's own tally. */
export function hurtFoe(world: World, g: GameV, e: Entity, amount: number, head: boolean): number {
  const f = must(world.get(e, Foe));
  if (f.state === FoeState.Pooled || f.state === FoeState.Dying) return 0;
  f.health -= amount;
  f.flash = 1;
  if (f.health > 0) {
    must(world.set(e, Foe, f));
    return 0;
  }
  const t = lazyTuning(world);
  f.state = FoeState.Dying;
  f.timer = 0.5;
  must(world.set(e, og.NavAgent, { seeking: false, drive: false }));
  if (f.kind === Kind.Drone) must(world.set(ent(f.beam), og.Beam, { opacity: 0 }));
  must(world.set(e, Foe, f));
  const points = (f.kind === Kind.Walker ? POINTS.walker : POINTS.drone) + (head ? POINTS.headshot : 0);
  g.kills += 1;
  g.score += points;
  if (head) g.headshots += 1;
  const at = must(world.get(e, og.Transform)).position;
  const y = f.kind === Kind.Walker ? 1.1 : DRONE.hover;
  burst(world, "pop", g.popNext, { x: at.x, y: at.y + y, z: at.z });
  burst(world, "smoke", g.popNext, { x: at.x, y: at.y + y, z: at.z });
  g.popNext = (g.popNext + 1) % POPS;
  g.sfx |= Sound.Pop;
  if (t.hitStop > 0) must(og.time.hitStop(world, t.hitStop));
  return points;
}

/** Ray against a foe's shape: the distance along the ray and whether it struck the head, or null. In isometric the test is on the floor's plane. */
export function rayFoe(world: World, e: Entity, o: Vector3, d: Vector3, flat: boolean): { t: number; head: boolean } | null {
  const f = must(world.get(e, Foe));
  if (f.state === FoeState.Pooled || f.state === FoeState.Dying) return null;
  const p = must(world.get(e, og.Transform)).position;
  if (flat) {
    const r = f.kind === Kind.Walker ? 0.62 : 0.66;
    const t = circle(o.x - p.x, o.z - p.z, d.x, d.z, r);
    return t === null ? null : { t, head: false };
  }
  if (f.kind === Kind.Drone) {
    const t = sphere(o.x - p.x, o.y - (p.y + DRONE.hover), o.z - p.z, d, DRONE.radius);
    return t === null ? null : { t, head: false };
  }
  const head = sphere(o.x - p.x, o.y - (p.y + WALKER.head), o.z - p.z, d, WALKER.headRadius);
  const tb = circle(o.x - p.x, o.z - p.z, d.x, d.z, WALKER.radius);
  let body: number | null = null;
  if (tb !== null) {
    const y = o.y + d.y * tb - p.y;
    if (y >= WALKER.low && y <= WALKER.high) body = tb;
  }
  if (head !== null && (body === null || head < body)) return { t: head, head: true };
  return body === null ? null : { t: body, head: false };
}

/** The first t ≥ 0 where o + t·d (2D) meets a circle of radius r round the origin. */
function circle(ox: number, oz: number, dx: number, dz: number, r: number): number | null {
  const a = dx * dx + dz * dz;
  if (a < 1e-9) return null;
  const b = ox * dx + oz * dz;
  const c = ox * ox + oz * oz - r * r;
  const disc = b * b - a * c;
  if (disc < 0) return null;
  const s = Math.sqrt(disc);
  const t = (-b - s) / a;
  if (t >= 0) return t;
  return c < 0 ? 0 : null;
}

function sphere(ox: number, oy: number, oz: number, d: Vector3, r: number): number | null {
  const b = ox * d.x + oy * d.y + oz * d.z;
  const c = ox * ox + oy * oy + oz * oz - r * r;
  const disc = b * b - c;
  if (disc < 0) return null;
  const t = -b - Math.sqrt(disc);
  return t >= 0 ? t : c < 0 ? 0 : null;
}

/** Whether nothing of the arena stands between two points. */
function clear(world: World, a: Vector3, b: Vector3): boolean {
  const d = { x: b.x - a.x, y: b.y - a.y, z: b.z - a.z };
  const len = Math.sqrt(d.x * d.x + d.y * d.y + d.z * d.z);
  if (len < 0.01) return true;
  const hit = og.physics.raycast(world, { origin: a, direction: d, maxDistance: len });
  if (!hit.ok || hit.value === null) return true;
  // A trooper's own capsule at the far end is the target, not cover.
  return hit.value.distance >= len - 0.5;
}

/**
 * Per tick: each foe hunts the nearest standing trooper on the navmesh: a
 * walker closes, winds up and strikes; a drone hovers, keeps its range and
 * charges a bolt down a clear line. The hit flash fades; the fallen sink
 * and go back to their pools. Counts the foes standing.
 */
export const Foes = defineSystem("Foes", { reads: [Tuning], writes: [Game, Hero, Foe, og.NavAgent, og.Transform, og.Model, og.AnimationGraph, og.PointLight, og.Beam, og.Mesh], after: ["Weapon"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  if (g.start) parkFoes(world);
  const dt = time.scaledDt;
  if (dt === 0) return;
  const t = lazyTuning(world);
  const playing = g.phase === Phase.Play && g.ending === Ending.None;
  let alive = 0;
  for (const e of foes(world)) {
    const f = must(world.get(e, Foe));
    if (f.state === FoeState.Pooled) continue;
    const p = must(world.get(e, og.Transform)).position;
    if (f.flash > 0) f.flash = Math.max(0, f.flash - dt * 7);
    if (f.state === FoeState.Dying) {
      f.timer -= dt;
      const k = 1 - Math.max(0, f.timer) / 0.5;
      if (f.kind === Kind.Walker) {
        // It topples forward and sinks.
        const tip = k * 1.3;
        must(world.set(ent(f.body), og.Transform, { quaternion: { x: math.sin(tip / 2), y: 0, z: 0, w: math.cos(tip / 2) }, position: { x: 0, y: -k * 0.5, z: 0 } }));
      } else {
        must(world.set(ent(f.body), og.Transform, { position: { x: 0, y: DRONE.hover * (1 - k * k), z: 0 }, quaternion: { x: math.sin(k), y: 0, z: math.sin(k * 0.7), w: math.cos(k) } }));
      }
      must(world.set(ent(f.body), og.Model, { flash: 1 - k * 0.5, flashColor: { r: 1, g: 0.85, b: 0.6 } }));
      if (f.timer <= 0) park(world, e, f);
      must(world.set(e, Foe, f));
      continue;
    }
    alive += 1;
    const prey = playing ? nearestHero(world, g, p) : null;
    const nav = must(world.get(e, og.NavAgent));
    if (f.kind === Kind.Walker) walker(world, g, t, e, f, p, prey, dt);
    else drone(world, g, t, e, f, p, prey, dt, time.elapsed);
    const v = nav.velocity;
    if (f.kind === Kind.Walker) must(world.set(ent(f.body), og.AnimationGraph, { p0: hypot(v.x, v.z) }));
    must(world.set(ent(f.body), og.Model, { flash: f.flash, flashColor: { r: 1, g: 1, b: 1 } }));
    must(world.set(e, Foe, f));
  }
  g.alive = alive;
});

function walker(world: World, g: GameV, t: TuneV, e: Entity, f: FoeV, p: Vector3, prey: { seat: number; pos: Vector3; dist: number } | null, dt: number): void {
  if (prey === null) {
    must(world.set(e, og.NavAgent, { seeking: false }));
    f.state = FoeState.Advance;
    return;
  }
  f.target = prey.seat;
  if (f.state === FoeState.Advance) {
    must(world.set(e, og.NavAgent, { target: prey.pos, seeking: true, drive: true }));
    if (prey.dist < REACH) {
      f.state = FoeState.Windup;
      f.timer = WINDUP;
      must(world.set(e, og.NavAgent, { seeking: false }));
      g.sfx |= Sound.Swipe;
    }
  } else if (f.state === FoeState.Windup) {
    // Turned on its prey, the visor flaring; then the strike lands on whoever is still in reach.
    must(world.set(e, og.Transform, { quaternion: yawQ(math.atan2(prey.pos.x - p.x, prey.pos.z - p.z)) }));
    must(world.set(ent(f.eye), og.PointLight, { intensity: 1.5 + 6 * (1 - f.timer / WINDUP) }));
    f.timer -= dt;
    if (f.timer <= 0) {
      if (prey.dist < STRIKE) hurtHero(world, g, prey.seat, t.walkerDamage, { x: p.x, y: p.y + 1.2, z: p.z });
      f.state = FoeState.Recover;
      f.timer = 0.7;
      must(world.set(ent(f.eye), og.PointLight, { intensity: 1.5 }));
    }
  } else if (f.state === FoeState.Recover) {
    f.timer -= dt;
    if (f.timer <= 0) f.state = FoeState.Advance;
  }
}

function drone(world: World, g: GameV, t: TuneV, e: Entity, f: FoeV, p: Vector3, prey: { seat: number; pos: Vector3; dist: number } | null, dt: number, clock: number): void {
  // The hover's bob and sway, by the clock (presentation of a pose the rules do not read).
  const bob = math.sin(clock * 2.2 + f.bob) * 0.12;
  const eye = { x: p.x, y: p.y + DRONE.hover + bob, z: p.z };
  if (f.state !== FoeState.Dying) must(world.set(ent(f.body), og.Transform, { position: { x: 0, y: DRONE.hover + bob, z: 0 }, quaternion: { x: math.sin(clock * 1.3 + f.bob) * 0.04, y: 0, z: 0, w: 1 } }));
  const beam = ent(f.beam);
  if (prey === null) {
    must(world.set(e, og.NavAgent, { seeking: false }));
    must(world.set(beam, og.Beam, { opacity: 0 }));
    f.state = FoeState.Advance;
    return;
  }
  f.target = prey.seat;
  const chest = { x: prey.pos.x, y: prey.pos.y + 0.25, z: prey.pos.z };
  const sees = prey.dist < t.droneRange && clear(world, eye, chest);
  if (f.state === FoeState.Advance) {
    if (sees) {
      f.state = FoeState.Windup;
      f.timer = CHARGE;
      must(world.set(e, og.NavAgent, { seeking: false }));
      g.sfx |= Sound.Charge;
    } else {
      // Close in on the trooper until it has a line (in range and clear): cover
      // between them keeps it coming round, never parked out of sight.
      must(world.set(e, og.NavAgent, { target: prey.pos, seeking: true, drive: true, stopDistance: 1.5 }));
    }
  }
  if (f.state === FoeState.Windup) {
    must(world.set(e, og.Transform, { quaternion: yawQ(math.atan2(prey.pos.x - p.x, prey.pos.z - p.z)) }));
    const k = 1 - f.timer / CHARGE;
    // The aim follows the trooper for most of the charge, then locks: the beat to dodge in.
    if (f.timer > CHARGE - TRACK) {
      f.aimX = chest.x;
      f.aimY = chest.y;
      f.aimZ = chest.z;
    }
    must(world.set(beam, og.Transform, { position: eye }));
    must(world.set(beam, og.Beam, { end: { x: f.aimX, y: f.aimY, z: f.aimZ }, opacity: 0.15 + 0.6 * k, width: 0.015 + 0.02 * k, color: { r: 4, g: 0.3, b: 0.15 } }));
    must(world.set(ent(f.eye), og.PointLight, { intensity: 1 + 8 * k }));
    f.timer -= dt;
    if (f.timer <= 0) fire(world, g, t, f, eye);
  } else if (f.state === FoeState.Recover) {
    const op = must(world.get(beam, og.Beam)).opacity;
    if (op > 0) must(world.set(beam, og.Beam, { opacity: Math.max(0, op - dt * 6) }));
    f.timer -= dt;
    if (f.timer <= 0) f.state = FoeState.Advance;
  }
}

/** A drone's bolt down its locked aim: it hits the first thing on the line (a trooper's capsule, or cover). */
function fire(world: World, g: GameV, t: TuneV, f: FoeV, eye: Vector3): void {
  const d = { x: f.aimX - eye.x, y: f.aimY - eye.y, z: f.aimZ - eye.z };
  const hit = og.physics.raycast(world, { origin: eye, direction: d, maxDistance: 60 });
  let end = { x: eye.x + d.x * 3, y: eye.y + d.y * 3, z: eye.z + d.z * 3 };
  if (hit.ok && hit.value !== null) {
    end = hit.value.point;
    for (let seat = 0; seat < SEATS; seat++) {
      if (hit.value.entity === heroOf(g, seat)) hurtHero(world, g, seat, t.droneDamage, eye);
    }
  }
  must(world.set(ent(f.beam), og.Beam, { end, opacity: 1, width: 0.09, color: { r: 8, g: 0.8, b: 0.3 } }));
  must(world.set(ent(f.eye), og.PointLight, { intensity: 1 }));
  g.sfx |= Sound.Bolt;
  f.state = FoeState.Recover;
  f.timer = 1.5;
}

/** How many foes wave `n` sends (two seats: half again). */
export function waveSize(n: number): number {
  const base = WAVE_FOES[n - 1]?.length ?? 0;
  return SEATS > 1 ? Math.round(base * 1.5) : base;
}

/** The kind of the i-th foe through the gates in wave n. */
function waveKind(n: number, i: number): number {
  const order = WAVE_FOES[n - 1] ?? "W";
  return order[i % order.length] === "D" ? Kind.Drone : Kind.Walker;
}

/** A new run's first breather (src/ui.ts startRun): wave 1 after a moment. */
export function beginWaves(g: GameV): void {
  g.wave = 1;
  g.waveState = WaveState.Breather;
  g.waveTimer = 2.5;
  g.toSpawn = 0;
  g.spawnTimer = 0;
}

/**
 * The holdout's waves: the breather, then the wave's foes through the
 * gates one by one (at most the pools' size at once); a wave cleared pays
 * its bonus, gives back health and ammunition, puts a downed trooper back
 * on its feet and starts the next breather, and the last one cleared holds
 * the arena. In a duel, each downed duellist is back after its beat at the
 * duel point farthest from the other.
 */
export const Waves = defineSystem("Waves", { reads: [Tuning], writes: [Game, Hero, Foe, SeatLook, og.NavAgent, og.Transform, og.Model, og.Mesh, og.PointLight, og.Beam, og.ParticleSystem, og.Rng, og.CharacterController], after: ["Foes"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const dt = time.scaledDt;
  if (g.phase !== Phase.Play || g.ending !== Ending.None || dt === 0) return;
  const t = lazyTuning(world);
  if (g.mode === Mode.Duel) {
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      if (!me.down) continue;
      me.respawn -= dt;
      if (me.respawn <= 0) {
        const other = must(world.get(heroOf(g, 1 - seat), og.Transform)).position;
        let far = 0;
        let best = -1;
        for (let i = 0; i < 4; i++) {
          const q = point(world, `duel-${i}`);
          const d = hypot(q.x - other.x, q.z - other.z);
          if (d > best) {
            best = d;
            far = i;
          }
        }
        const q = point(world, `duel-${far}`);
        const keep = { kills: me.kills, frags: me.frags, score: me.score, jumps: me.jumps };
        resetHero(world, hero, me, t, { x: q.x, y: q.y + FEET + 0.05, z: q.z });
        Object.assign(me, keep);
      }
      must(world.set(hero, Hero, me));
    }
    return;
  }
  if (g.waveState === WaveState.Breather) {
    g.waveTimer -= dt;
    if (g.waveTimer <= 0) {
      g.waveState = WaveState.Fighting;
      g.toSpawn = waveSize(g.wave);
      g.spawnTimer = 0;
      g.sfx |= Sound.Horn;
    }
    return;
  }
  g.spawnTimer -= dt;
  if (g.toSpawn > 0 && g.spawnTimer <= 0) {
    const i = waveSize(g.wave) - g.toSpawn;
    const gate = must(world.resource(og.Rng)).integer(0, GATES - 1);
    if (bring(world, g, t, waveKind(g.wave, i), gate)) {
      g.toSpawn -= 1;
      g.alive += 1;
      g.spawnTimer = Math.max(0.45, 1.25 - 0.12 * g.wave) / (SEATS > 1 ? 1.25 : 1);
    } else g.spawnTimer = 0.5;
  }
  if (g.toSpawn > 0 || g.alive > 0) return;
  // The wave is cleared.
  g.score += POINTS.wave * g.wave;
  g.sfx |= Sound.Clear;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const me = must(world.get(hero, Hero));
    if (me.down) {
      const keep = { kills: me.kills, frags: me.frags, score: me.score, jumps: me.jumps };
      resetHero(world, hero, me, t, spawnOf(world, seat));
      Object.assign(me, keep);
      me.health = t.health * 0.6;
    } else {
      me.health = Math.min(t.health, me.health + t.health * 0.35);
      me.reserve = t.reserve;
    }
    must(world.set(hero, Hero, me));
  }
  if (g.wave >= WAVES) {
    g.ending = Ending.Held;
    g.sfx |= Sound.Win;
    return;
  }
  g.wave += 1;
  g.waveState = WaveState.Breather;
  g.waveTimer = t.breather;
});

export { EYE };
