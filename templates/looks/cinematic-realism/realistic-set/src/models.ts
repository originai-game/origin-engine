// The realistic set's models (sets/realistic/, tools/art-samples/capture.ts):
// scanned and generated assets in place of the family's primitives, for the
// styles whose contract is realism (cinematic-realism, muted-cinematic-3d).
// Sources and how each was packed: sets/realistic/README.md.
export const MODELS = {
  ground: "models/ground.glb",
  far: "models/ridge.glb",
  tree: "models/pine.glb",
  lantern: "models/street-lamp.glb",
  crate: "models/crate.glb",
  hero: "models/hero.glb",
} as Record<string, string>;
/** Extra models, at (x, z) with a turn: the hand lantern on the crate, a stump by the tree, and the stand of pines behind (the same pine, turned, in depth). */
export const DRESSING: { path: string; x: number; z: number; turn?: number; y?: number }[] = [
  { path: "models/hand-lantern.glb", x: 1.15, z: 0.25, turn: 0.4, y: 0.5 },
  { path: "models/stump.glb", x: -1.9, z: -2.6, turn: 1.1 },
  ...[
    [-6.5, -4.5], [-5.2, -7.5], [-8.8, -9], [-3.6, -11], [-11, -6.5], [-7.4, -14], [-1.8, -16], [-12.5, -13],
    [5.8, -8], [7.6, -11.5], [4.2, -14], [10.5, -9.5], [9, -16], [2.4, -19], [13.5, -14], [-4.6, -20],
  ].map(([x, z], i) => ({ path: "models/pine.glb", x: x!, z: z!, turn: i * 1.7 })),
  { path: "models/stump.glb", x: 4.6, z: -4.2, turn: 2.6 },
];
