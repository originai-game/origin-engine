// Motion Course (README.md): og.motion's showcase. Two characters run a
// course through a temple's ruins: the runner walks, runs, brakes into a
// running jump over a fallen drum, walks on, steps up onto the terrace,
// turns to the camera and waves; the porter walks, hops, turns and cheers.
// The simulation owns every path and arc (og.MotionAgent.drive, the
// runner's og.CharacterController); the model draws the motion between the
// goals. The camera follows the runner along the way.
import { defineSystem, og } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { buildSet, TERRACE_TOP, TERRACE_X } from "./set.ts";
import { must, v3, yaw } from "./util.ts";

const EAST = Math.PI / 2;
/** The two lanes: the runner's along the way's near half, the porter's far. */
const RUNNER_Z = 0.1;
const PORTER_Z = -1.5;

function character(world: World, name: string, path: string, at: number, z: number): Entity {
  const anim = must(og.load.animation(world, path));
  const e = must(world.instantiate(must(og.load.gltf(world, path)), { name, position: v3(at, 0, z), quaternion: yaw(90) }));
  // On the way from the first frame, as the set is: drawn whole, not faded in.
  must(og.render.reveal(world, e, 0));
  must(world.insert(e, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" }));
  // The fallback until the motion model is resident: the Meshy idle.
  must(world.insert(e, og.AnimationPlayer, { clip: og.anim.clip(anim, "Idle"), loop: true }));
  must(world.insert(e, og.MotionAgent, { drive: true }));
  return e;
}

function runner(world: World, body: Entity): void {
  const z = RUNNER_Z;
  must(og.motion.walkTo(world, body, { position: v3(2.0, 0, z), speed: 1.4 }));
  must(og.motion.walkTo(world, body, { position: v3(4.9, 0, z), speed: 3.8 }));
  must(og.motion.jump(world, body, { height: 0.55, distance: 1.7, heading: EAST }));
  must(og.motion.walkTo(world, body, { position: v3(TERRACE_X - 0.9, 0, z), speed: 1.4 }));
  must(og.motion.jump(world, body, { height: TERRACE_TOP + 0.35, distance: 1.0, heading: EAST, prepare: 0.35 }));
  must(og.motion.walkTo(world, body, { position: v3(TERRACE_X + 1.6, TERRACE_TOP, z), speed: 1.3 }));
  must(og.motion.face(world, body, { heading: 0 }));
  must(og.motion.pose(world, body, { pose: "wave", duration: 3 }));
}

/** Where the porter sets out. His course ends 7.3 m on, at x 6.9: seen from
 * the camera's last place he cheers in the middle of the gap between two
 * columns of the row behind, his raised arms clear of their shafts and of
 * the pine's trunk beyond. */
const PORTER_FROM = -0.4;

function porter(world: World, body: Entity): void {
  const z = PORTER_Z;
  const x = PORTER_FROM;
  must(og.motion.walkTo(world, body, { position: v3(x + 2.0, 0, z), speed: 1.2 }));
  must(og.motion.jump(world, body, { height: 0.35, heading: EAST }));
  must(og.motion.walkTo(world, body, { position: v3(x + 5.0, 0, z), speed: 1.2 }));
  must(og.motion.jump(world, body, { height: 0.45, distance: 1.2, heading: EAST }));
  must(og.motion.walkTo(world, body, { position: v3(x + 7.3, 0, z), speed: 1.2 }));
  must(og.motion.face(world, body, { heading: 0 }));
  must(og.motion.pose(world, body, { pose: "cheer", duration: 4 }));
}

function light(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true, shadowDistance: 60 }));
  const day = must(world.resource(og.DayNight));
  day.enabled = true;
  day.start = 9.2;
  day.speed = 0;
  // The morning sun from behind the colonnade's left, so the cast stand
  // in its light and their shadows fall towards the camera.
  day.azimuth = 150;
  day.tilt = 30;
  must(world.resource(og.PhysicalSky)).enabled = true;
  must(world.resource(og.RenderSettings)).sky = true;
  const clouds = must(world.resource(og.Clouds));
  clouds.enabled = true;
  clouds.coverage = 0.35;
  clouds.scale = 6000;
  clouds.base = 1400;
  const fog = must(world.resource(og.Fog));
  fog.enabled = true;
  fog.density = 0.005;
  fog.sky = 0.2;
}

/** The camera's place along the way, and how it trails the runner. */
const CAMERA = { from: 2.6, to: TERRACE_X + 0.4, lead: 0.6, ease: 0.04, back: 0.6 };

function frame(world: World, x: number): void {
  const cam = must(world.resource(og.Camera));
  const eye = v3(x - CAMERA.back, 2.6, 8.2);
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, v3(x + 0.4, 1.7, -1.0));
  cam.fov = 45;
}

/** The camera eases after the runner, a little ahead of it, between the way's ends. */
const Follow = defineSystem("Follow", { reads: [og.Transform], writes: [og.Camera] }, (_q, _time, world) => {
  const r = world.find("runner");
  if (r === null) return;
  const t = must(world.get(r, og.Transform));
  const want = Math.min(CAMERA.to, Math.max(CAMERA.from, t.position.x + CAMERA.lead));
  // Where it stands now is the camera's own (snapshotted with the world).
  const now = must(world.resource(og.Camera)).position.x + CAMERA.back;
  frame(world, now + (want - now) * CAMERA.ease);
});

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    buildSet(world);
    light(world);
    const r = character(world, "runner", "models/runner.glb", -1.2, RUNNER_Z);
    must(world.insert(r, og.Collider, { shape: "capsule", radius: 0.28, height: 1.15, offset: v3(0, 0.85, 0) }));
    must(world.insert(r, og.CharacterController, {}));
    runner(world, r);
    porter(world, character(world, "porter", "models/porter.glb", PORTER_FROM, PORTER_Z));
    frame(world, CAMERA.from);
  },
  systems: [Follow],
});
