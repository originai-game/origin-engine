import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "browser-host",
  "entry": "src/main.ts",
  "create": {
    "form": "2d"
  }
});
