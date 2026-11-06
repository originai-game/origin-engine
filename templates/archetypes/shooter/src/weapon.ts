// The pulse rifle: hitscan, automatic. Held Fire (F or /, the right
// trigger, the left mouse button, Fire on the touch HUD) fires
// Tuning.fireRate rounds a second while the magazine lasts; each round
// leaves along the seat's ray (src/hero.ts rayOf) within its spread (the
// rest spread plus the bloom firing adds), climbs the view by the recoil
// (it settles back when the trigger rests), flashes the muzzle (a light
// and a bright burst on the barrel) and draws a tracer. It strikes the
// first thing on its line: a foe (a walker's head counts double), the
// other duellist in a duel, or the arena, where it leaves sparks and a
// scorched hole. Reload (R or ., X, Reload) refills the magazine from the
// reserve over Tuning.reloadSeconds; firing an empty magazine clicks and
// reloads. Every random number is og.Rng's, so a room fires the same
// rounds on every peer.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { DUEL_FRAGS, Ending, Foe, Game, Hero, MARKS, Mode, Phase, Sound, TRACERS, Tuning } from "./components.ts";
import { foes, hurtFoe, hurtHero, rayFoe } from "./foes.ts";
import { heroOf, lazyTuning, muzzleOf, rayOf } from "./hero.ts";
import { held, localSeats, pressedNow, SEATS } from "./seats.ts";
import { FIRST_PERSON, ISOMETRIC, SeatLook } from "./view.ts";
import { burst, DOWN, entity, must, qmul, SPARKS, SPRAYS, upTo } from "./world.ts";

type GameV = Value<typeof Game>;
type HeroV = Value<typeof Hero>;
type TuneV = Value<typeof Tuning>;

const DEG = Math.PI / 180;
/** How far a round reaches. */
const RANGE = 80;
/** Seconds the trigger rests before the recoil settles, and how fast it does (per second). */
const SETTLE_AFTER = 0.12;
const SETTLE = 7;
/** The muzzle flash's seconds. */
const FLASH = 0.05;
/** The flash's size: in first person it sits a hand from the eye, so smaller (the others see it full size). */
const FLASH_SIZE = FIRST_PERSON ? 0.45 : 1;

const ent = (n: number): Entity => n as unknown as Entity;
const tracerName = (seat: number, i: number) => `tracer-${seat}-${i}`;

/** Each seat's tracers (og.Beam, fading) and its muzzle burst. */
export function spawnWeapons(world: World): void {
  const flashGeo = must(og.render.boxGeometry(world, { width: 0.1, height: 0.1, depth: 0.22 }));
  const flashMat = must(og.render.standardMaterial(world, { color: { r: 1, g: 0.8, b: 0.5 }, emissive: { r: 1, g: 0.7, b: 0.35 }, emissiveIntensity: 9, unlit: true }));
  for (let seat = 0; seat < SEATS; seat++) {
    for (let i = 0; i < TRACERS; i++) {
      const e = must(world.spawn({ name: tracerName(seat, i), position: { x: 0, y: -50, z: 0 } }));
      must(world.insert(e, og.Beam, { end: { x: 0, y: -50, z: 0 }, width: 0.035, color: { r: 5, g: 3.4, b: 1.6 }, opacity: 0, blend: "additive" }));
    }
    const f = must(world.spawn({ name: `flash-${seat}`, position: { x: 0, y: -50, z: 0 } }));
    must(world.insert(f, og.Mesh, { geometry: flashGeo, material: flashMat, castShadow: false, visible: false }));
  }
}

/** A seat's view climbs by `rad` (or is let back by a negative one), within the look's limits. */
function climb(world: World, hero: Entity, rad: number): void {
  if (ISOMETRIC) return;
  const look = must(world.get(hero, SeatLook));
  must(world.set(hero, SeatLook, { pitch: Math.max(-1.4, Math.min(1.4, look.pitch + rad)) }));
}

