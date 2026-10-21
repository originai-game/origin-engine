// Watch replay: the world saved in the countdown (world.snapshot() through
// og.platform.saveSlot) is restored from the results (world.restore) and
// the run's taped inputs drive the car again; the same world and the same
// inputs give the same race, every lap on the tick it was driven.
import { playtest } from "@origingame/engine/playtest";
import { FINISHED, race, script, tap, texts } from "./lib.ts";

interface Call { method: string; args: Record<string, unknown>; result?: { ok: boolean; data?: Record<string, unknown> } }

export default playtest({
  name: "Watch replay restores the countdown's world and drives the taped inputs to the same laps, tick for tick",
  commands: script(
    "0.5:race",
    "3.6:drive:auto",
    "101:get:Race:raced.json",
    "103:dump_ui:results.json",
    tap("Actions/replay", 103.2),
    "104:get:Race:replaying.json",
    "204:get:Race:replayed.json",
    "204:dump_ui:replay-results.json",
    "204.1:dump_platform:calls.json",
    "204.2:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const a = race(run, "raced.json");
    t.equal(a.phase, FINISHED, "the race finished");
    t.ok(texts(run, "results.json").includes("Watch replay"), "the results offer the replay");
    const calls = run.json<{ calls: Call[] }>("calls.json").calls;
    const slot = calls.find((c) => c.method === "slot.save" || (c.method === "store.set" && c.args.key === "replay"));
    t.ok(slot !== undefined, "the countdown's world was saved in the replay slot");
    t.ok(calls.some((c) => c.method === "store.set" && c.args.key === "replay.tape" && c.result?.ok === true), "the run's inputs were stored at the finish");
    const mid = race(run, "replaying.json");
    t.ok(mid.replaying, "the replay is running");
    t.equal(mid.replays, 1, "once");
    const b = race(run, "replayed.json");
    t.equal(b.phase, FINISHED, "the replay finished");
    t.equal(b.replayMatched, 1, "on the race's own tick");
    t.equal(b.total, a.total, `the same total (${b.total} ticks)`);
    t.equal([b.lap1, b.lap2, b.lap3].join(","), [a.lap1, a.lap2, a.lap3].join(","), "the same three laps");
    t.equal([b.recordS1, b.recordS2, b.recordS3].join(","), [a.recordS1, a.recordS2, a.recordS3].join(","), "the same sectors");
    t.ok(texts(run, "replay-results.json").includes("Replay: the same race, tick for tick"), "the results say so");
    const submits = calls.filter((c) => c.method === "leaderboard.submit").length;
    t.equal(submits, 1, "the replay put nothing on the board");
  },
});
