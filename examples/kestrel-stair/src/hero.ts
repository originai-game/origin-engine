// The courier: a capsule on og.CharacterController carrying the skinned
// body (models/courier.glb, the library's clips retargeted at cook time,
// run by the blend tree of src/anim.ts) and a red cape (og.Cloth pinned at
// the shoulders, its pins following the body as it turns). WASD or the
// arrows (the left stick) walk relative to the view, Shift (a full stick)
// runs, Space (A) jumps, J or the left button (X) strikes with the sword:
// the hit lands on the attack clip's `hit` marker (og.AnimationEvent), an
// og.physics.overlap in front of the courier on the foes' layer.

import { og, defineSystem, CHARACTER_JUMPED, CHARACTER_LANDED, SHAPE_CAPSULE, SHAPE_SPHERE } from "@origingame/engine";
import type { ClipAsset, Entity, Value, World } from "@origingame/engine";
import { courierRig } from "./anim.ts";
import { Ending, Foe, Game, Hero, MAX_HP, OneByOne, Phase, Sound, Tuning } from "./components.ts";
import { begin, flush } from "./state.ts";
import { strikeFoe } from "./foes.ts";
import { START } from "./layout.ts";
import { entity, LAYER, must } from "./world.ts";

type GameV = Value<typeof Game>;
type TuneV = Value<typeof Tuning>;

/** The capsule: radius 0.3, a 1.1 m segment (1.7 m tall), its centre this far above the feet. */
export const FEET = 0.85;
const UP = { x: 0, y: 1, z: 0 };

/** The courier's one-shot clips (the library's, retargeted by the cook), kept on its Hero row. */
export function heroClip(world: World, which: "attack" | "hurt" | "die"): ClipAsset {
  const h = must(world.get(entity(world, "hero"), Hero));
  return (which === "attack" ? h.clipAttack : which === "hurt" ? h.clipHurt : h.clipDie) as unknown as ClipAsset;
}

