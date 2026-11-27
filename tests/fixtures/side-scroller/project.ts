// A test fixture, not a template (tests/fixtures/README.md): a night cave,
// a knight run by a sprite animation graph, pixel terrain, 2D lights.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "side-scroller",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
