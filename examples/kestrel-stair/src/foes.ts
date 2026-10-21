// The foes: thorn imps and stone wardens, each an instantiated skinned
// model on the navmesh (og.NavAgent, with og.Steering's separation and
// collider avoidance), minded by og.ai (assets/ai/<name>.ogt: the imps'
// tree patrols, chases, bites and flees; the wardens' machine patrols,
// alerts, closes in, slams and searches) and seeing through og.Perception.
// Their behaviours raise the game's commands `bite`, `alert` and `slam`
// (src/main.ts), handled here. The warden's slam is a long telegraphed
// wind-up (1.4 s clip, the hit at 0.82 s) whose lunge is root motion.
//
// A strike from the courier (src/hero.ts) costs a foe a thorn or a stone:
// hit stop by weight, a white flash, a shake, sparks, a number, and the foe
// thrown back (its NavAgent leaves the motion to the game a moment). An imp
// that has lost most of its thorns flees; a foe with none falls (its die
// clip), and the terrace warden drops the third sigil.

import { og, defineSystem, BODY_KINEMATIC, SHAPE_CAPSULE } from "@origingame/engine";
import type { Behaviour, ClipAsset, Entity, Value, World } from "@origingame/engine";
import { foeRig } from "./anim.ts";
import { Ending, Foe, Game, Hero, Kind, OneByOne, Phase, Sound, Tuning } from "./components.ts";
import { hurtHero } from "./hero.ts";
import { FOES, MIND } from "./layout.ts";
import { say } from "./text.ts";
import { entity, LAYER, must } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

/** The bite's and the slam's lengths (tools/rig.ts); their hits are the clips' `hit` markers. */
const BITE_LENGTH = 0.5;
const SLAM_LENGTH = 1.4;

