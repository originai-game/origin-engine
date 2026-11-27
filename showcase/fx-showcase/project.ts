import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "fx-showcase",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1,
  "publish": {
    "probe": "player",
    "out": "dist"
  },
  "create": {
    "form": "3d",
    "genre": "open-world",
    "view": "third-person-3d",
    "look": "cinematic-realism",
    "ui": "minimal",
    "platform": "desktop",
    "input": "keyboard-mouse",
    "players": "single",
    "lang": [
      "en"
    ],
    "music": "ambient"
  }
});
