import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

interface Inputs {
  value: { sum0: number; sum1: number };
}

test("two seats apply distinct inputs to one shared world", async () => {
  const run = await runPlaytest({
    commands: "0:bot:1;2:get:Duel:inputs.json;2.1:quit",
    room: {
      players: 2,
      commands: ["0:bot:1;2:get:Duel:inputs.json;2.1:quit", "0:bot:2;2:get:Duel:inputs.json;2.1:quit"],
      name: "inputs",
    },
    require: ["both seats supply inputs"],
  });
  expect(run.peers, "one peer beside the run").toHaveLength(1);
  const peer = run.peers[0]!;
  expect([run.tick, peer.tick], "both quit on the shared tick").toEqual([126, 126]);
  expect(run.lockstep?.running === true && peer.lockstep?.running === true, "both worlds ran lockstep").toBe(true);
  expect([run.lockstep?.peer, peer.lockstep?.peer].sort(), "each seat has one peer").toEqual([0, 1]);
  expect([run.lockstep?.desync, peer.lockstep?.desync], "no tick desynchronized").toEqual([null, null]);
  expect(run.netHash.length, "the run has a shared hash").toBeGreaterThan(0);
  expect(peer.netHash, "both worlds end with the same hash").toBe(run.netHash);
  const inputs = run.json<Inputs>("inputs.json").value;
  expect(inputs.sum0 > 0 && inputs.sum1 < 0, "each seat applied its distinct input").toBe(true);
  expect(peer.json<Inputs>("inputs.json").value, "both peers read the same input totals").toStrictEqual(inputs);
});
