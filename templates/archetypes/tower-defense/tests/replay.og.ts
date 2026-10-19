// Determinism: a round with plants by tap and keys, two waves called early,
// shots, frost, pops and a leak, run twice in fresh processes, ends on the
// same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, play, press, tapCell } from "./lib.ts";

export default playtest({
  name: "a round with plants, shots, pops and frost replays to the same hash",
  runs: 2,
  commands: [
    play(0.3),
    tapCell(3, 4, 1.2),
    press("Digit2", 1.4),
    press("ArrowRight", 1.5),
    press("ArrowRight", 1.6),
    press("Space", 1.7),
    press("KeyN", 1.9),
    "9.0:get:Game:end.json",
    "9.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.shots > 0 && end.kills > 0, `the round had shots (${end.shots}) and pops (${end.kills})`);
  },
});
