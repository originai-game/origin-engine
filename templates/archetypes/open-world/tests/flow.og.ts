// The core loop through real input: Set out on the title starts a run at
// the camp on day 1 at seven in the morning with the bag empty and no
// landmark claimed; W walks the explorer on the character controller and
// Shift runs. By the tree nearest the camp the prompt says how to fell it;
// holding Gather (E) fells it into the bag (two wood, shown in the inventory). Then
// short of the chapel it is discovered (its toast), and walking in claims it:
// the flag rises, the brazier catches, the HUD and the quest count it.
import { playtest } from "@origingame/engine/playtest";
import { approach, elements, game, GATHER, PLAY, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "Set out, walk and run, fell a tree into the bag, discover and claim the chapel",
  size: "1280x720",
  commands: [
    "0.3:get:Game:title.json",
    tap("Play", 0.5),
    "0.7:get:Game:started.json",
    "0.8:key_down:KeyW",
    "1.8:get:Game:walk.json",
    "1.9:key_down:ShiftLeft",
    "2.9:get:Game:run.json",
    "3.0:key_up:ShiftLeft",
    "3.0:key_up:KeyW",
    "3.2:warp:tree",
    "3.5:get:Game:near.json",
    "3.5:dump_ui:prompt.json",
    "3.6:key_down:KeyE",
    "4.2:get:Game:felling.json",
    "5.0:key_up:KeyE",
    "5.1:get:Game:felled.json",
    "5.1:dump_ui:bag.json",
    approach(2, 5.3),
    "5.6:get:Game:seen.json",
    "5.6:dump_ui:seen-ui.json",
    "5.7:key_down:KeyW",
    "8.9:key_up:KeyW",
    "9.0:get:Game:claimed.json",
    "9.0:get:site-2.Site:site.json",
    "9.0:dump_ui:claimed-ui.json",
    "9.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "Set out started a run");
    t.ok(started.claimed === 0 && started.wood === 0 && started.stone === 0 && !started.bridge, "with no landmark claimed, the bag empty and no bridge");
    t.ok(started.day === 1 && started.hour >= 7 && started.hour < 7.1, `on day 1 at seven in the morning (${started.hour.toFixed(2)})`);
    const walk = game(run, "walk.json");
    t.ok(walk.grounded && Math.abs(walk.speed - 3.4) < 0.15, `W walks on the ground at the walk speed (${walk.speed.toFixed(2)} m/s)`);
    t.ok(Math.abs(game(run, "run.json").speed - 6.8) < 0.2, `Shift runs at the run speed (${game(run, "run.json").speed.toFixed(2)} m/s)`);
    t.ok(game(run, "near.json").near >= 0, "by the tree it is within reach");
    t.ok(texts(run, "prompt.json").includes(`Hold ${GATHER} to fell the tree`), "and the prompt says how to fell it");
    t.ok(game(run, "felling.json").gathering, "holding E works the tree");
    const felled = game(run, "felled.json");
    t.ok(felled.felled === 1 && felled.wood === 2 && felled.woodTotal === 2, `it fell into the bag: two wood (${felled.wood})`);
    const bag = elements(run, "bag.json").filter((e) => e.path.startsWith("Status/Bag/"));
    t.ok(bag.some((e) => e.text === "2"), "and the inventory shows the two wood");
    const seen = game(run, "seen.json");
    t.ok((seen.seenMask & 4) !== 0 && (seen.claimMask & 4) === 0, "short of the chapel it is discovered, not yet claimed");
    t.ok(texts(run, "seen-ui.json").includes("Discovered: The Chapel"), "and its toast says so");
    const claimed = game(run, "claimed.json");
    t.ok(claimed.claimed === 1 && claimed.claimMask === 4, `walking in claimed it (${claimed.claimed}, mask ${claimed.claimMask})`);
    t.equal(value<{ claimed: boolean }>(run, "site.json").claimed, true, "the chapel's Site is claimed");
    const words = texts(run, "claimed-ui.json");
    t.ok(words.includes("The Chapel claimed · 1/5") && words.includes("1/5"), "the toast and the HUD count it");
    t.ok(words.includes("Landmarks claimed 1/5"), "and so does the quest");
  },
});
