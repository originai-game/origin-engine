// The motion-feet fixture course (README.md): the strider jumps with an og.CharacterController (a
// running jump, then up onto a 0.4 m ledge); the stout without one (in
// place, then forward). The simulation owns every arc; the model draws it.
import { og } from "@origingame/engine";
import type { Entity, Quaternion, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const floor = (): Quaternion => og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);
const EAST = Math.PI / 2;
const CLOSE = false;

function strider(world: World, body: Entity): void {
  const z = 0;
  must(og.motion.walkTo(world, body, { position: { x: 2.5, y: 0, z }, speed: 1.3 }));
  must(og.motion.walkTo(world, body, { position: { x: 5, y: 0, z }, speed: 3.6 }));
  must(og.motion.jump(world, body, { height: 0.5, distance: 1.6, heading: EAST }));
  must(og.motion.walkTo(world, body, { position: { x: 8.6, y: 0, z }, speed: 1.3 }));
  must(og.motion.jump(world, body, { height: 0.75, distance: 1.0, heading: EAST, prepare: 0.35 }));
  must(og.motion.walkTo(world, body, { position: { x: 11, y: 0.4, z }, speed: 1.3 }));
  must(og.motion.pose(world, body, { pose: "crouch", duration: 1.0 }));
  must(og.motion.face(world, body, { heading: 0 }));
  must(og.motion.pose(world, body, { pose: "wave", duration: 2 }));
}

function stout(world: World, body: Entity): void {
  const z = -1.8;
  must(og.motion.walkTo(world, body, { position: { x: 2.5, y: 0, z }, speed: 1.2 }));
  must(og.motion.jump(world, body, { height: 0.35, heading: EAST }));
  must(og.motion.jump(world, body, { height: 0.45, distance: 1.2, heading: EAST }));
  must(og.motion.walkTo(world, body, { position: { x: 6, y: 0, z }, speed: 1.2 }));
  must(og.motion.face(world, body, { heading: 0 }));
  must(og.motion.pose(world, body, { pose: "cheer", duration: 2 }));
}

function block(world: World, name: string, at: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }, color: { r: number; g: number; b: number }): void {
  const box = must(og.render.boxGeometry(world, {}));
  const m = must(og.render.standardMaterial(world, { color, roughness: 0.9 }));
  const e = must(world.spawn({ name, position: at }));
  world.set(e, og.Transform, { scale: size });
  world.insert(e, og.Mesh, { geometry: box, material: m, castShadow: true, receiveShadow: true });
}

function build(world: World): void {
  const plane = must(og.render.planeGeometry(world, { width: 40, height: 40 }));
  const field = must(og.render.standardMaterial(world, { color: { r: 0.34, g: 0.36, b: 0.33 }, roughness: 0.95 }));
  const ground = must(world.spawn({ name: "ground", position: { x: 5, y: 0, z: 0 }, quaternion: floor() }));
  world.insert(ground, og.Mesh, { geometry: plane, material: field, receiveShadow: true });
  const slab = must(world.spawn({ name: "ground-collider", position: { x: 5, y: -0.5, z: 0 } }));
  world.insert(slab, og.Collider, { shape: 0, size: { x: 40, y: 1, z: 40 } });
  // The ledge: 0.4 m high from x 9.5 on.
  block(world, "ledge", { x: 11.5, y: 0.2, z: 0 }, { x: 4, y: 0.4, z: 1.6 }, { r: 0.5, g: 0.42, b: 0.34 });
  const ledge = must(world.spawn({ name: "ledge-collider", position: { x: 11.5, y: 0.2, z: 0 } }));
  world.insert(ledge, og.Collider, { shape: 0, size: { x: 4, y: 0.4, z: 1.6 } });
  // A metre grid, so travel and planted feet read against the ground.
  for (let k = -2; k <= 14; k++) {
    block(world, `grid-x${k}`, { x: k, y: 0.002, z: -1 }, { x: 0.02, y: 0.004, z: 6 }, { r: 0.25, g: 0.27, b: 0.25 });
  }
  // A height scale behind the lanes: a bar every 0.25 m.
  for (let k = 1; k <= 8; k++) {
    block(world, `scale-${k}`, { x: 4, y: k * 0.25, z: -3.2 }, { x: 8, y: 0.01, z: 0.02 }, { r: 0.6, g: 0.6, b: 0.62 });
  }

  const s = must(world.instantiate(must(og.load.gltf(world, "models/strider.glb")), { name: "strider", position: { x: 0, y: 0, z: 0 } }));
  world.insert(s, og.Skeleton, { asset: must(og.load.animation(world, "models/strider.glb")).skeleton, rootMotion: "off" });
  world.insert(s, og.MotionAgent, { rig: must(og.load.rig(world, "rigs/strider.ogt")), drive: true });
  world.insert(s, og.Collider, { shape: 2, radius: 0.3, height: 1.2, offset: { x: 0, y: 0.9, z: 0 } });
  world.insert(s, og.CharacterController, {});
  strider(world, s);

  const t = must(world.instantiate(must(og.load.gltf(world, "models/stout.glb")), { name: "stout", position: { x: 0, y: 0, z: -1.8 } }));
  world.insert(t, og.Skeleton, { asset: must(og.load.animation(world, "models/stout.glb")).skeleton, rootMotion: "off" });
  world.insert(t, og.MotionAgent, { drive: true });
  stout(world, t);

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.7, z: -0.55 }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true });
  const cam = must(world.resource(og.Camera));
  const eye = CLOSE ? { x: 9.2, y: 1.1, z: 4.2 } : { x: 6, y: 1.4, z: 9.5 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, CLOSE ? { x: 9.2, y: 0.8, z: 0 } : { x: 6, y: 0.7, z: -1 });
  cam.fov = 50;
  must(world.resource(og.RenderSettings)).sky = true;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world);
  },
});
