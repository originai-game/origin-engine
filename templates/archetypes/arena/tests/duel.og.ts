// Two processes play one duel in lockstep through a relay (docs/design.md
// §2.10): each drives its own seat by a different scripted bot, and both
// quit on tick 600 (the steps run on the room's shared ticks), where both
// worlds hash the same, no og.Desync was sent, and the coin moved.
import { playtest } from "@origingame/engine/playtest";

interface Duel {
  value: { score0: number; score1: number; coins: number };
}

export default playtest({
  name: "two processes in one room play the same duel to tick 600",
  commands: "0:bot:1;9.5:get:Duel:duel.json;10:quit",
  room: { players: 2, commands: ["0:bot:1;9.5:get:Duel:duel.json;10:quit", "0:bot:2;9.5:get:Duel:duel.json;10:quit"], name: "duel" },
  check(run, t) {
    t.equal(run.peers.length, 1, "one peer beside the run");
    const peer = run.peers[0]!;
    t.equal(run.tick, 600, "the run quit on tick 600");
    t.equal(peer.tick, 600, "the peer quit on tick 600");
    t.ok(run.lockstep?.running === true && peer.lockstep?.running === true, "lockstep ran on both");
    t.equal([run.lockstep?.peer, peer.lockstep?.peer].sort(), [0, 1], "the peers are 0 and 1");
    t.equal(run.lockstep?.desync ?? null, null, "no desync on the run");
    t.equal(peer.lockstep?.desync ?? null, null, "no desync on the peer");
    t.ok(run.netHash.length > 0, "the run has a shared hash");
    t.equal(peer.netHash, run.netHash, `both worlds hash ${run.netHash} at tick 600`);
    const a = run.json<Duel>("duel.json").value;
    const b = peer.json<Duel>("duel.json").value;
    t.ok(a.coins > 0, `the fighters took coins (${a.coins})`);
    // The reads ran on one tick of the room's clock: the same duel, field for field.
    t.equal(b, a, `the peer read the same duel at 9.5 s`);
  },
});
