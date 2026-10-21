// Determinism (og-core §2, og-feel §13.8): a session with the imps' chase and
// bites, strikes with their hit stops, a warden's slam with its root-motion
// lunge, the cape and the banners in the wind, a rest at a waystone (a
// snapshot saved), the gate, the bell and the results, run twice in fresh
// processes, ends on one world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, script } from "./lib.ts";

export default playtest({
  name: "a session of chases, bites, strikes, a slam, a rest, the gate and the bell replays to the same hash",
  runs: 2,
  commands: script(
    "0.3:play:0:quick",
    "0.4:warp:-3:31",
    hold("KeyA", 0.6, 0.5),
    press("KeyJ", 1.4),
    press("KeyJ", 1.9),
    press("KeyJ", 2.4),
    press("KeyJ", 2.9),
    "3.3:warp:waystone-1",
    press("KeyF", 3.5),
    "3.8:warp:6:-8",
    press("KeyJ", 5.0),
    press("KeyJ", 5.5),
    hold("KeyS", 6.0, 0.8),
    "7.2:give:sigils:3",
    "7.3:warp:gate",
    press("KeyF", 7.5),
    "8.0:warp:bell",
    press("KeyF", 8.3),
    press("Enter", 9.0),
    press("Enter", 9.4),
    press("Enter", 9.8),
    press("Enter", 10.2),
    press("Enter", 10.6),
    press("Enter", 11.0),
    "12.6:get:Game:end.json",
    "12.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.hits >= 2 && end.hurts >= 2 && end.saves >= 1 && end.phase === 3, `the session struck, was bitten and slammed, rested and rang the bell (${end.hits} hits, ${end.hurts} hurts, ${end.saves} saves, phase ${end.phase})`);
  },
});
