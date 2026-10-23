// The shaders station: og.load.shader materials under a CRT custom post
// pass. The sphere's dissolve amount changes between the two frames
// (og.render.setShaderParam) and the grass sways; the panel drawn by a full
// custom shader (mode full lit) cuts its shadow with its holes; a warm
// grade in PostEffects.custom5 then the CRT in custom8, the last slot
// (og.load.post; the slots between are 0), warm the ground and bend the
// image, leaving its corners black. The ground behind the panel is near
// grey without the grade (red over blue by 8 on lavapipe) and 60 with it.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("shaders", 1.5, [], ({ t, frame, before }) => {
  t.ok(moved(before, frame, 300, 160, 620, 460) > 1, "the sphere dissolves further");
  t.ok(moved(before, frame, 0, 560, 1280, 720) > 1, "the grass sways");
  t.ok(moved(before, frame, 740, 120, 900, 360) > 1, "the full shader's cut opens");
  const ground = mean(frame, 900, 280, 1100, 380);
  t.ok(ground[0] - ground[2] > 35, `the warm grade in slot 5 runs (red over blue by ${(ground[0] - ground[2]).toFixed(1)})`);
  for (const [x, y] of [[4, 4], [1270, 4], [4, 712], [1270, 712]] as const) {
    const c = mean(frame, x, y, x + 6, y + 6);
    t.ok(c[0] + c[1] + c[2] < 10, `the CRT pass in slot 8 leaves the corner at ${x},${y} black`);
  }
});
