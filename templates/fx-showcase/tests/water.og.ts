// The water station: og.Water over an og.WaterSurface. A crate dropped in
// at 0.33 s floats at the surface (buoyancy) and its splash still ripples
// the water when the frame is taken.
import { moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("water", 3, ["water-crate"], ({ t, frame, before, field }) => {
  const y = field<number[]>("water-crate", "Transform", "position")[1]!;
  t.ok(y > -0.7 && y < 0.6, `the crate floats at the surface (y ${y.toFixed(2)})`);
  t.ok(moved(before, frame, 200, 360, 1080, 720) > 0.5, "the water moves");
});
