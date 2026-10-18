// The joints station: the motorised hinge turns, the chain hangs from its
// anchor, the spring holds its box below its anchor, and the walker went
// limp (og.Ragdoll.blend reached 1).
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("joints", 3, ["spinner", "chain-5", "bob", "ragdoll"], ({ t, field }) => {
  const angle = field<number>("spinner", "Joint", "value");
  t.ok(Math.abs(angle) > 1, `the hinge turned (${angle.toFixed(2)} rad)`);
  const end = field<number[]>("chain-5", "Transform", "position")[1]!;
  t.ok(end < 3.2, `the chain hangs (end y ${end.toFixed(2)})`);
  const bob = field<number[]>("bob", "Transform", "position")[1]!;
  t.ok(bob < 3.4 - 0.8 && bob > 0.5, `the spring holds the box (y ${bob.toFixed(2)})`);
  t.equal(field<number>("ragdoll", "Ragdoll", "blend"), 1, "the ragdoll blended fully to its bones");
});
