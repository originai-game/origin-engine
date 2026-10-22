// The two endings, through Tune writes: with the target at 30 the first line
// wins the round; with one move and a target out of reach, that move loses
// it. Each end card waits for a press, which deals the next round.
import { playtest } from "@origingame/engine/playtest";
import { CASCADE_BOARD, PHASE, matchOf, tap } from "./lib.ts";

export default playtest({
  name: "reaching the target wins, running out of moves loses, a press deals again",
  commands: [
    "0.1:set:board.Tuning.target=30",
    tap(0.5),
    `0.8:deal:${CASCADE_BOARD}`,
    "1.0:swap:2,0:2,1",
    "3.0:dump_entity:board:won.json",
    "3.0:screenshot:won.png",
    "3.1:set:board.Tuning.target=10000",
    "3.1:set:board.Tuning.moves=1",
    tap(3.2),
    "3.3:dump_entity:board:second.json",
    `3.5:deal:${CASCADE_BOARD}`,
    "3.7:swap:2,0:2,1",
    "5.7:dump_entity:board:lost.json",
    "5.7:screenshot:lost.png",
    "5.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const won = matchOf(run, "won.json");
    t.equal(won.phase, PHASE.won, "the first line reached the target of 30");
    t.equal(won.best, won.score, "the best score took the round's");
    const second = matchOf(run, "second.json");
    t.equal(second.phase, PHASE.idle, "a press after the card's hold dealt round two");
    t.equal(second.rounds, 2, "two rounds");
    t.equal(second.movesLeft, 1, "round two has the tuned single move");
    const lost = matchOf(run, "lost.json");
    t.equal(lost.phase, PHASE.lost, "the last move ran out below the target");
    t.equal(lost.movesLeft, 0, "no moves left");
    t.equal(lost.best, won.best, "the best score is kept");
  },
});
