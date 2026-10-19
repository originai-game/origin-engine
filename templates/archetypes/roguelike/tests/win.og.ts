// The whole dive's end: seed 7's stairs at (23, 11) take the lamp-bearer
// to floor 2 (the foes left behind take no turn); the moss-faced guardian
// stands before the relic at (24, 14), glows red and shows "!" the turn
// before its charged strike; felled, the relic is taken, and the summary
// shows the win. Meanwhile another device wrote the dive record, so the
// end's write is refused as stale, read again and merged (this dive's
// relic added to the other device's counts).
import { playtest } from "@origingame/engine/playtest";
import { foe, game, OVER, type PlatformDump, press, presses, tap, texts } from "./lib.ts";

export default playtest({
  name: "down the stairs, past the guardian, the relic, a merged record and the summary",
  commands: [
    "0.3:dive:7",
    "0.5:warp:23:10",
    "0.6:get:Game:above.json",
    press("ArrowDown", 0.7),
    "1.0:get:Game:below.json",
    "1.0:get:Foes:below-foes.json",
    "1.1:warp:22:14",
    "1.15:stale_record",
    presses(Array(8).fill("ArrowRight"), 1.3),
    "3.9:get:Game:relic.json",
    "5.0:get:Game:over.json",
    "5.0:dump_ui:summary.json",
    "5.0:dump_platform:platform.json",
    "5.05:screenshot:won.png",
    tap("Menu", 5.2),
    "5.8:get:Game:menu.json",
    "5.9:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const above = game(run, "above.json");
    t.equal(`${above.stairsX},${above.stairsY}`, "23,11", "seed 7's stairs are at (23, 11)");
    const below = game(run, "below.json");
    t.equal(below.floor, 2, "the stairs took the lamp-bearer down");
    t.equal(below.deepest, 2, "the deepest floor is 2");
    t.equal(below.hurts, 0, "the foes left behind took no turn");
    t.equal(foe(run, "below-foes.json", 9).join(","), "23,14", "floor 2's guardian stands before the relic");
    const relic = game(run, "relic.json");
    t.ok(relic.won, "the relic was taken");
    t.ok(relic.kills >= 1 && relic.hits >= 5, `after the guardian fell (${relic.hits} strikes)`);
    t.ok(relic.hurts >= 1, `it struck back (${relic.hurts} times)`);
    t.ok(relic.hp > 0, `and the lamp still burns (${relic.hp})`);
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "the summary came");
    t.equal(over.recordConflicts, 1, "the record write was refused once as stale");
    t.ok(over.dives === 7 && over.relics === 3, `and merged: the other device's 7 dives and 2 relics, plus this relic (${over.dives}, ${over.relics})`);
    const words = texts(run, "summary.json");
    t.ok(words.includes("The relic is yours"), "the summary shows the win");
    t.ok(words.includes("Dives 7 · relics 3"), "and the merged record");
    const p = run.json<PlatformDump>("platform.json");
    const writes = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record");
    const last = writes[writes.length - 1];
    t.equal(JSON.stringify(last?.args["value"]), JSON.stringify({ dives: 7, relics: 3 }), "the merge was written");
    t.equal(last?.result?.ok, true, "at the revision read again");
    t.equal(game(run, "menu.json").phase, 0, "Title went back to the title");
  },
});
