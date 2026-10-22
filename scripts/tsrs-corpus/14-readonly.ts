// Corpus 14-readonly: assigning a readonly System member
import { og, defineComponent, defineResource, defineSystem, bool, f32, Vector3 } from "@origingame/engine";
import type { World, Entity } from "@origingame/engine";
const Health = defineComponent("Health", { hp: f32(100), max: f32(100) });
const Velocity = defineComponent("Velocity", { v: Vector3() });
const Run = defineResource("Run", { started: bool() });
void og; void defineSystem; void Health; void Velocity; void Run;
declare const world: World;
declare const e: Entity;

const S = defineSystem("S", {}, () => {});
S.name = "T";
