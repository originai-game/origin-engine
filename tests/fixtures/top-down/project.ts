// A test fixture, not a template (tests/fixtures/README.md): a farm at dusk,
// a tilemap, a walking farmer, packed crop sprites and 2D lamps.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "top-down",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
