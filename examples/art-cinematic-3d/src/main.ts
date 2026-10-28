// The cinematic 3D family's sample scene (docs/art.md §2.6): the shared beats
// (src/scene.ts) under this family's camera. The cook conforms every model
// and texture to project.ogt's look; nothing moves.
import { og } from "@origingame/engine";
import { build } from "./scene.ts";

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world, { eye: { x: 2.4, y: 1.7, z: 7.5 }, at: { x: -0.4, y: 1.3, z: -1.2 }, fov: 42, sun: 1.2 });
  },
});
