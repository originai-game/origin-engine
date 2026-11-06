// The clock and the loss: daylight runs to nightfall (the first night's
// toast), the next morning breaks and the felled woods grow back; the third
// nightfall ends the run with what was claimed, the summary says the third
// night fell, and Title goes back to the title with the isle reset.
import { playtest } from "@origingame/engine/playtest";
import { game, NIGHT, OVER, PLAY, SUMMARY_READY, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "nightfall, a dawn that regrows the woods, the third nightfall ends the run, then Title",
  size: "1280x720",
  commands: [
    "0.3:run",
    "0.4:warp:tree",
    "0.6:key_down:KeyE",
    "2.0:key_up:KeyE",
    "2.1:get:Game:felled.json",
    "2.2:clock:110.5",
    "2.5:get:Game:night1.json",
    "2.5:dump_ui:night1-ui.json",
    "2.6:clock:132.5",
    "2.9:get:Game:day2.json",
    "2.9:dump_ui:day2-ui.json",
    "3.0:claim:2",
    "3.1:clock:373.6",
    "3.6:get:Game:night.json",
    "5.4:get:Game:over.json",
    "5.6:dump_ui:over-ui.json",
    tap("Menu", 3.4 + SUMMARY_READY + 0.2),
    "6.2:get:Game:title.json",
    "6.2:get:site-2.Site:site.json",
    "6.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "felled.json").felled, 1, "a tree felled on the first morning");
    const n1 = game(run, "night1.json");
    t.ok(n1.night && n1.day === 1 && n1.phase === PLAY, `daylight ran out: the first night (hour ${n1.hour.toFixed(2)})`);
    t.ok(texts(run, "night1-ui.json").includes("Night 1 falls"), "and its toast says so");
    const d2 = game(run, "day2.json");
    t.ok(!d2.night && d2.day === 2 && d2.hour >= 7 && d2.hour < 7.5, `the second day broke (${d2.hour.toFixed(2)})`);
    t.ok(texts(run, "day2-ui.json").includes("Day 2 breaks: the woods have grown back"), "its toast says the woods grew back");
    const night = game(run, "night.json");
    t.ok(night.ending === NIGHT && night.day === 3 && night.night, "the third nightfall ends the run");
    t.equal(game(run, "over.json").phase, OVER, "the summary after a beat");
    const words = texts(run, "over-ui.json");
    t.ok(words.includes("The third night fell") && words.includes("1 of 5 landmarks claimed"), "it says the third night fell, one landmark claimed");
    const title = game(run, "title.json");
    t.equal(title.phase, TITLE, "Title went back to the title");
    t.ok(title.claimed === 0 && title.day === 1 && !title.night, "with the isle reset behind it");
    t.equal(value<{ claimed: boolean }>(run, "site.json").claimed, false, "the chapel unclaimed again");
  },
});
