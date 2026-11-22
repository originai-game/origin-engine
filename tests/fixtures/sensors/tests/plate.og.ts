// The plate as `og test` plays it (src/main.ts): every tick the Plate
// system's og.assert.always sites hold (sensorContents in entity order, the
// crates over the plate, what the events add up to) and two runs end on
// the same world hash.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

test("the plate holds the crates over it every tick and replays to one hash", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["two crates on the plate at once"],
    commands: [
      // Crate b and crate c are both on the plate from tick 33 to 37.
      "0.6:expect:state:count==2",
      "0.6:expect:state:inside==6",
      // Every crate has crossed: each entered once and left once.
      "1:expect:state:count==0",
      "1:expect:state:entered==3",
      "1:expect:state:exited==3",
      "1.05:quit",
    ].join(";"),
  });
  expect(run.hashes.length).toBe(2);
  expect(run.hashes[1]).toBe(run.hashes[0]);
});
