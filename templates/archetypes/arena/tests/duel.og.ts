// Two processes play one duel in lockstep through a relay (docs/design.md
// §2.10): each drives its own seat by a different scripted bot, and at tick
// 600 both worlds hash the same, no og.Desync was sent, and the coin moved.
import { playtest } from "@origingame/engine/playtest";

interface Duel {
  value: { score0: number; score1: number; coins: number };
}

export default playtest({
  name: "two processes in one room play the same duel to tick 600",
  commands: "0:bot:1;9.5:get:Duel:duel.json",
  room: { players: 2, ticks: 600, commands: ["0:bot:1;9.5:get:Duel:duel.json", "0:bot:2;9.5:get:Duel:duel.json"], name: "duel" },
  check(run, t) {
    t.equal(run.peers.length, 1, "one peer beside the run");
    const peer = run.peers[0]!;
    t.equal(run.tick, 600, "the run stopped at tick 600");
    t.equal(peer.tick, 600, "the peer stopped at tick 600");
    t.ok(run.lockstep?.running === true && peer.lockstep?.running === true, "lockstep ran on both");
    t.equal([run.lockstep?.peer, peer.lockstep?.peer].sort(), [0, 1], "the peers are 0 and 1");
    t.equal(run.lockstep?.desync ?? null, null, "no desync on the run");
    t.equal(peer.lockstep?.desync ?? null, null, "no desync on the peer");
    t.ok(run.netHash.length > 0, "the run has a shared hash");
    t.equal(peer.netHash, run.netHash, `both worlds hash ${run.netHash} at tick 600`);
    const a = run.json<Duel>("duel.json").value;
    const b = peer.json<Duel>("duel.json").value;
    t.ok(a.coins > 0, `the fighters took coins (${a.coins})`);
    // A script's steps run on its own process's ticks (the peer that joins moves to the
    // authority's tick at the start), so the two reads may be a few ticks apart: the
    // hashes above are the exact comparison.
    t.ok(b.coins > 0 && Math.abs(b.coins - a.coins) <= 1, `the peer saw the same duel (${b.coins} coins)`);
  },
});
