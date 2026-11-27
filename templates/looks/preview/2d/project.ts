import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "look-preview",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1,
  "create": {
    "form": "2d"
  }
});
