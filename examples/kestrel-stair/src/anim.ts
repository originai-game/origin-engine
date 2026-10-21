// The characters' animation: one clip library (anims/humanoid.glb, a
// T-posed skeleton with Blender-style names) played on three bodies of
// other proportions, rest poses and bone names, and the blend trees that
// pick the motion by speed.
//
// - The courier (models/courier.glb, Mixamo names) has the library's clips
//   retargeted onto it at cook time (models/courier.glb.ogt: `animation {
//   library }`).
// - The imp (models/imp.glb, Unreal names) gets them at run time,
//   og.anim.retarget with its bones detected.
// - The warden (models/warden.glb, names no scheme knows) gets them at run
//   time with a rig file (rigs/warden.ogt) naming each bone.
//
// Each plays an og.AnimationGraph: a 1D blend of idle, walk and run by
// speed (and for the courier an air state), with an og.AnimationPlayer
// over it for the one-shots (strike, bite, slam, hurt, die) at weight 1
// while they play and 0 otherwise.

import { og } from "@origingame/engine";
import type { ClipAsset, ClipInfo, GraphAsset, SkeletonAsset, World } from "@origingame/engine";
import { must } from "./world.ts";

/**
 * The courier's graph: p0 the speed across the ground, p1 1 in the air. The
 * `type animgraph` grammar is og-game §9 since wave 11 (FINDINGS #2, fixed);
 * these lines follow the third-person archetype's.
 */
const HERO_GRAPH = `type animgraph
version 1
---
param speed
param air
clip idle "idle"
clip walk "walk" {
  speed 1.05
}
clip run "run"
clip jump "jump"
blend1d move speed {
  child idle 0
  child walk 3.2
  child run 6.2
}
layer base {
  state ground move
  state air jump
  entry ground
  transition ground air {
    when air gt 0.5
    fade 0.12
  }
  transition air ground {
    when air lt 0.5
    fade 0.1
  }
}
`;

/** A foe's graph: p0 its speed (a patrol walks, a chase runs). */
function foeGraph(walk: number, run: number): string {
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

function byName(clips: readonly ClipInfo[]): Record<string, ClipAsset> {
  const out: Record<string, ClipAsset> = {};
  for (const c of clips) out[c.name] = c.clip;
  return out;
}

/** The courier: its own clips (the library's, retargeted by the cook) and its graph. */
export function courierRig(world: World): Rigged {
  const anim = must(og.load.animation(world, "models/courier.glb"));
  return { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text: HERO_GRAPH, clips: anim.clips })), clips: byName(anim.clips) };
}

/** A foe's skeleton with the library's clips retargeted onto it now, and its graph. */
export function foeRig(world: World, kind: "imp" | "warden"): Rigged {
  const anim = must(og.load.animation(world, `models/${kind}.glb`));
  const library = must(og.load.animation(world, "anims/humanoid.glb"));
  const targetRig = kind === "warden" ? must(og.load.rig(world, "rigs/warden.ogt")) : undefined;
  const clips: ClipInfo[] = library.clips.map((c) => ({ ...c, clip: must(og.anim.retarget(world, { clip: c.clip, source: library.skeleton, target: anim.skeleton, ...(targetRig ? { targetRig } : {}) })).clip }));
  const text = kind === "imp" ? foeGraph(1.8, 4.4) : foeGraph(1.1, 2.4);
  return { skeleton: anim.skeleton, graph: must(og.anim.graph(world, { text, clips })), clips: byName(clips) };
}
