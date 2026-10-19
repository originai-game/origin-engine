// Determinism: a dive on the seed Dive picks, walked, fought and drunk
// through, run twice in fresh processes, ends on the same world hash, and
// the floor it carved is the same both times.
import { playtest } from "@origingame/engine/playtest";
import { game, press, presses, tap } from "./lib.ts";

export default playtest({
  name: "a dive with steps, strikes and the foes' turns replays to the same hash",
  runs: 2,
  commands: [
    tap("play", 0.3),
    presses(["ArrowUp", "ArrowRight", "ArrowRight", "ArrowDown", "ArrowLeft", "ArrowUp", "ArrowUp", "Space", "ArrowRight", "ArrowRight"], 0.9),
    press("KeyQ", 3.5),
    "4.0:get:Game:end.json",
    "4.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.turns >= 5, `the dive took turns (${end.turns})`);
  },
});
