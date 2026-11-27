// A test fixture, not a template (tests/fixtures/README.md): og.systems.combat
// playing a light-light-heavy combo into a dummy, with a dodge cancel.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "combat-arena",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
