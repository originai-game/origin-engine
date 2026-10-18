// The controller station: an og.CharacterController walked up three steps
// (each under stepHeight) and a slope, jumping on the way, and off the
// landing onto a lift, which carries it: grounded on the lift when the
// frame is taken.
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("controller", 5, ["hero", "platform"], ({ t, field }) => {
  const p = field<number[]>("hero", "Transform", "position");
  t.ok(p[0]! > 806.2, `it walked up and over (x ${p[0]!.toFixed(2)})`);
  t.ok(field<boolean>("hero", "CharacterController", "grounded"), "it stands on something");
  const top = field<number[]>("platform", "Transform", "position")[1]! + 0.125;
  t.ok(Math.abs(p[1]! - (top + 0.75)) < 0.15, `it rides the lift (hero y ${p[1]!.toFixed(2)}, lift top ${top.toFixed(2)})`);
});
