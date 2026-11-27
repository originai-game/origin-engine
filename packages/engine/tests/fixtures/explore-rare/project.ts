import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "explore-rare",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 17
});
