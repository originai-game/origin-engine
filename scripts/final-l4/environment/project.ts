// The LDR environment scene the final L4 pass shoots (docs/ci.md §The final L4 pass).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "environment-scene",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
