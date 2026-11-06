// The party: one hero per seat (src/seats.ts), each a capsule on
// og.CharacterController carrying a KayKit body (the knight for seat 0, the
// barbarian for seat 1), moved by its own og.PlayerInput alone.
//
// The move walks relative to the view (src/view.ts): on the isometric
// view relative to the screen (up the stick is up the screen), on the
// others relative to where the seat looks. The look turns the seat's view;
// on the isometric view, whose camera never turns, it turns the hero
// instead, and a hero that walks without the look faces where it walks.
// Attack (Space, J, the left button, A, the touch Attack) swings: the blow
// lands on the attack clip's `hit` marker on every standing risen within
// reach in front (a little help turns the swing to the nearest), with a
// hit stop, a flash, the numbers, sparks and a shake. Potion (Q, Y) drinks
// one of the party's draughts. Talk (E, X) speaks to Maren, opens the chest
// and moves a conversation on (src/story.ts).
//
// A hero struck to nothing falls. Alone that ends the run; with two, the
// fallen rises at the gate after Tuning.respawn while the other stands,
// and the run ends when both are down. Experience from every risen put
// down goes to each standing hero, and a level brings health and a harder
// blow.

import { og, defineSystem, math, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { animate, oneShot, release, rigOf } from "./anim.ts";
import { creation } from "./creation.ts";
import { Ending, Foe, Game, Hero, Loot, MAX_LEVEL, Near, Phase, Sound, Tuning } from "./components.ts";
import { CHEST, STARTS } from "./layout.ts";
import { held, LOCAL, localSeats, SEATS } from "./seats.ts";
import { answer, openChest, startTalk } from "./story.ts";
import { bodyLayer, FIRST_PERSON, ISOMETRIC, SeatLook, walkDirection } from "./view.ts";
import { strikeFoe } from "./foes.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type HeroV = Value<typeof Hero>;

const DEG = Math.PI / 180;
const UP = { x: 0, y: 1, z: 0 };
/** The capsule's centre above the feet (radius 0.32, a 0.9 m segment). */
export const FEET = 0.77;
/** The eye above the capsule's centre (1.62 m above the feet): the first-person camera, and what the others look at. */
export const EYE = 0.85;
/** The heroes' models (tools/compose.py) by seat, and their scale (about 1.75 m tall). */
const MODELS = ["knight", "barbarian"] as const;
export const BODY_SCALE = 0.7;
/** The isometric camera's fixed heading (the look's ArtCamera yaw): the screen's up is this way. */
const ISO_YAW = ((creation.camera as { yaw?: number }).yaw ?? 45) * DEG;
/** Where a new run's look points (third and first person): north-east, into the yard. */
const START_YAW = -0.75;
/** How near Talk reaches Maren and the chest. */
export const TALK_REACH = 2.6;
/** How long a buffered Attack waits for the swing it starts. */
const BUFFER = 0.25;
/** A foe's body radius, for the reach of a blow and keeping heroes out of it. */
export const FOE_RADIUS = 0.45;

/** sqrt(x² + y²) the same on every JavaScript engine (Math.hypot is not). */
export const hypot = (x: number, y: number): number => Math.sqrt(x * x + y * y);

export function wrap(a: number): number {
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

/** Turns `from` towards `to` by at most `step` radians. */
export function turn(from: number, to: number, step: number): number {
  const d = wrap(to - from);
  return wrap(from + Math.max(-step, Math.min(step, d)));
}

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

/** Seat `seat`'s hero's name: `hero`, `hero-1`. */
export const heroName = (seat: number): string => (seat === 0 ? "hero" : `hero-${seat}`);

/** Seat `seat`'s hero (seat 0's is `hero`, seat 1's `hero-1`). */
export function heroOf(g: GameV, seat: number): Entity {
  return (seat === 0 ? g.hero : g.hero1) as unknown as Entity;
}

/** The experience a hero at `level` needs for the next: Tuning.levelXp more each level. */
export function xpFor(t: TuneV, level: number): number {
  return t.levelXp * level;
}

/** A hero's blow at its level. */
export function damageOf(t: TuneV, level: number): number {
  return t.damage + (level - 1) * t.damagePerLevel;
}

/** First person: a seat's blade before its eye, on a layer only that seat's camera draws (its body is on src/view.ts's bodyLayer). */
const bladeLayer = (seat: number) => 8 << seat;

/** Spawns a hero per seat at the gate, each with its body, its heal and level-up sparkle, and in first person its blade. */
export function spawnHeroes(world: World): Entity[] {
  const g = must(world.resource(Game));
  const t = must(world.resource(Tuning));
  const heal = must(og.load.emitter(world, "fx/heal.ogt"));
  const rise = must(og.load.emitter(world, "fx/levelup.ogt"));
  const heroes: Entity[] = [];
  for (let seat = 0; seat < SEATS; seat++) {
    const name = heroName(seat);
    const at = STARTS[seat]!;
    const hero = must(world.spawn({ name, position: { x: at.x, y: FEET + 0.05, z: at.z } }));
    must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.32, height: 0.9 }));
    must(world.insert(hero, og.CharacterController, { maxSlope: 40, stepHeight: 0.3, snapDistance: 0.3 }));
    must(world.insert(hero, og.PlayerInput, { player: seat }));
    // The risen see the nearest standing hero (og.Perception) and hunt it.
    must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
    const rig = rigOf(world, MODELS[seat]!);
    const model = must(og.load.gltf(world, `models/${MODELS[seat]}.glb`));
    const s = BODY_SCALE;
    const body = must(world.instantiate(model, { name: `${name}-body`, parent: hero, position: { x: 0, y: -FEET, z: 0 }, scale: { x: s, y: s, z: s } }));
    animate(world, body, rig);
    if (FIRST_PERSON) must(world.set(body, og.Model, { layers: bodyLayer(seat) }));
    const sparkle = must(world.spawn({ name: `${name}-heal`, parent: hero, position: { x: 0, y: -0.2, z: 0 } }));
    must(world.insert(sparkle, og.ParticleSystem, { emitter: heal, seed: 3 + seat, visible: false }));
    const glow = must(world.spawn({ name: `${name}-rise`, parent: hero, position: { x: 0, y: -FEET + 0.1, z: 0 } }));
    must(world.insert(glow, og.ParticleSystem, { emitter: rise, seed: 7 + seat, visible: false }));
    if (FIRST_PERSON) {
      const blade = must(og.load.gltf(world, seat === 0 ? "models/gear/sword_1handed.gltf" : "models/gear/axe_1handed.gltf"));
      const vm = must(world.instantiate(blade, { name: `${name}-blade`, scale: { x: 0.55, y: 0.55, z: 0.55 } }));
      must(world.set(vm, og.Model, { layers: bladeLayer(seat), castShadow: false }));
    }
    must(world.insert(hero, Hero, { seat, body: body as unknown as number, hp: t.health, maxHp: t.health }));
    if (seat === 0) g.hero = hero as unknown as number;
    else g.hero1 = hero as unknown as number;
    heroes.push(hero);
  }
  return heroes;
}

