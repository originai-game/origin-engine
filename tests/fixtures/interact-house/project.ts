// A test fixture, not a template (tests/fixtures/README.md): og.systems.interact
// in two rooms joined by a door on a navmesh link.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "interact-house",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
