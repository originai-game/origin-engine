// The cloth station: a 20 × 20 sheet (6 cm apart, self-collision at a
// thickness of 6 cm) lowered onto the floor while its bar sweeps back and
// forth lays itself in folds: no two of its points that are not near
// neighbours came within 0.7 of the thickness at any tick (none passed
// through a layer), and many points lie on layers above the floor's.
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("cloth", 3, ["anim-physics"], ({ t, field }) => {
  const closest = field<number>("anim-physics", "AnimPhysics", "clothClosest");
  t.ok(closest >= 0.7, `no layer passed through another (closest pair ${closest.toFixed(2)} × the thickness)`);
  const above = field<number>("anim-physics", "AnimPhysics", "clothAbove");
  // The bar's own row (20 points at 12 cm) counts too: 120 is five rows more.
  t.ok(above >= 120, `the sheet piled up in layers (${above} points above the floor's layer)`);
});
