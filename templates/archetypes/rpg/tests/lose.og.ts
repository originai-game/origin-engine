// The loss: a hero struck to nothing falls. Alone that ends the run; with
// two heroes the fallen rises at the gate after Tuning.respawn while the
// other stands, and the run ends only when both are down. The summary says
// the party has fallen, and Title goes back to the title with the yard
// reset behind it.
import { playtest } from "@origingame/engine/playtest";
import { FALLEN, game, OVER, PLAY, script, SEATS, SUMMARY_READY, tap, texts, TITLE, value, type Hero } from "./lib.ts";

const two = SEATS > 1;
const END = two ? 4.0 : 1.0;

export default playtest({
  name: two ? "a fallen hero rises while the other stands; both down ends the run" : "the knight struck to nothing ends the run; Title goes back",
  commands: script(
    "0.3:run",
    "0.4:set:Tuning.respawn=1.5",
    "0.8:hp:0:0",
    "0.9:get:hero.Hero:fallen.json",
    "0.9:get:Game:one-down.json",
    ...(two ? ["2.8:get:hero.Hero:risen.json", "3.0:hp:0:0", "3.1:hp:1:0", "3.2:get:Game:both.json"] : []),
    `${END + 0.5}:get:Game:ending.json`,
    `${END + 2.2}:get:Game:over.json`,
    `${END + 2.2}:dump_ui:summary.json`,
    tap("Menu", END + 1.5 + SUMMARY_READY + 0.2),
    `${END + 1.5 + SUMMARY_READY + 0.6}:get:Game:title.json`,
    `${END + 1.5 + SUMMARY_READY + 0.6}:get:hero.Hero:reset.json`,
    `${END + 1.5 + SUMMARY_READY + 0.7}:quit`,
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const fallen = value<Hero>(run, "fallen.json");
    t.ok(fallen.dead && fallen.hp === 0, "struck to nothing, the knight falls");
    if (two) {
      t.equal(game(run, "one-down.json").ending, 0, "the barbarian stands: the run goes on");
      const risen = value<Hero>(run, "risen.json");
      t.ok(!risen.dead && risen.hp > 0, `after Tuning.respawn the knight rises at the gate (${risen.hp})`);
      t.equal(game(run, "both.json").ending, FALLEN, "both down ends the run");
    } else {
      t.equal(game(run, "one-down.json").ending, FALLEN, "alone, that ends the run");
    }
    t.equal(game(run, "ending.json").phase, PLAY, "the summary waits a beat");
    t.equal(game(run, "over.json").phase, OVER, "then the run is over");
    t.ok(texts(run, "summary.json").includes("The party has fallen"), "the summary says the party has fallen");
    t.equal(game(run, "title.json").phase, TITLE, "Title goes back to the title");
    const reset = value<Hero>(run, "reset.json");
    t.ok(!reset.dead && reset.hp === reset.maxHp && reset.level === 1, "the knight whole again behind it");
  },
});
