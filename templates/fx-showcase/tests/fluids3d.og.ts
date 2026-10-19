// The fluids3d station: two og.Fluid3D of one group in a tiled basin. The
// emitter poured all 500 particles of the heavy orange fluid (density 3,
// mixing 0) into the blue pool's left end and they sank under it; no
// particle left the basin. In the frame the pool is blue over the tiles,
// the orange shows through it where it lies, the surface still moves half a
// second on, and the low front wall in front of it does not.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("fluids3d", 5, ["sph"], ({ t, frame, before, field }) => {
  const n = (f: string) => field<number>("sph", "SPH", f);
  t.equal(n("poolCount"), 1600, "the pool keeps its 1600 particles");
  t.equal(n("honeyCount"), 500, "the emitter filled the orange fluid to its maxCount (500)");
  t.equal(n("outside"), 0, "no particle left the basin");
  const pool = n("poolY");
  const honey = n("honeyY");
  t.ok(honey + 0.03 < pool, `the heavy orange fluid sank under the blue (orange ${honey.toFixed(3)} m, blue ${pool.toFixed(3)} m)`);
  const blue = mean(frame, 760, 300, 900, 360);
  t.ok(blue[2] > blue[0] + 50, `the pool's right end is blue (${blue.map((c) => c.toFixed(0))})`);
  const orange = mean(frame, 480, 320, 640, 360);
  t.ok(orange[0] > orange[2] + 80, `the orange fluid shows through where it lies (${orange.map((c) => c.toFixed(0))})`);
  const surface = moved(before, frame, 280, 200, 960, 480);
  t.ok(surface > 1, `the surface still moves (${surface.toFixed(2)} per channel over the basin)`);
  const wall = moved(before, frame, 300, 520, 900, 580);
  t.ok(wall < 0.5, `the front wall does not (${wall.toFixed(2)})`);
});
