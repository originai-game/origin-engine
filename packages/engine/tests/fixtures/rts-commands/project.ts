import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "rts-commands",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60,
  "seed": 1
});
