// The sentinels: three stone stouts walking their rounds of the lake path
// on the vale's navmesh (og.NavAgent), each thinking with its behaviour
// (og.ai, assets/ai/sentinel-<name>.ogt) and watching with og.Perception
// (sight, a cone, the terrain and the plinths hide the hero). Seen, a
// sentinel raises `alarm` and chases; within reach it raises `grab`: the
// hero loses a heart and is thrown back. The navmesh stops at the shore, so
// the lake is a refuge. Their walk is the library's clips retargeted onto
// the stout (src/anim.ts), its speed the graph's parameter.

import { og, defineSystem } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { sentinelRig } from "./anim.ts";
import { Ending, Game, Phase, Sentinel, Sound, Tuning } from "./components.ts";
import { entity, FEET } from "./hero.ts";
import { ground, must } from "./world.ts";

/** The sentinels, by behaviour (assets/ai/sentinel-<name>.ogt). */
export const SENTINELS = ["west", "north", "east"] as const;
/** The stout's scale: a head taller than the hero. */
const BODY_SCALE = 1.25;
/** Seconds the hero is thrown back by a grab, and how long it cannot be grabbed again. */
const KNOCK = 0.35;
const SPARED = 1.6;

/** The first point of each sentinel's round (tools/make-assets.py ROUTES writes the rounds). */
const ROUND_STARTS = [{ x: -27, z: 12 }, { x: -20, z: -31 }, { x: 27, z: -22 }];
/** The behaviours' `alert` and `grab` states: indexes into BehaviourInfo.states, the same in all three. */
const ALERT_STATES = [1, 2];

export function spawnSentinels(world: World): void {
  const g = must(world.resource(Game));
  const rig = sentinelRig(world);
  const model = must(og.load.gltf(world, "models/stout.glb"));
  const eyeLight = { color: { r: 1, g: 0.18, b: 0.08 }, intensity: 0, distance: 6, castShadow: false };
  SENTINELS.forEach((name, i) => {
    const mind = must(og.ai.load(world, `ai/sentinel-${name}.ogt`));
    if (mind.states[1] !== "alert" || mind.states[2] !== "grab") throw new Error(`ai/sentinel-${name}.ogt: states ${mind.states.join(", ")}`);
    const first = ROUND_STARTS[i]!;
    const at = { x: first.x, y: ground(world, g, first.x, first.z).height, z: first.z };
    const s = must(world.spawn({ name: `sentinel-${i}`, position: at }));
    must(world.insert(s, og.NavAgent, { nav: g.nav as never, target: at, seeking: false, speed: 1.6, acceleration: 12, radius: 0.45, stopDistance: 0.3, faceMovement: true, turnSpeed: 6 }));
    must(world.insert(s, og.AiAgent, { behaviour: mind.behaviour, active: true, thinkEvery: 2 }));
    must(world.insert(s, og.Perception, { sight: 13, fov: 120, eyeHeight: 1.6, occlude: true, team: 2, memory: 3 }));
    must(world.insert(s, Sentinel, { index: i }));
    const body = must(world.instantiate(model, { name: `sentinel-${i}-body`, parent: s, scale: { x: BODY_SCALE, y: BODY_SCALE, z: BODY_SCALE } }));
    must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(body, og.AnimationGraph, { asset: rig.graph, speed: 1 }));
    const eye = must(world.spawn({ name: `sentinel-${i}-eye`, parent: s, position: { x: 0, y: 2.05, z: 0.3 } }));
    must(world.insert(eye, og.PointLight, eyeLight));
  });
}

/** Puts the sentinels back at the start of their rounds, calm (a new run or the title). */
function reset(world: World, g: { terrain: number }): void {
  SENTINELS.forEach((_, i) => {
    const s = entity(world, `sentinel-${i}`);
    const first = ROUND_STARTS[i]!;
    const at = { x: first.x, y: ground(world, g, first.x, first.z).height, z: first.z };
    must(world.set(s, og.Transform, { position: at }));
    must(world.set(s, og.NavAgent, { target: at, seeking: false }));
    must(world.set(s, Sentinel, { alert: false }));
  });
}

/**
 * Per tick: the sentinels' sight and pace from Tuning, the graph's speed
 * from how fast each walks, the body's feet on the terrain (the navmesh
 * lies a little above or below it), and the eye's light while it chases.
 */
