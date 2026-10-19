// Determinism: a play with hits on time, late and early, misses and the
// band's stems, run twice in fresh processes, ends on the same world hash.
import { playtest } from "@origingame/engine/playtest";
import { chart, game, play, songStart, sorted } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.0);

export default playtest({
  name: "a play with hits, misses and the band replays to the same hash",
  runs: 2,
  commands: sorted([
    "1.0:play",
    ...play(notes, t0, 0, 4),
    ...play(notes, t0, 4, 8, 0.07),
    ...play(notes, t0, 8, 10, -0.1),
    `${(t0 + 13 * 0.5).toFixed(3)}:get:Game:end.json`,
    `${(t0 + 13.1 * 0.5).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const end = game(run, "end.json");
    t.ok(end.perfect >= 4 && end.great >= 4 && end.good >= 1 && end.miss >= 1, `perfects, greats, goods and misses (${end.perfect}, ${end.great}, ${end.good}, ${end.miss})`);
  },
});
