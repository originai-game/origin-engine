// A default-pack game with Chinese strings: scripts/text-l4/cold-*.ts time its first Han glyph.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "cjk-cold",
  "entry": "src/main.tsx",
  "tick": 60,
  "seed": 1,
  "publish": {
    "probe": "all"
  }
});
