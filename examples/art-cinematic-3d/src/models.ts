// The models the scene places (src/scene.ts), by beat. A family's other
// asset set (sets/<name>/, tools/art-samples/capture.ts) brings its own
// copy of this file.
export const MODELS = {
  ground: "models/ground.glb",
  far: "models/hills.ogt",
  tree: "models/tree.ogt",
  lantern: "models/lantern.ogt",
  crate: "models/crate.glb",
  hero: "models/hero.ogt",
} as Record<string, string>;
/** Extra models, at (x, z) with a turn: the set's own dressing. */
export const DRESSING: { path: string; x: number; z: number; turn?: number; y?: number }[] = [];
