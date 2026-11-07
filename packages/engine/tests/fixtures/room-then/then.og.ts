// A room playtest with a `then` step, on the input fixture (rooms.test.ts):
// each place ends its script by `quit`, not by room.ticks, and the step
// relaunches both places into a room of their own. Lockstep keeps every
// step on the room's shared ticks, so both quits land on one tick.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("a room's quit lands on one tick for every place, in the script and in its then step", async () => {
  const run = await runPlaytest({
    commands: "0:bot:1;3:get:Duel:duel.json;4:quit",
    room: { players: 2, commands: ["0:bot:1;3:get:Duel:duel.json;4:quit", "0:bot:2;3:get:Duel:duel.json;4:quit"], name: "then" },
    then: [{ commands: "0:bot:2;2:get:Duel:duel.json;2.5:quit" }],
  });
  const peer = run.peers[0]!;
  expect([run.tick, peer.tick], "both places quit on tick 240").toEqual([240, 240]);
  expect(run.quit && peer.quit, "both quit").toBe(true);
  expect(peer.netHash, "the worlds hash the same where they quit").toBe(run.netHash);
  expect(peer.json("duel.json"), "the reads at 3 s are of one world").toStrictEqual(run.json("duel.json"));
  const step = run.then[0]!;
  expect(step.peers, "the then step relaunched both places").toHaveLength(1);
  expect([step.tick, step.peers[0]!.tick], "the step's quit lands on tick 150 for both").toEqual([150, 150]);
  expect(step.peers[0]!.netHash, "the step's worlds hash the same").toBe(step.netHash);
  expect(step.peers[0]!.json("duel.json"), "the step's reads are of one world").toStrictEqual(step.json("duel.json"));
});
