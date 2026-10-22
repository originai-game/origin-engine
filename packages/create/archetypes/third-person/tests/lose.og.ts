// The two losses and the way back: the last heart gone ends the run caught
// (the summary with the skull); Again starts afresh; the clock run to
// Tuning.duskSeconds brings night (the vale goes dark, the summary says
// night took it); Title goes back to the title, the vale reset behind it.
import { playtest } from "@origingame/engine/playtest";
import { CAUGHT, game, NIGHT, OVER, PLAY, SUMMARY_READY, tap, texts, TITLE } from "./lib.ts";

export default playtest({
  name: "caught with no hearts left, night at dusk's end, Again and Title",
  commands: [
    "0.3:run",
    "0.5:hearts:0",
    "0.6:get:Game:caught.json",
    "2.3:get:Game:caught-over.json",
    "2.6:dump_ui:caught-ui.json",
    tap("Again", 2.1 + SUMMARY_READY),
    "4.1:get:Game:again.json",
    "4.2:dusk:149.6",
    "4.7:get:Game:night.json",
    "6.4:get:Game:night-over.json",
    "6.7:dump_ui:night-ui.json",
    tap("Menu", 6.2 + SUMMARY_READY),
    "8.6:get:Game:title.json",
    "8.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "caught.json").ending, CAUGHT, "no hearts left: caught");
    const co = game(run, "caught-over.json");
    t.equal(co.phase, OVER, "the summary after a beat");
    t.ok(texts(run, "caught-ui.json").includes("The sentinels caught you"), "it says the sentinels caught you");
    const again = game(run, "again.json");
    t.ok(again.phase === PLAY && again.hearts === 3 && again.ending === 0 && again.time < 0.5, "Again started a fresh run");
    const night = game(run, "night.json");
    t.equal(night.ending, NIGHT, "dusk's end brings night");
    t.ok(night.hour >= 19.5, `at half past seven (${night.hour.toFixed(2)})`);
    t.equal(game(run, "night-over.json").phase, OVER, "the summary after a beat");
    const words = texts(run, "night-ui.json");
    t.ok(words.includes("Night took the vale") && words.includes("0/4"), "it says night took the vale, no beacon lit");
    const title = game(run, "title.json");
    t.equal(title.phase, TITLE, "Title went back to the title");
    t.ok(title.hour > 16 && title.hour < 17, "where the vale is in late afternoon again");
  },
});
