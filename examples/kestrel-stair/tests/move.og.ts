// The courier on og.CharacterController over the level's triangle meshes
// (og-game §13): walking and running at the tuned speeds on the court, up
// the broad ramp (18 degrees), up the steep path (34 degrees, under
// maxSlope 45), sliding back off the scree (63 degrees), up the stairs
// (0.25 m steps under stepHeight 0.32) to the high terrace, and a jump onto
// the first ledge, 1 m up, with its take-off and landing heard.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, press, script } from "./lib.ts";

export default playtest({
  name: "the courier walks and runs, climbs the ramp, the steep path and the stairs, slides off the scree and jumps onto a ledge",
  commands: script(
    "0.3:play:0:quick",
    "0.35:set:Tuning.sight=0",
    // Flat ground: a second of walking, then of running.
    "0.4:warp:-6:40",
    hold("KeyW", 0.5, 1.6),
    "1.0:get:Game:walk0.json",
    "1.5:get:Game:walk1.json",
    hold("ShiftLeft", 2.2, 1.6),
    hold("KeyW", 2.2, 1.6),
    "2.7:get:Game:run0.json",
    "3.2:get:Game:run1.json",
    // The ramp, from the court.
    "3.9:warp:0:26",
    hold("KeyW", 4.0, 2.4),
    "6.4:get:Game:ramp.json",
    // The steep path, and the scree beside the crypt terrace's wall.
    "6.6:warp:16:20",
    hold("KeyW", 6.7, 3.4),
    "10.1:get:Game:steep.json",
    "10.3:warp:-16:16",
    hold("KeyW", 10.4, 2.0),
    "12.4:get:Game:scree.json",
    // The stairs, to the high terrace.
    "12.6:warp:0:-14",
    hold("KeyW", 12.7, 4.6),
    "17.3:get:Game:stairs.json",
    // A jump onto the first ledge.
    "17.5:warp:16.5:-1.6",
    hold("KeyW", 17.6, 0.9),
    press("Space", 17.8),
    "19.4:get:Game:ledge.json",
    "19.4:get:hero.og.CharacterController:cc.json",
    "19.5:get:sfx-32.Sfx:jump.json",
    "19.5:get:sfx-64.Sfx:land.json",
    "19.6:screenshot:ledge.png",
    "19.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const w0 = game(run, "walk0.json");
    const w1 = game(run, "walk1.json");
    t.near(Math.abs(w1.heroZ - w0.heroZ) / 0.5, 3.2, 0.25, `walks at Tuning.walkSpeed (${(Math.abs(w1.heroZ - w0.heroZ) / 0.5).toFixed(2)} m/s)`);
    const r0 = game(run, "run0.json");
    const r1 = game(run, "run1.json");
    t.near(Math.abs(r1.heroZ - r0.heroZ) / 0.5, 6.2, 0.4, `runs at Tuning.runSpeed (${(Math.abs(r1.heroZ - r0.heroZ) / 0.5).toFixed(2)} m/s)`);
    const ramp = game(run, "ramp.json");
    // On the ramp the feet follow its slope: 4 m over its 12.
    t.ok(ramp.grounded && ramp.heroZ < 20 && Math.abs(ramp.heroY - ((24 - ramp.heroZ) / 12) * 4) < 0.1, `up the ramp (feet at ${ramp.heroY.toFixed(2)} m at z ${ramp.heroZ.toFixed(2)}, on its surface)`);
    const steep = game(run, "steep.json");
    t.ok(steep.heroY > 3.5, `up the steep path to the terrace (${steep.heroY.toFixed(2)} m)`);
    const scree = game(run, "scree.json");
    t.ok(scree.heroY < 1.0 && scree.heroZ > 13.5, `the scree is too steep: the courier stays at its foot (${scree.heroY.toFixed(2)} m up, z ${scree.heroZ.toFixed(2)})`);
    const stairs = game(run, "stairs.json");
    t.ok(stairs.heroY > 7.6 && stairs.heroZ < -23.5, `up the stairs to the high terrace (${stairs.heroY.toFixed(2)} m, z ${stairs.heroZ.toFixed(2)})`);
    const ledge = game(run, "ledge.json");
    t.ok(Math.abs(ledge.heroY - 5) < 0.15 && ledge.grounded, `onto the first ledge (feet at ${ledge.heroY.toFixed(2)} m)`);
    t.ok(ledge.jumps >= 1 && ledge.landings >= 1, `a jump and a landing (${ledge.jumps}, ${ledge.landings})`);
    // FINDINGS #16: og.CharacterController.sliding reads true standing still on level ground.
    const cc = run.json<{ value: { sliding: boolean; groundNormal: number[] } }>("cc.json").value;
    t.ok(cc.groundNormal[1]! > 0.99, `standing on the ledge's level top (normal y ${cc.groundNormal[1]!.toFixed(3)}, sliding ${cc.sliding})`);
    t.ok(run.json<{ value: { plays: number } }>("jump.json").value.plays >= 1 && run.json<{ value: { plays: number } }>("land.json").value.plays >= 1, "both heard");
  },
});
