// The wanderer: seen from behind on the path, breathing and looking about,
// a hand on the staff, the lantern hanging from its hook, the cloak in the
// wind.
import { math, og, quat } from "@origingame/engine";
import type { World } from "@origingame/engine";
import type { Coast } from "./coast.ts";
import { WANDERER } from "./layout.ts";
import { must } from "./must.ts";

/** The rig's right hand, among its skin's joints. */
const RIGHT_HAND = 15;
/** Which way the wanderer faces: down the path, turned a little to the light. */
const FACING = Math.PI - 0.12;

export function buildWanderer(world: World, coast: Coast): void {
  const up = { x: 0, y: 1, z: 0 };
  const turn = og.transform.fromAxisAngle(up, FACING);
  const feet = { x: WANDERER.x, y: coast.heightAt(WANDERER.x, WANDERER.z), z: WANDERER.z };
  // A point given in the wanderer's own frame (x to their left, z ahead).
  const c = math.cos(FACING), s = math.sin(FACING);
  const at = (x: number, y: number, z: number) => ({ x: feet.x + x * c + z * s, y: feet.y + y, z: feet.z - x * s + z * c });

  const rig = must(og.load.animation(world, "models/wanderer.glb"));
  const body = must(world.instantiate(must(og.load.gltf(world, "models/wanderer.glb")), { name: "wanderer", position: feet, quaternion: turn }));
  must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.AnimationPlayer, { clip: og.anim.clip(rig, "Long_Breathe_and_Look_Around")! }));
  // The cloak (tools/cloak.ts hangs it on the rig as the model's second
  // mesh): held at the collar, free towards the hem, blown inland.
  must(world.insert(body, og.Garment, { mesh: 1, maxDistance: 0.42, pinTop: 0.05, backstop: 0.03, bending: 0.05, damping: 1.4, gust: 1.3, wind: { x: -3, y: 0.4, z: 1.5 } }));

  // The staff stands planted by the right foot, its hook outwards; the
  // hand rests on its grip.
  const staff = at(-0.36, -0.03, 0.3);
  const lean = 0.04;
  must(world.instantiate(must(og.load.gltf(world, "props/staff.ogt")), {
    name: "staff", position: staff, quaternion: quat.multiply(og.transform.fromAxisAngle(up, FACING + Math.PI), og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, lean)),
  }));
  const grip = must(world.spawn({ name: "staff-grip" }));
  must(world.insert(grip, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 2, target: at(-0.36 + 1.3 * lean, 1.3, 0.3), pole: at(-0.55, 0.8, -0.6) }));

  // The lantern hangs by a cord from the staff's hook.
  const hook = at(-0.36 - 0.13 + 1.74 * lean, 1.74, 0.3);
  const cord = 0.42, size = 1.25;
  const lantern = must(world.instantiate(must(og.load.gltf(world, "models/lantern.glb")), {
    name: "lantern", position: { x: hook.x, y: hook.y - cord - 0.294 * size, z: hook.z }, scale: { x: size, y: size, z: size },
  }));
  const line = must(world.spawn({ name: "lantern-cord", position: hook }));
  must(world.insert(line, og.Rope, { segments: 6, length: cord, end: lantern, endOffset: { x: 0, y: 0.294 * size, z: 0 }, width: 0.012, radius: 0, color: { r: 0.12, g: 0.09, b: 0.06 } }));
  const flame = must(world.spawn({ name: "lantern-flame", position: { x: hook.x, y: hook.y - cord - 0.2 * size, z: hook.z } }));
  must(world.insert(flame, og.Mesh, {
    geometry: must(og.render.sphereGeometry(world, { radius: 0.03 * size })),
    material: must(og.render.standardMaterial(world, { color: { r: 1, g: 0.75, b: 0.4 }, emissive: { r: 1, g: 0.62, b: 0.22 }, emissiveIntensity: 2.5 })),
  }));
  must(world.insert(flame, og.PointLight, { color: { r: 1, g: 0.62, b: 0.26 }, intensity: 2.2, distance: 7, castShadow: true }));
}
