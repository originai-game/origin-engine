// Determinism: a run with taps, purchases, a milestone and the motes
// circling, run twice in fresh processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, ORB, press, tap, tapAt } from "./lib.ts";

export default playtest({
  name: "a run with taps, purchases and a milestone replays to the same hash",
  runs: 2,
  commands: [
    tap("play", 0.3),
    ...Array.from({ length: 8 }, (_, i) => tapAt(ORB.x, ORB.y, +(0.9 + i * 0.1).toFixed(2))),
    "1.8:sparks:400",
    press("Digit1", 1.9),
    press("Digit1", 2.0),
    press("Digit2", 2.1),
    "2.2:own:0:9",
    press("Digit1", 2.3),
    "3.5:get:Game:end.json",
    "3.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.taps === 8 && end.owned0 === 10, `the run had its taps (${end.taps}) and a milestone (${end.owned0} embers)`);
  },
});
