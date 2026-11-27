// R6's forest fixture: palms from near to far, the far ones drawn as their
// impostor (assets/models/palm.glb.ogt).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "forest-fixture",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
