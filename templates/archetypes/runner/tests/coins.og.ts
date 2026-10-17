// Coins: one placed at the runner's height is collected and raises the score
// by Tuning.coinValue; one hung above the reach of a standing runner is not.
import { playtest } from "@origingame/engine/playtest";
import { RUNNING, runOf, tap } from "./lib.ts";

export default playtest({
  name: "a coin in reach is collected and scores; one out of reach is not",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    tap(0.5),
    "1.0:dump_entity:runner:before.json",
    "1.0:place:coin:-2",
    "2.0:dump_entity:runner:collected.json",
    "2.0:place:coin:-2:3.2",
    "3.0:dump_entity:runner:missed.json",
    "3.0:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = runOf(run, "before.json");
    const collected = runOf(run, "collected.json");
    const missed = runOf(run, "missed.json");
    t.equal(before.coins, 0, "no coin before the first one arrives");
    t.equal(collected.phase, RUNNING, "still running");
    t.equal(collected.coins, 1, "the coin at the runner's height was collected");
    t.equal(collected.score, Math.floor(collected.distance) + 10, "the score is the distance plus one coin's value");
    t.ok(collected.score > before.score + 10, `collecting raised the score (${before.score} -> ${collected.score})`);
    t.equal(missed.coins, 1, "the coin hung at 3.2 m passed over the standing runner");
  },
});
