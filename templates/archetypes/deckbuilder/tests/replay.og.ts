// Determinism: a run begun on the seed Begin draws, its hand walked and
// played and its turns ended, run twice in fresh processes, ends on the
// same world hash: every shuffle, reward and intent's target is drawn from
// og.Rng, so a room's peers deal the same cards.
import { playtest } from "@origingame/engine/playtest";
import { beginRun, endTurn, game, press, script } from "./lib.ts";

export default playtest({
  name: "a run with plays, shuffles and the foes' turns replays to the same hash",
  runs: 2,
  commands: script([
    beginRun(0.6),
    press("Enter", 2.2),
    press("ArrowRight", 2.4),
    press("Enter", 2.6),
    endTurn(2.9),
    press("Enter", 5.2),
    press("ArrowUp", 5.4),
    press("Enter", 5.6),
    endTurn(5.9),
    "8.4:get:Game:end.json",
    "8.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.turn >= 3 && end.played >= 3, `the fight took turns and plays (turn ${end.turn}, ${end.played} played)`);
  },
});
