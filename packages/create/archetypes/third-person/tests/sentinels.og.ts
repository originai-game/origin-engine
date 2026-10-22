// The sentinels on the navmesh: the west one walks its round (og.NavAgent
// moving it at the patrol speed); the hero standing ahead on its round raises
// its alarm (og.ai's behaviour enters `alert` and commands `alarm`), its
// eye burns red and it chases; within reach its grab costs a heart, throws
// the hero back and spares it a moment (no second grab while spared).
import { playtest } from "@origingame/engine/playtest";
import { game, position, value } from "./lib.ts";

interface Sentinel { alarms: number; grabs: number; alert: boolean; speed: number }

export default playtest({
  name: "a sentinel on its round sees the hero, raises the alarm, chases and grabs a heart",
  commands: [
    "0.3:run",
    "2.0:get:sentinel-0.og.Transform:round-a.json",
    "2.5:get:sentinel-0.og.Transform:round-b.json",
    "2.5:get:sentinel-0.Sentinel:calm.json",
    "2.6:warp:-28.5:3",
    "3.2:get:Game:seen.json",
    "3.2:get:sentinel-0.Sentinel:alert.json",
    "3.2:get:sentinel-0-eye.og.PointLight:eye.json",
    "3.25:screenshot:alarm.png",
    "7.6:get:Game:grabbed.json",
    "7.6:get:sentinel-0.Sentinel:grabber.json",
    "7.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const [ax, , az] = position(run, "round-a.json");
    const [bx, , bz] = position(run, "round-b.json");
    const walked = Math.hypot(bx - ax, bz - az);
    t.ok(walked > 0.4 && walked < 1.2, `the west sentinel walks its round at the patrol pace (${walked.toFixed(2)} m in 0.5 s, after its first wait)`);
    const calm = value<Sentinel>(run, "calm.json");
    t.ok(!calm.alert && calm.alarms === 0, "calm while the hero is far");
    const seen = game(run, "seen.json");
    const alert = value<Sentinel>(run, "alert.json");
    t.ok(alert.alert && alert.alarms === 1 && seen.alarms >= 1, "the hero in its sight: the alarm, and it gives chase");
    t.ok(value<{ intensity: number }>(run, "eye.json").intensity > 5, "its eye burns red");
    const grabbed = game(run, "grabbed.json");
    const grabber = value<Sentinel>(run, "grabber.json");
    t.ok(grabber.grabs >= 1, `it caught the hero (${grabber.grabs} grabs)`);
    t.equal(grabbed.hearts, 3 - grabbed.grabs, `each grab cost a heart (${grabbed.hearts} left)`);
    t.equal(grabbed.hurts, grabbed.grabs, "and counts as a heart lost");
    t.ok(grabbed.grabs <= 2, `spared a moment after each grab: at most two in 4.4 s (${grabbed.grabs})`);
    t.equal(grabbed.phase, 1, "the run goes on with hearts left");
    t.frame("alarm.png", () => t.equal(run.png("alarm.png").width, 1280, "alarm.png captured"));
  },
});
