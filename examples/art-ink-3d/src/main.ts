// The ink and line 3D family's sample scene (docs/art.md §2.6): the shared beats
// (src/scene.ts) under this family's camera. The cook conforms every model
// and texture to project.ogt's look; nothing moves.
import { og } from "@origingame/engine";
import { build } from "./scene.ts";

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    build(world, { eye: { x: 1.2, y: 2.6, z: 8.5 }, at: { x: 0, y: 1.3, z: -1 }, fov: 45, sun: 0.6 });
  },
});
