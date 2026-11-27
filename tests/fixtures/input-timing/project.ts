// A test fixture, not a template (tests/fixtures/README.md): when in its
// tick an action went down (ActionState.pressedAt) and a buffered press
// (ActionDesc.bufferTicks, og.input.consume).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "input-timing",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1
});
