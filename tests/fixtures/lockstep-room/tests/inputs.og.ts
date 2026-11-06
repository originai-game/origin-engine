import { playtest } from "@origingame/engine/playtest";

interface Inputs {
  value: { sum0: number; sum1: number };
}

export default playtest({
  name: "two seats apply distinct inputs to one shared world",
  commands: "0:bot:1;2:get:Duel:inputs.json;2.1:quit",
  room: {
    players: 2,
    commands: ["0:bot:1;2:get:Duel:inputs.json;2.1:quit", "0:bot:2;2:get:Duel:inputs.json;2.1:quit"],
    name: "inputs",
  },
  require: ["both seats supply inputs"],
  check(run, t) {
    t.equal(run.peers.length, 1, "one peer beside the run");
    const peer = run.peers[0]!;
    t.equal([run.tick, peer.tick], [126, 126], "both quit on the shared tick");
    t.ok(run.lockstep?.running === true && peer.lockstep?.running === true, "both worlds ran lockstep");
    t.equal([run.lockstep?.peer, peer.lockstep?.peer].sort(), [0, 1], "each seat has one peer");
    t.equal([run.lockstep?.desync, peer.lockstep?.desync], [null, null], "no tick desynchronized");
    t.ok(run.netHash.length > 0, "the run has a shared hash");
    t.equal(peer.netHash, run.netHash, "both worlds end with the same hash");
    const inputs = run.json<Inputs>("inputs.json").value;
    t.ok(inputs.sum0 > 0 && inputs.sum1 < 0, "each seat applied its distinct input");
    t.equal(peer.json<Inputs>("inputs.json").value, inputs, "both peers read the same input totals");
  },
});
