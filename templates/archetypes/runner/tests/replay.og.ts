// Determinism: the same script twice, in fresh processes, ends on the same
// world hash. It covers what the feel added to the world state: dice-made
// patterns, jumps (stretch and squash tweens), a coin's pickup (its pop and
// floating text), the run clip's footfalls (dust puffs), a crash (hit stop,
// shake, knock-back, the flash) and a second round.
import { playtest } from "@origingame/engine/playtest";
import { OVER, RUNNING, runOf, tap } from "./lib.ts";

export default playtest({
  name: "replay: two runs of jumps, coins and a crash end on one hash",
  runs: 2,
  commands: [
    tap(0.5),
    "1.0:place:coin:-2",
    "2.0:place:crate:1",
    tap(2.8),
    "3.6:dump_entity:runner:round1.json",
    "4.0:place:fence:1",
    "5.2:dump_entity:runner:crashed.json",
    tap(5.8),
    "6.4:key_down:Space",
    "6.7:key_up:Space",
    tap(7.6),
    "9.0:dump_entity:runner:round2.json",
    "9.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const round1 = runOf(run, "round1.json");
    t.ok(round1.phase === RUNNING && round1.coins >= 1, `round one jumped the crate and took the coin (${round1.coins} coins)`);
    t.equal(runOf(run, "crashed.json").phase, OVER, "the fence ended round one");
    t.equal(runOf(run, "round2.json").rounds, 2, "round two started");
    t.equal(run.hashes.length, 2, "the script ran twice");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on the same world hash (${run.hashes.join(" vs ")})`);
  },
});
