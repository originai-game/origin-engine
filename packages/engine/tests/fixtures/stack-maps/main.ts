// A game whose system throws on its third tick (tests/sourcemap.test.ts):
// its stack must name this file's lines, not the bundle's.
import { og, defineSystem } from "@origingame/engine";

interface Well {
  readonly depth: number;
}

function drawWater(well: Well): number {
  if (well.depth > 2) throw new Error(`the well is dry at depth ${well.depth}`);
  return well.depth;
}

export const Dig = defineSystem("Dig", {}, (_q, time) => {
  drawWater({ depth: time.tick });
});

export default og.game({ tick: 60, systems: [Dig] });
