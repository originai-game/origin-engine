// A test fixture, not a template (tests/fixtures/README.md): og.systems.inventory
// with a bag, a chest, pickups on a walk and an item table.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "inventory-chest",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 7
});
