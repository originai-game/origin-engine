// A test fixture, not a template (tests/fixtures/README.md): og.systems.abilities
// with a dash of two charges, a fireball that costs mana and three statuses.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "abilities",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 3
});
