// The hero: a capsule on og.CharacterController carrying the voxel
// lamplighter, and the camera that looks down on it. WASD or the arrows
// (the left stick) walk on the valley's terrain, Shift (a full stick) runs;
// the controller shoves the rocks of the pile (pushBodies). A slime's touch
// throws the hero back (Game.knock, written by src/slimes.ts).

import { og, defineSystem, SHAPE_CAPSULE } from "@origingame/engine";
import type { Entity, Value, Vector3, World } from "@origingame/engine";
import { Ending, Game, Hero, Phase, Rock, Sound, Tuning } from "./components.ts";
import { groundY, LAYER, must, START } from "./world.ts";

type GameV = Value<typeof Game>;

/** The capsule's centre above the feet (radius 0.3, a 1.0 m segment: 1.6 m tall). */
export const FEET = 0.8;
const UP = { x: 0, y: 1, z: 0 };

export function entity(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`${name} is missing`);
  return e;
}

/** Spawns the hero at the start: the capsule and its voxel body. */
export function spawnHero(world: World): void {
  const g = must(world.resource(Game));
  const y = groundY(world, g, START.x, START.z) + FEET + 0.05;
  const hero = must(world.spawn({ name: "hero", position: { x: START.x, y, z: START.z } }));
  must(world.insert(hero, og.Collider, { shape: SHAPE_CAPSULE, radius: 0.3, height: 1.0, layers: LAYER.hero }));
  must(world.insert(hero, og.CharacterController, { maxSlope: 40, stepHeight: 0.35, snapDistance: 0.3, pushBodies: true }));
  must(world.insert(hero, Hero));
  g.hero = hero as unknown as number;
  must(world.instantiate(must(og.load.gltf(world, "models/hero.vox")), { name: "hero-body", parent: hero, position: { x: 0, y: -FEET, z: 0 } }));
}

function wrap(a: number): number {
  while (a > Math.PI) a -= 2 * Math.PI;
  while (a < -Math.PI) a += 2 * Math.PI;
  return a;
}

/** Puts the hero back at the start, facing north. */
function reset(world: World, g: GameV, hero: Entity): void {
  const y = groundY(world, g, START.x, START.z) + FEET + 0.05;
  must(world.set(hero, og.Transform, { position: { x: START.x, y, z: START.z } }));
  must(world.set(hero, og.CharacterController, { move: { x: 0, y: 0, z: 0 }, jump: false }));
  must(world.set(hero, Hero, { airTime: 0, stepClock: 0 }));
  g.facing = Math.PI;
  g.knock = 0;
  g.invulnerable = 0;
  g.kindling = false;
}

/** Puts the pile back as it was set (a new run). */
function restack(world: World): void {
  for (let i = 0; ; i++) {
    const e = world.find(`rock-${i}`);
    if (e === null) break;
    const r = must(world.get(e, Rock));
    const p = must(world.get(e, og.Transform)).position;
    if (Math.hypot(p.x - r.x0, p.z - r.z0) > 0.01) must(world.set(e, og.Transform, { position: { x: r.x0, y: p.y + 0.02, z: r.z0 } }));
    must(world.set(e, og.Velocity, { linear: { x: 0, y: 0, z: 0 }, angular: { x: 0, y: 0, z: 0 } }));
  }
}

/**
 * Per tick: reads the controls, moves the capsule, turns and bobs the body,
 * makes the steps heard, and measures how far the pile was shoved.
 */
