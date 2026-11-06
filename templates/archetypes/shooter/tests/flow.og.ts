// The core loop through real input: Hold the line on the title starts a
// holdout with full health and a full magazine; after the first breather
// wave 1 comes through the gates on the navmesh. A walker straight ahead
// takes rounds (F held): hits, its fall, the kill and the points on the
// trooper's own tally and the run's score. R reloads from the reserve, and
// an empty magazine fired clicks and reloads by itself.
import { playtest } from "@origingame/engine/playtest";
import { AIM_RIGHT, FIGHTING, game, hold, PLAY, press, tap, TITLE, value, type Hero } from "./lib.ts";

export default playtest({
  name: "Hold the line, wave 1 through the gates, a walker shot down, a reload and an empty magazine",
  commands: [
    "0.3:get:Game:title.json",
    tap("Play", 0.5),
    "0.7:get:Game:started.json",
    "0.7:get:hero.Hero:fresh.json",
    "3.6:get:Game:wave1.json",
    "3.6:get:walker-0.og.Transform:w-a.json",
    "4.4:get:walker-0.og.Transform:w-b.json",
    "4.5:clear",
    "4.6:warp:-3:2",
    "4.7:aim:0:-5",
    `4.8:spawn:walker:${-3 + AIM_RIGHT}:-8`,
    "4.9:get:Game:before.json",
    hold("KeyF", 5.0, 2.2),
    "7.3:get:Game:shot.json",
    "7.3:get:hero.Hero:shooter.json",
    press("KeyR", 7.4),
    "7.5:get:hero.Hero:reloading.json",
    "9.0:get:hero.Hero:reloaded.json",
    "9.1:ammo:0",
    hold("KeyF", 9.2, 0.1),
    "9.4:get:hero.Hero:empty.json",
    "11.0:get:hero.Hero:auto.json",
    "11.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "Hold the line started a run");
    t.ok(started.runs === 1 && started.score === 0 && started.kills === 0, "the first run, nothing scored");
    const fresh = value<Hero>(run, "fresh.json");
    t.ok(fresh.health === 100 && fresh.mag === 24 && fresh.reserve === 120 && !fresh.down, `with full health and rounds (${fresh.health}, ${fresh.mag}/${fresh.reserve})`);
    const w1 = game(run, "wave1.json");
    t.ok(w1.wave === 1 && w1.waveState === FIGHTING, `after the breather wave 1 is on (wave ${w1.wave}, state ${w1.waveState})`);
    t.ok(w1.alive >= 1, `and its foes have come in (${w1.alive} standing)`);
    const a = value<{ position: number[] }>(run, "w-a.json").position;
    const b = value<{ position: number[] }>(run, "w-b.json").position;
    t.ok(Math.hypot(b[0]! - a[0]!, b[2]! - a[2]!) > 0.8, `the first walker paths in on the navmesh (${Math.hypot(b[0]! - a[0]!, b[2]! - a[2]!).toFixed(2)} m)`);
    const before = game(run, "before.json");
    const shot = game(run, "shot.json");
    const fired = shot.shots - before.shots;
    t.ok(fired >= 12, `F held fires the rifle (${fired} rounds)`);
    t.ok(shot.hits - before.hits >= 5 && shot.kills > before.kills, `the walker ahead takes hits and falls (${shot.hits - before.hits} hits, ${shot.kills - before.kills} down)`);
    const me = value<Hero>(run, "shooter.json");
    t.ok(me.kills >= 1 && me.score >= 100 && shot.score - before.score >= me.score, `the kill and its points are the trooper's (${me.kills}, ${me.score}) and the run's (${shot.score - before.score})`);
    t.ok(me.mag === 24 - fired, `each round came from the magazine (${me.mag} left)`);
    const reloading = value<Hero>(run, "reloading.json");
    t.ok(reloading.reload > 0, "R starts a reload");
    const reloaded = value<Hero>(run, "reloaded.json");
    t.ok(reloaded.mag === 24 && reloaded.reserve === 120 - fired && reloaded.reload === 0, `which tops the magazine up from the reserve (${reloaded.mag}/${reloaded.reserve})`);
    const empty = value<Hero>(run, "empty.json");
    t.ok(empty.mag === 0 && empty.reload > 0, "an empty magazine fired reloads by itself");
    t.equal(value<Hero>(run, "auto.json").mag, 24, "and the magazine is full again");
  },
});
