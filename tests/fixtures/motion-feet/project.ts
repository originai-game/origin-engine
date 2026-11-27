// A test fixture, not a template (tests/fixtures/README.md): og.motion's
// drawn feet over a course of walks, a run, two jumps, a crouch, a turn and
// held gestures, on two rigs.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "motion-feet",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
