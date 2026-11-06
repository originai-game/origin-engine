// The risen: four skeleton minions in the yard and their captain by the
// crypt's chest (src/layout.ts FOES), each on the keep's navmesh
// (og.NavAgent, og.Crowd so they part round each other) and thinking with
// its behaviour (og.ai: assets/ai/minion.ogt and captain.ogt), watching
// with og.Perception (the walls and pillars hide the party; of two heroes
// it hunts the nearest it sees). They wander until a hero comes into
// sight, raise `alarm` and chase; within reach they stop and raise
// `strike`: the game winds the blow up (a red glint, the body turning to
// its mark), plays the clip and lands it on the clip's `hit` marker on a
// hero still in front and in reach (src/heroes.ts hurtHero).
//
// A hero's blow (strikeFoe) costs health, flashes and squashes the body,
// throws it back a step and flinches it; at nothing it falls: its mind
// stops, the quest counts it (og.story.progress), every standing hero
// gains its experience, and it drops gold or a draught into the loot pool,
// picked up by walking over it.

import { og, defineSystem, math } from "@origingame/engine";
import type { Entity, Value, World } from "@origingame/engine";
import { animate, oneShot, release, rigOf } from "./anim.ts";
import { Ending, Foe, Game, Hero, Kind, Loot, LootKind, Phase, Sound, Tuning } from "./components.ts";
import { award, damageOf, entity, heroOf, hurtHero, hypot, lazyTuning } from "./heroes.ts";
import { FOES, NAV_WARD } from "./layout.ts";
import { SEATS } from "./seats.ts";
import { QUEST } from "./story.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;
type FoeV = Value<typeof Foe>;
type HeroV = Value<typeof Hero>;

const UP = { x: 0, y: 1, z: 0 };
/** The models by kind (tools/compose.py) and their scale: a minion a little shorter than the heroes, the captain a head taller. */
const MODELS = ["skeleton", "captain"] as const;
const SCALE = [0.66, 0.86];
/** Seconds of the wind-up before the clip, the clip that strikes, its speed and the strike's whole length, by kind. */
const WINDUP = [0.32, 0.2];
const CLIP = ["attack", "slam"] as const;
const CLIP_SPEED = [0.8, 1.05];
const STRIKE = [0.95, 1.5];
/** How far in front a strike reaches past the foe's own reach, and how wide. */
const STRIKE_SLACK = 0.55;
const STRIKE_ARC = 70;
/** Loot slots (loot-0 to loot-7). */
export const LOOT_SLOTS = 8;
/** How near a hero walks to pick loot up. */
const PICK = 1.1;

/** The foe's behaviour states (assets/ai/*.ogt): index 1 and 2 are the hunt. */
const HUNT_STATES = [1, 2];

