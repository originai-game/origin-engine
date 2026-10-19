// The core loop through real input: Play on the title starts the count-in
// (four clicks, 4 3 2 1), then bar 1. The first two bars' eight notes are
// hit with A S D on their times: eight Perfects, a combo of 8, 2,400
// points, the groove rising. Bar 3's first note is hit 80 ms late (Great,
// late), its second 120 ms early (Good, early), its third and fourth let
// pass (two Misses, the combo broken); a press with no note near is only a
// sound.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, KEYS, PLAY, play, press, songStart, sorted, tap, TITLE } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.05);
const bar3 = notes.filter((n) => n.beat >= 8 && n.beat < 12);

export default playtest({
  name: "Play, the count-in, perfect hits, a late Great, an early Good, misses and an empty press",
  commands: sorted([
    "0.3:get:Game:title.json",
    tap("play", 1.0),
    `${(t0 - 1.0).toFixed(3)}:get:Game:count.json`,
    ...play(notes, t0, 0, 8),
    `${(t0 + 7.8 * BEAT).toFixed(3)}:get:Game:perfect.json`,
    `${(t0 + 7.8 * BEAT).toFixed(3)}:screenshot:play.png`,
    press(KEYS[bar3[0]!.lane]!, t0 + bar3[0]!.beat * BEAT + 0.08),
    press(KEYS[bar3[1]!.lane]!, t0 + bar3[1]!.beat * BEAT - 0.12),
    `${(t0 + 9.6 * BEAT).toFixed(3)}:get:Game:early.json`,
    press("KeyS", t0 + 10.5 * BEAT),
    `${(t0 + 11.6 * BEAT).toFixed(3)}:get:Game:missed.json`,
    `${(t0 + 11.7 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const count = game(run, "count.json");
    t.equal(count.phase, PLAY, "Play started the count-in");
    t.ok(Math.abs(count.songStart - t0) < 1e-6, `bar 1 starts at ${t0.toFixed(3)} s, four beats after the next tick`);
    const p = game(run, "perfect.json");
    t.equal(p.perfect, 8, "the first two bars' eight notes were Perfect");
    t.equal(p.combo, 8, "a combo of 8");
    t.equal(p.score, 8 * 300, "300 each at ×1");
    t.ok(p.groove > 0.5, `the groove rose (${p.groove.toFixed(2)})`);
    const e = game(run, "early.json");
    t.equal(e.great, 1, "80 ms late was Great");
    t.equal(e.good, 1, "120 ms early was Good");
    t.equal(e.combo, 10, "the combo went on: 10");
    const m = game(run, "missed.json");
    t.equal(m.miss, 2, "the two notes let pass were Misses");
    t.equal(m.combo, 0, "the combo broke");
    t.equal(m.judged, 12, "twelve notes judged; the empty press judged nothing");
    t.equal(m.taps, 11, "eleven presses heard");
    t.equal(run.png("play.png").width, 1280, "play.png captured");
  },
});
