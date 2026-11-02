// og.motion on two game rigs: the engine's motion model animates the strider
// (Mixamo names, long legs) and the stout (Unreal names, short legs) live, on
// the GPU, from goals the game queues. og.motion drives their Transforms with
// fixed math (drive: true); the generated pose is drawn on top, its feet
// planted, and never fed back into the simulation.
import { og } from "@origingame/engine";
import type { Entity, Quaternion, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const floor = (): Quaternion => og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);

/** The characters: model, rig file (or detection), and where its lane is (side by side). */
const CHARACTERS = [
  { name: "strider", model: "models/strider.glb", rig: "rigs/strider.ogt", x: 0, z: 0 },
  { name: "stout", model: "models/stout.glb", rig: null, x: -1.6, z: 0 },
] as const;

/** One lane's goals: walk out, turn, run, jump, crouch, walk back, wave, cheer. */
function course(world: World, body: Entity, x0: number, z: number): void {
  const at = (x: number, dz = 0) => ({ x: x0 + x, y: 0, z: z + dz });
  must(og.motion.walkTo(world, body, { position: at(3), speed: 1.3 }));
  must(og.motion.walkTo(world, body, { position: at(4.5, 1.5), speed: 1.3 }));
  must(og.motion.walkTo(world, body, { position: at(8, 1.5), speed: 3.6 }));
  must(og.motion.place(world, body, { pose: "jump", position: at(9, 1.5), heading: Math.PI / 2, duration: 0.8 }));
  must(og.motion.pose(world, body, { pose: "crouch", duration: 1.2 }));
  must(og.motion.walkTo(world, body, { position: at(5, 0), speed: 1.3 }));
  must(og.motion.face(world, body, { heading: 0 }));
  must(og.motion.pose(world, body, { pose: "wave", duration: 2 }));
  must(og.motion.pose(world, body, { pose: "cheer", duration: 2 }));
}

function build(world: World): void {
  const plane = must(og.render.planeGeometry(world, { width: 40, height: 40 }));
  const field = must(og.render.standardMaterial(world, { color: { r: 0.34, g: 0.36, b: 0.33 }, roughness: 0.95 }));
  const ground = must(world.spawn({ name: "ground", position: { x: 4, y: 0, z: 1 }, quaternion: floor() }));
  world.insert(ground, og.Mesh, { geometry: plane, material: field, receiveShadow: true });
  // A metre grid, so travel and planted feet read against the ground.
  const box = must(og.render.boxGeometry(world, {}));
  const line = must(og.render.standardMaterial(world, { color: { r: 0.25, g: 0.27, b: 0.25 }, roughness: 1 }));
  for (let k = -6; k <= 14; k++) {
    const x = must(world.spawn({ name: `grid-x${k}`, position: { x: k, y: 0.002, z: 1 } }));
    world.set(x, og.Transform, { scale: { x: 0.02, y: 0.004, z: 20 } });
    world.insert(x, og.Mesh, { geometry: box, material: line });
    const z = must(world.spawn({ name: `grid-z${k}`, position: { x: 4, y: 0.002, z: k - 4 } }));
    world.set(z, og.Transform, { scale: { x: 20, y: 0.004, z: 0.02 } });
    world.insert(z, og.Mesh, { geometry: box, material: line });
  }

  for (const c of CHARACTERS) {
    const anim = must(og.load.animation(world, c.model));
    const rig = c.rig === null ? undefined : must(og.load.rig(world, c.rig));
    const body = must(world.instantiate(must(og.load.gltf(world, c.model)), { name: c.name, position: { x: c.x, y: 0, z: c.z } }));
    // og.anim poses the skeleton for the simulation and draws it until the
    // model is resident (rest pose here: a game gives it an idle clip).
    world.insert(body, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
    world.insert(body, og.MotionAgent, rig === undefined ? { drive: true } : { rig, drive: true });
    course(world, body, c.x, c.z);
  }

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.7, z: -0.55 }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true });
  const cam = must(world.resource(og.Camera));
  // The lanes from the front and above: x -1.6 to 9, z 0 to 1.5.
  const eye = { x: 4.5, y: 3.4, z: 8.5 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: 4.5, y: 0.7, z: -0.2 });
  cam.fov = 55;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world);
  },
});
