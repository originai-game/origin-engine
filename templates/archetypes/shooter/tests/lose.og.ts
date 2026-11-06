// Overrun: a walker's strikes hurt the trooper standing in its way (the
// health falls and the hurt is marked for the damage arc), and with every
// trooper down the holdout is overrun. The summary says so, and its Title
// button goes back to the title screen.
import { playtest } from "@origingame/engine/playtest";
import { downAll, game, OVERRUN, SUMMARY_READY, tap, texts, TITLE, value, type Hero } from "./lib.ts";

export default playtest({
  name: "a walker's strikes hurt, every trooper down is an overrun, the summary and back to the title",
  commands: [
    "0.3:run",
    "0.5:warp:-3:2",
    "0.6:spawn:walker:-3:-1",
    "4.0:get:hero.Hero:struck.json",
    downAll(4.2),
    "4.4:get:Game:overrun.json",
    `${(4.4 + SUMMARY_READY).toFixed(2)}:dump_ui:summary-ui.json`,
    tap("Menu", 4.5 + SUMMARY_READY),
    `${(4.9 + SUMMARY_READY).toFixed(2)}:get:Game:title.json`,
    `${(5.0 + SUMMARY_READY).toFixed(2)}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const me = value<Hero & { hurtTick: number }>(run, "struck.json");
    t.ok(me.health < 100 && me.hurtTick > 0, `the walker's strikes hurt the trooper (health ${me.health})`);
    const over = game(run, "overrun.json");
    t.ok(over.ending === OVERRUN && over.downs >= 1, `every trooper down: overrun (${over.downs} down)`);
    t.ok(texts(run, "summary-ui.json").includes("Overrun"), "the summary says so");
    const back = game(run, "title.json");
    t.ok(back.phase === TITLE && back.ending === 0 && back.wins === 0 && back.runs === 1, "its Title button goes back to the title, no win counted");
  },
});
