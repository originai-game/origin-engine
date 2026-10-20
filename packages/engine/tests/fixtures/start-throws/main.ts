// A game whose GameDesc.start throws (examples/lantern-vale/FINDINGS.md #10):
// og run, og test and og dev fail with the error and its stack, rather than
// running on a half-built world (tests/og-run.test.ts).
import { og } from "@origingame/engine";

function buildTheVillage(): void {
  throw new Error("the village has no well");
}

export default og.game({ start: () => buildTheVillage() });
