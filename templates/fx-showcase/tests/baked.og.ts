// The baked station (depth S7): a Cornell box whose static model carries
// the lightmap `og bake lightmaps` wrote (scenes/cornell.lightmap.*). The
// tall block's side facing the red wall is red from its bounce, and the
// ceiling, which the spot under it does not light, is lit by bounces
// alone; unbaked, that side is the sky's blue-grey and the ceiling dark
// (tests/evidence/depth/render-light: 128,80,82 against 94,102,119, and
// 128,127,125 against 48,44,42, at 640 × 360).
import { mean, stationPlaytest } from "./station.ts";

export default stationPlaytest("baked", 1, ["render-light"], ({ t, field, frame }) => {
  t.equal(field<number>("render-light", "LightStage", "baked"), 1, "the station draws its baked lighting");
  const [r, g, b] = mean(frame, 472, 340, 488, 580);
  t.ok(r > 1.3 * g && r > 1.3 * b, `the tall block's side takes red from the wall beside it (${r.toFixed(0)},${g.toFixed(0)},${b.toFixed(0)})`);
  const ceiling = mean(frame, 480, 16, 800, 40);
  const y = 0.2126 * ceiling[0] + 0.7152 * ceiling[1] + 0.0722 * ceiling[2];
  t.ok(y > 90, `the ceiling is lit by bounces (luma ${y.toFixed(0)}; unbaked about 45)`);
});
