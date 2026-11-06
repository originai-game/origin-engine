// The win and the record: the fifth landmark claimed lights the isle and
// ends the run; after a beat the summary says so with the day and hour, the
// landmarks, the bag's haul, the bridge and the best time. Meanwhile another
// device wrote the record, so the end's write is refused as stale, read
// again and merged (the other device's runs and wins plus this win, the
// better best) and written at the new revision; the end goes out as a
// `run.over` event. Again starts a second run from nothing.
import { playtest } from "@origingame/engine/playtest";
import { game, LIT, OVER, PLAY, type PlatformDump, SUMMARY_READY, tap, texts, value } from "./lib.ts";

const OVER_AT = 1.1;

export default playtest({
  name: "five landmarks claimed: the isle lit, a merged record, the summary, then Again from nothing",
  size: "1280x720",
  commands: [
    "0.3:run",
    "0.6:stale_record",
    "0.7:claim:0",
    "0.8:claim:1",
    "0.9:claim:2",
    "1.0:claim:3",
    "1.05:get:Game:four.json",
    "1.1:claim:4",
    "1.15:get:Game:lit.json",
    `${OVER_AT + 1.6}:get:Game:over.json`,
    `${OVER_AT + 1.7}:dump_platform:platform.json`,
    `${OVER_AT + 1.9}:dump_ui:summary.json`,
    tap("Again", OVER_AT + SUMMARY_READY + 0.1),
    `${OVER_AT + SUMMARY_READY + 0.5}:get:Game:again.json`,
    `${OVER_AT + SUMMARY_READY + 0.5}:get:site-3.Site:site.json`,
    `${OVER_AT + SUMMARY_READY + 0.6}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const four = game(run, "four.json");
    t.ok(four.phase === PLAY && four.claimed === 4, "four landmarks claimed, the run goes on");
    const lit = game(run, "lit.json");
    t.ok(lit.claimed === 5 && lit.claimMask === 31 && lit.ending === LIT, "the fifth lights the isle");
    t.equal(lit.phase, PLAY, "the summary waits a beat");
    t.equal(game(run, "over.json").phase, OVER, "then the run is over");
    const p = run.json<PlatformDump>("platform.json");
    const writes = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record");
    t.ok(writes.some((c) => c.result?.ok === false && c.result.error?.code === "conflict"), "the record write was refused once as stale");
    const last = writes[writes.length - 1];
    const merged = last?.args["value"] as { runs: number; wins: number; best: number } | undefined;
    t.ok(merged?.runs === 9 && merged.wins === 4, `and merged: the other device's 9 runs and 3 wins, plus this win (${JSON.stringify(merged)})`);
    t.ok(merged !== undefined && merged.best > 0 && merged.best < 2, "this run's time is the best (not the other device's 300 s)");
    t.equal(last?.result?.ok, true, "written at the revision read again");
    const event = p.calls.find((c) => c.method === "telemetry" && c.args["name"] === "run.over");
    t.ok(event !== undefined, "the end went out as a run.over event");
    const words = texts(run, "summary.json");
    t.ok(words.includes("Every light burns on the isle") && words.includes("All 5 landmarks claimed"), "the summary shows the win");
    t.ok(words.includes("5/5") && words.some((w) => w.endsWith(" · new best")), "five landmarks and a new best time");
    t.ok(words.includes("Day 1, 7:00") || words.some((w) => w.startsWith("Day 1, 7:")), "ended on the first morning");
    t.ok(words.includes("Runs 9 · isle lit 4"), "and the merged record");
    const again = game(run, "again.json");
    t.equal(again.phase, PLAY, "Again started a run");
    t.ok(again.claimed === 0 && again.ending === 0 && again.wood === 0 && !again.bridge && again.time < 0.5, "from nothing: no landmark, an empty bag, no bridge, the clock back at the start");
    t.equal(value<{ claimed: boolean }>(run, "site.json").claimed, false, "the landmarks unclaimed again");
  },
});
