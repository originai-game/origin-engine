// The win and the replay: the four beacons lit end the run at dawn; after
// a beat the summary says so with the time, the beacons and the best. Meanwhile
// another device wrote the record, so the end's write is refused as stale,
// read again and merged (the other device's runs, its wins plus this one,
// the better best) and written at the new revision; the end goes out as a
// `run.over` event. Again starts a second run from nothing: hearts full,
// the beacons dark, the clock back at half past three.
import { playtest } from "@origingame/engine/playtest";
import { DAWN, game, OVER, PLAY, type PlatformDump, SUMMARY_READY, tap, texts, value } from "./lib.ts";

const OVER_AT = 2.1;

export default playtest({
  name: "four beacons lit: dawn, a merged record, the summary, then Again from nothing",
  commands: [
    "0.3:run",
    "0.6:stale_record",
    "0.7:light:0",
    "0.8:light:1",
    "0.9:light:2",
    "0.95:get:Game:three.json",
    "1.0:light:3",
    "1.05:get:Game:dawn.json",
    `${OVER_AT + 0.6}:get:Game:over.json`,
    `${OVER_AT + 0.6}:dump_platform:platform.json`,
    `${OVER_AT + 1.2}:dump_ui:summary.json`,
    tap("Again", OVER_AT + SUMMARY_READY + 0.1),
    `${OVER_AT + SUMMARY_READY + 0.5}:get:Game:again.json`,
    `${OVER_AT + SUMMARY_READY + 0.5}:get:beacon-2.Beacon:beacon.json`,
    `${OVER_AT + SUMMARY_READY + 0.6}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const three = game(run, "three.json");
    t.ok(three.phase === PLAY && three.lit === 3, "three beacons lit, the run goes on");
    const dawn = game(run, "dawn.json");
    t.ok(dawn.lit === 4 && dawn.litMask === 15 && dawn.ending === DAWN, "the fourth brings the dawn");
    t.equal(dawn.phase, PLAY, "the summary waits a beat");
    const over = game(run, "over.json");
    t.equal(over.phase, OVER, "then the run is over");
    t.equal(over.recordConflicts, 1, "the record write was refused once as stale");
    t.ok(over.runs === 9 && over.wins === 4, `and merged: the other device's 9 runs and 3 wins, plus this win (${over.runs}, ${over.wins})`);
    t.ok(over.best > 0 && over.best < 2 && over.newBest, `this run's time is the best (${over.best.toFixed(2)} s, not the other device's 200 s)`);
    const p = run.json<PlatformDump>("platform.json");
    const writes = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record");
    const last = writes[writes.length - 1];
    const merged = last?.args["value"] as { runs: number; wins: number; best: number } | undefined;
    t.ok(merged?.runs === 9 && merged.wins === 4 && Math.abs(merged.best - over.best) < 1e-3, `the merge was written (${JSON.stringify(merged)})`);
    t.equal(last?.result?.ok, true, "at the revision read again");
    const event = p.calls.find((c) => c.method === "telemetry" && c.args["name"] === "run.over");
    t.ok(event !== undefined, "the end went out as a run.over event");
    const words = texts(run, "summary.json");
    t.ok(words.includes("Dawn will find the vale lit"), "the summary shows the win");
    t.ok(words.includes("4/4") && words.some((w) => w.endsWith(" ★")), "four beacons and a new best time");
    t.ok(words.includes("Runs 9 · beacons all lit 4"), "and the merged record");
    const again = game(run, "again.json");
    t.equal(again.phase, PLAY, "Again started a run");
    t.ok(again.runs === 10 && again.hearts === 3 && again.lit === 0 && again.ending === 0, "the tenth, from nothing: hearts full, no beacon lit");
    t.ok(again.hour < 15.6, "the clock back at half past three");
    t.equal(value<{ lit: boolean }>(run, "beacon.json").lit, false, "the beacons dark again");
  },
});