/**
 * First person: each seat's camera draws the world (layer 0), the other
 * hero's body and its own blade, never its own body (the eye is inside it).
 */
export function layerCameras(world: World): void {
  if (!FIRST_PERSON) return;
  const other = (seat: number) => (SEATS > 1 ? bodyLayer(1 - seat) : 0);
  if (LOCAL) {
    for (let seat = 0; seat < SEATS; seat++) must(world.set(entity(world, `view-${seat}`), og.CameraView, { layers: 1 | other(seat) | bladeLayer(seat) }));
    return;
  }
  const seat = localSeats(world)[0] ?? 0;
  must(world.resource(og.Camera)).layers = 1 | other(seat) | bladeLayer(seat);
}

/** Puts a hero back at the gate for a new run: level 1, whole, its look into the yard. */
function reset(world: World, t: TuneV, hero: Entity, me: HeroV): void {
  const at = STARTS[me.seat]!;
  must(world.set(hero, og.Transform, { position: { x: at.x, y: FEET + 0.05, z: at.z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 } }));
  Object.assign(me, { hp: t.health, maxHp: t.health, level: 1, xp: 0, attack: 0, struck: false, buffer: 0, hurt: 0, invulnerable: 0, knock: 0, dead: false, respawn: 0, near: Near.None, speed: 0, stepClock: 0, swings: 0, kills: 0 });
  const yaw = ISOMETRIC ? ISO_YAW : START_YAW;
  me.facing = wrap(yaw + Math.PI);
  must(world.set(hero, SeatLook, { yaw, pitch: ISOMETRIC || FIRST_PERSON ? 0 : -0.25 }));
  const body = me.body as unknown as Entity;
  release(world, body);
  must(world.set(body, og.Model, { flash: 0 }));
  if (!world.get(hero, og.Perceivable).ok) must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
}

