// The skinned station: the walker walking, its model's materials given the
// ripple shader with og.render.setMaterialShader. Gold bands climb the
// skinned body (the shader's surface_color on a skinned mesh) and its
// shadow on the ground moves with the walk.
import { changed, stationPlaytest } from "./station.ts";

export default stationPlaytest("skinned", 1, [], ({ t, frame, before }) => {
  let gold = 0;
  for (let y = 200; y < 560; y++) {
    for (let x = 600; x < 900; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      // Yellow: green near red, blue well below (the walker's own browns have green far below red).
      if (r > 150 && g > 0.88 * r && b < 0.75 * r) gold++;
    }
  }
  t.ok(gold > 400, `gold bands on the walker (${gold} pixels)`);
  t.ok(changed(before, frame) > 500, "the walker and its shadow move");
});
