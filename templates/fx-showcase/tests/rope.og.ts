// The rope station: a ball swinging on a rope joint with an og.Rope drawn
// to it, a slack rope in the wind, and two og.Cloth flags in gusts. The
// ball stays within the rope's reach of its anchor; the flags move.
import { moved, stationPlaytest } from "./station.ts";

export default stationPlaytest("rope", 2, ["rope-ball"], ({ t, frame, before, field }) => {
  const p = field<number[]>("rope-ball", "Transform", "position");
  const reach = Math.hypot(p[0]! - 1098, p[1]! - 4.05);
  t.ok(reach <= 2.35, `the ball is within the rope's reach (${reach.toFixed(2)} m)`);
  t.ok(moved(before, frame, 700, 100, 1180, 420) > 1, "the flags move in the wind");
});
