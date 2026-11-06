// The explorer's blend tree (og.anim.graph) and where its clips come from:
// the strider's import settings retarget the engine's clip library
// (anims/humanoid-clips.glb) onto it at cook time.

import { og } from "@origingame/engine";
import type { GraphAsset, SkeletonAsset, World } from "@origingame/engine";
import { must } from "./world.ts";

/**
 * The explorer's graph: p0 is the speed across the ground, p1 is 1 in the air.
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
  child walk 3.0
  child run 6.6
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

export interface Rigged {
  skeleton: SkeletonAsset;
  graph: GraphAsset;
}

/** The strider's skeleton and the explorer's graph. */
export function heroRig(world: World): Rigged {
  const anim = must(og.load.animation(world, "models/strider.glb"));
  return { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text: HERO_GRAPH, clips: anim.clips })) };
}
