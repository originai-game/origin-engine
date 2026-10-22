// Combat: the slime's bite costs a heart and knocks the hero back; the
// sword's strike frame (the attack clip's "hit" event) lands 12, 12, then a
// 26 crit, and each hit holds the world for Tuning.hitStop ticks with the
// slime flashed white and squashed; the third kills it, and it fades and
// hides.
//
// A press at t = 0.65 s (tick 39) reaches the strike frame six ticks later
// (tick 45): frame 0 of the attack lasts 0.07 s, then Control reads the
// event on the next tick.
import { playtest } from "@origingame/engine/playtest";
import { component, position, press, report, scale } from "./lib.ts";

/** Hero at x, slime 1.1 m to its right, facing it. */
const setup = (t: number) => `${t}:place:12.3:6.5;${t}:slime:13.4:6.5;${t}:face:right`;

export default playtest({
  name: "a bite hurts the hero; three sword hits kill the slime with hit stop, flash and a crit",
  commands: [
    // The bite: the slime starts on the hero.
    "0.1:place:12.3:6.5",
    "0.1:slime:12.8:6.5",
    // Read after the hit stop: the bite lands about tick 11, the stop holds the
    // hero four ticks, and the knock-back (9 m/s, decaying) has moved it past
    // 0.3 m by 0.3 s (12.3 to 11.80; 11.70 by 0.6 s).
    "0.3:dump_entity:hero:bitten.json",
    // First hit, then two ticks into the stop and the tick after it.
    setup(0.6),
    press("Space", 0.65),
    "0.768:dump_entity:slime:stop-a.json",
    "0.801:dump_entity:slime:stop-b.json",
    "0.851:dump_entity:slime:after.json",
    "0.9:dump_entity:hero:hit1.json",
    setup(1.3),
    press("Space", 1.35),
    "1.6:dump_entity:hero:hit2.json",
    setup(2.0),
    press("Space", 2.05),
    "2.3:dump_entity:hero:hit3.json",
    "3.0:dump_entity:slime:dead.json",
    "3.0:dump_entity:hero:end.json",
    "3.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const bitten = report(run, "bitten.json");
    t.equal(bitten.hp, 2, "the bite cost one heart");
    t.equal(bitten.hurts, 1, "one hurt");
    t.ok(position(run, "bitten.json").x < 12.0, `the bite knocked the hero back (x ${position(run, "bitten.json").x.toFixed(3)})`);

    // The hit stop: the slime holds its place, white and squashed, then moves off.
    const a = position(run, "stop-a.json");
    const b = position(run, "stop-b.json");
    t.equal(b.x, a.x, "the slime holds still during the hit stop");
    t.equal(component<{ flash: number }>(run, "stop-a.json", "og.Sprite").flash, 1, "the struck slime is white during the stop");
    t.near(scale(run, "stop-a.json").x, 1.45, 0.001, "the struck slime is squashed during the stop");
    t.equal(component<{ hp: number }>(run, "stop-a.json", "Slime").hp, 38, "the first hit dealt 12");
    t.ok(position(run, "after.json").x > b.x + 0.1, "after the stop the knockback carries the slime away");
    t.ok(scale(run, "after.json").x < 1.45, "after the stop the squash recovers");

    t.equal(report(run, "hit1.json").enemyHp, 38, "Game.enemyHp after one hit");
    t.equal(report(run, "hit2.json").enemyHp, 26, "a second 12");
    const end = report(run, "end.json");
    t.equal(end.enemyHp, 0, "the crit (26) finished it");
    t.equal(end.hits, 3, "three hits");
    t.equal(end.swings, 3, "three strike frames: one per press");
    t.equal(end.hp, 2, "the hero still has two hearts");
    const slime = component<{ dead: boolean }>(run, "dead.json", "Slime");
    t.equal(slime.dead, true, "the slime is dead");
    t.equal(component<{ visible: boolean }>(run, "dead.json", "og.Sprite").visible, false, "the dead slime is hidden");
    t.near(scale(run, "dead.json").y, 0, 0.001, "the death tween squashed it to nothing");
  },
});
