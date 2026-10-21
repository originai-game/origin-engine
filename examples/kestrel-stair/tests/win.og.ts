// The climb's end. At the summit gate with two sigils F is refused (the
// prompt says how many it needs and shakes, the error sound); with three it
// rises (its collider gone, its navmesh obstacle lifted) and the courier
// walks through. F at the bell rings it (its spatial toll, the win) and
// plays the bell-keeper's ending on og.ui.cutscene under the summit look
// (looks/summit.ogt: depth bands down the stair). The results
// (og.ui.resultsScreen) give the score, three stars and a new record; the
// ascent's time goes on the `ascent` leaderboard (og.platform, lower is
// better) and the Records screen (og.ui.leaderboardScreen) shows it,
// highlighted as the courier's own. Again starts a new journey under the
// stair's look.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, OVER, press, RUNG, script, tap, texts, value } from "./lib.ts";

const ENTERS = [6.6, 7.0, 7.4, 7.8, 8.2, 8.6];

export default playtest({
  name: "the gate refuses two sigils and rises for three; the bell ends the climb under the summit look; the results, the leaderboard and Again",
  commands: script(
    "0.3:play:0:quick",
    "0.35:set:Tuning.sight=0",
    "0.4:give:sigils:2",
    "0.5:warp:gate",
    "1.0:dump_ui:shut.json",
    press("KeyF", 1.1),
    "1.15:get:Game:refused.json",
    "1.15:get:sfx-262144.Sfx:error.json",
    "1.2:give:sigils:3",
    press("KeyF", 1.5),
    "1.6:get:Game:rising.json",
    "3.6:get:gate.og.Mesh:gate-mesh.json",
    "3.6:get:Game:open.json",
    "3.7:key_down:KeyW",
    "5.3:key_up:KeyW",
    "5.4:get:Game:through.json",
    "5.5:warp:bell",
    press("KeyF", 5.8),
    "6.0:get:bell.og.AudioSource:bell.json",
    "6.3:get:Game:ending.json",
    "6.3:dump_ui:ending-ui.json",
    ...ENTERS.map((t) => press("Enter", t)),
    "10.4:get:Game:results.json",
    "10.4:dump_ui:results-ui.json",
    "10.6:dump_platform:calls.json",
    tap("records", 10.7),
    "11.6:get:Game:records.json",
    "11.6:dump_ui:records-ui.json",
    "11.65:screenshot:records.png",
    press("Escape", 11.8),
    tap("again", 12.3),
    "13.4:get:Game:again.json",
    "13.5:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(texts(run, "shut.json").includes("The gate needs three sigils (2/3)"), "the prompt says the gate needs three sigils");
    const refused = game(run, "refused.json");
    t.ok(!refused.gateOpen && refused.refusedAt > 0, "F with two is refused (the prompt shakes)");
    t.ok(value<{ plays: number }>(run, "error.json").plays >= 1, "with the error sound");
    t.ok(game(run, "rising.json").gateOpen, "with three it rises");
    t.equal(value<{ visible: boolean }>(run, "gate-mesh.json").visible, false, "into the wall");
    t.ok(game(run, "through.json").heroZ < -44.6, `the courier walked through (z ${game(run, "through.json").heroZ.toFixed(2)})`);
    t.ok(value<{ playing: boolean; spatial: boolean }>(run, "bell.json").playing, "the bell tolls");
    const ending = game(run, "ending.json");
    t.equal([ending.phase, ending.ending, ending.scene, ending.look], [2, RUNG, 2, 1], "the ending plays under the summit look");
    t.ok(texts(run, "ending-ui.json").includes("Oswin the bell-keeper"), "with the bell-keeper's words");
    const results = game(run, "results.json");
    t.equal([results.phase, results.ending], [OVER, RUNG], "then the results");
    t.ok(results.newBest && results.best > 0 && results.score > 0, `a new record (${results.best} s, score ${results.score})`);
    const shown = texts(run, "results-ui.json");
    t.ok(shown.includes("The bell rings") && shown.includes("New record") && shown.includes("3/3"), "the results say so, with the sigils");
    const calls = run.json<{ calls: Array<{ method: string; args: { board?: string; order?: string; score?: number } }> }>("calls.json").calls;
    const submit = calls.find((c) => c.method === "leaderboard.submit");
    t.ok(submit !== undefined && submit.args.board === "ascent" && submit.args.order === "asc" && submit.args.score === results.best, "the ascent's time went on the board, lower being better");
    t.ok(results.submitted === 1 && results.rank === 1, `and ranked (${results.rank})`);
    const records = game(run, "records.json");
    t.equal(records.recordsState, 2, "the Records screen read the board");
    const rows = elements(run, "records-ui.json");
    t.ok(rows.some((e) => e.text === "You") && rows.some((e) => e.text === "Fastest"), "and shows the courier's row among the fastest");
    const again = game(run, "again.json");
    t.equal([again.phase, again.ending, again.look, again.gateOpen], [1, 0, 0, false], "Again: a new journey under the stair's look, the gate shut");
  },
});