/** One round from a seat: the ray, what it meets, the feedback. */
function shoot(world: World, g: GameV, t: TuneV, seat: number, hero: Entity, me: HeroV): void {
  const rng = must(world.resource(og.Rng));
  const { origin, dir } = rayOf(world, hero);
  // The spread: a random direction within the cone (degrees at its edge).
  const cone = (t.spread + me.bloom) * DEG;
  const a = rng.range(0, Math.PI * 2);
  const r = Math.sqrt(rng.next()) * cone;
  const side = ISOMETRIC ? { x: -dir.z, y: 0, z: dir.x } : norm(cross(dir, { x: 0, y: 1, z: 0 }));
  const up = ISOMETRIC ? { x: 0, y: 0, z: 0 } : cross(side, dir);
  const k = math.cos(a) * r, j = ISOMETRIC ? 0 : math.sin(a) * r;
  const d = norm({ x: dir.x + side.x * k + up.x * j, y: dir.y + side.y * k + up.y * j, z: dir.z + side.z * k + up.z * j });
  // The arena (and in a duel the other trooper) by physics; the foes by their shapes.
  let wall = RANGE;
  let wallHit: { point: Vector3; normal: Vector3; seat: number } | null = null;
  let from = origin;
  let gone = 0;
  for (let tries = 0; tries < 3; tries++) {
    const hit = og.physics.raycast(world, { origin: from, direction: d, maxDistance: RANGE - gone, filter: { exclude: hero } });
    if (!hit.ok || hit.value === null) break;
    const other = SEATS > 1 && hit.value.entity === heroOf(g, 1 - seat) ? 1 - seat : -1;
    if (other >= 0 && (g.mode !== Mode.Duel || must(world.get(hit.value.entity, Hero)).down)) {
      // A partner in a holdout is never hit: the round passes them.
      gone += hit.value.distance + 0.8;
      from = { x: origin.x + d.x * gone, y: origin.y + d.y * gone, z: origin.z + d.z * gone };
      continue;
    }
    wall = gone + hit.value.distance;
    wallHit = { point: hit.value.point, normal: hit.value.normal, seat: other };
    break;
  }
  let foe: Entity | null = null;
  let foeT = wall;
  let head = false;
  if (g.mode === Mode.Holdout) {
    for (const e of foes(world)) {
      const h = rayFoe(world, e, origin, d, ISOMETRIC);
      if (h !== null && h.t < foeT) {
        foeT = h.t;
        foe = e;
        head = h.head;
      }
    }
  }
  const end = foe !== null ? along(origin, d, foeT) : wallHit ? wallHit.point : along(origin, d, RANGE);
  g.shots += 1;
  if (foe !== null) {
    const points = hurtFoe(world, g, foe, t.damage * (head ? 2 : 1), head);
    if (points > 0) {
      me.kills += 1;
      me.score += points;
    }
    g.hits += 1;
    me.hitTick = world.tick;
    me.hitKind = points > 0 ? 2 : head ? 1 : 0;
    g.sfx |= Sound.Hit;
    burst(world, "spray", g.sprayNext, end, { x: -d.x, y: -d.y, z: -d.z });
    g.sprayNext = (g.sprayNext + 1) % SPRAYS;
  } else if (wallHit && wallHit.seat >= 0) {
    const downed = hurtHero(world, g, wallHit.seat, t.damage, origin);
    if (downed) {
      // A frag: DUEL_FRAGS of them win the duel.
      me.frags += 1;
      if (me.frags >= DUEL_FRAGS) {
        g.ending = Ending.Duel;
        g.winner = seat;
        g.sfx |= Sound.Win;
      }
    }
    g.hits += 1;
    me.hitTick = world.tick;
    me.hitKind = downed ? 2 : 0;
    g.sfx |= Sound.Hit;
    burst(world, "spray", g.sprayNext, end, { x: -d.x, y: -d.y, z: -d.z });
    g.sprayNext = (g.sprayNext + 1) % SPRAYS;
  } else if (wallHit) {
    burst(world, "sparks", g.sparkNext, end, wallHit.normal);
    g.sparkNext = (g.sparkNext + 1) % SPARKS;
    // The hole: projected into the surface along its normal, turned at random.
    const mark = entity(world, `mark-${g.markNext}`);
    const spin = rng.range(0, Math.PI * 2);
    const q = qmul(upTo(wallHit.normal), qmul({ x: 0, y: math.sin(spin / 2), z: 0, w: math.cos(spin / 2) }, DOWN));
    must(world.set(mark, og.Transform, { position: along(end, wallHit.normal, 0.02), quaternion: q }));
    must(world.set(mark, og.Decal, { visible: true }));
    g.markNext = (g.markNext + 1) % MARKS;
  }
  // The tracer, from the muzzle the others see (in first person the hands' muzzle).
  const tracer = entity(world, tracerName(seat, me.tracer));
  me.tracer = (me.tracer + 1) % TRACERS;
  const muzzle = muzzleOf(world, hero);
  must(world.set(tracer, og.Transform, { position: muzzle }));
  must(world.set(tracer, og.Beam, { end, opacity: 1 }));
  const flash = entity(world, `flash-${seat}`);
  const roll = rng.range(0, Math.PI);
  const look = must(world.get(hero, SeatLook));
  const yaw = { x: 0, y: math.sin(look.yaw / 2), z: 0, w: math.cos(look.yaw / 2) };
  must(world.set(flash, og.Transform, { position: muzzle, quaternion: qmul(yaw, { x: 0, y: 0, z: math.sin(roll / 2), w: math.cos(roll / 2) }), scale: { x: rng.range(0.8, 1.4) * FLASH_SIZE, y: rng.range(0.8, 1.4) * FLASH_SIZE, z: 1 } }));
  must(world.set(flash, og.Mesh, { visible: true }));
  must(world.set(ent(me.muzzle), og.Transform, { position: muzzle }));
  must(world.set(ent(me.muzzle), og.PointLight, { intensity: 14 }));
  // The recoil: the view climbs and drifts a little; the bloom opens.
  const kick = t.recoil * DEG * rng.range(0.8, 1.2);
  climb(world, hero, kick);
  me.kick += kick;
  if (!ISOMETRIC) must(world.set(hero, SeatLook, { yaw: look.yaw + rng.range(-0.25, 0.25) * t.recoil * DEG }));
  me.bloom = Math.min(4, me.bloom + t.bloom);
  me.firing = 0;
  g.sfx |= Sound.Shot;
  // Feel for the one who fired, on the device that plays them: a camera kick and a touch of shake.
  if (localSeats(world).includes(seat)) {
    must(og.camera.shake(world, FIRST_PERSON ? 0.05 : 0.035));
    if (FIRST_PERSON) must(og.camera.kick(world, { x: -d.x, y: 0, z: -d.z }, 0.025));
  }
}

