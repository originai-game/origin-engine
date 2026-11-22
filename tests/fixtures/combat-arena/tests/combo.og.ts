// The combo as `og test` plays it: J three times chains light 1, light 2
// and the heavy; light 2 lands inside the dummy's i-frames and takes
// nothing, the heavy kills it and Died is sent once; a fourth light strikes
// the dead dummy for nothing; a dodge during its active frames is refused
// and one in recovery cuts it. Two runs end on one hash. Tick-exact windows
// and saving mid-swing are in packages/engine/tests/combat.test.ts.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.02).toFixed(2)}:key_up:${key}`;

test("light-light-heavy chains, i-frames ignore a hit, Died once, dodge only in the cancel window", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["the dummy takes damage"],
    invariants: ["state:died<=1", "state:damaged<=2", "state:health>=0"],
    commands: [
      tap(0.5, "KeyJ"), // light 1 (tick 30)
      tap(0.65, "KeyJ"), // light 2, inside light 1's link window
      tap(0.8, "KeyJ"), // the heavy, inside light 2's
      "1.4:expect:state:seq==57", // 1 | 2 << 2 | 3 << 4
      "1.4:expect:state:damaged==2",
      "1.4:expect:state:dealt==40",
      "1.4:expect:state:died==1",
      "1.4:expect:state:dead==1",
      tap(1.5, "KeyJ"), // light 1 again, into a dead dummy
      tap(1.55, "KeyK"), // its active frames: refused
      tap(1.75, "KeyK"), // its recovery: cut
      "2.2:expect:state:seq==121", // ... | 1 << 6
      "2.2:expect:state:hits==4",
      "2.2:expect:state:damaged==2",
      "2.2:expect:state:died==1",
      "2.2:expect:state:refused==1",
      "2.2:expect:state:dodges==1",
      "2.2:expect:state:idle==1",
      "2.22:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
  expect(run.hashes[0], "two runs end on one hash").toBe(run.hashes[1]!);
});
