// The UI filter scenes the final L4 pass shoots and times (docs/ci.md §The final L4 pass).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "filter-scenes",
  "entry": "src/main.tsx",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
