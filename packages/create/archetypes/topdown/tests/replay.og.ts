// Determinism: a whole visit (walking past the pillar, four swings of which
// two land with their hit stops, a coin, the key, the door, the east room,
// a dash) replayed in two fresh processes ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { hold, press, report } from "./lib.ts";

export default playtest({
  name: "a whole visit replays to the same hash",
  runs: 2,
  commands: [
    "0.2:key_down:KeyD",
    "0.9:key_down:KeyW",
    "1.3:key_up:KeyW",
    "1.6:key_up:KeyD",
    press("Space", 1.7),
    press("Space", 2.2),
    press("Space", 2.7),
    press("Space", 3.2),
    "3.5:key_down:KeyS",
    "4.0:key_down:KeyD",
    "4.5:key_up:KeyS",
    "5.0:key_up:KeyD",
    "5.1:place:15.2:2.3",
    hold("KeyD", 5.1, 5.5),
    "5.6:place:17.5:6",
    "5.6:key_down:KeyD",
    press("ShiftLeft", 6.8),
    "7.4:key_up:KeyD",
    "7.5:dump_entity:hero:end.json",
    "7.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = report(run, "end.json");
    t.equal(end.doorOpen, true, "the visit opened the door");
    t.equal(end.room, 2, "and ended in room 2");
  },
});
