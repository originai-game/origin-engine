// Permadeath: a lamp-bearer of three health and a blade found waits by a
// rat until the lamp goes out; a beat later the summary says how and how
// deep; Dive again starts from nothing (a new seed, floor 1, the blade
// gone, no kills). With two delvers the lamp goes out only when both are
// down: the second is struck down first, and the dive goes on until the
// rat fells the lamp-bearer too.
import { playtest } from "@origingame/engine/playtest";
import { game, OVER, PLAY, presses, RAT, script, tap, texts, TWO } from "./lib.ts";

export default playtest({
  name: "the lamp goes out: the summary, and the next dive starts from nothing",
  commands: script([
    "0.2:set:Tuning.heroHp=3",
    "0.3:dive:7",
    "0.4:give:blade",
    "0.5:warp:5:7",
    // Two delvers: the second is down (as a foe's strike leaves it), and the dive goes on.
    ...(TWO ? ["0.55:hp:0:1", "0.65:get:Game:one-down.json"] : []),
    "0.6:get:Game:armed.json",
    presses(["Space", "Space", "Space"], 0.7),
    "1.75:get:Game:out.json",
    "3.0:get:Game:over.json",
    "3.0:dump_ui:summary.json",
    "3.05:screenshot:lost.png",
    tap("Again", 3.2),
    "3.6:get:Game:again.json",
    "3.7:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "armed.json").attack, 4, "the blade raised the strike to 4");
    if (TWO) {
      const one = game(run, "one-down.json");
      t.ok(one.hp2 === 0 && one.hp > 0 && one.phase === PLAY, `one delver down: the dive goes on (health ${one.hp}, ${one.hp2})`);
    }
    const out = game(run, "out.json");
    t.equal(out.hp, 0, "the rat put the lamp out");
    if (TWO) t.equal(out.hp2, 0, "both delvers down: the lamp is out");
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
    if (TWO) t.equal(again.hp2, 3, "and the second delver whole again");
    t.equal(again.dives, 2, "the record counts the second dive");
  },
});