export function spawnFoes(world: World): void {
  const g = must(world.resource(Game));
  const t = must(world.resource(Tuning));
  const minds = [must(og.ai.load(world, "ai/minion.ogt")), must(og.ai.load(world, "ai/captain.ogt"))];
  for (const m of minds) if (m.states[0] !== "idle" || m.states[1] !== "hunt" || m.states[2] !== "strike") throw new Error(`a behaviour's states are ${m.states.join(", ")}: idle, hunt, strike, search`);
  const models = MODELS.map((m) => must(og.load.gltf(world, `models/${m}.glb`)));
  FOES.forEach((f, i) => {
    const at = og.nav.nearest(world, g.nav as never, { x: f.x, y: 0, z: f.z }) ?? { x: f.x, y: 0, z: f.z };
    const e = must(world.spawn({ name: `foe-${i}`, position: at }));
    const captain = f.kind === Kind.Captain;
    must(world.insert(e, og.NavAgent, { nav: g.nav as never, target: at, seeking: false, speed: captain ? 1.1 : 1.3, acceleration: 14, radius: captain ? 0.5 : 0.4, stopDistance: 0.3, faceMovement: true, turnSpeed: captain ? 6 : 9 }));
    must(world.insert(e, og.Crowd, {}));
    // The gate's ward is not theirs to walk (src/layout.ts WARD).
    must(world.insert(e, og.NavFilter, { excluded: [NAV_WARD] }));
    must(world.insert(e, og.AiAgent, { behaviour: minds[f.kind]!.behaviour, active: true, thinkEvery: 2 }));
    must(world.insert(e, og.Perception, { sight: t.sight, fov: 150, eyeHeight: 1.4, occlude: true, team: 2, memory: 3, hearing: 4 }));
    const s = SCALE[f.kind]!;
    const rig = rigOf(world, MODELS[f.kind]!);
    const body = must(world.instantiate(models[f.kind]!, { name: `foe-${i}-body`, parent: e, scale: { x: s, y: s, z: s } }));
    animate(world, body, rig);
    // The wind-up's glint: a red light in the chest, lit while it gathers the blow.
    const glint = must(world.spawn({ name: `foe-${i}-glint`, parent: e, position: { x: 0, y: captain ? 1.5 : 1.15, z: 0.25 } }));
    must(world.insert(glint, og.PointLight, { color: { r: 1, g: 0.2, b: 0.08 }, intensity: 0, distance: 4, castShadow: false }));
    const hp = captain ? t.captainHealth : t.minionHealth;
    must(world.insert(e, Foe, { index: i, kind: f.kind, hp, maxHp: hp, walk: captain ? 1.0 : 1.2, chase: captain ? 2.5 : 3.1, reach: captain ? 1.7 : 1.35, body: body as unknown as number, homeX: at.x, homeZ: at.z }));
  });
  // The loot pool: a coin and a draught per slot, hidden until a risen drops one there.
  const coin = must(og.load.gltf(world, "models/keep/coin.gltf"));
  const bottle = must(og.load.gltf(world, "models/keep/bottle_A_brown.gltf"));
  for (let i = 0; i < LOOT_SLOTS; i++) {
    const e = must(world.spawn({ name: `loot-${i}`, position: { x: 0, y: -5, z: 0 } }));
    must(world.instantiate(coin, { name: `loot-${i}-gold`, parent: e, scale: { x: 1.1, y: 1.1, z: 1.1 } }));
    must(world.instantiate(bottle, { name: `loot-${i}-potion`, parent: e, scale: { x: 0.8, y: 0.8, z: 0.8 } }));
    const light = must(world.spawn({ name: `loot-${i}-light`, parent: e, position: { x: 0, y: 0.4, z: 0 } }));
    must(world.insert(light, og.PointLight, { color: { r: 1, g: 0.8, b: 0.4 }, intensity: 0, distance: 2.5, castShadow: false }));
    must(world.insert(e, Loot, {}));
    showLoot(world, i, null);
  }
}

/** Shows slot `i` as `kind` (null hides it). */
function showLoot(world: World, i: number, kind: number | null): void {
  must(world.set(entity(world, `loot-${i}-gold`), og.Model, { visible: kind === LootKind.Gold }));
  must(world.set(entity(world, `loot-${i}-potion`), og.Model, { visible: kind === LootKind.Potion }));
  must(world.set(entity(world, `loot-${i}-light`), og.PointLight, { intensity: kind === null ? 0 : kind === LootKind.Gold ? 2.2 : 1.6, color: kind === LootKind.Potion ? { r: 0.5, g: 1, b: 0.55 } : { r: 1, g: 0.8, b: 0.4 } }));
}

/** Every risen back where it rose, whole and calm, and the loot gone (a new run or the title). */
function reset(world: World, t: TuneV): void {
  FOES.forEach((_, i) => {
    const e = entity(world, `foe-${i}`);
    const f = must(world.get(e, Foe));
    const at = { x: f.homeX, y: 0, z: f.homeZ };
    must(world.set(e, og.Transform, { position: at, quaternion: og.transform.fromAxisAngle(UP, 0) }));
    must(world.set(e, og.NavAgent, { target: at, seeking: false, drive: true }));
    must(world.set(e, og.AiAgent, { active: true }));
    must(og.ai.restart(world, e));
    const body = f.body as unknown as Entity;
    release(world, body);
    must(world.set(body, og.Transform, { position: { x: 0, y: 0, z: 0 } }));
    must(world.set(body, og.Model, { visible: true, flash: 0 }));
    const hp = f.kind === Kind.Captain ? t.captainHealth : t.minionHealth;
    must(world.set(e, Foe, { hp, maxHp: hp, attack: 0, landed: false, stagger: 0, dying: 0, flash: 0, oneShot: false, alert: false, strikes: 0 }));
  });
  for (let i = 0; i < LOOT_SLOTS; i++) {
    must(world.set(entity(world, `loot-${i}`), Loot, { active: false }));
    showLoot(world, i, null);
  }
}

