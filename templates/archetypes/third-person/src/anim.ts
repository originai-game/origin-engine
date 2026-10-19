// The two characters' blend trees (og.anim.graph) and where their clips
// come from. The hero is the strider: its import settings retarget the
// clip library (anims/humanoid-clips.glb) at cook time. The sentinels are
// the stout, whose Unreal-named bones get the library's clips retargeted
// here at run time (og.anim.retarget).

import { og } from "@origingame/engine";
import type { ClipInfo, GraphAsset, SkeletonAsset, World } from "@origingame/engine";
import { must } from "./world.ts";

/**
 * The hero's graph: p0 is the speed across the ground, p1 is 1 in the air.
 * On the ground a 1D blend runs idle, walk and run by speed (the clips'
 * phases kept in step); a jump or a fall cross-fades to the run held
 * mid-stride, slowed.
 */
const HERO_GRAPH = `type animgraph
version 1
---
param speed
param air
clip idle "idle"
clip walk "walk" {
  speed 1.1
}
clip run "run"
blend1d move speed {
  child idle 0
  child walk 2.2
  child run 5.6
}
layer base {
  state ground move
  state air run {
    speed 0.3
  }
  entry ground
  transition ground air {
    when air gt 0.5
    fade 0.15
  }
  transition air ground {
    when air lt 0.5
    fade 0.1
  }
}
`;

/** A sentinel's graph: p0 is its speed (a patrol walks, a chase runs). */
const SENTINEL_GRAPH = `type animgraph
version 1
---
param speed
clip idle "idle"
clip walk "walk" {
  speed 0.9
}
clip run "run"
blend1d move speed {
  child idle 0
  child walk 1.6
  child run 4.2
}
layer base {
  state move move
  entry move
}
`;

export interface Rigged {
  skeleton: SkeletonAsset;
  graph: GraphAsset;
}

/** The strider's skeleton and the hero's graph. */
export function heroRig(world: World): Rigged {
  const anim = must(og.load.animation(world, "models/strider.glb"));
  return { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text: HERO_GRAPH, clips: anim.clips })) };
}

/** The stout's skeleton and the sentinels' graph, its clips retargeted from the library. */
export function sentinelRig(world: World): Rigged {
  const anim = must(og.load.animation(world, "models/stout.glb"));
  const library = must(og.load.animation(world, "anims/humanoid-clips.glb"));
  const clips: ClipInfo[] = library.clips.map((c) => ({ ...c, clip: must(og.anim.retarget(world, { clip: c.clip, source: library.skeleton, target: anim.skeleton })).clip }));
  return { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text: SENTINEL_GRAPH, clips })) };
}
