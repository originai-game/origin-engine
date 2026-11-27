// A test fixture, not a template (tests/fixtures/README.md): two composed
// effects (type vfx), a hit and a heal, on a dark ground.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "vfx-hit",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
