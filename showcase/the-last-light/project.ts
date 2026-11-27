import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "the-last-light",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1,
  "publish": {
    "probe": "player",
    "out": "dist"
  },
  "create": {
    "genre": "open-world",
    "view": "third-person-3d",
    "look": "cinematic-realism",
    "ui": "default",
    "platform": "desktop",
    "input": "keyboard-mouse",
    "players": "single",
    "lang": [
      "en"
    ],
    "music": "ambient"
  }
});
