// A whole time trial driven by the autopilot (`drive:auto`, a pure-pursuit
// driver on the centre line): three laps with both splits each, in order;
// the laps add up to the race; the record is the fastest lap and its
// sectors add up to it; the record and its poses are stored, the best lap
// goes on the `laps` board (seconds, ascending); the results show the
// times, and the leaderboard from the results shows the player's row.
import { playtest } from "@origingame/engine/playtest";
import { FINISHED, race, script, tap, texts } from "./lib.ts";

interface Call { method: string; args: Record<string, unknown>; state: string; result?: { ok: boolean; data?: Record<string, unknown> } }

export default playtest({
  name: "three autopilot laps: splits in order, laps summing to the race, the record and its poses stored, the best lap on the board, the results and the leaderboard",
  commands: script(
    "0.5:race:go",
    "0.6:drive:auto",
    "15:get:Race:lap1-mid.json",
    "97.2:get:Race:finish.json",
    "100.5:dump_ui:results.json",
    tap("Actions/board", 101),
    "103:dump_ui:board.json",
    "103:get:Race:board-race.json",
    "103.1:dump_platform:calls.json",
    "103.2:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const mid = race(run, "lap1-mid.json");
    t.ok(mid.split1 > 0 && mid.split2 === 0, `15 s in, the first split is crossed (${mid.split1} ticks) and the second not yet`);
    const r = race(run, "finish.json");
    t.equal(r.phase, FINISHED, "the race is over after three laps");
    t.equal(r.laps, 3, "three laps counted");
    t.equal(r.resets, 0, "no reset");
    t.ok(r.valid, "every lap counted");
    const laps = [r.lap1, r.lap2, r.lap3];
    t.equal(laps[0]! + laps[1]! + laps[2]!, r.total, `the laps add up to the race (${laps.join(" + ")} = ${r.total} ticks)`);
    t.ok(laps.every((l) => l > 25 * 60 && l < 45 * 60), `every lap between 25 and 45 s (${laps.map((l) => (l / 60).toFixed(2)).join(", ")} s)`);
    t.ok(laps[1]! < laps[0]!, "a flying lap beats the standing start");
    t.equal(r.record, Math.min(...laps), "the record is the fastest lap");
    t.equal(r.runBest, r.record, "the run's best is the record in a first session");
    t.equal(r.recordS1 + r.recordS2 + r.recordS3, r.record, `the record's sectors add up to it (${r.recordS1} + ${r.recordS2} + ${r.recordS3})`);
    t.ok(r.recordS1 > 0 && r.recordS2 > 0 && r.recordS3 > 0, "three sectors, each timed");
    t.equal(r.splits, 6, "both splits crossed on every lap");
    t.ok(r.newRecord, "the run set a new record");
    t.equal(r.bestLen, Math.ceil(r.record / 2), `the best lap's poses: one every second tick (${r.bestLen})`);
    t.equal(r.runLen, r.total + 1, `the run's inputs taped from GO to the line (${r.runLen} ticks)`);
    const shown = texts(run, "results.json");
    t.ok(shown.includes("New lap record"), "the results are titled for the new record");
    const lapText = (x: number) => {
      const ms = Math.round((x * 1000) / 60);
      return `${Math.floor(ms / 60000)}:${String(Math.floor((ms % 60000) / 1000)).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
    };
    for (const l of laps) t.ok(shown.includes(lapText(l)), `the results show lap ${lapText(l)}`);
    t.ok(shown.includes(lapText(r.total)) && shown.includes("Total time"), `the results' headline is the total, ${lapText(r.total)}, under "Total time"`);
    const calls = run.json<{ calls: Call[] }>("calls.json").calls;
    const submit = calls.find((c) => c.method === "leaderboard.submit");
    t.ok(submit !== undefined, "the best lap was submitted");
    t.equal(submit?.args.board, "laps", "on the laps board");
    t.equal(submit?.args.order, "asc", "lower first");
    t.equal(submit?.args.score, Math.round((r.record * 1000) / 60) / 1000, "in seconds, to the millisecond");
    const stored = calls.filter((c) => c.method === "store.set" && c.args.key === "record");
    t.ok(stored.length >= 1 && stored.every((c) => c.result?.ok === true), `the record stored at every new best (${stored.length} writes, each answered)`);
    const last = stored[stored.length - 1]?.args.value as { ticks: number; len: number; poses: string } | undefined;
    t.equal(last?.ticks, r.record, "the stored record is the fastest lap");
    t.equal(last?.poses.length, (last?.len ?? 0) * 15, "with its poses, fifteen characters each");
    const b = race(run, "board-race.json");
    t.ok(b.boardOpen && b.boardState === 2, "the leaderboard opened from the results and answered");
    t.ok(texts(run, "board.json").includes("You"), "the player's lap is on it");
    t.ok(texts(run, "board.json").includes(lapText(r.record)), "as its time");
  },
});
