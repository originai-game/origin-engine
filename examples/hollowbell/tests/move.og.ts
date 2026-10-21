// Moving on og.PlatformerController: the arrows run Wren at Tuning.maxSpeed
// on the tick they are read, the run clip's `step` events are heard; Space
// jumps to Tuning.jumpHeight at Tuning.timeToApex (og-feel §12's check),
// with a stretch on take-off and a squash, dust and a thud on landing; the
// root climb's one-way planks are passed from below and stood on, and Down
// drops back through one; the lift, moving by its velocity, carries Wren over the pit.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, position, press, script, value } from "./lib.ts";

export default playtest({
  name: "running, a jump to its height, one-way planks up and down, the lift's ride",
  commands: script(
    "0.3:play",
    "0.5:get:Game:rest.json",
    "0.6:key_down:ArrowRight",
    "0.617:get:hero.og.PlatformerController:first.json",
    "1.1:key_up:ArrowRight",
    "1.1:get:Game:ran.json",
    "1.3:get:hero.og.Transform:before.json",
    "1.4:key_down:Space",
    "1.45:get:hero.og.Transform:stretch.json",
    "1.82:get:hero.og.Transform:apex.json",
    "1.82:get:hero.og.PlatformerController:apexpc.json",
    "2.3:key_up:Space",
    "2.6:get:Game:landed.json",
    "2.6:get:fx-1.Fx:dust.json",
    // The root climb: up from the floor through three planks, three rows apart.
    "2.7:warp:3:3",
    hold("Space", 3.0, 0.5),
    "3.8:get:Game:p1.json",
    hold("ArrowRight", 3.9, 0.25),
    hold("Space", 4.0, 0.5),
    "4.8:get:Game:p2.json",
    hold("ArrowLeft", 4.9, 0.25),
    hold("Space", 5.0, 0.5),
    "5.8:get:Game:p3.json",
    // Down drops back through the plank.
    hold("ArrowDown", 6.0, 0.3),
    "6.8:get:Game:dropped.json",
    // The lift over the pit carries Wren along.
    "7.0:warp:lift:0:0.2",
    "7.5:get:hero.og.Transform:lift0.json",
    "8.5:get:hero.og.Transform:lift1.json",
    "8.5:get:Game:end.json",
    "8.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const rest = game(run, "rest.json");
    t.ok(rest.grounded && Math.abs(rest.heroX - 16.5) < 0.1, `Wren stands at the spawn (${rest.heroX.toFixed(2)})`);
    const first = value<{ moveX: number; velocity: [number, number, number] }>(run, "first.json");
    t.ok(first.moveX === 1 && first.velocity[0] > 0, `the press is answered on its tick: moveX ${first.moveX}, ${first.velocity[0].toFixed(2)} m/s`);
    const ran = game(run, "ran.json");
    // At Tuning.maxSpeed tick after tick once accelerated (70 m/s²): 0.5 × 6.5 less 6.5² / 140 for
    // the start, 2.95 m (FINDINGS.md #16, fixed: the run stuttered and covered about two thirds of it).
    t.near(ran.heroX - rest.heroX, 0.5 * 6.5 - (6.5 * 6.5) / 140, 0.15, `half a second of Right ran ${(ran.heroX - rest.heroX).toFixed(2)} m`);
    t.ok(ran.steps >= 2, `the run clip's step events were heard (${ran.steps})`);
    const y0 = position(run, "before.json")[1];
    const stretch = value<{ scale: [number, number, number] }>(run, "stretch.json").scale;
    t.ok(stretch[1] > 1.02 && stretch[0] < 0.98, `take-off stretches Wren (${stretch[0].toFixed(2)} x ${stretch[1].toFixed(2)})`);
    const apex = position(run, "apex.json")[1] - y0;
    t.near(apex, 3.6, 0.36, `the jump reaches Tuning.jumpHeight 3.6 at timeToApex (${apex.toFixed(2)} m)`);
    const landed = game(run, "landed.json");
    t.ok(landed.jumps === 1 && landed.landings >= 1 && landed.grounded, `a jump and a landing were reported (${landed.jumps}, ${landed.landings})`);
    t.ok(value<{ bursts: number }>(run, "dust.json").bursts >= 2, "dust puffed on take-off and landing");
    const p1 = game(run, "p1.json");
    const p2 = game(run, "p2.json");
    const p3 = game(run, "p3.json");
    t.ok(p1.grounded && Math.abs(p1.heroY - 6) < 0.1, `the first plank, passed from below (y ${p1.heroY.toFixed(2)})`);
    t.ok(p2.grounded && Math.abs(p2.heroY - 9) < 0.1, `the second (y ${p2.heroY.toFixed(2)})`);
    t.ok(p3.grounded && Math.abs(p3.heroY - 12) < 0.1, `the third (y ${p3.heroY.toFixed(2)})`);
    const dropped = game(run, "dropped.json");
    t.ok(dropped.heroY < p3.heroY - 1, `Down dropped back through the plank (y ${dropped.heroY.toFixed(2)})`);
    const l0 = position(run, "lift0.json");
    const l1 = position(run, "lift1.json");
    // The lift moves by its own og.Velocity, 2 m/s (FINDINGS.md #13, fixed), and carries Wren a second's worth.
    t.ok(Math.abs(Math.abs(l1[0] - l0[0]) - 2) < 0.1 && Math.abs(l1[1] - 5) < 0.1, `the lift carried Wren ${(l1[0] - l0[0]).toFixed(2)} m along at its height`);
    t.equal(game(run, "end.json").hurts, 0, "no harm done");
  },
});
void press;
