// Physics: the lamplighter's character controller walks into the pile of
// boulders (dynamic bodies on spheres) and shoves them aside (pushBodies);
// the trees' trunks stop it; a slime's knock-back throws it.
import { playtest } from "@origingame/engine/playtest";
import { game, position } from "./lib.ts";

export default playtest({
  name: "the character controller shoves the pile of rocks and is stopped by a trunk",
  commands: [
    "0.3:run",
    "0.4:warp:4.5:13.5",
    "0.5:get:rock-4.og.Transform:rock-before.json",
    "0.5:key_down:KeyW",
    "2.5:key_up:KeyW",
    "2.6:get:Game:pushed.json",
    "2.6:get:rock-4.og.Transform:rock-after.json",
    "2.7:warp:-6:7",
    "2.8:key_down:KeyW",
    "4.3:key_up:KeyW",
    "4.4:get:hero.og.Transform:trunk.json",
    "4.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = game(run, "pushed.json");
    t.ok(g.rockMoved > 0.5, `the pile was shoved (a rock moved ${g.rockMoved.toFixed(2)} m)`);
    const [x0, , z0] = position(run, "rock-before.json");
    const [x1, , z1] = position(run, "rock-after.json");
    t.ok(Math.hypot(x1 - x0, z1 - z0) > 0.2, "the middle rock was pushed off its spot");
    const [, , z] = position(run, "trunk.json");
    t.ok(z > 4.3 && z < 5.5, `the tree at (-6, 4) stopped the walk north (z ${z.toFixed(2)})`);
  },
});
