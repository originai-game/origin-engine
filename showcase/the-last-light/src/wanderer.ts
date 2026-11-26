// The wanderer, the player's character: a capsule on the cliff path that the
// player walks and runs (player.ts), its walk cross-faded in as it moves; the staff and the lantern's hook carried in its frame, the
// lantern swinging on its cord, the cloak in the wind.
import { defineResource, f64, math, og, quat } from "@origingame/engine";
import type { ClipAsset, World } from "@origingame/engine";
import type { Coast } from "./coast.ts";
import { WANDERER } from "./layout.ts";
import { must } from "./must.ts";

/** The rig's right hand, among its skin's joints. */
const RIGHT_HAND = 15;
/** Which way the wanderer faces: down the path, turned a little to the light. */
const FACING = Math.PI - 0.12;

/** Points carried in the wanderer's frame (x to their left, y up, z ahead, from the feet): the staff's foot, the hand's grip and its pole, the lantern's hook. */
export const CARRY = {
  lean: 0.04,
  staff: { x: -0.36, y: -0.03, z: 0.3 },
  grip: { x: -0.36 + 1.3 * 0.04, y: 1.3, z: 0.3 },
  pole: { x: -0.55, y: 0.8, z: -0.6 },
  hook: { x: -0.36 - 0.13 + 1.74 * 0.04, y: 1.74, z: 0.3 },
} as const;
/** The wanderer's two clips (handles): standing and walking. */
export const Clips = defineResource("Clips", { idle: f64(), walk: f64() });

/** The lantern: its cord, its size, and how far below the hook its origin hangs. */
export const LANTERN = { cord: 0.42, size: 1.25, drop: 0.42 + 0.294 * 1.25 } as const;
/** Where the wanderer starts and which way it faces: down the path, turned a little to the light. */
export const START_HEADING = FACING;

export function buildWanderer(world: World, coast: Coast): void {
  const up = { x: 0, y: 1, z: 0 };
  const turn = og.transform.fromAxisAngle(up, FACING);
  const feet = { x: WANDERER.x, y: coast.heightAt(WANDERER.x, WANDERER.z), z: WANDERER.z };
  const c = math.cos(FACING), s = math.sin(FACING);
  const at = (p: { x: number; y: number; z: number }) => ({ x: feet.x + p.x * c + p.z * s, y: feet.y + p.y, z: feet.z - p.x * s + p.z * c });

  const rig = must(og.load.animation(world, "models/wanderer.glb"));
  const body = must(world.instantiate(must(og.load.gltf(world, "models/wanderer.glb")), { name: "wanderer", position: feet, quaternion: turn }));
  must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  // Standing: breathing and looking about. Walking: a casual walk from the
  // same Meshy rig family (models/walk-clips.glb), retargeted onto the
  // wanderer; player.ts cross-fades between them by the speed it walks at.
  const library = must(og.load.animation(world, "models/walk-clips.glb"));
  const walk = must(og.anim.retarget(world, { clip: og.anim.clip(library, "Casual_Walk")!, source: library.skeleton, target: rig.skeleton }));
  const clips = must(world.resource(Clips));
  clips.idle = og.anim.clip(rig, "Long_Breathe_and_Look_Around")! as unknown as number;
  clips.walk = walk.clip as unknown as number;
  must(world.insert(body, og.AnimationPlayer, { clip: clips.idle as unknown as ClipAsset, loop: true, fade: 0.35 }));
  // Walked by the player through a kinematic controller on the terrain.
  must(world.insert(body, og.Collider, { shape: "capsule", radius: 0.3, height: 1.0, offset: { x: 0, y: 0.82, z: 0 } }));
  must(world.insert(body, og.CharacterController, { maxSlope: 50, stepHeight: 0.35 }));
  // The cloak (tools/cloak.ts hangs it on the rig as the model's second
  // mesh): held at the collar, free towards the hem, blown inland. XPBD
  // garment (physics §8): the default physical stiffness, four substeps,
  // drag and lift so the wind (air in m/s) moves it, a spherical backstop
  // behind the back and skeleton capsules so it never folds into the body.
  must(world.insert(body, og.Garment, {
    mesh: 1, maxDistance: 0.42, pinTop: 0.05, backstop: 0.03, backstopRadius: 0.25, bodyRadius: 0.08, teleportDistance: 0.5,
    bending: 0.1, damping: 2, substeps: 4, drag: 1, lift: 0.3, gust: 0.9, wind: { x: -3, y: 0.4, z: 1.5 },
  }));

  // The staff is carried upright by the right side, its hook outwards; the
  // hand rests on its grip (player.ts moves the grip with the wanderer).
  must(world.instantiate(must(og.load.gltf(world, "props/staff.ogt")), {
    name: "staff", parent: body, position: CARRY.staff,
    quaternion: quat.multiply(og.transform.fromAxisAngle(up, Math.PI), og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, CARRY.lean)),
  }));
  const grip = must(world.spawn({ name: "staff-grip" }));
  must(world.insert(grip, og.IkChain, { skeleton: body, tip: RIGHT_HAND, bones: 2, target: at(CARRY.grip), pole: at(CARRY.pole) }));

  // The lantern hangs by a cord from the staff's hook; player.ts swings it
  // as the wanderer starts, stops and turns.
  const lantern = must(world.instantiate(must(og.load.gltf(world, "models/lantern.glb")), {
    name: "lantern", parent: body, position: { x: CARRY.hook.x, y: CARRY.hook.y - LANTERN.drop, z: CARRY.hook.z }, scale: { x: LANTERN.size, y: LANTERN.size, z: LANTERN.size },
  }));
  const line = must(world.spawn({ name: "lantern-cord", parent: body, position: CARRY.hook }));
  must(world.insert(line, og.Rope, { segments: 6, length: LANTERN.cord, end: lantern, endOffset: { x: 0, y: 0.294 * LANTERN.size, z: 0 }, width: 0.012, radius: 0, color: { r: 0.12, g: 0.09, b: 0.06 } }));
  const flame = must(world.spawn({ name: "lantern-flame", parent: lantern, position: { x: 0, y: 0.06, z: 0 } }));
  must(world.insert(flame, og.Mesh, {
    geometry: must(og.render.sphereGeometry(world, { radius: 0.03 })),
    material: must(og.render.standardMaterial(world, { color: { r: 1, g: 0.75, b: 0.4 }, emissive: { r: 1, g: 0.62, b: 0.22 }, emissiveIntensity: 2.5 })),
  }));
  must(world.insert(flame, og.PointLight, { color: { r: 1, g: 0.62, b: 0.26 }, intensity: 2.2, distance: 7, castShadow: true }));
}
