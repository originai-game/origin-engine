// The fixture story.test.ts cooks and loads (assets/story, content/strings).
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "story-fixture",
  "entry": "main.ts",
  "create": {
    "form": "2d"
  },
  "tick": 60
});
