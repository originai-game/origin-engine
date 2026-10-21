// Determinism: a session with running and jumping, glimmers, a beetle's
// strikes and a touch (hit stops, knock-back), the lift, the gloam's fall
// in slow motion, a lantern's save and the bell's results, run twice in
// fresh processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, script } from "./lib.ts";

export default playtest({
  name: "a session with strikes, a touch, the lift, the gloam's slow motion, a save and the bell replays to the same hash",
  runs: 2,
  commands: script(
    "0.2:set:Tuning.gloamHp=1",
    "0.3:play",
    hold("ArrowRight", 0.5, 0.6),
    hold("Space", 0.8, 0.3),
    "1.4:warp:beetle-0:-1.6",
    press("KeyX", 1.5),
    press("KeyX", 2.0),
    "2.4:warp:beetle-1:-0.4",
    "3.0:warp:lift:0:0.2",
    "3.6:warp:lantern-1:-1",
    hold("ArrowRight", 3.8, 0.3),
    "4.4:warp:gloam:-1.8",
    press("KeyX", 4.6),
    press("KeyX", 5.0),
    press("KeyX", 5.4),
    "6.4:warp:chime-2:0:-1",
    "6.8:give:chimes:3",
    "6.9:warp:bell:1.2",
    press("ArrowUp", 7.3),
    "10.6:get:Game:end.json",
    "10.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.kills >= 2 && end.hurts >= 1 && end.bossDown && end.saves >= 1 && end.phase === 2, `the session beat a beetle and the gloam, was touched, saved and rang the bell (${end.kills} kills, ${end.hurts} hurts, ${end.saves} saves, phase ${end.phase})`);
  },
});
