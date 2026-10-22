// The core loop through real input: Set out on the title starts a run at
// the south end of the vale with three hearts, no beacon lit and the clock
// at half past three; W walks the hero north on the character controller,
// Shift runs, and on into the lake it wades (slower, splashing). Then by
// the first beacon the prompt shows, holding F kindles it: the right hand
// reaches into the bowl (og.IkChain, solved within its tolerance) and after
// Tuning.kindleSeconds the beacon catches, the HUD counts it and says so.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, position, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "Set out, walk, run and wade on the terrain, kindle a beacon with the hand in the bowl",
  commands: [
    "0.3:get:Game:title.json",
    tap("play", 0.5),
    "0.7:get:Game:started.json",
    "0.7:get:hero.og.Transform:start.json",
    "0.8:key_down:KeyW",
    "1.8:get:Game:walk.json",
    "1.9:key_down:ShiftLeft",
    "2.9:get:Game:run.json",
    "7.6:get:Game:wade.json",
    "7.6:get:hero.og.Transform:wade-at.json",
    "7.7:key_up:ShiftLeft",
    "7.7:key_up:KeyW",
    "7.8:warp:0",
    "8.1:get:Game:near.json",
    "8.1:dump_ui:prompt.json",
    "8.2:key_down:KeyF",
    "9.3:get:hero-reach.og.IkChain:reach.json",
    "9.3:get:Game:kindling.json",
    "10.4:key_up:KeyF",
    "10.5:get:Game:lit.json",
    "10.5:get:beacon-0.Beacon:beacon.json",
    "10.5:dump_ui:lit-ui.json",
    "10.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "Set out started a run");
    t.ok(started.hearts === 3 && started.lit === 0 && started.runs === 1, "with three hearts, no beacon lit, the first run");
    t.ok(started.hour >= 15.5 && started.hour < 15.6, `at half past three (${started.hour.toFixed(2)})`);
    const walk = game(run, "walk.json");
    t.ok(walk.grounded && Math.abs(walk.speed - 3) < 0.1, `W walks on the ground at the walk speed (${walk.speed.toFixed(2)} m/s)`);
    t.ok(Math.abs(game(run, "run.json").speed - 6) < 0.1, "Shift runs at the run speed");
    const wade = game(run, "wade.json");
    t.ok(wade.wading, "on into the lake the hero wades");
    t.ok(Math.abs(wade.speed - 6 * 0.45) < 0.15, `slowed to Tuning.wade of a run (${wade.speed.toFixed(2)} m/s)`);
    t.ok(wade.splashes >= 1, `and splashes (${wade.splashes})`);
    const [, , z0] = position(run, "start.json");
    const [x1, , z1] = position(run, "wade-at.json");
    t.ok(z0 - z1 > 20 && Math.abs(x1) < 0.5, `the hero went north across the terrain (z ${z0.toFixed(1)} → ${z1.toFixed(1)})`);
    t.equal(game(run, "near.json").near, 0, "by the first beacon it is within reach");
    t.ok(texts(run, "prompt.json").includes("Hold F to kindle the beacon"), "and the prompt says how to kindle it");
    const k = game(run, "kindling.json");
    t.ok(k.kindling && k.kindle > 0.5, `holding F kindles it (${k.kindle.toFixed(2)} s)`);
    const reach = value<{ reached: boolean; error: number; weight: number }>(run, "reach.json");
    t.ok(reach.weight > 0.9, `the arm's IK chain is on (weight ${reach.weight.toFixed(2)})`);
    t.ok(reach.reached && reach.error <= 0.02, `the hand reached into the bowl (${(reach.error * 100).toFixed(1)} cm off)`);
    const lit = game(run, "lit.json");
    t.ok(lit.lit === 1 && lit.litMask === 1, "the beacon caught");
    const b = value<{ lit: boolean; litAt: number }>(run, "beacon.json");
    t.ok(b.lit && b.litAt - k.time > 0 && b.litAt < k.time + 1.6, `after Tuning.kindleSeconds of kindling (at ${b.litAt.toFixed(2)} s)`);
    const words = texts(run, "lit-ui.json");
    t.ok(words.includes("1/4") && words.includes("Beacon 1 of 4 burns"), "the HUD counts it and says so");
  },
});
