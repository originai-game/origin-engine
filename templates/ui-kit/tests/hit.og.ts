// A hit the HUDs show is felt (og-feel §3, §4, §7; og-audio §1): the
// touch HUD's Hit takes 15 health and, on the hit's tick, holds a short hit
// stop, adds camera trauma, floats the number up from the health bar and
// plays a thud on the sfx bus. Ways it could fail: the health drops with
// nothing seen or heard (og polish's hit feedback 0), or the thud plays on
// the ui bus (a click, not a hit).
import { playtest } from "@origingame/engine/playtest";
import { demo, tap } from "./lib.ts";

interface Pulse {
  hitStop: number;
  scale: number;
  trauma: number;
  spawned: number;
  voices: Array<[number, number, number]>;
}

export default playtest({
  name: "a hit stops time briefly, shakes, floats its number and thuds on the sfx bus",
  commands: [
    "0.3:show:widgets",
    "0.8:dump_pulse:before.json",
    "0.8:dump_entity:demo:before-demo.json",
    tap(0.9, "Hit"),
    "1.02:dump_pulse:hit.json",
    "1.1:dump_entity:demo:after.json",
    "1.2:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "after.json").hp, demo(run, "before-demo.json").hp - 15, "Hit took 15 health");
    const before = run.json<Pulse>("before.json");
    const hit = run.json<Pulse>("hit.json");
    t.ok(hit.hitStop > 0 || hit.scale < 1, `time holds after the hit (hit stop ${hit.hitStop}, scale ${hit.scale})`);
    t.ok(hit.trauma > before.trauma, `the camera takes trauma (${before.trauma} to ${hit.trauma})`);
    t.ok(hit.spawned > before.spawned, `a floating text spawned (${before.spawned} to ${hit.spawned})`);
    t.ok(hit.voices.some((v) => v[1] === 2), `the thud plays on the sfx bus (voices ${JSON.stringify(hit.voices)})`);
  },
});
