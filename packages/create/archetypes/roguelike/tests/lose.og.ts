// Permadeath: a lamp-bearer of three health and a blade found waits by a
// rat until the lamp goes out; a beat later the summary says how and how
// deep; Dive again starts from nothing (a new seed, floor 1, the blade
// gone, no kills).
import { playtest } from "@origingame/engine/playtest";
import { game, OVER, PLAY, presses, RAT, tap, texts } from "./lib.ts";

export default playtest({
  name: "the lamp goes out: the summary, and the next dive starts from nothing",
  commands: [
    "0.2:set:Tuning.heroHp=3",
    "0.3:dive:7",
    "0.4:give:blade",
    "0.5:warp:5:7",
    "0.6:get:Game:armed.json",
    presses(["Space", "Space", "Space"], 0.7),
    "1.75:get:Game:out.json",
    "3.0:get:Game:over.json",
    "3.0:dump_ui:summary.json",
    "3.05:screenshot:lost.png",
    tap("Again", 3.2),
    "3.6:get:Game:again.json",
    "3.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "armed.json").attack, 4, "the blade raised the strike to 4");
    const out = game(run, "out.json");
    t.equal(out.hp, 0, "the rat put the lamp out");
    t.equal(out.killedBy, RAT, "the summary will name the rat");
    t.equal(out.phase, PLAY, "the summary waits a beat");
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "then the summary");
    const words = texts(run, "summary.json");
    t.ok(words.includes("The lamp went out on floor 1"), "it says where the lamp went out");
    t.ok(words.includes("a tomb rat"), "and what put it out");
    t.ok(words.includes("Dives 1 · relics 0"), "and the record");
    const again = game(run, "again.json");
    t.equal(again.phase, PLAY, "Dive again started a new dive");
    t.ok(again.seed !== over.seed, `on a new seed (${over.seed} → ${again.seed})`);
    t.ok(again.floor === 1 && again.hp === 3 && again.attack === 3 && again.kills === 0 && again.turns === 0, "from nothing: floor 1, full health, the blade gone, no kills, no turns");
    t.equal(again.dives, 2, "the record counts the second dive");
  },
});