/** The foe a command names: its index in FOES (as a playtest types it) or its entity (what og.ai sends). */
export function foeOf(world: World, args: readonly string[]): Entity {
  const n = Number(args[0]);
  const e = (Number.isInteger(n) && n >= 0 && n < FOES.length ? entity(world, `foe-${n}`) : n) as Entity;
  if (!world.get(e, Foe).ok) throw new Error(`${args[0]} is not a risen (0 to ${FOES.length - 1}, or a risen's entity)`);
  return e;
}

/** The nearest standing hero to a point, and how far. */
function nearestHero(world: World, g: GameV, x: number, z: number): { hero: Entity; x: number; z: number; d: number } | null {
  let best: { hero: Entity; x: number; z: number; d: number } | null = null;
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    if (must(world.get(hero, Hero)).dead) continue;
    const p = must(world.get(hero, og.Transform)).position;
    const d = hypot(p.x - x, p.z - z);
    if (best === null || d < best.d) best = { hero, x: p.x, z: p.z, d };
  }
  return best;
}

/** `alarm <risen>`: it has seen a hero (its behaviour raises it on entering the hunt). */
export function alarm(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const g = must(world.resource(Game));
  const f = must(world.get(e, Foe));
  if (g.phase !== Phase.Play || g.ending !== Ending.None || f.dying > 0) return;
  g.sfx |= Sound.Alarm;
  const at = must(world.get(e, og.Transform)).position;
  must(og.render.floatingText(world, { text: "!", position: { x: at.x, y: at.y + (f.kind === Kind.Captain ? 2.4 : 1.9), z: at.z }, size: 0.8, color: { r: 1, g: 0.32, b: 0.15 }, outline: 0.15, rise: 0.5, spread: 0, lifetime: 0.9 }));
}

/** `strike <risen>`: it is within reach of a hero (its behaviour raises it): the wind-up starts. */
export function strike(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const g = must(world.resource(Game));
  const f = must(world.get(e, Foe));
  if (g.phase !== Phase.Play || g.ending !== Ending.None || f.dying > 0 || f.attack > 0) return;
  must(world.set(e, Foe, { attack: 1e-4, landed: false, strikes: f.strikes + 1, oneShot: false }));
  must(world.set(e, og.NavAgent, { drive: false }));
}

/**
 * A hero's blow reaches risen `e`: its health less the hero's damage, the
 * flash and squash, the numbers, a step back and a flinch; at nothing it
 * falls (fall).
 */
