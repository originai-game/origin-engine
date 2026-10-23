// Physics: the lamplighter's character controller walks into the pile of
// boulders (dynamic bodies on their convex hulls) and shoves them aside (pushBodies,
// at most pushStrength): it parts the front two and shoves the middle one
// ahead along the ground, walking on into the pile; the trees' trunks stop
// it; a slime's knock-back throws it.
import { playtest } from "@origingame/engine/playtest";
import { position } from "./lib.ts";

export default playtest({
  name: "the character controller shoves the pile of rocks and is stopped by a trunk",
  commands: [
    "0.3:run",
    "0.4:warp:4.5:13.5",
    "0.5:get:rock-4.og.Transform:rock-before.json",
    "0.5:get:rock-6.og.Transform:left-before.json",
    "0.5:get:rock-7.og.Transform:right-before.json",
    "0.5:key_down:KeyW",
    "2.5:key_up:KeyW",
    "2.6:get:rock-4.og.Transform:rock-after.json",
    "2.6:get:rock-6.og.Transform:left-after.json",
    "2.6:get:rock-7.og.Transform:right-after.json",
    "2.6:get:hero.og.Transform:hero-pushed.json",
    "2.7:warp:-6:7",
    "2.8:key_down:KeyW",
    "4.3:key_up:KeyW",
    "4.4:get:hero.og.Transform:trunk.json",
    "4.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    // The checks read the rocks the hero walks into, from the moment it sets
    // off: the front two (rock-6 and rock-7, the hero aims at the gap between
    // them) and the middle one behind. With nobody walking in, each of them
    // stays within a millimetre (the pile stands by itself).
    const gap = (a: string, b: string) => {
      const [ax, , az] = position(run, a);
      const [bx, , bz] = position(run, b);
      return Math.hypot(ax - bx, az - bz);
    };
    const parted = gap("left-after.json", "right-after.json") - gap("left-before.json", "right-before.json");
    t.ok(parted > 0.15, `the hero parted the front two rocks (their gap widened ${parted.toFixed(2)} m)`);
    const [x0, y0, z0] = position(run, "rock-before.json");
    const [x1, y1, z1] = position(run, "rock-after.json");
    const shoved = Math.hypot(x1 - x0, z1 - z0);
    t.ok(shoved > 0.5, `the middle rock was shoved off its spot (${shoved.toFixed(2)} m)`);
    t.ok(y1 - y0 < 0.3, `it slid along the ground, not thrown onto the pile (rose ${(y1 - y0).toFixed(2)} m)`);
    const [, , hz] = position(run, "hero-pushed.json");
    t.ok(hz < 10.2, `the hero walked on into the pile, not wedged at its front (z ${hz.toFixed(2)})`);
    const [, , z] = position(run, "trunk.json");
    t.ok(z > 4.3 && z < 5.5, `the tree at (-6, 4) stopped the walk north (z ${z.toFixed(2)})`);
  },
});
