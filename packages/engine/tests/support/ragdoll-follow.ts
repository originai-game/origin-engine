// The template's walker as an og.Ragdoll on the native host: it walks with
// the ragdoll's weight at 0 for --ticks ticks, then goes limp. Prints JSON
// for e5.test.ts: the bones (og.physics.ragdollPose) two ticks apart while
// animated, the rig's rest pose placed at the entity, and the bones on the
// first tick limp.
//
//   bun tests/support/ragdoll-follow.ts --project <templates/game> [--ticks 40]

import { og, defineSystem } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";
import { loadNativeHost } from "../../src/node/index.ts";
import { createEngine, type GameToken } from "../../src/runtime.ts";

const argv = process.argv.slice(2);
const opt = (name: string, fallback: string) => {
  const i = argv.indexOf(name);
  return i >= 0 ? argv[i + 1]! : fallback;
};
const project = opt("--project", "");
const ticks = Number(opt("--ticks", "40"));
loadNativeHost();

let doll: Entity | null = null;
let rest: Array<{ x: number; y: number; z: number }> = [];

function start(world: World): void {
  const a = og.load.animation(world, "models/walker.glb");
  if (!a.ok) throw new Error(a.reason);
  const walk = a.value.clips.find((c) => c.name === "walk") ?? a.value.clips[0]!;
  const e = world.spawn({ name: "doll", position: { x: 1, y: 0, z: 2 } });
  if (!e.ok) throw new Error(e.reason);
  world.insert(e.value, og.Skeleton, { asset: a.value.skeleton, rootMotion: "off" });
  world.insert(e.value, og.AnimationPlayer, { clip: walk.clip, speed: 1, loop: true });
  world.insert(e.value, og.Ragdoll, { weight: 0, blendTime: 0.2 });
  const rig = og.physics.ragdollRigFromSkeleton(world, a.value.skeleton);
  if (!rig.ok) throw new Error(rig.reason);
  const set = og.physics.setRagdollRig(world, e.value, rig.value);
  if (!set.ok) throw new Error(set.reason);
  rest = (rig.value.bones ?? []).map((b) => ({ x: 1 + (b.position?.x ?? 0), y: b.position?.y ?? 0, z: 2 + (b.position?.z ?? 0) }));
  doll = e.value;
}

const Limp = defineSystem("RdLimp", { writes: [og.Ragdoll] }, (_q, time, world) => {
  if (doll !== null && time.tick === ticks) world.set(doll, og.Ragdoll, { weight: 1 });
});

const game = og.game({ tick: 60, seed: 1, systems: [Limp], start }) as unknown as GameToken;
const created = createEngine(game, { root: project, profile: "3d" });
if (!created.ok) throw new Error(created.reason);
const engine = created.value;
const pose = () => {
  const p = og.physics.ragdollPose(engine.world as never, doll as Entity);
  if (!p.ok) throw new Error(p.reason);
  return p.value.map((b) => b.position);
};
const adv = engine.advance(ticks - 2);
if (!adv.ok) throw new Error(adv.reason);
const a = pose();
engine.advance(2);
const b = pose();
engine.advance(1);
const limp = pose();
console.log(JSON.stringify({ a, b, rest, limp }));
