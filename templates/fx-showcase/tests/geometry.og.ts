// The geometry station (depth S1): a procedural terrain rewritten every
// tick, a tube along a spline, the new primitives, a point cloud, dashed
// screen lines and the physics debug view. The probe's render raycast hits
// the terrain under it; the dashed frame along the bottom edge is dashes,
// not a solid line; the physics view draws the terrain's collider edges in
// orange over it; the terrain moves between the two frames.
import { changed, stationPlaytest } from "./station.ts";

export default stationPlaytest("geometry", 1.5, ["geo"], ({ t, field, frame, before }) => {
  t.equal(field<number>("geo", "GEO", "hits"), 1, "the probe's ray hits the terrain (og.render.raycast, nearest only)");
  const d = field<number>("geo", "GEO", "distance");
  t.ok(d > 2.4 && d < 3.9, `the hit is on the terrain, 4 m under the probe less its height (${d.toFixed(2)} m)`);
  // The screen frame's bottom edge (y 696): yellow dashes and gaps.
  let on = 0;
  for (let x = 330; x < 1250; x++) {
    const i = (696 * frame.width + x) * 4;
    if (frame.rgba[i]! > 200 && frame.rgba[i + 1]! > 170 && frame.rgba[i + 2]! < 160) on++;
  }
  const dashed = on / 920;
  t.ok(dashed > 0.4 && dashed < 0.8, `the screen frame is dashed (${(dashed * 100).toFixed(0)}% of its bottom edge drawn)`);
  // The physics view: rapier's orange collider edges over the terrain.
  let orange = 0;
  for (let y = 300; y < 640; y++) {
    for (let x = 400; x < 1100; x++) {
      const i = (y * frame.width + x) * 4;
      const [r, g, b] = [frame.rgba[i]!, frame.rgba[i + 1]!, frame.rgba[i + 2]!];
      if (r > 200 && g > 110 && g < 200 && b < 70) orange++;
    }
  }
  t.ok(orange > 10000, `the physics view draws the terrain's collider (${orange} orange pixels)`);
  const moving = changed(frame, before, 20);
  t.ok(moving > 2000, `the terrain is rewritten every tick (${moving} pixels changed in half a second)`);
});
