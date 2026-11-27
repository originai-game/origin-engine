// The text sheet scripts/text-l4/run.sh measures against Chrome.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "text-sheet",
  "entry": "src/main.tsx",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
