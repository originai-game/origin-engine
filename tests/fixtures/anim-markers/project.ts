// A test fixture, not a template (tests/fixtures/README.md): where an
// og.AnimationEvent came from in a cross-fade and a three-way blend.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "anim-markers",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
