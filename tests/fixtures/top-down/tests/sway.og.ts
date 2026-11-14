// The west meadow in a fresh breeze as `og test` plays it: the tree and the
// pine are textures cooked with wind parts (assets/trees/*.png.ogt), drawn
// by og.Sprite.sway as og.Wind moves them, and the frame matches its L4
// reference (references/vulkan/farm-wind.png). In still air the same trees
// are their pictures: tests/walk.og.ts's frame.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("the meadow's trees sway in a fresh breeze as recorded", async () => {
  const run = await runPlaytest({
    size: "960x540",
    commands: [
      "0.1:look:10:11.5",
      "0.1:wind:8",
      "2.2:screenshot:farm-wind.png",
      "2.2:expect:frame:references/vulkan/farm-wind.png:rmse:<=3",
      "2.3:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
});
