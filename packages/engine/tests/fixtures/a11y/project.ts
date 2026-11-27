// tests/web/a11y.ts's game: a settings dialog (Audio and Graphics tabs, a
// Master slider, switches, Done) and a pause dialog, opened by `show`.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "a11y",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
