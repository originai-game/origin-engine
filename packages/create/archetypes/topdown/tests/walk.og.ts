// Walking: the keys move the hero at Tuning.speed, the walk clip's "step"
// events count footsteps, the facing follows the keys (left is the side row
// flipped), and the wall terrain's colliders stop the body.
//
// The slime waits in the east room, out of sight, so nothing interrupts.
import { playtest } from "@origingame/engine/playtest";
import { LEFT, MAP, RIGHT, UP, component, hold, position, report } from "./lib.ts";

export default playtest({
  name: "the hero walks at its speed, steps on the walk frames and stops at walls",
  commands: [
    "0.1:slime:35:3",
    // Right for one second from the spawn: 4.5 m at 4.5 m/s.
    hold("KeyD", 0.2, 1.2),
    "1.3:dump_entity:hero:right.json",
    // Up into the top wall (row 0 covers y 11..12; the body's box reaches 0.35 above the feet).
    hold("KeyW", 1.4, 3.0),
    "3.1:dump_entity:hero:up.json",
    // Left along the wall to the west wall (column 0 covers x 0..1; the box is 0.6 wide).
    "3.2:key_down:ArrowLeft",
    "3.6:dump_entity:hero:walking-left.json",
    "3.6:screenshot:walk.png",
    "5.4:key_up:ArrowLeft",
    "5.5:dump_entity:hero:left.json",
    "5.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const right = position(run, "right.json");
    t.near(right.x, MAP.spawn.x + 4.5, 0.1, `one second of D walks 4.5 m (x ${right.x.toFixed(3)})`);
    t.near(right.y, MAP.spawn.y, 0.001, "walking right keeps y");
    const r = report(run, "right.json");
    t.equal(r.facing, RIGHT, "the hero faces right");
    t.ok(r.steps >= 3 && r.steps <= 5, `the walk clip's step events fired twice a cycle (${r.steps} steps in 1 s at 8 fps)`);
    const up = position(run, "up.json");
    t.near(up.y, 11 - 0.35, 0.03, `the top wall stops the body (feet at y ${up.y.toFixed(3)}, box top at 11)`);
    t.near(up.x, right.x, 0.01, "walking up keeps x");
    t.equal(report(run, "up.json").facing, UP, "the hero faces up");
    t.equal(component<{ clip: number }>(run, "walking-left.json", "og.Flipbook").clip, 5, "walking left plays walk_side (clip 5)");
    t.equal(component<{ flipX: boolean }>(run, "walking-left.json", "og.Sprite").flipX, true, "facing left flips the side row");
    const left = position(run, "left.json");
    t.near(left.x, 1 + 0.3, 0.03, `the west wall stops the body (x ${left.x.toFixed(3)})`);
    t.equal(report(run, "left.json").facing, LEFT, "the hero faces left");
    t.equal(component<{ clip: number }>(run, "left.json", "og.Flipbook").clip, 2, "standing still shows idle_side (clip 2)");
  },
});
