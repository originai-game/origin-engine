// The flow acceptance fixture (crates/og-runtime/tests/flow_acceptance.rs
// and docs/design.md's film plan): one flow plays a CG intro on footage, a
// 3D physics level the game plays, an in-engine cutscene, a 2D physics
// minigame and a live-action ending. The engine starts and draws the flow;
// src/main.ts holds only the two games.
import { defineProject } from "@origingame/engine/author";

export default defineProject({
  "id": "fused-flow-fixture",
  "entry": "src/main.ts",
  "create": {
    "form": "3d",
    "genre": "interactive-film"
  },
  "tick": 60,
  "seed": 1,
  "flow": {
    "source": {
      "$ogRef": "base/story/game"
    }
  }
});
