// A test fixture, not a template (tests/fixtures/README.md): og.play's hit
// volumes on a posed bone, swept between ticks, and its candidates.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "hit-volumes",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
