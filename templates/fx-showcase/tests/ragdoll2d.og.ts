// The ragdoll2d station: in a 2D world (og.Physics.profile 2) the puppet
// walks on its ledge for half a second, then goes limp: og.Ragdoll's blend
// reached 1, the hit knocked it off the 1 m ledge (its lowest bone fell
// below 0.6 m), and its bones stayed in the xy plane at their animated z
// (near limbs at +0.06, far ones at -0.06).
import { stationPlaytest } from "./station.ts";

export default stationPlaytest("ragdoll2d", 2, ["p2", "puppet"], ({ t, field }) => {
  t.equal(field<number>("puppet", "Ragdoll", "blend"), 1, "the puppet went fully limp");
  const low = field<number>("p2", "P2", "ragdollLow");
  t.ok(low < 0.6, `it fell off the ledge (lowest bone at ${low.toFixed(2)} m)`);
  const z = field<number>("p2", "P2", "ragdollZ");
  t.ok(z > 0.01 && z < 0.07, `its bones kept their layers in the plane (largest |z| ${z.toFixed(3)})`);
});
