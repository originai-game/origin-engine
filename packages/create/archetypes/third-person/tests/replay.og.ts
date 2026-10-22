// Determinism: a run with walking, running, a jump, a beacon kindled and
// a sentinel's chase and grab (the character controller, the animation
// graph and IK, og.nav and og.ai), run twice in fresh processes, ends on
// the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, press, tap } from "./lib.ts";

export default playtest({
  name: "a run with movement, a jump, kindling and a sentinel's grab replays to the same hash",
  runs: 2,
  commands: [
    tap("play", 0.7),
    "1.0:key_down:KeyW",
    "1.2:key_down:ShiftLeft",
    press("Space", 1.6),
    "2.2:key_up:ShiftLeft",
    "2.4:key_up:KeyW",
    "2.5:warp:0",
    "2.7:key_down:KeyF",
    "4.6:key_up:KeyF",
    "4.7:warp:-29.5:1",
    "8.0:get:Game:end.json",
    "8.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.jumps === 1 && end.lit === 1 && end.grabs >= 1, `the run jumped, lit a beacon and was grabbed (${end.jumps}, ${end.lit}, ${end.grabs})`);
  },
});
