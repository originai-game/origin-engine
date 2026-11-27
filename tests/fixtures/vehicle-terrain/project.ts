// A test fixture, not a template (tests/fixtures/README.md): a car driven
// on cooked terrain, its HUD, its chase camera and its effects.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "vehicle-terrain",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
