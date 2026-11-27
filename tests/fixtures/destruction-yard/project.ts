// A test fixture, not a template (tests/fixtures/README.md): og.destruction
// on a brick wall, a concrete pillar and a glass pane.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "destruction-yard",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
