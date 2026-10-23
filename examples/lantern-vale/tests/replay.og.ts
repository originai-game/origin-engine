// Determinism: a session with walking, running, the pile shoved, a lantern
// lit, a slime's chase and touch (hit stop, knock-back), the far side
// streamed in and a look switched at the results, run twice in fresh
// processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { game, position } from "./lib.ts";

export default playtest({
  name: "a session with the pile, a lantern, a slime's touch, the far side and the results replays to the same hash",
  runs: 2,
  commands: [
    "0.3:run",
    "0.4:warp:4.5:13.5",
    "0.5:get:rock-4.og.Transform:rock-before.json",
    "0.5:key_down:KeyW",
    "1.5:key_down:ShiftLeft",
    "2.5:key_up:ShiftLeft",
    "2.5:key_up:KeyW",
    "2.6:get:rock-4.og.Transform:rock-after.json",
    "2.6:get:hero.og.Transform:hero-pushed.json",
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
    t.ok(end.lit === 4 && end.hurts >= 1 && end.farLoads === 1 && end.look === 1, `the session lit four, was touched, streamed the far side and reached the dawn look (${end.lit}, ${end.hurts}, ${end.farLoads}, ${end.look})`);
    // The shove, as physics.og.ts reads it: the middle rock off its spot
    // along the ground and the hero on into the pile. With nobody walking
    // in, rock-4 stays put and the hero at the warp's z 13.5.
    const [x0, y0, z0] = position(run, "rock-before.json");
    const [x1, y1, z1] = position(run, "rock-after.json");
    const shoved = Math.hypot(x1 - x0, z1 - z0);
    const [, , hz] = position(run, "hero-pushed.json");
    t.ok(shoved > 0.5 && y1 - y0 < 0.3 && hz < 10.2, `the session shoved the pile: the middle rock slid ${shoved.toFixed(2)} m (rose ${(y1 - y0).toFixed(2)} m), the hero walked on to z ${hz.toFixed(2)}`);
  },
});
