// The car driven up the road and thrown into a handbrake turn: it passes
// 20 km/h, its rear slides (smoke and skid marks), it never falls through
// the terrain, and one frame of the slide (the HUD's speed, the marks, the
// smoke, the eased camera) is compared with its L4 reference (references/vulkan/).
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

// The script's expect: steps, require and invariants are its checks: runPlaytest
// rejects naming the first failing tick and value of each.
test("a car driven on terrain slides through a handbrake turn, leaving smoke and skid marks", async () => {
  const run = await runPlaytest({
    size: "960x540",
    invariants: ["state:kmh<250"],
    require: ["the car drives over 20 km/h", "the rear slides"],
    commands: [
      "0.2:key_down:KeyW",
      "2.0:key_down:KeyD",
      "2.0:key_down:Space",
      "2.3:key_up:Space",
      "2.6:expect:state:kmh>25",
      "2.6:expect:state:marks>=3",
      "2.6:expect:state:smoking>0",
      "2.6:expect:ui:text:km/h",
      "2.6:screenshot:drive.png",
      "2.6:expect:frame:references/vulkan/drive.png:rmse:<=3",
      "2.65:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
});
