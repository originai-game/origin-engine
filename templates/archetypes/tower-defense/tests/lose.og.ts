// The failure path: no plants, the light set to two. The first visitors
// walk the open lawn to the doorstep; each takes its light with a shake, a
// hit stop and a red number; the light at 0 ends the round on the end card
// ("The lantern went out"), and Play again starts a fresh round.
import { playtest } from "@origingame/engine/playtest";
import { game, OVER, PLAY, play, tap } from "./lib.ts";

export default playtest({
  name: "unguarded, visitors reach the doorstep, the lantern goes out, the end card offers another round",
  commands: [
    "0.1:set:Tuning.startLives=2",
    play(0.3),
    "1.0:get:Game:start.json",
    "15.0:get:Game:over.json",
    "15.0:get:og.TimeScale:time.json",
    "16.6:screenshot:lost.png",
    tap("Retry", 17.0),
    "17.8:get:Game:again.json",
    "17.9:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = game(run, "start.json");
    t.equal(start.phase, PLAY, "the round started");
    t.equal(start.lives, 2, "with the tuned light");
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "the light ran out: the round is over");
    t.equal(over.won, false, "lost");
    t.equal(over.lives, 0, "no light left");
    t.ok(over.leaks >= 2, `${over.leaks} visitors reached the doorstep`);
    t.equal(over.kills, 0, "nothing was turned away");
    t.equal(run.json<{ value: { scale: number } }>("time.json").value.scale, 1, "the end card leaves the time scale at 1");
    t.equal(run.png("lost.png").width, 1280, "the end card was captured");
    const again = game(run, "again.json");
    t.equal(again.phase, PLAY, "Play again started another round");
    t.equal(again.rounds, 2, "the second round");
    t.equal(again.lives, 2, "with the light back");
    t.equal(again.leaks, 0, "and the count cleared");
  },
});
