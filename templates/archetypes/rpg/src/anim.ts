// The characters' animation: one clip library (anims/humanoid.glb: idle,
// walk, run, attack, hurt, die and slam, with `hit` markers on the blows)
// retargeted by the cook onto KayKit's Rig_Medium (every models/*.glb.ogt
// names rigs/kaykit.ogt), so the knight, the barbarian, the castellan and
// the skeletons share it. Each body plays an og.AnimationGraph (idle, walk
// and run blended by speed) with an og.AnimationPlayer over it for the
// one-shots (a swing, a hurt, a fall), weighted 1 while one plays.

import { og } from "@origingame/engine";
import type { ClipAsset, ClipInfo, Entity, GraphAsset, SkeletonAsset, World } from "@origingame/engine";
import { must } from "./world.ts";

/** A body's graph: p0 is its speed across the ground. */
function graphText(walk: number, run: number): string {
  return `type animgraph
version 1
---
param speed
clip idle "idle"
clip walk "walk"
clip run "run"
blend1d move speed {
  child idle 0
  child walk ${walk}
  child run ${run}
}
layer base {
  state move move
  entry move
}
`;
}

export interface Rigged {
  skeleton: SkeletonAsset;
  graph: GraphAsset;
  clips: Record<string, ClipAsset>;
}

const rigs = new Map<string, Rigged>();

/** A character's skeleton, its clips by name and its graph (loaded once per model). */
export function rigOf(world: World, model: string, walk = 1.6, run = 4.2): Rigged {
  const known = rigs.get(model);
  if (known) return known;
  const anim = must(og.load.animation(world, `models/${model}.glb`));
  const clips: Record<string, ClipAsset> = {};
  for (const c of anim.clips as readonly ClipInfo[]) clips[c.name] = c.clip;
  for (const name of ["idle", "walk", "run", "attack", "hurt", "die"]) if (!clips[name]) throw new Error(`models/${model}.glb has no ${name} clip (its import retargets anims/humanoid.glb)`);
  const r = { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text: graphText(walk, run), clips: anim.clips })), clips };
  rigs.set(model, r);
  return r;
}

/** Gives a body its skeleton, graph and one-shot player. */
export function animate(world: World, body: Entity, rig: Rigged): void {
  must(world.insert(body, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.AnimationGraph, { asset: rig.graph }));
  must(world.insert(body, og.AnimationPlayer, { clip: rig.clips.attack!, weight: 0, loop: false }));
}

/** Plays a one-shot over the graph from its start, at `speed` (a fall holds its last frame until released). */
export function oneShot(world: World, body: Entity, rig: Rigged, clip: string, speed = 1): void {
  must(world.set(body, og.AnimationPlayer, { clip: rig.clips[clip]!, time: 0, weight: 1, loop: false, speed, fade: 0.06 }));
}

/** Hands the pose back to the graph. */
export function release(world: World, body: Entity): void {
  must(world.set(body, og.AnimationPlayer, { weight: 0 }));
}
