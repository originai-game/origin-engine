// The pixel in 3D family's sample scene (docs/art.md §2.6): the shared beats
// (src/scene.ts) under this family's camera. The cook conforms every model
// and texture to project.ogt's look; nothing moves.
import { og } from "@origingame/engine";
import { build } from "./scene.ts";

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world, { eye: { x: 0.6, y: 3.4, z: 9 }, at: { x: 0, y: 1.2, z: -1 }, fov: 40, sprites: { hero: { path: "art/hero.png", width: 1.07, height: 2 }, tree: { path: "art/tree.png", width: 3.37, height: 3.4 } } });
  },
});
