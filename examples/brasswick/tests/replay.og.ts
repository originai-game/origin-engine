// Determinism: the Boiler Stair played by keys alone from New game to the
// hatch (gears, the spike pit, two crates, the plate and its door, the lamp's
// save, four wall jumps, the exit's cheer and the results), run twice in
// fresh processes, ends on the same world hash, and on the same tick each
// time: the crates' physics, the controller and the save replay exactly.
import { playtest } from "@origingame/engine/playtest";
import { boilerStair, game, RESULTS, script, tap } from "./lib.ts";

export default playtest({
  name: "the first level played by keys from the title to the results replays to the same hash",
  runs: 2,
  commands: script(
    tap("new", 0.5),
    ...boilerStair(0.5),
    "8.0:get:Game:mid.json",
    "16.0:get:Game:end.json",
    "16.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const mid = game(run, "mid.json");
    t.ok(mid.checkpoint === 0 && mid.switches === 1, `half-way the lamp is lit and the crate holds the plate (checkpoint ${mid.checkpoint}, switches ${mid.switches})`);
    const end = game(run, "end.json");
    t.equal(end.phase, RESULTS, "the hatch led to the results");
    t.equal(end.gears, 5, "five of the six gears on the way");
    t.equal(end.deaths, 0, "without a fall");
    t.ok(end.wallJumps >= 3 && end.presses >= 1 && end.opens >= 1 && end.saves >= 1, `wall jumps (${end.wallJumps}), a plate (${end.presses}), its door (${end.opens}) and the lamp's save (${end.saves})`);
    t.ok(end.unlocked >= 2, "the second level unlocked");
  },
});
