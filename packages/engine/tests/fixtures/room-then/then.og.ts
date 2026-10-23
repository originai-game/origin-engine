// A room playtest with a `then` step, on the arena archetype (rooms.test.ts):
// each place ends its script by `quit`, not by room.ticks, and the step
// relaunches both places into a room of their own. Lockstep keeps every
// step on the room's shared ticks, so both quits land on one tick.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "a room's quit lands on one tick for every place, in the script and in its then step",
  commands: "0:bot:1;3:get:Duel:duel.json;4:quit",
  room: { players: 2, commands: ["0:bot:1;3:get:Duel:duel.json;4:quit", "0:bot:2;3:get:Duel:duel.json;4:quit"], name: "then" },
  then: [{ commands: "0:bot:2;2:get:Duel:duel.json;2.5:quit" }],
  check(run, t) {
    const peer = run.peers[0]!;
    t.equal([run.tick, peer.tick], [240, 240], "both places quit on tick 240");
    t.ok(run.quit && peer.quit, "both quit");
    t.equal(peer.netHash, run.netHash, "the worlds hash the same where they quit");
    t.equal(peer.json("duel.json"), run.json("duel.json"), "the reads at 3 s are of one world");
    const step = run.then[0]!;
    t.equal(step.peers.length, 1, "the then step relaunched both places");
    t.equal([step.tick, step.peers[0]!.tick], [150, 150], "the step's quit lands on tick 150 for both");
    t.equal(step.peers[0]!.netHash, step.netHash, "the step's worlds hash the same");
    t.equal(step.peers[0]!.json("duel.json"), step.json("duel.json"), "the step's reads are of one world");
  },
});
