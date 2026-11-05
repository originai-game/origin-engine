// Engine defaults that moved after the fixture saves were written (the
// 0.7.7 save, tests/fixtures/light2d-v1, tests/fixtures/video-v1). A save
// holds every field of an engine resource, and a restore keeps what it
// holds: it cannot tell a value a game chose from the default the game ran
// with, so it never rewrites one. A test that compares a restored world
// with the same game run on this build gives the fresh world the saved
// values first.
//
// - og.PostEffects.ssrMaxDistance 20 -> 100 (37329a702: the hierarchical
//   march's cost is bounded by its steps, not the ray's length).
// - og.PostEffects.ssrSteps 32 -> 64 (ccb28fb57, the stochastic SSR).
import { og } from "@origingame/engine";
import type { Engine } from "../../src/runtime.ts";

export function asSaved(engine: Engine): void {
  const post = engine.world.resource(og.PostEffects);
  if (!post.ok) throw new Error(post.reason);
  Object.assign(post.value, { ssrMaxDistance: 20, ssrSteps: 32 });
}
