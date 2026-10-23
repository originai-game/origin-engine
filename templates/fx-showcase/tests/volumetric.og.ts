// The volumetric station (depth S7): fog lit as a medium (og.Fog
// volumetric) in a room the sun of an og.DayNight cycle shines into
// through a window, an og.RectLight panel under the ceiling, a spot with a
// lattice cookie and a lantern with a cube cookie. The sun's patch under
// the window is the floor's brightest; the lattice cookie cuts the spot's
// pool into bright cells and dark bars; the shafts move with the hours
// (an hour every four seconds) between the two frames.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("volumetric", 1, ["rect-panel"], ({ t, field, frame, before }) => {
  t.equal(field<number>("rect-panel", "RectLight", "width"), 1.6, "the panel is a 1.6 m rect light");
  const luma = (c: [number, number, number]) => 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  const patch = luma(mean(frame, 660, 540, 940, 600));
  const shade = luma(mean(frame, 1000, 450, 1180, 500));
  t.ok(patch > shade + 30, `the sun's patch through the window lights the floor (luma ${patch.toFixed(0)} against ${shade.toFixed(0)})`);
  // The spot's pool: the lattice's bars against its cells (lavapipe: the
  // 10th percentile of luma 119, the 90th 151; without a cookie the pool
  // is smooth).
  const pool: number[] = [];
  for (let y = 560; y < 700; y++) {
    for (let x = 380; x < 640; x++) {
      const i = (y * frame.width + x) * 4;
      pool.push(0.2126 * frame.rgba[i]! + 0.7152 * frame.rgba[i + 1]! + 0.0722 * frame.rgba[i + 2]!);
    }
  }
  pool.sort((p, q) => p - q);
  const [low, high] = [pool[Math.floor(pool.length / 10)]!, pool[Math.floor((pool.length * 9) / 10)]!];
  t.ok(high - low > 20, `the lattice cookie cuts the spot's pool into bars and cells (luma ${low.toFixed(0)} to ${high.toFixed(0)})`);
  const sweep = moved(frame, before, 600, 360, 960, 620);
  t.ok(sweep > 2, `the shafts and the patch move with the hours (mean change ${sweep.toFixed(2)} in half a second)`);
});