export const Sentinels = defineSystem(
  "Sentinels",
  { reads: [Tuning, og.AiAgent], writes: [Game, Sentinel, og.NavAgent, og.Transform, og.Perception, og.AnimationGraph, og.PointLight], after: ["Hero"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) reset(world, g);
    const pulse = 0.75 + 0.25 * Math.sin(time.elapsed * 9);
    SENTINELS.forEach((_, i) => {
      const s = entity(world, `sentinel-${i}`);
      const nav = must(world.get(s, og.NavAgent));
      const ai = must(world.get(s, og.AiAgent));
      const me = must(world.get(s, Sentinel));
      const p = must(world.get(s, og.Transform)).position;
      const speed = Math.hypot(nav.velocity.x, nav.velocity.z);
      const alert = g.phase === Phase.Play && ALERT_STATES.includes(ai.state);
      if (alert !== me.alert || Math.abs(speed - me.speed) > 1e-3) must(world.set(s, Sentinel, { speed, alert }));
      must(world.set(s, og.Perception, { sight: t.sight }));
      const body = entity(world, `sentinel-${i}-body`);
      must(world.set(body, og.AnimationGraph, { p0: speed }));
      must(world.set(body, og.Transform, { position: { x: 0, y: ground(world, g, p.x, p.z).height - p.y, z: 0 } }));
      must(world.set(entity(world, `sentinel-${i}-eye`), og.PointLight, { intensity: alert ? 14 * pulse : 0 }));
    });
  },
);

/**
 * The sentinel a command names: its index, 0 to 2 (SENTINELS' order, as a
 * playtest types it), or its entity handle (what og.ai's `command` sends
 * first; no sentinel's handle is that small).
 */
function sentinel(world: World, args: readonly string[]): { me: Entity; at: { x: number; y: number; z: number } } {
  const n = Number(args[0]);
  const me = (Number.isInteger(n) && n >= 0 && n < SENTINELS.length ? entity(world, `sentinel-${n}`) : n) as Entity;
  const t = world.get(me, og.Transform);
  if (!t.ok || !world.get(me, Sentinel).ok) throw new Error(`${args[0]} is not a sentinel (0 to ${SENTINELS.length - 1}, or a sentinel's entity)`);
  return { me, at: t.value.position };
}

/** `alarm <sentinel>`: it has seen the hero (its behaviour raises it entering `alert`). */
export function alarm(world: World, args: readonly string[]): void {
  const { me, at } = sentinel(world, args);
  const g = must(world.resource(Game));
  if (g.phase !== Phase.Play || g.ending !== Ending.None) return;
  const s = must(world.get(me, Sentinel));
  must(world.set(me, Sentinel, { alarms: s.alarms + 1 }));
  g.alarms += 1;
  g.sfx |= Sound.Alarm;
  must(og.render.floatingText(world, { text: "!", position: { x: at.x, y: at.y + 2.7, z: at.z }, size: 0.9, color: { r: 1, g: 0.3, b: 0.15 }, rise: 0.6, spread: 0, lifetime: 1 }));
}

/**
 * `grab <sentinel>`: it reached the hero (its behaviour raises it within
 * reach). The hero loses a heart and is thrown back, the world holds for a
 * beat, the screen flashes red and shakes; then the hero is spared a moment.
 */
export function grab(world: World, args: readonly string[]): void {
  const { me, at } = sentinel(world, args);
  const g = must(world.resource(Game));
  if (g.phase !== Phase.Play || g.ending !== Ending.None || g.invulnerable > 0) return;
  const hero = must(world.get(g.hero as unknown as Entity, og.Transform)).position;
  if (Math.hypot(hero.x - at.x, hero.z - at.z) > 2.2 || hero.y - FEET > at.y + 1.5) return;
  const s = must(world.get(me, Sentinel));
  must(world.set(me, Sentinel, { grabs: s.grabs + 1 }));
  const dx = hero.x - at.x;
  const dz = hero.z - at.z;
  const d = Math.max(0.01, Math.hypot(dx, dz));
  g.hearts -= 1;
  g.hurts += 1;
  g.grabs += 1;
  g.knock = KNOCK;
  g.knockX = dx / d;
  g.knockZ = dz / d;
  g.invulnerable = SPARED;
  g.sfx |= Sound.Grab;
  const t = must(world.resource(Tuning));
  if (t.hitStop > 0) must(og.time.hitStop(world, t.hitStop));
  const post = must(world.resource(og.PostEffects));
  post.flash = 0.45;
  post.flashColor = { r: 0.8, g: 0.05, b: 0.02 };
  must(og.camera.shake(world, 0.55));
}
