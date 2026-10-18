// The fluid station: og.Fluid2D water held in a basin and drawn as
// metaballs, a viscous orange fluid, and an og.WaterLine pond (drawn by
// og.Water2D) where a dropped crate floats and the surface still waves.
import { mean, moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("fluid", 2, ["pond-crate"], ({ t, frame, before, field }) => {
  const y = field<number[]>("pond-crate", "Transform", "position")[1]!;
  t.ok(y > -2.6 && y < -1.0, `the crate floats in the pond (y ${y.toFixed(2)})`);
  const water = mean(frame, 420, 440, 700, 480);
  t.ok(water[2] > water[0] + 40, `the basin holds blue fluid (rgb ${water.map((c) => c.toFixed(0)).join(", ")})`);
  t.ok(moved(before, frame, 960, 300, 1280, 560) > 0.3, "the pond waves");
});
