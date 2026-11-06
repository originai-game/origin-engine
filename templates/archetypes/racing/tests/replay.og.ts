// Determinism: a race with the countdown, steering, a drift, the boost,
// contacts with the barriers and the other cars, and the rivals on the
// racing line, run twice in fresh processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, sorted } from "./lib.ts";

export default playtest({
  name: "a race with drifting, boost and contacts replays to the same hash",
  runs: 2,
  commands: sorted([
    "0.3:race",
    hold("KeyW", 0.5, 6.5),
    hold("KeyD", 3.6, 0.5),
    hold("KeyS", 3.7, 0.5),
    hold("KeyA", 4.6, 0.6),
    hold("Space", 5.3, 0.8),
    hold("KeyD", 6.2, 0.8),
    "7.2:get:Game:end.json",
    "7.3:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    t.ok(game(run, "end.json").bumps > 0, `the race had contacts (${game(run, "end.json").bumps})`);
  },
});
