// Holding the line: clearing a wave gives a breather before the next, the
// reserve refilled (and with two seats a downed partner back on its
// feet); clearing wave 5 holds the arena. The summary counts it a win and a
// best score in the device's record, and Again starts the next run.
import { playtest } from "@origingame/engine/playtest";
import { FIGHTING, game, HELD, hold, OVER, PLAY, press, SEATS, SUMMARY_READY, tap, texts, value, type Hero } from "./lib.ts";

const partnerDown = SEATS > 1 ? ["4.2:hurt:500:1", "4.3:get:hero-1.Hero:down.json"] : [];

export default playtest({
  name: "a wave cleared refills and revives, wave 5 cleared holds the arena, the summary and Again",
  commands: [
    "0.3:run",
    "3.5:get:Game:w1.json",
    "3.6:warp:-3:2",
    hold("KeyF", 3.7, 0.4),
    press("KeyR", 4.15),
    ...partnerDown,
    "5.7:clear",
    "6.0:get:Game:cleared.json",
    "6.0:get:hero.Hero:refilled.json",
    ...(SEATS > 1 ? ["6.0:get:hero-1.Hero:revived.json"] : []),
    "6.2:wave:5",
    "7.0:get:Game:w5.json",
    "7.1:clear",
    "7.3:get:Game:held.json",
    `${(7.3 + SUMMARY_READY).toFixed(2)}:get:Game:summary.json`,
    `${(7.4 + SUMMARY_READY).toFixed(2)}:dump_ui:summary-ui.json`,
    tap("Again", 7.5 + SUMMARY_READY),
    `${(7.9 + SUMMARY_READY).toFixed(2)}:get:Game:again.json`,
    `${(8.0 + SUMMARY_READY).toFixed(2)}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(game(run, "w1.json").wave === 1 && game(run, "w1.json").waveState === FIGHTING, "wave 1 is on");
    const cleared = game(run, "cleared.json");
    t.ok(cleared.wave === 2 && cleared.waveState !== FIGHTING && cleared.ending === 0, `clearing it gives the breather before wave 2 (wave ${cleared.wave})`);
    const refilled = value<Hero>(run, "refilled.json");
    t.equal(refilled.reserve, 120, "and refills the reserve the reload drew from");
    if (SEATS > 1) {
      t.ok(value<Hero>(run, "down.json").down, "seat 1's trooper went down");
      const revived = value<Hero>(run, "revived.json");
      t.ok(!revived.down && revived.health > 0, `and is back on its feet with the wave cleared (${revived.health})`);
    }
    const w5 = game(run, "w5.json");
    t.ok(w5.wave === 5 && w5.waveState === FIGHTING, "wave 5 comes on");
    const held = game(run, "held.json");
    t.equal(held.ending, HELD, "clearing the last wave holds the arena");
    const s = game(run, "summary.json");
    t.ok(s.phase === OVER && s.wins === 1 && s.best === s.score && s.score > 0, `the summary: a win, the best score (${s.best})`);
    const words = texts(run, "summary-ui.json");
    t.ok(words.includes("The line held") && words.includes("5/5"), "it says the line held, at wave 5 of 5");
    const again = game(run, "again.json");
    t.ok(again.phase === PLAY && again.runs === 2 && again.score === 0 && again.wave <= 1, "Again starts a fresh run");
  },
});
