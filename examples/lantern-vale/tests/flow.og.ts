// The core loop through real input: Set out on the title starts a run at
// the south end of the vale with three hearts, no lantern lit and the clock
// at half past three; W walks the lamplighter north on the character
// controller, Shift runs. By the first lantern the overlap query finds it
// and the prompt shows; holding E lights it after Tuning.kindleSeconds: the
// HUD counts it, a toast says so, the light swells in.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, position, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "Set out, walk and run on the terrain, find a lantern by an overlap query, light it with E",
  commands: [
    "0.3:get:Game:title.json",
    tap("play", 0.5),
    "0.7:get:Game:started.json",
    "0.7:get:hero.og.Transform:start.json",
    "0.8:key_down:KeyW",
    "1.8:get:Game:walk.json",
    "1.9:key_down:ShiftLeft",
    "2.9:get:Game:run.json",
    "2.9:get:hero.og.Transform:ran.json",
    "3.0:key_up:ShiftLeft",
    "3.0:key_up:KeyW",
    "3.1:warp:0",
    "3.4:get:Game:near.json",
    "3.4:dump_ui:prompt.json",
    "3.5:key_down:KeyE",
    "4.2:get:Game:kindling.json",
    "4.9:key_up:KeyE",
    "5.1:get:Game:lit.json",
    "5.1:get:lantern-0.Lantern:lantern.json",
    "5.1:dump_ui:lit-ui.json",
    "5.6:get:lantern-0-light.og.PointLight:light.json",
    "5.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "Set out started a run");
    t.ok(started.hearts === 3 && started.lit === 0, "with three hearts and no lantern lit");
    t.ok(started.hour >= 15.5 && started.hour < 15.52, `at half past three (${started.hour.toFixed(2)})`);
    const walk = game(run, "walk.json");
    t.ok(walk.grounded && Math.abs(walk.speed - 3.4) < 0.15, `W walks on the ground at the walk speed (${walk.speed.toFixed(2)} m/s)`);
    t.ok(walk.steps >= 2, `and the steps are counted (${walk.steps})`);
    t.ok(Math.abs(game(run, "run.json").speed - 6) < 0.2, "Shift runs at the run speed");
    const [, , z0] = position(run, "start.json");
    const [, , z1] = position(run, "ran.json");
    t.ok(z0 - z1 > 8, `the hero went north across the terrain (z ${z0.toFixed(1)} → ${z1.toFixed(1)})`);
    t.equal(game(run, "near.json").near, 0, "by the first lantern the overlap query finds it");
    t.ok(texts(run, "prompt.json").includes("Hold E to light the lantern"), "and the prompt says how to light it");
    const k = game(run, "kindling.json");
    t.ok(k.kindling && k.kindle > 0.5, `holding E kindles it (${k.kindle.toFixed(2)} s)`);
    const lit = game(run, "lit.json");
    t.ok(lit.lit === 1 && lit.litMask === 1, "the lantern caught");
    t.ok(value<{ lit: boolean }>(run, "lantern.json").lit, "its Lantern row says lit");
    const words = texts(run, "lit-ui.json");
    t.ok(words.includes("1/4") && words.includes("Lantern 1 of 4 glows"), "the HUD counts it and a toast says so");
    t.ok(value<{ intensity: number }>(run, "light.json").intensity > 4, "its light swelled in");
  },
});