export function strikeFoe(world: World, g: GameV, t: TuneV, me: HeroV, e: Entity, from: { x: number; z: number }): void {
  const f = must(world.get(e, Foe));
  if (f.hp <= 0) return;
  const at = must(world.get(e, og.Transform)).position;
  const dx = at.x - from.x;
  const dz = at.z - from.z;
  const d = Math.max(0.01, hypot(dx, dz));
  const amount = damageOf(t, me.level);
  const hp = Math.max(0, f.hp - amount);
  const captain = f.kind === Kind.Captain;
  g.hits += 1;
  g.sfx |= captain ? Sound.Clang | Sound.Hit : Sound.Hit;
  g.target = e as unknown as number;
  g.targetTicks = 0;
  const body = f.body as unknown as Entity;
  must(og.tween.flash(world, { entity: body, amount: 0.75, duration: 0.14 }));
  must(og.tween.squash(world, { entity: body, impact: { x: 1.12, y: 0.86, z: 1.12 }, duration: 0.16 }));
  must(og.camera.kick(world, { x: dx / d, y: 0, z: dz / d }, 0.1));
  must(og.render.floatingText(world, { text: `${Math.round(amount)}`, position: { x: at.x, y: at.y + (captain ? 2.1 : 1.6), z: at.z }, size: hp <= 0 ? 0.62 : 0.5, color: hp <= 0 ? { r: 1, g: 0.85, b: 0.3 } : { r: 1, g: 1, b: 0.94 }, outline: 0.15, rise: 1, spread: 0.35, lifetime: 0.85, pop: 0.3, crit: hp <= 0 }));
  const sparks = world.find("sparks");
  if (sparks !== null) {
    must(world.set(sparks, og.Transform, { position: { x: at.x - (dx / d) * 0.3, y: at.y + (captain ? 1.3 : 1.0), z: at.z - (dz / d) * 0.3 } }));
    must(world.set(sparks, og.ParticleSystem, { visible: true, startTick: world.tick }));
  }
  if (hp <= 0) {
    must(world.set(e, Foe, { hp: 0 }));
    fall(world, g, t, me, e, f);
    return;
  }
  // A step back and a flinch; the captain only staggers, and keeps a strike it has begun.
  const stagger = captain ? 0.12 : 0.32;
  must(world.set(e, Foe, { hp, stagger, flash: 0.14, oneShot: f.attack === 0 || !captain ? true : f.oneShot, attack: captain ? f.attack : 0 }));
  must(world.set(e, og.NavAgent, { drive: false }));
  if (f.attack === 0 || !captain) oneShot(world, body, rigOf(world, MODELS[f.kind]!), "hurt", 1.4);
  if (!captain && f.attack > 0) must(world.set(entity(world, `foe-${f.index}-glint`), og.PointLight, { intensity: 0 }));
}

/** `slay:<risen>`: it falls now, as to seat 0's blow (a playtest's shortcut through the fight). */
export function slay(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const g = must(world.resource(Game));
  const f = must(world.get(e, Foe));
  if (g.phase !== Phase.Play || f.hp <= 0) return;
  must(world.set(e, Foe, { hp: 0 }));
  const hero = heroOf(g, 0);
  const me = must(world.get(hero, Hero));
  fall(world, g, must(world.resource(Tuning)), me, e, f);
  must(world.set(hero, Hero, me));
}

/** Risen `e` falls: its die clip, its mind stopped, the quest counts it, the party its experience, and its drop. */
function fall(world: World, g: GameV, t: TuneV, me: HeroV, e: Entity, f: FoeV): void {
  const captain = f.kind === Kind.Captain;
  must(world.set(e, Foe, { dying: 1e-4, attack: 0, stagger: 0 }));
  must(world.set(e, og.AiAgent, { active: false }));
  must(world.set(e, og.NavAgent, { seeking: false, drive: false }));
  must(world.set(entity(world, `foe-${f.index}-glint`), og.PointLight, { intensity: 0 }));
  oneShot(world, f.body as unknown as Entity, rigOf(world, MODELS[f.kind]!), "die", 1);
  g.slain += 1;
  g.sfx |= Sound.Bones;
  must(og.time.hitStop(world, captain ? 12 : 7));
  must(og.camera.punch(world, captain ? 0.12 : 0.06));
  og.story.progress(world, QUEST, captain ? "captain" : "risen");
  me.kills += 1;
  award(world, g, t, captain ? t.captainXp : t.minionXp, me);
  // The drop: gold always, or a draught by chance (og.Rng: the same on every peer); the captain both.
  const rng = must(world.resource(og.Rng));
  const at = must(world.get(e, og.Transform)).position;
  const potion = captain || rng.next() < t.potionChance;
  drop(world, g, LootKind.Gold, captain ? 40 : 8 + Math.floor(rng.next() * 8), at.x + 0.35, at.z);
  if (potion) drop(world, g, LootKind.Potion, 1, at.x - 0.35, at.z + 0.2);
}

