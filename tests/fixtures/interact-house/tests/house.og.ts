// The house as `og test` plays it: a hold released early leaves the door
// shut, a full hold opens it and its link, and the guard walks through; the
// player turns to the lever (used), the lever behind the pillar (never
// focused) and the crate, picks it up and throws it onto the plate, which
// opens the gate. Two runs end on one hash. Tick-exact holds and saving
// while carrying are in packages/engine/tests/interact.test.ts.
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

const tap = (at: number, key: string) => `${at}:key_down:${key};${(at + 0.02).toFixed(2)}:key_up:${key}`;

test("hold opens the door and its link, focus follows the turn and sight, a thrown crate holds the plate down", async () => {
  const run = await runPlaytest({
    runs: 2,
    require: ["something is used", "the guard goes through the door"],
    invariants: ["state:hiddenOn==0", "state:interacted<=4"],
    commands: [
      "0.2:expect:state:focus==1", // the door, to the east
      "0.3:key_down:KeyE", // 12 ticks of a 30-tick hold
      "0.5:key_up:KeyE",
      "0.6:expect:state:doorOpen==0",
      "0.6:expect:state:progress==0",
      "0.7:key_down:KeyE", // the whole hold
      "1.4:key_up:KeyE",
      "1.5:expect:state:doorOpen==1",
      tap(1.6, "KeyD"), // north: the lever
      "1.7:expect:state:focus==2",
      tap(1.8, "KeyE"),
      "1.9:expect:state:leverOn==1",
      tap(2.0, "KeyD"), // west: the lever behind the pillar
      "2.1:expect:state:focus==0",
      tap(2.2, "KeyE"),
      tap(2.4, "KeyD"), // south: the crate
      "2.5:expect:state:focus==4",
      tap(2.6, "KeyE"), // picked up
      "2.8:expect:state:holding==1",
      tap(3.0, "KeyE"), // thrown
      "3.1:expect:state:holding==0",
      "4.5:expect:state:plateOn==1",
      "4.5:expect:state:gateOpen==1",
      "4.5:expect:state:interacted==3",
      "4.5:expect:state:linkReached==1",
      "4.5:expect:state:guardWest==1",
      "4.52:quit",
    ].join(";"),
  });
  expect(run.quit, "the script reached quit").toBe(true);
  expect(run.hashes[0], "two runs end on one hash").toBe(run.hashes[1]!);
});
