// Determinism: a session with walking, running, the pile shoved, a lantern
// lit, a slime's chase and touch (hit stop, knock-back), the far side
// streamed in and a look switched at the results, run twice in fresh
// processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game } from "./lib.ts";

export default playtest({
  name: "a session with the pile, a lantern, a slime's touch, the far side and the results replays to the same hash",
  runs: 2,
  commands: [
    "0.3:run",
    "0.4:warp:4.5:13.5",
    "0.5:key_down:KeyW",
    "1.5:key_down:ShiftLeft",
    "2.5:key_up:ShiftLeft",
    "2.5:key_up:KeyW",
    "2.6:warp:0",
    "2.7:key_down:KeyE",
    "4.1:key_up:KeyE",
    "4.2:warp:-19:0",
    "6.0:warp:8:-20",
    "7.0:light:1",
    "7.1:light:2",
    "7.2:light:3",
    "9.5:get:Game:end.json",
    "9.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.lit === 4 && end.hurts >= 1 && end.rockMoved > 0.3 && end.farLoads === 1 && end.look === 1, `the session lit four, was touched, shoved the pile, streamed the far side and reached the dawn look (${end.lit}, ${end.hurts}, ${end.rockMoved.toFixed(2)}, ${end.farLoads}, ${end.look})`);
  },
});
