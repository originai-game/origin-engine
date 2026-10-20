// The two losses and the way back: the last heart gone ends the run (the
// slimes won); Again starts afresh; the clock run to Tuning.duskSeconds
// brings night; Title goes back to the title, the vale reset behind it and
// the dusk look back.
import { playtest } from "@origingame/engine/playtest";
import { CAUGHT, game, NIGHT, OVER, PLAY, RESULTS_READY, tap, texts, TITLE } from "./lib.ts";

export default playtest({
  name: "no hearts left, night at dusk's end, Again and Title",
  commands: [
    "0.3:run",
    "0.5:hearts:0",
    "0.6:get:Game:caught.json",
    "2.2:get:Game:caught-over.json",
    "2.4:dump_ui:caught-ui.json",
    tap("again", 0.5 + RESULTS_READY),
    "4.0:get:Game:again.json",
    "4.1:dusk:299.6",
    "4.6:get:Game:night.json",
    "6.0:get:Game:night-over.json",
    "6.2:dump_ui:night-ui.json",
    tap("menu", 4.1 + RESULTS_READY),
    "7.6:get:Game:title.json",
    "7.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "caught.json").ending, CAUGHT, "no hearts left: the slimes won");
    t.equal(game(run, "caught-over.json").phase, OVER, "the results after a beat");
    t.ok(texts(run, "caught-ui.json").includes("The slimes won"), "they say the slimes won");
    const again = game(run, "again.json");
    t.ok(again.phase === PLAY && again.hearts === 3 && again.ending === 0 && again.time < 1, "Again started a fresh run");
    const night = game(run, "night.json");
    t.equal(night.ending, NIGHT, "dusk's end brings night");
    t.ok(night.hour >= 19.5, `at half past seven (${night.hour.toFixed(2)})`);
    t.equal(game(run, "night-over.json").phase, OVER, "the results after a beat");
    const words = texts(run, "night-ui.json");
    t.ok(words.includes("Night fell first") && words.includes("0/4"), "they say night fell first, no lantern lit");
    const title = game(run, "title.json");
    t.ok(title.phase === TITLE && title.look === 0, "Title went back to the title, under the dusk look");
    t.ok(title.hour > 16 && title.hour < 17, "where the vale is in late afternoon again");
  },
});