export const HeroMove = defineSystem(
  "Hero",
  { reads: [og.Input, og.Gamepad, Tuning, Rock], writes: [Game, Hero, og.CharacterController, og.Transform, og.Velocity, og.PostEffects], after: ["Ui"] },
  (_q, time, world) => {
    const g = must(world.resource(Game));
    const t = must(world.resource(Tuning));
    const hero = g.hero as unknown as Entity;
    if (g.start) {
      reset(world, g, hero);
      restack(world);
    }
    const inp = must(world.resource(og.Input));
    const gp = must(world.resource(og.Gamepad));
    const playing = g.phase === Phase.Play && g.ending === Ending.None;
    // Pause before the time gate, so a press in a hit stop still pauses; not
    // while held (the press that resumed this tick would open it again).
    if (playing && !g.held && (inp.pressed("Escape") || inp.pressed("KeyP") || gp.pressed("start"))) g.paused = true;
    const dt = time.scaledDt;
    if (dt === 0) return;
    const cc = must(world.get(hero, og.CharacterController));
    const me = must(world.get(hero, Hero));
    const p = must(world.get(hero, og.Transform)).position;

    // The controls: the camera looks north, so W is -z.
    let ix = inp.axis("KeyA", "KeyD") + inp.axis("ArrowLeft", "ArrowRight");
    let iz = inp.axis("KeyW", "KeyS") + inp.axis("ArrowUp", "ArrowDown");
    let analog = false;
    const sx = gp.axis("leftX");
    const sy = gp.axis("leftY");
    if (Math.hypot(sx, sy) > 0.2) {
      ix += sx;
      iz += sy;
      analog = true;
    }
    const mag = Math.min(1, Math.hypot(ix, iz));
    const run = analog ? mag > 0.85 : inp.down("ShiftLeft") || inp.down("ShiftRight");
    let dx = 0;
    let dz = 0;
    let speed = 0;
    if (g.knock > 0) {
      speed = 7 * (g.knock / 0.3);
      dx = g.knockX;
      dz = g.knockZ;
      g.knock = Math.max(0, g.knock - dt);
    } else if (playing && !g.kindling && mag > 0.01) {
      const n = Math.hypot(ix, iz);
      dx = ix / n;
      dz = iz / n;
      speed = (run ? t.runSpeed : t.walkSpeed) * (analog && !run ? mag / 0.85 : 1);
      g.facing = wrap(g.facing + Math.max(-12 * dt, Math.min(12 * dt, wrap(Math.atan2(dx, dz) - g.facing))));
    }
    must(world.set(hero, og.CharacterController, { move: { x: dx * speed, y: 0, z: dz * speed } }));
    if (g.invulnerable > 0) g.invulnerable = Math.max(0, g.invulnerable - dt);
    const post = must(world.resource(og.PostEffects));
    if (post.flash > 0) post.flash = Math.max(0, post.flash - dt * 2.5);

    // The body: turned to where it goes, bobbing a little with each stride, blinking while spared.
    const v = cc.velocity;
    const hs = Math.hypot(v.x, v.z);
    g.speed = hs;
    let stepClock = me.stepClock;
    if (cc.grounded && hs > 0.4) {
      stepClock += hs * dt;
      const stride = hs > 4.5 ? 1.5 : 1.05;
      if (stepClock >= stride) {
        stepClock -= stride;
        g.steps += 1;
        g.sfx |= Sound.Step;
      }
    }
    const bob = hs > 0.4 ? Math.abs(Math.sin((stepClock / (hs > 4.5 ? 1.5 : 1.05)) * Math.PI)) * 0.07 : 0;
    const blink = g.invulnerable > 0 && Math.floor(g.invulnerable * 12) % 2 === 0;
    const body = entity(world, "hero-body");
    must(world.set(body, og.Transform, { position: { x: 0, y: -FEET + bob, z: 0 }, quaternion: og.transform.fromAxisAngle(UP, g.facing), scale: blink ? { x: 0.001, y: 0.001, z: 0.001 } : { x: 1, y: 1, z: 1 } }));
    g.grounded = cc.grounded;
    must(world.set(hero, Hero, { airTime: cc.grounded ? 0 : me.airTime + dt, stepClock }));

    // How far the pile has been shoved (a fact the physics playtest reads).
    let moved = 0;
    for (let i = 0; ; i++) {
      const e = world.find(`rock-${i}`);
      if (e === null) break;
      const r = must(world.get(e, Rock));
      const rp = must(world.get(e, og.Transform)).position;
      moved = Math.max(moved, Math.hypot(rp.x - r.x0, rp.z - r.z0));
    }
    if (Math.abs(moved - g.rockMoved) > 1e-4) g.rockMoved = moved;
  },
);

/**
 * The camera: behind the title a slow pass high over the valley; in a run
 * it hangs above and behind the hero (Tuning.cameraHeight, cameraBack),
 * easing after it.
 */
export const View = defineSystem("View", { reads: [Game, og.Transform, Tuning], writes: [og.Camera], after: ["Hero"] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const cam = must(world.resource(og.Camera));
  const t = must(world.resource(Tuning));
  let eye: Vector3;
  let focus: Vector3;
  if (g.phase === Phase.Title || g.phase === Phase.Over) {
    const a = time.elapsed * 0.03;
    focus = { x: Math.sin(a) * 6, y: 0, z: -2 + Math.cos(a) * 4 };
    eye = { x: focus.x - 4, y: 26, z: focus.z + 22 };
  } else {
    const p = must(world.get(g.hero as unknown as Entity, og.Transform)).position;
    focus = { x: p.x, y: p.y, z: p.z - 1 };
    eye = { x: p.x, y: p.y + t.cameraHeight, z: p.z + t.cameraBack };
  }
  const at = cam.position;
  const far = Math.hypot(eye.x - at.x, eye.y - at.y, eye.z - at.z) > 12 || g.start;
  const k = far ? 1 : 1 - Math.exp(-time.dt * 7);
  const pos = { x: at.x + (eye.x - at.x) * k, y: at.y + (eye.y - at.y) * k, z: at.z + (eye.z - at.z) * k };
  cam.position = pos;
  cam.quaternion = og.transform.lookAt(pos, focus);
  cam.fov = 50;
  cam.far = 300;
});
