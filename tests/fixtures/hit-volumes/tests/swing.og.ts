// The spinning walker as `og test` plays it: its hand's volume strikes each
// thin dummy and the wall once a turn, though no tick's pose touches them,
// and the lever behind the wall is a candidate out of sight. Two runs end
// on one hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("a swept hand strikes each target once a swing; a lever behind the wall is not visible", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["a dummy is struck"],
    invariants: ["state:dups==0", "state:dummyA<=$cap", "state:dummyB<=$cap", "state:wall<=$cap"],
    commands: [
      // 180 ticks: 90 radians, 14 whole turns.
      "3:expect:state:turns==14",
      // Struck in every turn but at most the one under way.
      "3:expect:state:dummyA>=14",
      "3:expect:state:dummyB>=14",
      "3:expect:state:wall>=14",
      "3:expect:state:leversSeen==1",
      "3:expect:state:leversHidden==1",
      "3:expect:state:nearestLever==1",
      "3.02:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
  expect(run.hashes[0], "two runs end on one hash").toBe(run.hashes[1]!);
});
