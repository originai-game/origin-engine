import { defineProject, ref } from "@origingame/engine/author";
import { timing } from "./lib/timing.ts";

export default defineProject({
  id: "ts-startup",
  entry: "src/main.ts",
  ...timing,
  saveVersion: 4,
  scene: "harbor",
  scenes: {
    harbor: ref("base/scenes/harbor"),
    inland: ref("base/scenes/inland"),
  },
  roots: [ref("base/fields/depth")],
  modules: [{ id: "island", root: "modules/island" }],
});
