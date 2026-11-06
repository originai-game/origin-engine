// The win: the map's last step is the Ashen King; struck down, the run is
// won. A card given to the hand (Heavy Blow), the d-pad's left wrapping the
// focus to it, the play key: the King falls, the summary says so after a
// beat, the record counts a crown, and Title goes back to the title.
import { playtest } from "@origingame/engine/playtest";
import { BATTLE, foe, game, OVER, press, script, tap, texts, TITLE } from "./lib.ts";

export default playtest({
  name: "the Ashen King struck down wins the run; the summary, then the title",
  commands: script([
    "0.3:goto:5",
    "0.4:foehp:0:1",
    "0.4:give:heavy-blow",
    "0.6:get:Game:boss.json",
    "0.6:dump_entity:foe-0:king.json",
    press("ArrowLeft", 0.7),
    press("Enter", 0.9),
    "1.1:dump_entity:foe-0:fallen.json",
    "2.9:get:Game:won.json",
    "3.0:dump_ui:summary.json",
    tap("Menu", 3.2),
    "3.9:get:Game:title.json",
    "4.0:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const boss = game(run, "boss.json");
    t.equal(boss.phase, BATTLE, "the last step's fight");
    t.equal(foe(run, "king.json").kind, 4, "against the Ashen King");
    t.equal(foe(run, "fallen.json").hp, 0, "Heavy Blow struck him down");
    const won = game(run, "won.json");
    t.equal(won.phase, OVER, "the summary after a beat");
    t.equal(won.won, true, "the run is won");
    t.equal(won.wins, 1, "the record counts a crown");
    t.ok(texts(run, "summary.json").includes("The Ashen King falls"), "the summary says the King fell");
    t.equal(game(run, "title.json").phase, TITLE, "Title went back to the title");
  },
});