const along = (o: Vector3, d: Vector3, s: number): Vector3 => ({ x: o.x + d.x * s, y: o.y + d.y * s, z: o.z + d.z * s });
const cross = (a: Vector3, b: Vector3): Vector3 => ({ x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x });
function norm(v: Vector3): Vector3 {
  const n = Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z) || 1;
  return { x: v.x / n, y: v.y / n, z: v.z / n };
}

/**
 * Per tick, for each trooper: reload (asked, or an empty magazine fired),
 * fire while Fire is held and the rifle is ready, settle the recoil and the
 * bloom while it rests, and fade the flash and the tracers.
 */
export const Weapon = defineSystem("Weapon", { reads: [og.PlayerInput, Tuning, og.FollowCamera, og.Lockstep], writes: [Game, Hero, SeatLook, Foe, og.NavAgent, og.Beam, og.Transform, og.Mesh, og.PointLight, og.ParticleSystem, og.Decal, og.Rng, og.Camera], after: ["Hero"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const dt = time.scaledDt;
  if (dt === 0) return;
  const t = lazyTuning(world);
  const playing = g.phase === Phase.Play && g.ending === Ending.None;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const me = must(world.get(hero, Hero));
    const input = must(world.get(hero, og.PlayerInput));
    const buttons = input.buttons;
    const live = playing && !me.down && !g.start;
    me.cooldown = Math.max(0, me.cooldown - dt);
    me.firing += dt;
    if (me.reload > 0) {
      me.reload -= dt;
      if (me.reload <= 0) {
        const take = Math.min(t.magazine - me.mag, me.reserve);
        me.mag += take;
        me.reserve -= take;
        me.reload = 0;
      }
    }
    const canReload = live && me.reload === 0 && me.mag < t.magazine && me.reserve > 0;
    if (canReload && pressedNow(buttons, me.before, "reload")) startReload(g, t, me);
    if (live && held(buttons, "fire") && me.reload === 0 && me.cooldown === 0) {
      if (me.mag > 0) {
        me.mag -= 1;
        me.cooldown = 1 / t.fireRate;
        shoot(world, g, t, seat, hero, me);
      } else {
        // Empty: a click on the pull, and the reload starts by itself.
        if (pressedNow(buttons, me.before, "fire")) g.sfx |= Sound.Empty;
        if (canReload) startReload(g, t, me);
        me.cooldown = 0.25;
      }
    }
    // Resting: the view comes back down by the climb the recoil left, and the bloom closes.
    if (me.firing > SETTLE_AFTER) {
      if (me.kick > 0) {
        const back = Math.min(me.kick, me.kick * SETTLE * dt + 0.002);
        climb(world, hero, -back * 0.85);
        me.kick -= back;
      }
      me.bloom = Math.max(0, me.bloom - dt * 6);
    }
    if (me.firing > FLASH) {
      must(world.set(entity(world, `flash-${seat}`), og.Mesh, { visible: false }));
      must(world.set(ent(me.muzzle), og.PointLight, { intensity: 0 }));
    }
    for (let i = 0; i < TRACERS; i++) {
      const tr = entity(world, tracerName(seat, i));
      const op = must(world.get(tr, og.Beam)).opacity;
      if (op > 0) must(world.set(tr, og.Beam, { opacity: Math.max(0, op - dt * 14) }));
    }
    me.before = buttons;
    must(world.set(hero, Hero, me));
  }
});

function startReload(g: GameV, t: TuneV, me: HeroV): void {
  me.reload = t.reloadSeconds;
  g.sfx |= Sound.Reload;
}
