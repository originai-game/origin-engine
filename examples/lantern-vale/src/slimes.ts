// The slimes: three skinned jelly domes (models/slime.glb, a transmissive
// material) bouncing on og.AnimationPlayer with stepFps 12 (on twos), each
// with an expression bubble over it: an og.Sprite drawing a layer of the
// texture array sprites/faces.ogt, standing in the 3D set with a drop
// shadow. A slime patrols between two points; one that sees the hero
// within Tuning.sight turns alert ("!") and bounds after it; past 1.6 times
// that it loses the hero ("?") and goes back to its round. Its touch costs
// a heart: hit stop, a red flash, a shake, the hero thrown back, a "-1"
// rising, the hurt sound. At night they doze ("z").

import { og, defineSystem } from "@origingame/engine";
import type { ClipAsset, Entity, Value, World } from "@origingame/engine";
import { Ending, Face, Game, Phase, Slime, Sound, Tuning } from "./components.ts";
import { FEET } from "./hero.ts";
import { say } from "./ui.ts";
import { groundY, must } from "./world.ts";

type GameV = Value<typeof Game>;

/** The rounds: from a to b and back (x, z). */
export const ROUNDS: readonly [number, number, number, number][] = [
  [-24, -2, -8, 3],
  [10, 0, 26, 5],
  [4, -24, 22, -20],
];
/** The hold of each pose: 12 a second is on twos at 24 fps. */
export const STEP_FPS = 12;
/** How near counts as a touch, and how long the hero is spared after one. */
const TOUCH = 0.95;
const SPARED = 1.6;
const KNOCK = 0.3;
const UP = { x: 0, y: 1, z: 0 };

/** The slime's clips (from og.load.animation), by name. */
function clips(world: World): { idle: ClipAsset; chase: ClipAsset; skeleton: Value<typeof og.Skeleton>["asset"] } {
  const anim = must(og.load.animation(world, "models/slime.glb"));
  const find = (name: string) => {
    const c = anim.clips.find((x) => x.name === name);
    if (!c) throw new Error(`models/slime.glb has no clip "${name}" (it has ${anim.clips.map((x) => x.name).join(", ")})`);
    return c.clip;
  };
  return { idle: find("idle"), chase: find("chase"), skeleton: anim.skeleton };
}