/** The foes as the heroes see them this tick: where each stands and whether it still does. */
interface FoeAt { e: Entity; x: number; z: number; up: boolean }

function foesNow(world: World): FoeAt[] {
  const out: FoeAt[] = [];
  for (let i = 0; ; i++) {
    const e = world.find(`foe-${i}`);
    if (e === null) return out;
    const p = must(world.get(e, og.Transform)).position;
    const f = must(world.get(e, Foe));
    out.push({ e, x: p.x, z: p.z, up: f.hp > 0 });
  }
}

/**
 * Per tick, for each hero: its presses, its move by its view, its swing and
 * blow, its potion, its talk, its fall and rise, the graph's speed and its
 * body's turn.
 */
export const Heroes = defineSystem(
  "Heroes",
  { reads: [og.PlayerInput, og.AnimationEvent, Tuning], writes: [Game, Hero, Loot, og.Rng, og.Camera, og.PointLight, SeatLook, og.CharacterController, og.Transform, og.AnimationGraph, og.AnimationPlayer, og.Model, og.ParticleSystem, og.PostEffects, og.Perceivable, Foe, og.AiAgent, og.NavAgent], after: ["Ui"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = lazyTuning(world);
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    const dt = time.scaledDt;
    const foes = foesNow(world);
    const events = world.events(og.AnimationEvent);
    let standing = 0;
    for (let seat = 0; seat < SEATS; seat++) {
      const hero = heroOf(g, seat);
      const me = must(world.get(hero, Hero));
      const input = must(world.get(hero, og.PlayerInput));
      if (g.start) reset(world, t, hero, me);
      // Press edges against the buttons of the last tick that stepped (Ui read them first): a press during a hit stop waits for it.
      const pressed = input.buttons & ~me.before;
      if (dt > 0 || g.start || g.held || g.phase !== Phase.Play) me.before = input.buttons;
      if (dt > 0 && !g.start) step(world, g, t, hero, me, input, pressed, playing, dt, foes, events);
      if (!me.dead) standing += 1;
      must(world.set(hero, Hero, me));
    }
    const post = must(world.resource(og.PostEffects));
    if (post.flash > 0 && dt > 0) post.flash = Math.max(0, post.flash - dt * 2.2);
    if (playing && standing === 0) {
      g.ending = Ending.Fallen;
      g.sfx |= Sound.Lose;
    }
  },
);

const pressedBit = (pressed: number, name: string) => held(pressed, name);

/** One hero's tick. */
function step(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV, input: { x: number; y: number; aimX: number; aimY: number; buttons: number }, pressed: number, playing: boolean, dt: number, foes: FoeAt[], events: readonly { entity: Entity; name: string }[]): void {
  const body = me.body as unknown as Entity;
  const rig = rigOf(world, MODELS[me.seat]!);
  const p = must(world.get(hero, og.Transform)).position;
  const cc = must(world.get(hero, og.CharacterController));
  const look = must(world.get(hero, SeatLook));
  if (me.hurt > 0) {
    me.hurt = Math.max(0, me.hurt - dt);
    must(world.set(body, og.Model, { flash: me.hurt / 0.3 * 0.7 }));
  }
  if (me.invulnerable > 0) me.invulnerable = Math.max(0, me.invulnerable - dt);

  // The fallen: lie still, and rise at the gate once the other has held long enough.
  if (me.dead) {
    must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 } }));
    must(world.set(body, og.AnimationGraph, { p0: 0 }));
    if (playing && SEATS > 1 && me.respawn > 0) {
      me.respawn = Math.max(0, me.respawn - dt);
      if (me.respawn === 0) rise(world, g, t, hero, me);
    }
    return;
  }

  // What Talk would act on.
  const maren = must(world.get(g.maren as unknown as Entity, og.Transform)).position;
  me.near = hypot(p.x - maren.x, p.z - maren.z) < TALK_REACH ? Near.Maren : !g.chestOpen && hypot(p.x - CHEST.x, p.z - CHEST.z) < TALK_REACH - 0.6 ? Near.Chest : Near.None;
  const talking = g.conversation !== 0;
  if (playing) {
    if (talking) answer(world, g, me.seat, pressed);
    else if (pressedBit(pressed, "talk")) {
      if (me.near === Near.Maren) startTalk(world, g, me.seat);
      else if (me.near === Near.Chest) openChest(world, g, me.seat, p);
    }
  }

  // The move, by the view: the screen's up on the isometric view, the look's forward on the others.
  const mag = Math.min(1, hypot(input.x, input.y));
  const free = playing && !talking && me.knock === 0;
  let dir = { x: 0, z: 0 };
  if (free && mag > 0.05) {
    const w = walkDirection(ISOMETRIC ? ISO_YAW : look.yaw, input.x, input.y);
    const n = hypot(w.x, w.z);
    dir = { x: w.x / n, z: w.z / n };
  }
  let speed = free ? t.walkSpeed * Math.min(1, mag / 0.8) : 0;

  // The swing: a press (buffered a moment) starts one once the last is mostly done.
  if (playing && !talking && pressedBit(pressed, "attack")) me.buffer = BUFFER;
  else me.buffer = Math.max(0, me.buffer - dt);
  if (free && me.buffer > 0 && (me.attack === 0 || me.attack > t.swing * 0.8)) {
    me.buffer = 0;
    me.attack = 1e-4;
    me.struck = false;
    me.swings += 1;
    g.sfx |= Sound.Swing;
    if (speed > 0) me.facing = math.atan2(dir.x, dir.z);
    const aim = aimAt(me, p, foes, t);
    if (aim !== null) me.facing = aim;
    // First person: the look is the facing, so the help turns the look, only for a risen nearly in front (a little magnetism, never a spin).
    if (FIRST_PERSON && aim !== null && Math.abs(wrap(aim - wrap(look.yaw + Math.PI))) < 35 * DEG) must(world.set(hero, SeatLook, { yaw: wrap(aim - Math.PI) }));
    oneShot(world, body, rig, "attack", 0.55 / t.swing);
  }
  if (me.attack > 0) {
    me.attack += dt;
    // A swing roots the hero for its first part.
    if (me.attack < t.swing * 0.6) speed *= 0.3;
    for (const ev of events) {
      if (ev.entity !== body || ev.name !== "hit" || me.struck) continue;
      me.struck = true;
      blow(world, g, t, me, p, foes);
    }
    if (me.attack >= t.swing) {
      if (!me.struck) blow(world, g, t, me, p, foes);
      me.attack = 0;
      release(world, body);
    }
  }

  // The potion.
  if (playing && !talking && pressedBit(pressed, "potion")) drink(world, g, t, hero, me, p);

  // Thrown back by a blow.
  if (me.knock > 0) {
    speed = 6 * (me.knock / 0.2);
    dir = { x: me.knockX, z: me.knockZ };
    me.knock = Math.max(0, me.knock - dt);
  }
  // Kept out of the risen's bodies.
  let push = { x: 0, z: 0 };
  for (const f of foes) {
    if (!f.up) continue;
    const dx = p.x - f.x;
    const dz = p.z - f.z;
    const d = hypot(dx, dz);
    const min = FOE_RADIUS + 0.35;
    if (d < min && d > 1e-4) push = { x: push.x + (dx / d) * (min - d) * 8, z: push.z + (dz / d) * (min - d) * 8 };
  }
  must(world.set(hero, og.CharacterController, { move: { x: dir.x * speed + push.x, y: 0, z: dir.z * speed + push.z } }));

  // The facing: the look on first person and (when it turns) isometric; else where it walks.
  const turned = input.aimX !== 0;
  if (FIRST_PERSON) me.facing = wrap(look.yaw + Math.PI);
  else if (ISOMETRIC && turned && me.attack === 0) me.facing = wrap(look.yaw + Math.PI);
  else if (speed > 0.1 && me.knock === 0 && me.attack === 0) me.facing = turn(me.facing, math.atan2(dir.x, dir.z), 14 * dt);
  // On the isometric view the look follows the hero, so the next turn starts from where it faces.
  if (ISOMETRIC) must(world.set(hero, SeatLook, { yaw: wrap(me.facing - Math.PI) }));
  must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, me.facing) }));

  // The graph, the steps.
  const v = cc.velocity;
  me.speed = hypot(v.x, v.z);
  must(world.set(body, og.AnimationGraph, { p0: me.attack > 0 ? 0 : me.speed }));
  if (cc.grounded && me.speed > 0.5) {
    me.stepClock += me.speed * dt;
    if (me.stepClock >= 1.25) {
      me.stepClock -= 1.25;
      g.sfx |= Sound.Step;
    }
  }
  if (FIRST_PERSON) blade(world, me, p, look);
}

