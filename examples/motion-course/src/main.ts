// og-motion's demo course on two game rigs. assets/anims/course.glb is the
// course the model generated for SOMA's skeleton, its feet planted from the
// model's contacts (crates/og-motion `examples/course.rs`); here og.anim.retarget
// puts it on the strider (Mixamo names, long legs) and the stout (Unreal
// names, short legs), each walking its own lane: the walk, turns, run, jump,
// crouch and wave of the course, as generated.
import { og } from "@origingame/engine";
import type { Quaternion, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const floor = (): Quaternion => og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);

/** The characters: model, rig file (or detection), and where its lane starts. */
const CHARACTERS = [
  { name: "strider", model: "models/strider.glb", rig: "rigs/strider.ogt", z: 0 },
  { name: "stout", model: "models/stout.glb", rig: null, z: -2.2 },
] as const;

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

  const course = must(og.load.animation(world, "anims/course.glb"));
  const clip = course.clips.find((c) => c.name === "course")!;
  for (const c of CHARACTERS) {
    const anim = must(og.load.animation(world, c.model));
    const rig = c.rig === null ? undefined : must(og.load.rig(world, c.rig));
    const target = rig === undefined ? { skeleton: anim.skeleton } : { skeleton: anim.skeleton, rig };
    must(og.anim.humanoid(world, target));
    const retargeted = must(og.anim.retarget(world, { clip: clip.clip, source: course.skeleton, target: anim.skeleton, ...(rig === undefined ? {} : { targetRig: rig }) }));
    const body = must(world.instantiate(must(og.load.gltf(world, c.model)), { name: c.name, position: { x: 0, y: 0, z: c.z } }));
    world.insert(body, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
    world.insert(body, og.AnimationPlayer, { clip: retargeted.clip, speed: 1, loop: true, fade: 0 });
  }

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.7, z: -0.55 }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true });
  const cam = must(world.resource(og.Camera));
  // The whole course in view: x -0.5 to 9, z -3.2 (the stout's lane) to 4.5.
  const eye = { x: 4.3, y: 10, z: 15 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: 4.3, y: 0.4, z: 0.8 });
  cam.fov = 50;
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
