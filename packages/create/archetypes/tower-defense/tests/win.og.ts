// The round held to the end: a stand of thorns and frost lilies at the
// gate, the last wave called early with N (its bonus paid), every visitor
// turned away; the end card lands with its stars and score, the score is
// saved as the best (a store write at the revision read on tick 0,
// whose answer moves the revision on for the next one),
// submitted to the board and filed as telemetry; Title goes back.
import { playtest } from "@origingame/engine/playtest";
import { game, OVER, type PlatformDump, press, tap, TITLE } from "./lib.ts";

const STAND = ["thorn:3:2", "thorn:5:2", "frost:4:3", "thorn:2:4", "thorn:6:4", "frost:3:5", "frost:5:5", "thorn:4:7", "thorn:2:7", "thorn:6:7"].map((p) => `0.5:plant:${p}`).join(";");

export default playtest({
  name: "a strong stand holds the last wave, the end card shows the win, the best is saved and submitted",
  commands: [
    "0.1:set:Tuning.spawnGap=0.5",
    "0.3:begin",
    "0.4:gold:2000",
    STAND,
    "0.6:wave:5",
    "0.7:get:Game:before.json",
    press("KeyN", 1.0),
    "1.2:get:Game:called.json",
    "22.0:get:Game:won.json",
    "23.0:screenshot:won.png",
    "23.1:dump_platform:platform.json",
    tap("Menu", 23.5),
    "24.4:get:Game:title.json",
    "24.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = game(run, "before.json");
    t.equal(before.placed, 10, "ten plants in the stand");
    const called = game(run, "called.json");
    t.equal(called.wave, 5, "N called the last wave at once");
    t.ok(called.inWave, "and it is under way");
    const won = game(run, "won.json");
    t.equal(won.phase, OVER, "the round is over");
    t.equal(won.won, true, "won");
    t.equal(won.leaks, 0, "nobody reached the doorstep");
    t.equal(won.kills, 18, "all eighteen visitors of the last wave turned away");
    t.equal(won.score, won.kills * 10 + won.lives * 50 + won.gold, "the score: kills × 10 + light × 50 + coins");
    t.ok(won.newBest && won.best === won.score, "a new best");
    const p = run.json<PlatformDump>("platform.json");
    const of = (m: string) => p.calls.filter((c) => c.method === m);
    const read = of("store.get").find((c) => c.args["key"] === "best");
    t.ok(read?.tick === 0 && read.result?.data["value"] === null, "tick 0 read `best` from the store (never written)");
    const set = of("store.set")[0];
    t.equal(set?.args["value"], won.score, "the best was written");
    t.equal(set?.args["revision"], 0, "at the revision read (never written before)");
    t.equal(set?.result?.data["revision"], 1, "its first revision");
    t.equal(of("leaderboard.submit")[0]?.args["score"], won.score, "the score went to the board");
    t.equal(of("telemetry")[0]?.args["name"], "round.over", "a round.over event was filed");
    const title = game(run, "title.json");
    t.equal(title.phase, TITLE, "Title went back to the title");
    t.equal(title.bestRevision, 1, "the write's answer moved the revision on, so the next best is written at 1 and not refused as stale");
  },
});