/** The help a swing gets: the nearest standing risen within reach and a wide arc in front, else null. */
function aimAt(me: HeroV, p: Vector3, foes: FoeAt[], t: TuneV): number | null {
  let best: number | null = null;
  let bestD = t.reach + FOE_RADIUS + 1.2;
  for (const f of foes) {
    if (!f.up) continue;
    const d = hypot(f.x - p.x, f.z - p.z);
    const a = math.atan2(f.x - p.x, f.z - p.z);
    if (d < bestD && Math.abs(wrap(a - me.facing)) < 100 * DEG) {
      bestD = d;
      best = a;
    }
  }
  return best;
}

/** The blow lands: every standing risen within reach and the arc takes it (src/foes.ts). */
function blow(world: World, g: GameV, t: TuneV, me: HeroV, p: Vector3, foes: FoeAt[]): void {
  let landed = 0;
  for (const f of foes) {
    if (!f.up) continue;
    const d = hypot(f.x - p.x, f.z - p.z);
    const a = math.atan2(f.x - p.x, f.z - p.z);
    if (d > t.reach + FOE_RADIUS || Math.abs(wrap(a - me.facing)) > t.arc * DEG) continue;
    strikeFoe(world, g, t, me, f.e, { x: p.x, z: p.z });
    landed += 1;
  }
  if (landed > 0) {
    if (t.hitStop > 0) must(og.time.hitStop(world, t.hitStop));
    must(og.camera.shake(world, 0.22 + 0.08 * landed));
  }
}