function drop(world: World, g: GameV, kind: number, amount: number, x: number, z: number): void {
  const i = g.loot % LOOT_SLOTS;
  g.loot += 1;
  const e = entity(world, `loot-${i}`);
  must(world.set(e, og.Transform, { position: { x, y: 0.25, z } }));
  must(world.set(e, Loot, { active: true, kind, amount, baseY: 0.25, born: world.tick }));
  showLoot(world, i, kind);
}

/** A strike lands on the clip's `hit`: the nearest standing hero still in front and in reach takes the blow. */
function land(world: World, g: GameV, t: TuneV, e: Entity, f: FoeV): void {
  const captain = f.kind === Kind.Captain;
  const q = must(world.get(e, og.Transform));
  const yaw = 2 * math.atan2(q.quaternion.y, q.quaternion.w);
  const fx = math.sin(yaw);
  const fz = math.cos(yaw);
  g.sfx |= captain ? Sound.Crush : Sound.Swing;
  if (captain) must(og.camera.shake(world, 0.3));
  for (let seat = 0; seat < SEATS; seat++) {
    const hero = heroOf(g, seat);
    const p = must(world.get(hero, og.Transform)).position;
    const hx = p.x - q.position.x;
    const hz = p.z - q.position.z;
    const d = hypot(hx, hz);
    const front = d < 0.01 ? 1 : (hx * fx + hz * fz) / d;
    if (d > f.reach + STRIKE_SLACK || front < math.cos((STRIKE_ARC * Math.PI) / 180)) continue;
    hurtHero(world, g, t, hero, captain ? t.captainDamage : t.minionDamage, { x: q.position.x, z: q.position.z });
  }
}

/**
 * Per tick: the risen's sight and paces from Tuning, the graph's speed,
 * the wind-ups, clips and blows of their strikes, the reeling after a
 * blow, the fallen sinking away, and the loot bobbing and picked up.
 */
