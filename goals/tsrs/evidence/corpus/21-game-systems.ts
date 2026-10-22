// Corpus 21-game-systems: og.game given a component where a system goes
import { og, defineComponent, defineResource, defineSystem, bool, f32, Vector3 } from "@origingame/engine";
import type { World, Entity } from "@origingame/engine";
const Health = defineComponent("Health", { hp: f32(100), max: f32(100) });
const Velocity = defineComponent("Velocity", { v: Vector3() });
const Run = defineResource("Run", { started: bool() });
void og; void defineSystem; void Health; void Velocity; void Run;
declare const world: World;
declare const e: Entity;

export default og.game({ systems: [Health] });
