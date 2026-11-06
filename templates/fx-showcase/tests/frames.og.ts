// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";

export const SCRIPT = [
  "1.0:screenshot:post.png",
  "1.1:station:lens",
  "2.1:station:fx",
  "4.1:station:2d",
  "5.1:station:stylised",
  "7.0:screenshot:stylised.png",
  "7.1:dump_entity:stage:stage.json",
  "7.1:dump_world:world.json",
  "7.2:quit",
];
const FILES = ["post.png", "stylised.png"];

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: SCRIPT.join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    const stage = run.json<{ components: { Stage: { station: number } } }>("stage.json").components.Stage;
    t.equal(stage.station, 4, "the last station is stylised");
    // The E5 stations are built when first shown (src/e5.ts): none was, so the world holds none of them.
    const world = run.json<{ entities: number }>("world.json");
    t.ok(world.entities < 150, `no E5 station was built (${world.entities} entities)`);
    checkFrames(run, t, FILES, "");
  },
});