export const Foes = defineSystem(
  "Foes",
  { reads: [og.AnimationEvent], writes: [Game, Tuning, Foe, Hero, Loot, og.Transform, og.NavAgent, og.AiAgent, og.Perception, og.AnimationGraph, og.AnimationPlayer, og.Model, og.PointLight, og.ParticleSystem, og.PostEffects, og.Camera, og.CharacterController, og.Perceivable, og.Rng], after: ["Heroes"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = lazyTuning(world);
    if (g.start) {
      reset(world, t);
      return;
    }
    const dt = time.scaledDt;
    if (dt === 0) return;
    const hunting = g.phase === Phase.Play && g.ending === Ending.None;
    const hits = new Set<number>();
    for (const ev of world.events(og.AnimationEvent)) if (ev.name === "hit") hits.add(ev.entity as unknown as number);
    if (g.target !== 0) g.targetTicks += 1;
    FOES.forEach((_, i) => {
      const e = entity(world, `foe-${i}`);
      const f = must(world.get(e, Foe));
      const body = f.body as unknown as Entity;
      const captain = f.kind === Kind.Captain;
      if (f.dying > 0) {
        // Fallen: the die clip plays out, then it sinks into the flagstones and is gone.
        const dying = f.dying + dt;
        must(world.set(e, Foe, { dying }));
        if (dying > 2 && dying < 4) must(world.set(body, og.Transform, { position: { x: 0, y: -(dying - 2) * 0.6, z: 0 } }));
        if (dying >= 4 && f.dying < 4) must(world.set(body, og.Model, { visible: false }));
        return;
      }
      const agent = must(world.get(e, og.NavAgent));
      const ai = must(world.get(e, og.AiAgent));
      const sight = hunting ? t.sight : 0;
      if (must(world.get(e, og.Perception)).sight !== sight) must(world.set(e, og.Perception, { sight }));
      const v = agent.velocity;
      const speed = Math.round(hypot(v.x, v.z) * 20) / 20;
      let { attack, landed, stagger, oneShot: shot } = f;
      const rig = rigOf(world, MODELS[f.kind]!);
      const glint = entity(world, `foe-${i}-glint`);
      // Reeling: thrown back along the blow, the agent's motion the game's.
      if (stagger > 0) {
        stagger = Math.max(0, stagger - dt);
        const p = must(world.get(e, og.Transform)).position;
        const near = nearestHero(world, g, p.x, p.z);
        if (near !== null && near.d > 0.01) {
          const k = (captain ? 1.5 : 3.2) * dt;
          const away = { x: (p.x - near.x) / near.d, z: (p.z - near.z) / near.d };
          const to = og.nav.nearest(world, g.nav as never, { x: p.x + away.x * k, y: p.y, z: p.z + away.z * k });
          if (to !== null) must(world.set(e, og.Transform, { position: to }));
        }
        if (stagger === 0 && attack === 0) must(world.set(e, og.NavAgent, { drive: true }));
      }
      if (attack > 0) {
        attack += dt;
        const p = must(world.get(e, og.Transform)).position;
        const w = WINDUP[f.kind]!;
        // The wind-up: it turns to its mark and the glint gathers; then the clip.
        if (attack < w) {
          const near = nearestHero(world, g, p.x, p.z);
          if (near !== null) must(world.set(e, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, math.atan2(near.x - p.x, near.z - p.z)) }));
          must(world.set(glint, og.PointLight, { intensity: 10 * (attack / w) }));
        } else if (attack - dt < w) {
          oneShot(world, body, rig, CLIP[f.kind]!, CLIP_SPEED[f.kind]!);
          shot = false;
          must(world.set(glint, og.PointLight, { intensity: 0 }));
        }
        if (!landed && hits.has(body as unknown as number) && attack >= w) {
          landed = true;
          if (hunting) land(world, g, t, e, f);
        }
        if (attack >= STRIKE[f.kind]!) {
          attack = 0;
          release(world, body);
          if (stagger === 0) must(world.set(e, og.NavAgent, { drive: true }));
        }
      } else if (shot) {
        const pl = must(world.get(body, og.AnimationPlayer));
        if (pl.finished) {
          shot = false;
          release(world, body);
        }
      }
      must(world.set(body, og.AnimationGraph, { p0: attack > 0 ? 0 : speed }));
      const alert = hunting && HUNT_STATES.includes(ai.state);
      must(world.set(e, Foe, { attack, landed, stagger, oneShot: shot, speed, alert, flash: Math.max(0, f.flash - dt) }));
    });
    if (g.targetTicks > 300) {
      g.target = 0;
      g.targetTicks = 0;
    }
    // The loot: it bobs and turns, and the first hero to walk over it takes it into the party's bag.
    for (let i = 0; i < LOOT_SLOTS; i++) {
      const e = entity(world, `loot-${i}`);
      const l = must(world.get(e, Loot));
      if (!l.active) continue;
      const p = must(world.get(e, og.Transform)).position;
      const age = (world.tick - l.born) / 60;
      must(world.set(e, og.Transform, { position: { x: p.x, y: l.baseY + 0.12 * math.sin(age * 3), z: p.z }, quaternion: og.transform.fromAxisAngle(UP, age * 2) }));
      if (!hunting || age < 0.4) continue;
      const near = nearestHero(world, g, p.x, p.z);
      if (near === null || near.d > PICK) continue;
      must(world.set(e, Loot, { active: false }));
      showLoot(world, i, null);
      const hp = must(world.get(near.hero, og.Transform)).position;
      let text: string;
      if (l.kind === LootKind.Gold) {
        g.gold += l.amount;
        g.sfx |= Sound.Coin;
        text = og.i18n.t(world, "float.gold", { n: l.amount });
      } else {
        g.potions += 1;
        g.sfx |= Sound.Coin;
        text = og.i18n.t(world, "float.potion");
      }
      must(og.render.floatingText(world, { text, position: { x: hp.x, y: hp.y + 1.2, z: hp.z }, size: 0.38, color: l.kind === LootKind.Gold ? { r: 1, g: 0.84, b: 0.35 } : { r: 0.55, g: 1, b: 0.6 }, outline: 0.13, rise: 0.8, lifetime: 1.1 }));
    }
  },
);
