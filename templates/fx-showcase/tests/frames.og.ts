// Every station's frame at 1280×720 after its effects have run, compared
// with references/<tier>/<station>.png: post (AO, outlines, god rays,
// bloom, the teal-orange LUT, vignette, aberration, grain), lens (depth of
// field, motion blur), fx (fire, smoke and colliding sparks, decals, a
// trail, a jittering beam, a ribbon), 2d (2D lights, occluders and sprite
// shadows, glow, flash, 2D embers), stylised (toon, rim, dissolve, hit
// flash, outline, the skinned walker). Every frame carries the screen-space
// HUD and the og.ui badge over the finished frame.
import { playtest } from "@origingame/engine/playtest";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { checkFrames, decodePng, outDir } from "./frames.ts";

export const SCRIPT = [
  "1.0:screenshot:post.png",
  "1.1:station:lens",
  "2.0:screenshot:lens.png",
  "2.1:station:fx",
  "4.0:screenshot:fx.png",
  "4.1:station:2d",
  "5.0:screenshot:2d.png",
  "5.1:station:stylised",
  "7.0:screenshot:stylised.png",
  "7.1:dump_entity:stage:stage.json",
  "7.1:dump_world:world.json",
  "7.2:quit",
];
const FILES = ["post.png", "lens.png", "fx.png", "2d.png", "stylised.png"];

export default playtest({
  name: "every station matches its reference frame",
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
    // The HUD's accent bar is opaque linear (1, 0.62, 0.08): sRGB (255, 207,
    // 79) exactly on every station, after tone mapping, the LUT, vignette,
    // aberration, grain, bloom and TAA, because the overlay comes after them.
    t.frame(FILES, () => {
      for (const f of FILES) {
        const img = decodePng(readFileSync(join(outDir(), f)));
        const at = (18 * img.width + 19) * 4;
        const px = [img.rgba[at]!, img.rgba[at + 1]!, img.rgba[at + 2]!];
        t.ok(Math.abs(px[0]! - 255) <= 2 && Math.abs(px[1]! - 207) <= 2 && Math.abs(px[2]! - 79) <= 2, `${f}: the HUD bar is exactly its colour (${px.join(", ")})`);
      }
    });
    checkFrames(run, t, FILES, "");
  },
});
