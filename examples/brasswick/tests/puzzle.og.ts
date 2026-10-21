// The puzzle pieces: a plate is down while Wick or a crate stands on it, and
// its door slides up into the ceiling (a kinematic body moved by its
// og.Velocity) and back down when it lets go; a crate shoved onto the plate
// holds the door; the second level's crate shoved off its slab falls onto
// its plate and the other drops into the pit onto its own; the third level's
// gate slides its whole travel on its prismatic joint; the lift carries
// Wick; and R puts the crates back where the last lamp saw them. The
// second level's tick is stomped first: it walks the pit crate's floor.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, position, press, script, value } from "./lib.ts";

export default playtest({
  name: "plates hold doors, crates hold plates, a crate drops onto its plate, the gate's joint slides, the lift carries Wick, R puts the crates back",
  commands: script(
    "0.5:play:1",
    // Wick on the plate, then off it.
    "1.0:warp:l1-plate-a:0:0",
    "2.0:get:Game:on-plate.json",
    "2.0:get:l1-door-a.og.Transform:door-up.json",
    "2.1:warp:l1-plate-a:2:0",
    "3.0:get:Game:off-plate.json",
    "3.0:get:l1-door-a.og.Transform:door-down.json",
    // The lamp, then the crate onto the plate: it holds the door; R brings it back to where the lamp saw it.
    "3.1:warp:l1-lamp-0:-1.2:0",
    hold("ArrowRight", 3.3, 1.85),
    "5.8:get:Game:crate-on.json",
    "5.8:get:l1-door-a.og.Transform:door-held.json",
    "5.8:get:l1-crate-0.og.Transform:crate-on-plate.json",
    press("KeyR", 5.9),
    "6.6:get:Game:retried.json",
    "6.6:get:l1-crate-0.og.Transform:crate-back.json",
    // The second level: the slab's crate onto plate a, the pit's onto plate b.
    "6.8:play:2",
    "7.4:warp:15.6:-14",
    hold("ArrowRight", 7.8, 1.6),
    "11.2:get:Game:slab.json",
    // The tick walking this floor goes first (a stomp), or it would knock Wick back to the spawn.
    "11.25:warp:l2-tick-1:0:2.5",
    "11.6:warp:34.5:-20",
    hold("ArrowRight", 11.75, 2.55),
    "15.0:get:Game:pit.json",
    "15.0:get:l2-door-b.og.Transform:door-b.json",
    // The third level: the gate's plate, then the lift.
    "15.2:play:3",
    "15.8:warp:22.5:-20",
    hold("ArrowRight", 15.9, 2.6),
    "19.2:get:l3-gate-j.og.Joint:joint.json",
    "19.2:get:Game:gate.json",
    "19.3:warp:l3-lift-0:0:0.1",
    "19.6:get:Game:ride0.json",
    "20.6:get:Game:ride1.json",
    "20.6:get:l3-lift-0.og.Transform:lift.json",
    "20.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const on = game(run, "on-plate.json");
    t.equal(on.switches, 1, "Wick on the first plate holds its switch");
    t.near(position(run, "door-up.json")[1], -14 + 4, 0.05, "and the door has slid its height up into the ceiling");
    t.equal(game(run, "off-plate.json").switches, 0, "off the plate the switch lets go");
    t.near(position(run, "door-down.json")[1], -14, 0.05, "and the door is back down");
    const held = game(run, "crate-on.json");
    t.ok(held.switches === 1 && held.checkpoint === 0, `the crate shoved onto the plate holds it, the lamp lit on the way (switches ${held.switches}, checkpoint ${held.checkpoint})`);
    t.near(position(run, "crate-on-plate.json")[0], 31.5, 0.85, "the crate stands on the plate");
    t.ok(position(run, "door-held.json")[1] > -11, "the door stays up while the crate holds it");
    const retried = game(run, "retried.json");
    t.ok(retried.deaths === 1 && Math.abs(retried.heroX - 25.5) < 0.3, `R put Wick back at the lamp (${retried.heroX.toFixed(2)})`);
    t.near(position(run, "crate-back.json")[0], 28.5, 0.05, "and the crate back where the lamp saw it");
    t.equal(retried.switches, 0, "so the plate is up again");
    t.equal(game(run, "slab.json").switches & 2, 2, "the second level's slab crate fell onto plate a");
    t.equal(game(run, "pit.json").switches & 4, 4, "its other crate dropped into the pit onto plate b");
    t.ok(position(run, "door-b.json")[1] > -16.5, "and door b is up");
    const joint = value<{ value: number; motorTarget: number }>(run, "joint.json");
    t.ok(game(run, "gate.json").switches === 8 && joint.motorTarget === 3 && joint.value > 2.9, `the gate's plate set its joint's motor to its travel and it slid (${joint.value.toFixed(3)} m)`);
    const r0 = game(run, "ride0.json");
    const r1 = game(run, "ride1.json");
    const lift = position(run, "lift.json");
    t.ok(Math.abs(r1.heroX - r0.heroX) > 1.5 && Math.abs(r1.heroX - lift[0]) < 1.5 && r1.grounded, `the lift carried Wick ${Math.abs(r1.heroX - r0.heroX).toFixed(2)} m, standing on it`);
  },
});
