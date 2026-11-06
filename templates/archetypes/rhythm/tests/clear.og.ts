// The song's end and its results: from bar 21 (`play:21`, the earlier
// notes skipped) every note is hit on time, the last chord on all three
// lanes; a beat and a half later the results: cleared, grade S, 100 %
// accuracy, the score, the best kept at the revision read and submitted to
// the board, the play reported; Play again starts over from bar 1. In
// versus (seat 1 idle, out once its groove runs dry) the results have two
// columns and name P1 the winner, and no best is kept or submitted.
import { playtest } from "@origingame/engine/playtest";
import { chart, game, PLAY, type PlatformDump, play, RESULTS, rival, SEATS, songStart, sorted, tap, text } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.0, 21);
const last = notes[notes.length - 1]!.beat;
const end = t0 + last * 0.5;

export default playtest({
  name: "bar 21 to the end hit on time: cleared, grade S, the best kept and submitted, Play again",
  commands: sorted([
    "1.0:play:21",
    ...play(notes, t0, 80, last + 1),
    `${(end + 0.2).toFixed(3)}:get:Game:last.json`,
    `${(end + 2.4).toFixed(3)}:get:Game:results.json`,
    ...(SEATS > 1 ? [`${(end + 2.4).toFixed(3)}:get:Rival:rival.json`] : []),
    `${(end + 2.4).toFixed(3)}:dump_ui:results-ui.json`,
    `${(end + 2.4).toFixed(3)}:dump_platform:platform.json`,
    `${(end + 2.45).toFixed(3)}:screenshot:results.png`,
    tap("Again", +(end + 2.6).toFixed(3)),
    `${(end + 3.2).toFixed(3)}:get:Game:again.json`,
    `${(end + 3.3).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const expected = notes.filter((n) => n.beat >= 80).length;
    const l = game(run, "last.json");
    t.equal(l.notes, expected, `bar 21 on holds ${expected} notes`);
    t.equal(l.perfect, expected, "every one Perfect");
    t.equal(l.cleared, true, "the song is cleared");
    t.equal(l.phase, PLAY, "the results wait a beat and a half");
    const r = game(run, "results.json");
    t.equal(r.phase, RESULTS, "then the results");
    t.equal(text(run, "results-ui.json", "Letter"), "S", "grade S");
    const p = run.json<PlatformDump>("platform.json");
    if (SEATS > 1) {
      // Versus: seat 1 pressed nothing, so its groove ran out; seat 0 cleared and leads.
      const v = rival(run, "rival.json");
      t.equal(v.hits, 0, "seat 1 hit nothing");
      t.equal(v.failed, true, "seat 1's groove ran out: it is out, and the song played on for seat 0");
      t.equal(text(run, "results-ui.json", "Value", "Seats"), "P1  ·  P2", "two columns, P1 and P2");
      t.equal(text(run, "results-ui.json", "Value", "Score"), `${r.score}  ·  ${v.score}`, "each seat's score");
      t.equal(text(run, "results-ui.json", "Value", "Accuracy"), "100%  ·  0%", "each seat's accuracy");
      t.equal(text(run, "results-ui.json", "Value", "Miss"), `0  ·  ${v.miss}`, "each seat's misses");
      t.equal(text(run, "results-ui.json", "Winner"), "P1 wins", "the higher score is named: P1 wins");
      t.equal(r.newBest, false, "no new best in versus");
      t.equal(text(run, "results-ui.json", "Best"), undefined, "the results show no best");
      t.ok(!p.calls.some((c) => c.method === "store.set" && c.args["key"] === "best"), "no best was written");
      t.ok(!p.calls.some((c) => c.method === "leaderboard.submit"), "nothing was submitted to the board");
    } else {
      t.equal(text(run, "results-ui.json", "Value", "Accuracy"), "100%", "100 % accuracy");
      t.equal(text(run, "results-ui.json", "Value", "Score"), `${r.score}`, "the score");
      t.equal(r.newBest, true, "a new best");
      const best = p.calls.find((c) => c.method === "store.set" && c.args["key"] === "best");
      t.equal(best?.args["value"], r.score, "the best was written");
      t.equal(best?.result?.ok, true, "at the revision read");
      t.ok(p.calls.some((c) => c.method === "leaderboard.submit" && c.args["score"] === r.score), "and submitted to the board");
    }
    t.ok(p.calls.some((c) => c.method === "telemetry" && c.args["name"] === "song.over"), "the play was reported");
    const a = game(run, "again.json");
    t.ok(a.phase === PLAY && a.score === 0 && a.judged === 0 && a.notes === notes.length, "Play again started over from bar 1");
  },
});
