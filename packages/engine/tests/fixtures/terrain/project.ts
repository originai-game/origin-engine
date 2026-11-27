// The terrain fixture cooks and loads content/terrain/hill.ts.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "terrain-fixture",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