export function spawnFoes(world: World, g: GameV): void {
  const rigs = { imp: foeRig(world, "imp"), warden: foeRig(world, "warden") };
  const models = { imp: must(og.load.gltf(world, "models/imp.glb", { castShadow: true })), warden: must(og.load.gltf(world, "models/warden.glb", { castShadow: true })) };
  const chitter = must(og.load.sound(world, "audio/chitter.wav"));
  const slam = must(og.load.sound(world, "audio/slam.ogt"));
  const dust = must(og.load.emitter(world, "fx/dust.ogt"));
  FOES.forEach((f, i) => {
    const rig = rigs[f.kind];
    // Loaded twice: og.AiAgent starts a machine afresh when its behaviour changes, so a new run swaps handles.
    const mind = must(og.ai.load(world, `ai/${f.name}.ogt`));
    const spare = must(og.ai.load(world, `ai/${f.name}.ogt`));
    const [x, y, z] = f.route[0]!;
    const e = must(world.instantiate(models[f.kind], { name: f.name, position: { x, y, z } }));
    const imp = f.kind === "imp";
    must(world.insert(e, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(e, og.AnimationGraph, { asset: rig.graph }));
    must(world.insert(e, og.AnimationPlayer, { clip: rig.clips[imp ? "bite" : "slam"]!, loop: false, weight: 0, fade: 0.1 }));
    must(world.insert(e, og.RigidBody, { kind: BODY_KINEMATIC }));
    must(world.insert(e, og.Collider, { shape: SHAPE_CAPSULE, radius: imp ? 0.35 : 0.55, height: imp ? 0.4 : 1.3, offset: { x: 0, y: imp ? 0.55 : 1.2, z: 0 }, layers: LAYER.foe }));
    // Seeking its round's next point from the first tick, its mind off until it has planned once:
    // an agent that has never planned is moved from navmesh polygon 0, whatever its height (FINDINGS #4).
    const [nx, ny, nz] = f.route[1]!;
    must(world.insert(e, og.NavAgent, { nav: g.nav as never, target: { x: nx, y: ny, z: nz }, seeking: true, speed: imp ? 1.8 : 1.1, acceleration: imp ? 18 : 8, radius: imp ? 0.4 : 0.6, stopDistance: 0.3, faceMovement: true, turnSpeed: imp ? 10 : 5 }));
    must(world.insert(e, og.Steering, { separation: 1.5, separationRadius: 1.6, avoidance: 1.2, avoidDistance: 1.8, avoidMask: LAYER.level | LAYER.gate }));
    must(world.insert(e, og.AiAgent, { behaviour: mind.behaviour, active: false, thinkEvery: 2 }));
    must(world.insert(e, og.Perception, { sight: 11, fov: imp ? 140 : 110, eyeHeight: imp ? 0.9 : 2.2, occlude: true, occluders: LAYER.level | LAYER.gate, team: 2, memory: imp ? 2 : 4, hearing: 5 }));
    must(world.insert(e, Foe, {
      kind: imp ? Kind.Imp : Kind.Warden,
      index: i,
      hp: imp ? MIND.impHp : MIND.wardenHp,
      carries: f.name === "warden-0",
      homeX: x, homeY: y, homeZ: z,
      graph: rig.graph as unknown as number,
      clipAttack: rig.clips[imp ? "bite" : "slam"] as unknown as number,
      clipHurt: rig.clips.hurt as unknown as number,
      clipDie: rig.clips.die as unknown as number,
      mind: mind.behaviour as unknown as number,
      spare: spare.behaviour as unknown as number,
    }));
    // Heard from where it is: an imp's chitter loops; a warden's slam is a one-shot at the warden.
    must(world.insert(e, og.AudioSource, imp
      ? { sound: chitter, bus: "sfx", loop: true, spatial: true, refDistance: 1.5, maxDistance: 22, rolloffFactor: 1.4, occlude: true, volume: 0.45, priority: 90 }
      : { sound: slam, bus: "sfx", playing: false, spatial: true, refDistance: 3, maxDistance: 40, volume: 1, priority: 210 }));
    if (!imp) {
      const puff = must(world.spawn({ name: `${f.name}-dust`, parent: e, position: { x: 0, y: 0.1, z: 1.3 } }));
      must(world.insert(puff, og.ParticleSystem, { emitter: dust, seed: 40 + i, visible: false }));
    }
  });
}

/** The foe a command names: its entity handle (og.ai sends it) or its index in FOES. */
export function foeOf(world: World, args: readonly string[]): Entity {
  const n = Number(args[0]);
  const e = (Number.isInteger(n) && n >= 0 && n < FOES.length ? entity(world, FOES[n]!.name) : n) as Entity;
  if (!world.get(e, Foe).ok) throw new Error(`${args[0]} is not a foe (0 to ${FOES.length - 1}, or a foe's entity)`);
  return e;
}

function alive(f: Value<typeof Foe>): boolean {
  return f.dying === 0;
}

/** `bite`: an imp within reach after its wind-up: the lunge starts, the bite lands on the clip's hit frame. */
export function bite(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const f = must(world.get(e, Foe));
  const g = must(world.resource(Game));
  if (!alive(f) || f.slam > 0 || g.phase !== Phase.Play) return;
  must(world.set(e, Foe, { slam: 0.0001, slamHit: false, cooldown: 1.1 }));
  must(world.set(e, og.AnimationPlayer, { clip: f.clipAttack as unknown as ClipAsset, time: 0, weight: 1, loop: false }));
}

/** `alert`: a warden has seen the courier. */
export function alert(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const f = must(world.get(e, Foe));
  const g = must(world.resource(Game));
  if (!alive(f) || g.phase !== Phase.Play || g.ending !== Ending.None) return;
  must(world.set(e, Foe, { alerts: f.alerts + 1 }));
  g.sfx |= Sound.Alert;
  const at = must(world.get(e, og.Transform)).position;
  must(og.render.floatingText(world, { text: "!", position: { x: at.x, y: at.y + 3, z: at.z }, size: 0.9, color: { r: 1, g: 0.35, b: 0.15 }, rise: 0.5, spread: 0, lifetime: 1 }));
  if (f.alerts === 0) og.ui.toast(world, say(world, "toast.warden"), { tone: "warn" });
}

/** `slam`: a warden within reach raises both fists (a long wind-up), lunges and slams. */
export function slam(world: World, args: readonly string[]): void {
  const e = foeOf(world, args);
  const f = must(world.get(e, Foe));
  const g = must(world.resource(Game));
  if (!alive(f) || f.slam > 0 || g.phase !== Phase.Play) return;
  must(world.set(e, Foe, { slam: 0.0001, slamHit: false }));
  // The lunge is the clip's root motion (its hips travel 0.9 m forward): the warden's own
  // Transform moves by it, so the navmesh agent lets go of the motion until the slam ends.
  must(world.set(e, og.NavAgent, { drive: false }));
  must(world.set(e, og.Skeleton, { rootMotion: "planar" }));
  must(world.set(e, og.AnimationPlayer, { clip: f.clipAttack as unknown as ClipAsset, time: 0, weight: 1, loop: false }));
}

/**
 * A strike reaches foe `e`: a thorn or a stone lost, the hit's weight, the
 * foe thrown back; it falls with none left. Answers whether it landed.
 */
export function strikeFoe(world: World, g: GameV, t: TuneV, e: Entity, from: { x: number; z: number }): boolean {
  const f = world.get(e, Foe);
  if (!f.ok || !alive(f.value)) return false;
  const at = must(world.get(e, og.Transform)).position;
  const dx = at.x - from.x;
  const dz = at.z - from.z;
  const d = Math.max(0.01, Math.hypot(dx, dz));
  const warden = f.value.kind === Kind.Warden;
  const hp = f.value.hp - 1;
  must(world.set(e, Foe, { hp, hitsTaken: f.value.hitsTaken + 1, stun: warden ? 0.15 : 0.3, stunX: dx / d, stunZ: dz / d }));
  must(world.set(e, og.NavAgent, { drive: false }));
  g.sfx |= Sound.Hit;
  must(og.time.hitStop(world, warden ? t.heavyStop : t.hitStop));
  must(og.camera.shake(world, warden ? 0.3 : 0.2));
  must(og.camera.kick(world, { x: dx / d, y: 0, z: dz / d }, 0.12));
  must(og.render.floatingText(world, { text: hp <= 0 ? "×" : `${hp}`, position: { x: at.x, y: at.y + (warden ? 2.9 : 1.5), z: at.z }, size: warden ? 0.55 : 0.45, color: { r: 1, g: 1, b: 0.92 }, rise: 1, lifetime: 0.8, crit: hp <= 0 }));
  flash(world, e);
  if (hp <= 0) fall(world, g, e, f.value);
  else if (f.value.slam === 0) must(world.set(e, og.AnimationPlayer, { clip: f.value.clipHurt as unknown as ClipAsset, time: 0, weight: 1, loop: false }));
  return true;
}

/**
 * The hit's flinch on the foe's model. og-feel §3 pairs a hit stop with a flash, but
 * og.tween.flash takes an og.Sprite or an og.Mesh and a glTF instance draws through og.Model
 * ("has neither og.Sprite nor og.Mesh"), and its materials are shared by every instance, so a
 * foe's own flash cannot be drawn (FINDINGS #7): a squash and the sparks stand in for it.
 */
function flash(world: World, e: Entity): void {
  must(og.tween.squash(world, { entity: e, impact: { x: 1.12, y: 0.86 }, duration: 0.16 }));
}

/** Foe `e` falls: its die clip, its mind stopped, its collider gone; the warden's sigil dropped. */
function fall(world: World, g: GameV, e: Entity, f: Value<typeof Foe>): void {
  must(world.set(e, Foe, { dying: 0.0001, slam: 0 }));
  must(world.set(e, og.AiAgent, { active: false }));
  must(world.set(e, og.NavAgent, { seeking: false, drive: false }));
  must(world.set(e, og.Skeleton, { rootMotion: "off" }));
  must(world.set(e, og.Collider, { layers: 0, mask: 0 }));
  must(world.set(e, og.AnimationPlayer, { clip: f.clipDie as unknown as ClipAsset, time: 0, weight: 1, loop: false }));
  if (f.kind === Kind.Imp) must(world.set(e, og.AudioSource, { playing: false }));
  g.foesDown |= 1 << f.index;
  g.kills += 1;
  g.sfx |= Sound.Death;
  must(og.time.hitStop(world, f.kind === Kind.Warden ? 14 : 8));
  must(og.camera.punch(world, f.kind === Kind.Warden ? 0.12 : 0.06));
  if (f.carries) {
    const at = must(world.get(e, og.Transform)).position;
    const s = entity(world, "sigil-2");
    must(world.set(s, og.Transform, { position: { x: at.x, y: at.y + 0.8, z: at.z } }));
    must(world.set(s, og.Mesh, { visible: true }));
    must(world.set(s, og.PointLight, { intensity: 3 }));
    og.ui.toast(world, say(world, "toast.drop"));
  }
}

/** A bite or a slam lands: if the courier is still in front and in reach, it hurts. */
function landAttack(world: World, g: GameV, t: TuneV, e: Entity, name: string, f: Value<typeof Foe>, hunting: boolean): void {
  const imp = f.kind === Kind.Imp;
  const q = must(world.get(e, og.Transform));
  const fx = 2 * (q.quaternion.x * q.quaternion.z + q.quaternion.w * q.quaternion.y);
  const fz = 1 - 2 * (q.quaternion.x * q.quaternion.x + q.quaternion.y * q.quaternion.y);
  const hx = g.heroX - q.position.x;
  const hz = g.heroZ - q.position.z;
  const dist = Math.hypot(hx, hz);
  const front = dist < 0.01 ? 1 : (hx * fx + hz * fz) / dist;
  const reach = imp ? MIND.impReach + 0.35 : MIND.wardenReach + 0.5;
  const level = Math.abs(g.heroY - q.position.y) < 1.4;
  if (!imp) {
    g.sfx |= Sound.Slam;
    must(world.set(e, og.AudioSource, { playing: false }));
    must(world.set(e, og.AudioSource, { playing: true }));
    must(world.set(entity(world, `${name}-dust`), og.ParticleSystem, { visible: true, startTick: world.tick }));
    must(og.camera.shake(world, 0.35));
  } else g.sfx |= Sound.Bite;
  if (hunting && dist < reach && front > 0.2 && level) {
    hurtHero(world, g, t, imp ? t.biteDamage : t.slamDamage, { x: q.position.x, z: q.position.z }, !imp);
    must(world.set(e, Foe, { bites: f.bites + 1 }));
  }
}

/** Every foe back at the start of its round, whole and calm (a new run or the title). */
function reset(world: World, g: GameV): void {
  for (const f of FOES) {
    const e = entity(world, f.name);
    const me = must(world.get(e, Foe));
    const imp = me.kind === Kind.Imp;
    must(world.set(e, og.Transform, { position: { x: me.homeX, y: me.homeY, z: me.homeZ } }));
    // Back to planning before thinking (FINDINGS #4): a teleported agent that is not seeking moves from its old polygon.
    const [nx, ny, nz] = f.route[1]!;
    must(world.set(e, og.NavAgent, { target: { x: nx, y: ny, z: nz }, seeking: true, drive: true }));
    must(world.set(e, og.Skeleton, { rootMotion: "off" }));
    must(world.set(e, og.Collider, { layers: LAYER.foe, mask: 0xffffffff }));
    must(world.set(e, og.AnimationPlayer, { weight: 0 }));
    if (imp) must(world.set(e, og.AudioSource, { playing: true }));
    // A machine (the wardens') that left its patrol starts afresh: its other handle.
    const ai = must(world.get(e, og.AiAgent));
    const swap = !imp && ai.state !== 0;
    must(world.set(e, og.AiAgent, { active: false, ...(swap ? { behaviour: me.spare as unknown as Behaviour } : {}) }));
    const plans = must(world.get(e, og.NavAgent)).plans;
    must(world.set(e, Foe, { hp: imp ? MIND.impHp : MIND.wardenHp, cooldown: 0, stun: 0, slam: 0, slamHit: false, dying: 0, warm: plans, ...(swap ? { mind: me.spare, spare: me.mind } : {}) }));
  }
  g.foesDown = 0;
}

/**
 * Per tick: the foes' pace and sight from Tuning, the graph's speed, the
 * reeling after a strike, the bites and slams landing on their frames, the
 * imps' thorns growing back, and the fallen sinking away.
 */
export const Foes = defineSystem(
  "Foes",
  { reads: [Tuning, og.AnimationEvent, OneByOne], writes: [Game, Foe, Hero, og.Transform, og.NavAgent, og.AiAgent, og.Perception, og.AnimationGraph, og.AnimationPlayer, og.Skeleton, og.Collider, og.AudioSource, og.ParticleSystem, og.PostEffects, og.Camera, og.Mesh, og.PointLight, og.Steering], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) reset(world, g);
    const dt = time.scaledDt;
    if (dt === 0) return;
    const hunting = g.phase === Phase.Play && g.ending === Ending.None;
    // The bites' and slams' `hit` markers og.anim passed last tick (og.AnimationEvent).
    const hits = new Set<number>();
    for (const ev of world.events(og.AnimationEvent)) if (ev.name === "hit") hits.add(ev.entity as unknown as number);
    for (const def of FOES) {
      const e = entity(world, def.name);
      const f = must(world.get(e, Foe));
      const imp = f.kind === Kind.Imp;
      const tr = must(world.get(e, og.Transform));
      const p = tr.position;
      if (f.dying > 0) {
        // Fallen: the die clip plays out, then it sinks into the ground.
        const dying = f.dying + dt;
        must(world.set(e, Foe, { dying }));
        if (dying > 1.6) must(world.set(e, og.Transform, { position: { x: p.x, y: p.y - dt * 0.5, z: p.z } }));
        continue;
      }
      const agent = must(world.get(e, og.NavAgent));
      // Its mind wakes once the agent has planned on the navmesh (FINDINGS #4).
      if (agent.plans > f.warm && !must(world.get(e, og.AiAgent)).active) must(world.set(e, og.AiAgent, { active: true }));
      must(world.set(e, og.Perception, { sight: hunting ? t.sight : 0 }));
      if (agent.speed !== (imp ? t.impPatrol : t.wardenPatrol) && agent.status === 0) must(world.set(e, og.NavAgent, { speed: imp ? t.impPatrol : t.wardenPatrol }));
      const v = agent.velocity;
      must(world.set(e, og.AnimationGraph, { p0: Math.hypot(v.x, v.z) }));
      let { cooldown, stun, slam: attack, slamHit, hp } = f;
      cooldown = Math.max(0, cooldown - dt);
      // Reeling from a strike: thrown back along the strike, the agent's motion left to the game.
      if (stun > 0) {
        stun = Math.max(0, stun - dt);
        const k = (imp ? 5 : 2) * dt;
        must(world.set(e, og.Transform, { position: { x: p.x + f.stunX * k, y: p.y, z: p.z + f.stunZ * k } }));
        if (stun === 0 && attack === 0) must(world.set(e, og.NavAgent, { drive: true }));
      }
      // An attack in progress: its hit lands on the clip's `hit` marker (landAttack, below); it ends with the clip.
      if (attack > 0) {
        attack += dt;
        const length = imp ? BITE_LENGTH : SLAM_LENGTH;
        if (hits.has(e as unknown as number) && !slamHit) {
          slamHit = true;
          landAttack(world, g, t, e, def.name, f, hunting);
        }
        if (attack >= length) {
          attack = 0;
          must(world.set(e, og.AnimationPlayer, { weight: 0 }));
          if (!imp) must(world.set(e, og.Skeleton, { rootMotion: "off" }));
          if (stun === 0) must(world.set(e, og.NavAgent, { drive: true }));
        }
      } else {
        const pl = must(world.get(e, og.AnimationPlayer));
        if (pl.weight > 0 && pl.finished) must(world.set(e, og.AnimationPlayer, { weight: 0 }));
      }
      // An imp's thorns grow back while it keeps away.
      if (imp && hp < MIND.impHp && stun === 0 && attack === 0 && Math.floor(time.elapsed / 6) !== Math.floor((time.elapsed - dt) / 6)) hp += 1;
      must(world.set(e, Foe, { cooldown, stun, slam: attack, slamHit, hp }));
    }
  },
);