export function spawnHero(world: World, g: GameV): void {
  const rig = courierRig(world);
  const hero = must(world.spawn({ name: "hero", position: { x: START.x, y: START.y + FEET + 0.05, z: START.z } }));
  must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 1.1, layers: LAYER.hero }));
  must(world.insert(hero, og.CharacterController, { maxSlope: 45, stepHeight: 0.32, snapDistance: 0.3, coyoteTime: 0.12, jumpSpeed: 6.4 }));
  must(world.insert(hero, Hero, { clipAttack: rig.clips.attack as unknown as number, clipHurt: rig.clips.hurt as unknown as number, clipDie: rig.clips.die as unknown as number }));
  must(world.insert(hero, og.Perceivable, { team: 1, eyeHeight: 0.6 }));
  g.hero = hero as unknown as number;
  const body = must(world.instantiate(must(og.load.gltf(world, "models/courier.glb", { castShadow: true })), { name: "hero-body", parent: hero, position: { x: 0, y: -FEET, z: 0 } }));
  must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
  must(world.insert(body, og.AnimationPlayer, { clip: rig.clips.attack!, loop: false, weight: 0, fade: 0.08 }));
  // The cape: a child of the body, pinned along the shoulders behind the back (its pinned corner
  // the left shoulder), hanging to the knees, turned half round so the cloth's x runs from the
  // courier's left shoulder to its right. Its pins follow the body as it walks and turns
  // (FINDINGS #10, fixed: a cloth on a child entity hangs from its world transform).
  const cape = must(og.render.standardMaterial(world, { map: must(og.load.texture(world, "textures/cape.png")), roughness: 0.9, side: "double", sheen: 0.8, sheenColor: { r: 0.8, g: 0.3, b: 0.25 }, sheenRoughness: 0.6 }));
  const c = must(world.spawn({ name: "hero-cape", parent: body, position: { x: 0.24, y: 1.52, z: -0.2 }, quaternion: og.transform.fromAxisAngle(UP, Math.PI) }));
  must(world.insert(c, og.Cloth, { columns: 7, rows: 12, spacing: 0.08, pins: 1, iterations: 10, damping: 0.9, wind: { x: 0, y: 0, z: -0.6 }, gust: 0.5, radius: 0.03, thickness: 0, material: cape }));
  // The strike's sparks.
  const sparks = must(world.spawn({ name: "hero-sparks" }));
  must(world.insert(sparks, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/hit.ogt")), seed: 7, visible: false }));
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

/** Puts the courier at (x, y, z) facing `facing`, still, the view behind it. */
export function placeHero(world: World, g: GameV, x: number, y: number, z: number, facing: number): void {
  const hero = g.hero as unknown as Entity;
  must(world.set(hero, og.Transform, { position: { x, y: y + FEET + 0.05, z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  must(world.set(hero, Hero, { airTime: 0, stepClock: 0, fall: 0 }));
  g.facing = facing;
  g.yaw = wrap(facing + Math.PI);
  g.knock = 0;
  g.attack = 0;
  g.heroX = x;
  g.heroY = y;
  g.heroZ = z;
}

/** The courier is hurt: strength lost, the world holds a beat, a red flash, a shake, thrown back. */
export function hurtHero(world: World, g: GameV, t: TuneV, damage: number, from: { x: number; z: number }, heavy: boolean): void {
  if (g.invulnerable > 0 || g.phase !== Phase.Play || g.ending !== Ending.None) return;
  const dx = g.heroX - from.x;
  const dz = g.heroZ - from.z;
  const d = Math.max(0.01, Math.hypot(dx, dz));
  g.hp = Math.max(0, g.hp - damage);
  g.hurts += 1;
  g.knock = heavy ? 0.35 : 0.2;
  g.knockX = dx / d;
  g.knockZ = dz / d;
  g.invulnerable = heavy ? 1.2 : 0.8;
  g.attack = 0;
  g.sfx |= Sound.Hurt;
  must(og.time.hitStop(world, heavy ? t.heavyStop : t.hitStop));
  const post = must(world.resource(og.PostEffects));
  post.flash = heavy ? 0.5 : 0.35;
  post.flashColor = { r: 0.8, g: 0.08, b: 0.04 };
  must(og.camera.shake(world, heavy ? 0.55 : 0.3));
  must(og.camera.kick(world, { x: g.knockX, y: 0, z: g.knockZ }, heavy ? 0.3 : 0.15));
  must(og.render.floatingText(world, { text: `-${Math.round(damage)}`, position: { x: g.heroX, y: g.heroY + 2.1, z: g.heroZ }, size: 0.45, color: { r: 1, g: 0.25, b: 0.2 }, rise: 1.1, lifetime: 0.9 }));
  const body = entity(world, "hero-body");
  // The hurt clip over the blend tree; with no strength left, the fall.
  must(world.set(body, og.AnimationPlayer, { clip: heroClip(world, g.hp > 0 ? "hurt" : "die"), time: 0, weight: 1, loop: false, speed: 1 }));
}

/**
 * Per tick: reads the controls, moves the capsule, turns the body, drives
 * the graph, swings the sword and lands its hit, and makes the steps, the
 * jumps and the landings heard.
 */
export const HeroMove = defineSystem(
  "Hero",
  {
    reads: [og.Input, og.Gamepad, Tuning, og.CharacterEvent, og.AnimationEvent, OneByOne],
    writes: [Game, Hero, Foe, og.CharacterController, og.Transform, og.AnimationGraph, og.AnimationPlayer, og.ParticleSystem, og.PostEffects, og.Camera, og.NavAgent, og.AiAgent, og.Mesh, og.Collider, og.Skeleton, og.AudioSource, og.PointLight],
    after: ["Ui"],
  },
  (_q, time, world) => {
    const g = begin(world);
    try {
      const t = must(world.resource(Tuning));
      const hero = g.hero as unknown as Entity;
      const inp = must(world.resource(og.Input));
      const gp = must(world.resource(og.Gamepad));
      const playing = g.phase === Phase.Play && g.ending === Ending.None && !g.held;
      // Pause before the time gate, so a press during a hit stop still pauses.
      if (playing && (inp.pressed("Escape") || inp.pressed("KeyP") || gp.pressed("start"))) g.paused = true;
      // A strike pressed is kept for a fifth of a second (og-feel §1's buffer), so a press during a
      // hit stop, a knock-back or the end of a swing still swings when it can.
      if (playing && (inp.pressed("KeyJ") || inp.buttonPressed("left") || gp.pressed("x"))) g.strikeBuffer = 0.2;
      else if (g.strikeBuffer > 0) g.strikeBuffer = Math.max(0, g.strikeBuffer - time.dt);
      const dt = time.scaledDt;
      const p = must(world.get(hero, og.Transform)).position;
      g.heroX = p.x;
      g.heroY = p.y - FEET;
      g.heroZ = p.z;
      if (dt === 0) return;
      const body = entity(world, "hero-body");
      const cc = must(world.get(hero, og.CharacterController));
      const me = must(world.get(hero, Hero));

      // The view turns with Q and E (the right stick); the controls follow the view.
      const turnIn = inp.axis("KeyQ", "KeyE") + gp.axis("rightX");
      if (playing && Math.abs(turnIn) > 0.15) g.yaw = wrap(g.yaw - turnIn * 2.2 * dt);
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
      const mag = Math.min(1, Math.hypot(ix, iy));
      const run = analog ? mag > 0.85 : inp.down("ShiftLeft") || inp.down("ShiftRight");
      // The view looks along -z turned by yaw: forward is where the camera faces.
      const fwd = { x: -Math.sin(g.yaw), z: -Math.cos(g.yaw) };
      const right = { x: Math.cos(g.yaw), z: -Math.sin(g.yaw) };
      let dx = 0;
      let dz = 0;
      if (mag > 0.01) {
        const n = Math.hypot(ix, iy);
        dx = (right.x * ix + fwd.x * iy) / n;
        dz = (right.z * ix + fwd.z * iy) / n;
      }

      // The sword: a press starts a swing on this tick (the player's own attack has no wind-up).
      const strike = playing && g.knock === 0 && g.strikeBuffer > 0;
      if (strike && (g.attack === 0 || g.attack > t.swing * 0.75)) {
        g.strikeBuffer = 0;
        g.attack = 0.0001;
        g.struck = false;
        g.swings += 1;
        g.sfx |= Sound.Swing;
        if (mag > 0.01) g.facing = Math.atan2(dx, dz);
        // A little help with aim: the swing turns to the nearest standing foe within 3 m.
        const aim = aimAt(world, g);
        if (aim !== null) g.facing = aim;
        must(world.set(body, og.AnimationPlayer, { clip: heroClip(world, "attack"), time: 0, weight: 1, loop: false, speed: 0.55 / t.swing }));
      }
      let speed = 0;
      let face: number | null = null;
      if (g.knock > 0) {
        speed = 6 * (g.knock / 0.35);
        dx = g.knockX;
        dz = g.knockZ;
        g.knock = Math.max(0, g.knock - dt);
      } else if (playing && mag > 0.01) {
        speed = (run ? t.runSpeed : t.walkSpeed) * (analog && !run ? mag / 0.85 : 1);
        // A swing roots the courier for its first half.
        if (g.attack > 0 && g.attack < t.swing * 0.5) speed *= 0.25;
        face = Math.atan2(dx, dz);
      }
      const jump = playing && g.knock === 0 && (inp.down("Space") || gp.down("a"));
      must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed }, jump, jumpSpeed: t.jumpSpeed }));
      if (g.invulnerable > 0) g.invulnerable = Math.max(0, g.invulnerable - dt);
      const post = must(world.resource(og.PostEffects));
      if (post.flash > 0) post.flash = Math.max(0, post.flash - dt * 2.5);

      // The swing's hit lands on the attack clip's `hit` marker (og.AnimationEvent, read the tick
      // after og.anim passed it); the swing's end gives the pose back to the graph.
      if (g.attack > 0) {
        g.attack += dt;
        for (const ev of world.events(og.AnimationEvent)) {
          if (ev.entity !== body || ev.name !== "hit" || g.struck) continue;
          g.struck = true;
          landStrike(world, g, t);
        }
        if (g.attack >= t.swing) {
          g.attack = 0;
          must(world.set(body, og.AnimationPlayer, { weight: 0 }));
        }
      } else if (g.hp > 0) {
        const pl = must(world.get(body, og.AnimationPlayer));
        if (pl.weight > 0 && pl.finished) must(world.set(body, og.AnimationPlayer, { weight: 0 }));
      }

      // The body turns towards where it goes (not while thrown back).
      if (face !== null) g.facing = turn(g.facing, face, 12 * dt);
      // The view swings slowly behind a walking courier.
      if (playing && speed > 0.5 && Math.abs(turnIn) < 0.15) g.yaw = turn(g.yaw, wrap(g.facing + Math.PI), 0.9 * dt * Math.min(1, speed / t.walkSpeed));
      must(world.set(body, og.Transform, { quaternion: og.transform.fromAxisAngle(UP, g.facing) }));

      // The graph: speed across the ground, and whether it is in the air.
      const v = cc.velocity;
      const hs = Math.hypot(v.x, v.z);
      g.speed = hs;
      const airTime = cc.grounded ? 0 : me.airTime + dt;
      must(world.set(body, og.AnimationGraph, { p0: hs, p1: airTime > 0.15 ? 1 : 0 }));

      // Steps, jumps and landings.
      let stepClock = me.stepClock;
      if (cc.grounded && hs > 0.5) {
        stepClock += hs * dt;
        const stride = hs > 4.5 ? 1.7 : 1.15;
        if (stepClock >= stride) {
          stepClock -= stride;
          g.steps += 1;
          g.sfx |= Sound.Step;
        }
      } else if (speed > 0 && hs < 0.5) {
        // Setting off is heard at once: the first step lands on the press.
        stepClock = 1.7;
      }
      for (const e of world.events(og.CharacterEvent)) {
        if (e.entity !== hero) continue;
        if (e.kind === CHARACTER_JUMPED) {
          g.jumps += 1;
          g.sfx |= Sound.Jump;
          must(og.tween.squash(world, { entity: body, impact: { x: 0.9, y: 1.1, z: 0.9 }, duration: 0.12 })); // FINDINGS #4, fixed: z too
        } else if (e.kind === CHARACTER_LANDED) {
          g.landings += 1;
          g.sfx |= Sound.Land;
          const hard = e.speed < -7;
          must(og.tween.squash(world, { entity: body, impact: hard ? { x: 1.18, y: 0.8, z: 1.18 } : { x: 1.08, y: 0.92, z: 1.08 }, duration: hard ? 0.18 : 0.12 }));
          if (hard) must(og.camera.shake(world, 0.15));
        }
      }
      g.grounded = cc.grounded;
      must(world.set(hero, Hero, { airTime, stepClock, fall: cc.grounded ? 0 : Math.min(me.fall, v.y) }));
      if (g.hp > 0 && g.hp < MAX_HP && g.phase === Phase.Play) {
        // No regeneration: strength comes back from hillwort and the waystones.
      }
    } finally {
      flush(g);
    }
  },
);

/** The heading to the nearest standing foe within 3 m (those in front first), or null. */
function aimAt(world: World, g: GameV): number | null {
  const near = og.physics.overlap(world, { shape: { shape: SHAPE_SPHERE, radius: 3 }, position: { x: g.heroX, y: g.heroY + 1, z: g.heroZ }, filter: { mask: LAYER.foe } });
  if (!near.ok) return null;
  let best: number | null = null;
  let bestD = Infinity;
  for (const e of near.value) {
    const f = world.get(e, Foe);
    if (!f.ok || f.value.dying > 0) continue;
    const at = must(world.get(e, og.Transform)).position;
    const dx = at.x - g.heroX;
    const dz = at.z - g.heroZ;
    const d = Math.hypot(dx, dz);
    const h = Math.atan2(dx, dz);
    // The nearest wins, a foe behind counting as twice as far as one in front.
    const score = d * (1 + Math.abs(wrap(h - g.facing)) / Math.PI);
    if (score >= bestD) continue;
    bestD = score;
    best = h;
  }
  return best;
}

/** The swing's hit: every foe in an arc in front of the courier takes it. */
function landStrike(world: World, g: GameV, t: TuneV): void {
  const fx = Math.sin(g.facing);
  const fz = Math.cos(g.facing);
  const at = { x: g.heroX + fx * t.reach * 0.6, y: g.heroY + 1.0, z: g.heroZ + fz * t.reach * 0.6 };
  const hits = must(og.physics.overlap(world, { shape: { shape: SHAPE_SPHERE, radius: t.reach * 0.75 }, position: at, filter: { mask: LAYER.foe } }));
  let landed = false;
  for (const e of hits) {
    if (strikeFoe(world, g, t, e, { x: g.heroX, z: g.heroZ })) landed = true;
  }
  if (!landed) return;
  g.hits += 1;
  const sparks = entity(world, "hero-sparks");
  must(world.set(sparks, og.Transform, { position: { x: at.x + fx * 0.3, y: at.y, z: at.z + fz * 0.3 } }));
  must(world.set(sparks, og.ParticleSystem, { visible: true, startTick: world.tick }));
}
