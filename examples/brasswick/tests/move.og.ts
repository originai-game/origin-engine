// Wick's controller (og.PlatformerController with Tuning's numbers): the
// run, a full jump's height at its time to apex, coyote time (a jump three
// ticks after running off the ledge still jumps; one twelve ticks after does
// not), the wall slide held under Tuning.wallSlideSpeed, wall jumps up the
// shaft, and a grate dropped through with Down.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, script, value } from "./lib.ts";

interface Controller {
  velocity: [number, number, number];
  onWall: number;
  grounded: boolean;
}

export default playtest({
  name: "the run, a full jump, coyote time, the wall slide, wall jumps and a grate dropped through",
  commands: script(
    "0.5:play:1",
    // The run: half a second of Right from rest on the first floor.
    "1.0:get:Game:run0.json",
    hold("ArrowRight", 1.0, 0.5),
    "1.5:get:Game:run1.json",
    // A full jump from rest: its apex at Tuning.timeToApex.
    "2.2:get:Game:jump0.json",
    hold("Space", 2.2, 0.6),
    "2.583:get:Game:apex.json",
    // Coyote time: off the ledge's left edge (it leaves the ground at about tick 86 of this run's clock below).
    "3.4:warp:25:-16",
    "3.8:key_down:ArrowLeft",
    "4.283:key_down:Space",
    "4.4:key_up:Space",
    "4.4:get:Game:coyote.json",
    "4.6:key_up:ArrowLeft",
    "5.0:warp:25:-16",
    "5.4:key_down:ArrowLeft",
    "5.95:key_down:Space",
    "6.05:key_up:Space",
    "6.05:get:Game:late.json",
    "6.2:key_up:ArrowLeft",
    // The shaft: a jump into its right wall and a slide down it, then off each wall in turn.
    "6.6:warp:41.5:-16",
    "7.0:key_down:ArrowRight",
    hold("Space", 7.1, 0.35),
    "7.75:get:wick.og.PlatformerController:slide.json",
    "7.75:get:Game:slide-game.json",
    "7.9:key_up:ArrowRight",
    "7.9:key_down:ArrowLeft",
    hold("Space", 7.9, 0.2),
    "8.3:key_up:ArrowLeft",
    "8.3:key_down:ArrowRight",
    hold("Space", 8.3, 0.2),
    "8.7:key_up:ArrowRight",
    "8.7:key_down:ArrowLeft",
    hold("Space", 8.7, 0.2),
    "9.1:key_up:ArrowLeft",
    "9.1:key_down:ArrowRight",
    hold("Space", 9.1, 0.2),
    "9.5:get:Game:climbed.json",
    "9.6:key_up:ArrowRight",
    // A grate: Down drops through it.
    "10.0:warp:16.5:-16",
    "10.5:get:Game:on-grate.json",
    hold("ArrowDown", 10.6, 0.3),
    "11.0:get:Game:dropped.json",
    "11.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const r0 = game(run, "run0.json");
    const r1 = game(run, "run1.json");
    const ran = r1.heroX - r0.heroX;
    // 0.5 s at 6.5 m/s less the 70 m/s² start: 3.25 - 6.5² / 140 = 2.95.
    t.near(ran, 2.95, 0.25, `half a second of Right runs ${ran.toFixed(2)} m`);
    const j0 = game(run, "jump0.json");
    const apex = game(run, "apex.json");
    const rise = apex.heroY - j0.heroY;
    t.near(rise, 3.4, 0.35, `a full jump rises ${rise.toFixed(2)} m at its apex (Tuning.jumpHeight 3.4 at 0.38 s)`);
    const coyote = game(run, "coyote.json");
    t.ok(coyote.jumps === apex.jumps + 1 && coyote.heroVY > 0, `three ticks after leaving the ledge Space still jumps (jumps ${coyote.jumps}, vy ${coyote.heroVY.toFixed(2)})`);
    const late = game(run, "late.json");
    t.ok(late.jumps === coyote.jumps && late.heroVY < 0, `twelve ticks after, it does not (jumps ${late.jumps}, vy ${late.heroVY.toFixed(2)})`);
    const slide = value<Controller>(run, "slide.json");
    t.equal(slide.onWall, 1, "Wick clings to the shaft's right wall");
    t.ok(slide.velocity[1] < 0 && slide.velocity[1] >= -2.6 - 0.05, `and slides down it no faster than Tuning.wallSlideSpeed (${slide.velocity[1].toFixed(2)} m/s)`);
    const climbed = game(run, "climbed.json");
    t.ok(climbed.wallJumps >= 4 && climbed.heroY > -9.2, `four wall jumps climb the shaft to the shelf's height (${climbed.wallJumps} wall jumps, y ${climbed.heroY.toFixed(2)})`);
    const on = game(run, "on-grate.json");
    t.ok(on.grounded && Math.abs(on.heroY + 16) < 0.05, `Wick stands on the grate (y ${on.heroY.toFixed(2)})`);
    t.ok(game(run, "dropped.json").heroY < -17, "Down drops him through it");
  },
});
