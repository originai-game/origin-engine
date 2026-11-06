// Determinism: a run with walking, a jump, the rifle's spread and recoil
// (og.Rng), walkers on the navmesh and the crowd, a kill and a drone's
// charge and bolt, run twice in fresh processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { AIM_RIGHT, game, hold, press, tap } from "./lib.ts";

export default playtest({
  name: "a run with movement, a jump, firing, a kill and a drone's bolt replays to the same hash",
  runs: 2,
  commands: [
    tap("Play", 0.5),
    "0.8:key_down:KeyW",
    press("Space", 1.0),
    "1.4:key_up:KeyW",
    "1.6:warp:-3:2",
    "1.7:aim:0:-5",
    `1.8:spawn:walker:${-3 + AIM_RIGHT}:-8`,
    "1.8:spawn:drone:0:-15",
    hold("KeyF", 2.0, 1.6),
    "5.5:get:Game:end.json",
    "5.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.kills >= 1 && end.shots >= 8, `the run fired and downed a foe (${end.shots} rounds, ${end.kills} down)`);
  },
});
