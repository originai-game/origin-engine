// The win and the replay: with the yard cleared and the seal in the bag,
// Maren asks for it and the one reply hands it over; that completes the
// quest, and once she has said her thanks the run is won. After a beat the
// summary says so with the time, the risen put down and the record.
// Meanwhile another device wrote the record, so the end's write is
// refused as stale, read again and merged; the end goes out as a
// `run.over` event. Again starts a second run from nothing: no quest, the
// risen standing again, the bag as the party set out.
import { playtest } from "@origingame/engine/playtest";
import { game, OVER, PLAY, type PlatformDump, press, Q_DONE, Q_NONE, Q_RETURN, script, SUMMARY_READY, tap, texts, value, WON, type Foe } from "./lib.ts";

const OVER_AT = 3.6;

export default playtest({
  name: "the seal handed to Maren: the quest done, the run won, a merged record, then Again from nothing",
  commands: script(
    "0.3:run",
    "0.4:stale_record",
    "0.5:quest:report",
    "0.7:warp:maren",
    "0.9:get:Game:ready.json",
    press("KeyE", 1.0),
    "1.3:dump_ui:report.json",
    press("Digit1", 1.6),
    "2.0:get:Game:handed.json",
    "2.0:dump_ui:thanks.json",
    press("KeyE", 2.4),
    "2.6:get:Game:won.json",
    `${OVER_AT + 0.6}:get:Game:over.json`,
    `${OVER_AT + 0.6}:dump_platform:platform.json`,
    `${OVER_AT + 1.2}:dump_ui:summary.json`,
    tap("Again", OVER_AT + SUMMARY_READY + 0.1),
    `${OVER_AT + SUMMARY_READY + 0.5}:get:Game:again.json`,
    `${OVER_AT + SUMMARY_READY + 0.5}:get:foe-4.Foe:captain.json`,
    `${OVER_AT + SUMMARY_READY + 0.6}:quit`,
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const ready = game(run, "ready.json");
    t.ok(ready.quest === Q_RETURN && ready.seal && ready.slain === 5, "the yard is cleared and the seal is in the bag");
    t.ok(texts(run, "report.json").some((s) => s.startsWith("Is that the warden's seal?")), "Maren asks for the seal");
    t.ok(texts(run, "report.json").includes("Here. Close the crypt."), "with the reply that hands it over");
    const handed = game(run, "handed.json");
    t.equal(handed.quest, Q_DONE, "handing it over completes the quest");
    t.equal(handed.ending, 0, "the run goes on while she speaks");
    t.ok(texts(run, "thanks.json").some((s) => s.startsWith("It is done.")), "she thanks the party");
    const won = game(run, "won.json");
    t.ok(won.ending === WON && won.phase === PLAY, "her last word closed the conversation and won the run; the summary waits a beat");
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "then the run is over");
    t.equal(over.recordConflicts, 1, "the record write was refused once as stale");
    t.ok(over.runs === 9 && over.wins === 4, `and merged: the other device's 9 runs and 3 wins, plus this win (${over.runs}, ${over.wins})`);
    t.ok(over.best > 0 && over.best < 5 && over.newBest, `this run's time is the best (${over.best.toFixed(2)} s, not the other device's 200 s)`);
    const p = run.json<PlatformDump>("platform.json");
    const writes = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record");
    const last = writes[writes.length - 1];
    const merged = last?.args["value"] as { runs: number; wins: number; best: number } | undefined;
    t.ok(merged?.runs === 9 && merged.wins === 4 && Math.abs(merged.best - over.best) < 1e-3, `the merge was written (${JSON.stringify(merged)})`);
    t.equal(last?.result?.ok, true, "at the revision read again");
    t.ok(p.calls.some((c) => c.method === "telemetry" && c.args["name"] === "run.over"), "the end went out as a run.over event");
    const words = texts(run, "summary.json");
    t.ok(words.includes("The crypt is sealed"), "the summary shows the win");
    t.ok(words.some((w) => w.endsWith(" ★")), "with a new best time");
    t.ok(words.includes("Runs 9 · crypts sealed 4"), "and the merged record");
    const again = game(run, "again.json");
    t.equal(again.phase, PLAY, "Again started a run");
    t.ok(again.runs === 10 && again.quest === Q_NONE && again.slain === 0 && !again.seal && again.potions === 2 && again.gold === 0, "the tenth, from nothing: no quest, nothing slain, the bag as it set out");
    const captain = value<Foe>(run, "captain.json");
    t.ok(captain.hp === captain.maxHp && captain.dying === 0, "and the captain risen again");
  },
});