/** A hero's draught: health back, a green rise of light, one fewer in the bag. */
function drink(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV, p: Vector3): void {
  if (g.potions === 0 || me.hp >= me.maxHp) {
    must(og.render.floatingText(world, { text: og.i18n.t(world, g.potions === 0 ? "float.nopotion" : "float.full"), position: { x: p.x, y: p.y + 1.4, z: p.z }, size: 0.32, color: { r: 0.85, g: 0.82, b: 0.75 }, outline: 0.12, rise: 0.5, lifetime: 0.9 }));
    return;
  }
  g.potions -= 1;
  g.drunk += 1;
  const gain = Math.min(me.maxHp - me.hp, t.heal);
  me.hp += gain;
  g.sfx |= Sound.Potion;
  must(og.render.floatingText(world, { text: `+${Math.round(gain)}`, position: { x: p.x, y: p.y + 1.3, z: p.z }, size: 0.5, color: { r: 0.45, g: 1, b: 0.5 }, outline: 0.15, rise: 0.9, lifetime: 1.1, pop: 0.3 }));
  must(world.set(entity(world, `${heroName(me.seat)}-heal`), og.ParticleSystem, { visible: true, startTick: world.tick }));
}

/** A fallen hero rises at the gate, half whole and spared a moment. */
function rise(world: World, g: GameV, t: TuneV, hero: Entity, me: HeroV): void {
  const at = STARTS[me.seat]!;
  me.dead = false;
  me.hp = Math.round(me.maxHp / 2);
  me.invulnerable = 2;
  must(world.set(hero, og.Transform, { position: { x: at.x, y: FEET + 0.05, z: at.z } }));
  release(world, me.body as unknown as Entity);
  must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
  g.sfx |= Sound.Potion;
  must(world.set(entity(world, `${heroName(me.seat)}-heal`), og.ParticleSystem, { visible: true, startTick: world.tick }));
  void t;
}

/**
 * Experience for every standing hero; each level reached brings health, a
 * harder blow, a sparkle and a fanfare. `live` is the row of a hero its
 * caller holds and writes back itself (the one whose blow earned it).
 */
