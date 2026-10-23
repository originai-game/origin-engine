// The shaders station: og.load.shader materials under a CRT custom post
// pass. The sphere's dissolve amount changes between the two frames
// (og.render.setShaderParam) and the grass sways; the panel drawn by a full
// custom shader (mode full lit) cuts its shadow with its holes; the CRT
// (og.load.post, PostEffects.custom1) bends the image, leaving its corners
// black.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("shaders", 1.5, [], ({ t, frame, before }) => {
  t.ok(moved(before, frame, 300, 160, 620, 460) > 1, "the sphere dissolves further");
  t.ok(moved(before, frame, 0, 560, 1280, 720) > 1, "the grass sways");
  t.ok(moved(before, frame, 740, 120, 900, 360) > 1, "the full shader's cut opens");
  for (const [x, y] of [[4, 4], [1270, 4], [4, 712], [1270, 712]] as const) {
    const c = mean(frame, x, y, x + 6, y + 6);
    t.ok(c[0] + c[1] + c[2] < 10, `the CRT pass leaves the corner at ${x},${y} black`);
  }
});
