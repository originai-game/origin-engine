// A test fixture, not a template (tests/fixtures/README.md): a pressure
// plate's contents (og.physics.sensorContents) as crates cross it.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "sensors",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
