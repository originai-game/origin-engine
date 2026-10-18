// The same script twice, in fresh processes, ends in the same world: a
// refused swap (a yoyo tween), a swap whose line clears and cascades (tween
// groups, flipbook clips, floating scores, camera shake and the refill's
// dice), stopped mid-fall so running tweens, clips and floating text are in
// the hash too.
import { playtest } from "@origingame/engine/playtest";
import { CASCADE_BOARD, PHASE, matchOf, tap } from "./lib.ts";

export default playtest({
  name: "a replay of swaps, clears and a cascade lands on the same world hash",
  runs: 2,
  commands: [
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    "1.0:swap:5,0:6,0",
    "1.5:swap:2,0:2,1",
    "2.75:dump_entity:board:falling.json",
    "2.75:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const m = matchOf(run, "falling.json");
    t.equal([m.phase, m.chain, m.cleared], [PHASE.fall, 2, 6], "it stops while the cascade's gems fall");
    t.equal(run.hashes.length, 2, "the script ran twice");
    t.equal(run.hashes[1], run.hashes[0], "both runs end in the same world");
  },
});