export function spawnSlimes(world: World): void {
  const g = must(world.resource(Game));
  const model = must(og.load.gltf(world, "models/slime.glb"));
  const rig = clips(world);
  // The bubbles' texture array. og.load.textureArray("sprites/faces.ogt") is
  // missing from the runtime (FINDINGS.md), so the same layers come from an
  // `array true` flipbook sheet, whose FlipbookInfo.array is the array.
  const faces = must(og.load.flipbook(world, "sprites/faces-sheet.ogt"));
  ROUNDS.forEach(([ax, az, bx, bz], i) => {
    const y = groundY(world, g, ax, az);
    const e = must(world.instantiate(model, { name: `slime-${i}`, position: { x: ax, y, z: az } }));
    must(world.insert(e, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
    must(world.insert(e, og.AnimationPlayer, { clip: rig.idle, loop: true, fade: 0.12, stepFps: STEP_FPS, speed: 1 + i * 0.07 }));
    must(world.insert(e, Slime, { index: i, ax, az, bx, bz, leg: 1, idleClip: rig.idle as unknown as number, chaseClip: rig.chase as unknown as number }));
    // The bubble: a child sprite in the xy plane, standing over the slime, its shadow cast on the ground.
    const face = must(world.spawn({ name: `slime-${i}-face`, position: { x: ax, y: y + 1.05, z: az } }));
    must(world.insert(face, og.Sprite, { array: faces.array, arrayLayer: Face.Calm, width: 0.9, height: 0.9, anchor: { x: 0.5, y: 0 }, dropShadow: 0.45, layer: 1, color: { r: 0.62, g: 0.6, b: 0.58 } }));
  });
}

/** Puts every slime back at the start of its round, calm. */
function resetSlimes(world: World, g: GameV): void {
  ROUNDS.forEach(([ax, az], i) => {
    const e = world.find(`slime-${i}`);
    if (e === null) return;
    must(world.set(e, og.Transform, { position: { x: ax, y: groundY(world, g, ax, az), z: az } }));
    must(world.set(e, Slime, { leg: 1, chasing: false, face: Face.Calm, faceTime: 0, bounce: 0 }));
  });
}

/** A slime touched the hero: a heart, the hit's weight, the hero thrown back (away from `at`). */
export function hurt(world: World, g: GameV, t: Value<typeof Tuning>, at: { x: number; y: number; z: number }, hero: { x: number; y: number; z: number }): void {
  const dx = hero.x - at.x;
  const dz = hero.z - at.z;
  const d = Math.max(0.01, Math.hypot(dx, dz));
  g.hearts -= 1;
  g.hurts += 1;
  g.knock = KNOCK;
  g.knockX = dx / d;
  g.knockZ = dz / d;
  g.invulnerable = SPARED;
  g.sfx |= Sound.Hurt | Sound.Hit;
  if (t.hitStop > 0) must(og.time.hitStop(world, t.hitStop));
  const post = must(world.resource(og.PostEffects));
  post.flash = 0.5;
  post.flashColor = { r: 0.85, g: 0.08, b: 0.05 };
  must(og.camera.shake(world, 0.4));
  must(og.render.floatingText(world, { text: say(world, "hud.ouch"), position: { x: hero.x, y: hero.y + 1.3, z: hero.z }, size: 0.7, color: { r: 1, g: 0.25, b: 0.2 }, rise: 1.1, lifetime: 0.9 }));
}

export const Slimes = defineSystem(
  "Slimes",
  { reads: [Tuning], writes: [Game, Slime, og.Transform, og.Sprite, og.AnimationPlayer, og.PostEffects, og.Camera], after: ["Lanterns"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    if (g.start) resetSlimes(world, g);
    const dt = time.scaledDt;
    if (dt === 0) return;
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    const heroAt = must(world.get(g.hero as unknown as Entity, og.Transform)).position;
    const hero = { x: heroAt.x, y: heroAt.y - FEET, z: heroAt.z };
    for (let i = 0; i < ROUNDS.length; i++) {
      const e = world.find(`slime-${i}`);
      if (e === null) continue;
      const s = must(world.get(e, Slime));
      const p = must(world.get(e, og.Transform)).position;
      const d = Math.hypot(hero.x - p.x, hero.z - p.z);
      let { chasing, face, faceTime, leg, heading, bounce, hits } = s;
      const idle = s.idleClip as unknown as ClipAsset;
      const chaseClip = s.chaseClip as unknown as ClipAsset;
      faceTime += dt;
      const night = g.phase === Phase.Over || g.ending === Ending.Night;
      if (playing && !chasing && bounce <= 0 && g.invulnerable <= 0 && d < t.sight) {
        chasing = true;
        face = Face.Alert;
        faceTime = 0;
        g.sfx |= Sound.Alert;
      } else if (chasing && (!playing || d > t.sight * 1.6)) {
        chasing = false;
        face = Face.Lost;
        faceTime = 0;
      } else if (face === Face.Lost && faceTime > 1.5) {
        face = Face.Calm;
      }
      if (night) face = Face.Sleepy;
      else if (face === Face.Sleepy) face = Face.Calm;
      // Where it heads: the hero, back from it after a touch, or the end of its leg.
      let tx: number;
      let tz: number;
      let speed: number;
      if (bounce > 0) {
        bounce = Math.max(0, bounce - dt);
        tx = p.x - (hero.x - p.x);
        tz = p.z - (hero.z - p.z);
        speed = t.patrolSpeed;
      } else if (chasing) {
        tx = hero.x;
        tz = hero.z;
        speed = t.chaseSpeed;
      } else {
        const r = ROUNDS[i]!;
        tx = leg === 0 ? r[0] : r[2];
        tz = leg === 0 ? r[1] : r[3];
        speed = night ? 0 : t.patrolSpeed;
        if (Math.hypot(tx - p.x, tz - p.z) < 0.4) leg = 1 - leg;
      }
      const vx = tx - p.x;
      const vz = tz - p.z;
      const len = Math.hypot(vx, vz);
      let nx = p.x;
      let nz = p.z;
      if (len > 0.05 && speed > 0) {
        const step = Math.min(len, speed * dt);
        nx += (vx / len) * step;
        nz += (vz / len) * step;
        heading = Math.atan2(vx, vz);
      }
      const ny = groundY(world, g, nx, nz);
      must(world.set(e, og.Transform, { position: { x: nx, y: ny, z: nz }, quaternion: og.transform.fromAxisAngle(UP, heading) }));
      const face3 = world.find(`slime-${i}-face`);
      if (face3 !== null) must(world.set(face3, og.Sprite, { arrayLayer: face }));
      if (face3 !== null) must(world.set(face3, og.Transform, { position: { x: nx, y: ny + 1.05, z: nz } }));
      if (chasing !== s.chasing) must(world.set(e, og.AnimationPlayer, { clip: chasing ? chaseClip : idle }));
      if (playing && g.invulnerable <= 0 && d < TOUCH && Math.abs(hero.y - ny) < 1.2) {
        hurt(world, g, t, { x: nx, y: ny, z: nz }, heroAt);
        chasing = false;
        bounce = 0.9;
        hits += 1;
        must(world.set(e, og.AnimationPlayer, { clip: idle }));
      }
      must(world.set(e, Slime, { chasing, face, faceTime, leg, heading, bounce, hits }));
    }
  },
);
