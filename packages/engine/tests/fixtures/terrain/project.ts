// The fixture terrain.test.ts cooks and loads (assets/terrain/hill.ogt).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "terrain-fixture",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
