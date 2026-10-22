// Corpus 07-vector3-array: a Vector3 given as an array
import { og, defineComponent, defineResource, defineSystem, bool, f32, Vector3 } from "@origingame/engine";
import type { World, Entity } from "@origingame/engine";
const Health = defineComponent("Health", { hp: f32(100), max: f32(100) });
const Velocity = defineComponent("Velocity", { v: Vector3() });
const Run = defineResource("Run", { started: bool() });
void og; void defineSystem; void Health; void Velocity; void Run;
declare const world: World;
declare const e: Entity;

export const r = world.instantiate(og.load.gltf(world, "a.glb").ok ? (null as never) : (null as never), { position: [0, 1, 0] });
