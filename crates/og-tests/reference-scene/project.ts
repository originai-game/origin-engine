// The reference scene of docs/design.md §7: 500 instanced objects, two
// shadowed lights, 20 skinned characters, PBR materials and post. The lanes
// bench (packages/engine/bench/lanes.ts) times it natively and in Chrome.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "reference-scene",
  "entry": "src/main.ts",
  "tick": 60,
  "seed": 1,
  "publish": {
    "probe": "player",
    "out": "dist"
  }
});
