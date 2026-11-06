// The loss: every hero struck to nothing ends the run. A hero at 1 health
// ends its turn into the foes' attacks; the summary says what struck it
// down after a beat; March again starts a new run from nothing (whole
// health, the starting deck, the map's first step).
import { playtest } from "@origingame/engine/playtest";
import { endTurn, game, hero, MAP, OVER, script, tap, texts, TWO } from "./lib.ts";

export default playtest({
  name: "out of health the run is lost; the summary, then March again from nothing",
  commands: script([
    "0.3:goto:1",
    "0.4:give:cleave",
    "0.5:hp:1",
    // Two: the Hunter down already, so the foes' blows all fall on the Warden.
    ...(TWO ? ["0.5:hp:0:1"] : []),
    endTurn(0.7),
    "3.5:get:Game:lost.json",
    "3.6:dump_ui:summary.json",
    tap("Again", 3.8),
    "4.3:get:Game:again.json",
    "4.3:dump_entity:hero:again-hero.json",
    "4.4:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const lost = game(run, "lost.json");
    t.equal(lost.phase, OVER, "every hero down: the summary");
    t.equal(lost.won, false, "the run is lost");
    t.equal(lost.killedBy, 2, "struck down by a bog slime");
    t.ok(texts(run, "summary.json").includes("Struck down by the Bog slime"), "the summary says what struck the last hero down");
    const again = game(run, "again.json");
    t.equal(again.phase, MAP, "March again opened the map");
    t.equal(again.step, 0, "at its first step");
    const h = hero(run, "again-hero.json");
    t.equal(h.hp, h.hpMax, "whole again");
    t.equal(h.deck.length, 10, "the starting deck again (the given card gone)");
  },
});
