// The time away from the host's clock, across two processes (og test keeps
// the store between them): the first plays, buys ten embers and two storms
// and saves with Escape at its host time; the second launches two hours
// later by its own clock, offers Continue, and the welcome back credits
// the two hours (less the seconds the first run went on after the save) at
// half the rate. No relaunch command: the host's time alone.
import { playtest } from "@origingame/engine/playtest";
import { game, press, tap, text, TITLE } from "./lib.ts";

const T = 1767225600000;
const TWO_HOURS = 2 * 3600 * 1000;

export default playtest({
  name: "two hours away by the host's clock, across processes, credited at half the rate",
  store: "keep",
  clock: T,
  commands: ["0.3:play", "0.9:own:0:10", "0.9:own:1:2", "1.0:sparks:500", press("Escape", 1.2), "1.5:get:Game:saved.json", "1.6:quit"].join(";"),
  then: [
    {
      clock: T + TWO_HOURS,
      commands: ["0.5:get:Game:launched.json", "0.5:dump_ui:title.json", tap("play", 0.6), "1.5:get:Game:welcome.json", "1.5:dump_ui:welcome-ui.json", "1.6:quit"].join(";"),
    },
  ],
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the first run ran without warnings");
    const saved = game(run, "saved.json");
    t.equal(saved.saves, 1, "Escape saved");
    t.equal(saved.rate, 18, "ten embers (doubled) and two storms: 18 a second");
    const later = run.then[0]!;
    t.equal(later.warnings.join("; "), "", "the second run ran without warnings");
    const launched = game(later, "launched.json");
    t.equal(launched.phase, TITLE, "the second process opens on the title");
    t.ok(text(later, "title.json", "play")?.startsWith("Continue") ?? false, "which offers Continue: the save was kept");
    const w = game(later, "welcome.json");
    t.equal(w.owned0, 10, "the save came back");
    // Saved about 1.2 s into the first run, read on the second's first ticks: two hours less about a second.
    t.ok(w.away > 7198 && w.away < 7200, `away two hours by the hosts' clocks (${w.away.toFixed(3)} s)`);
    t.near(w.offline, 18 * 0.5 * w.away, 1e-6, "credited at half of 18 a second");
    t.equal(w.welcome, true, "the welcome back is up");
    t.ok(text(later, "welcome-ui.json", "Body")?.startsWith("While you were away (1 h 59 min)") ?? false, `and says how long (${text(later, "welcome-ui.json", "Body")})`);
  },
});
