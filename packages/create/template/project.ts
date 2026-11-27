import { defineProject } from "@origingame/engine/author";

export default defineProject({
  id: "origin-game",
  entry: "src/main.ts",
  tick: 60,
  seed: 1,
  publish: { probe: "player", out: "dist" },
});