export function award(world: World, g: GameV, t: TuneV, xp: number, live: HeroV | null = null): void {
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const me = live !== null && live.seat === seat ? live : must(world.get(hero, Hero));
    if (me.dead || me.level >= MAX_LEVEL) continue;
    me.xp += xp;
    let up = false;
    while (me.level < MAX_LEVEL && me.xp >= xpFor(t, me.level)) {
      me.xp -= xpFor(t, me.level);
      me.level += 1;
      me.maxHp += t.healthPerLevel;
      up = true;
    }
    if (me.level >= MAX_LEVEL) me.xp = 0;
    if (up) {
      me.hp = me.maxHp;
      g.levelUps += 1;
      g.sfx |= Sound.LevelUp;
      const p = must(world.get(hero, og.Transform)).position;
      must(og.render.floatingText(world, { text: og.i18n.t(world, "float.level", { n: me.level }), position: { x: p.x, y: p.y + 1.6, z: p.z }, size: 0.55, color: { r: 1, g: 0.82, b: 0.35 }, outline: 0.16, rise: 1.1, lifetime: 1.6, pop: 0.4 }));
      const name = heroName(seat);
      must(world.set(entity(world, `${name}-rise`), og.ParticleSystem, { visible: true, startTick: world.tick }));
    }
    if (me !== live) must(world.set(hero, Hero, me));
  }
}

/** A hero takes a risen's blow (src/foes.ts): health lost, a red flash, a shove; nothing left, it falls. */
export function hurtHero(world: World, g: GameV, t: TuneV, hero: Entity, amount: number, from: { x: number; z: number }): boolean {
  const me = must(world.get(hero, Hero));
  if (me.dead || me.invulnerable > 0) return false;
  const p = must(world.get(hero, og.Transform)).position;
  me.hp = Math.max(0, me.hp - amount);
  me.hurt = 0.3;
  me.invulnerable = 0.5;
  const dx = p.x - from.x;
  const dz = p.z - from.z;
  const d = Math.max(0.01, hypot(dx, dz));
  me.knock = 0.2;
  me.knockX = dx / d;
  me.knockZ = dz / d;
  g.hurts += 1;
  g.sfx |= Sound.Hurt;
  must(world.set(me.body as unknown as Entity, og.Model, { flashColor: { r: 1, g: 0.15, b: 0.1 } }));
  must(og.render.floatingText(world, { text: `-${Math.round(amount)}`, position: { x: p.x, y: p.y + 1.3, z: p.z }, size: 0.5, color: { r: 1, g: 0.32, b: 0.25 }, outline: 0.15, rise: 0.8, spread: 0.3, lifetime: 1, pop: 0.25 }));
  const post = must(world.resource(og.PostEffects));
  post.flash = 0.35;
  post.flashColor = { r: 0.75, g: 0.04, b: 0.02 };
  must(og.camera.shake(world, 0.45));
  if (t.hitStop > 0) must(og.time.hitStop(world, Math.max(1, t.hitStop - 1)));
  if (me.hp <= 0) {
    me.dead = true;
    me.attack = 0;
    me.knock = 0;
    me.respawn = t.respawn;
    oneShot(world, me.body as unknown as Entity, rigOf(world, MODELS[me.seat]!), "die", 1);
    must(world.remove(hero, og.Perceivable));
  }
  must(world.set(hero, Hero, me));
  return true;
}

/** First person: the seat's blade before its eye, swung through the strike. */
function blade(world: World, me: HeroV, p: Vector3, look: { yaw: number; pitch: number }): void {
  const name = heroName(me.seat);
  const vm = world.find(`${name}-blade`);
  if (vm === null) return;
  const t = me.attack > 0 ? Math.min(1, me.attack / 0.3) : 0;
  // Rest low on the right; a swing sweeps it across to the left and down.
  const side = 0.32 - 0.55 * t;
  const down = -0.3 - 0.1 * t;
  const fwd = 0.55 + 0.15 * math.sin(t * Math.PI);
  const sy = math.sin(look.yaw), cy = math.cos(look.yaw);
  const eye = { x: p.x, y: p.y + EYE, z: p.z };
  const f = { x: -sy, z: -cy };
  const r = { x: cy, z: -sy };
  const at = { x: eye.x + f.x * fwd + r.x * side, y: eye.y + down + look.pitch * 0.4, z: eye.z + f.z * fwd + r.z * side };
  const yawQ = og.transform.fromAxisAngle(UP, look.yaw - 0.5 + 1.4 * t);
  const tilt = og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -1.0 - 0.9 * t + look.pitch);
  must(world.set(vm, og.Transform, { position: at, quaternion: qmul(yawQ, tilt) }));
}

/** a * b: the rotation b, then a. */
function qmul(a: { x: number; y: number; z: number; w: number }, b: { x: number; y: number; z: number; w: number }) {
  return {
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  };
}
