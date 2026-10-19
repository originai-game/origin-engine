// The failure path: nothing is pressed; every note let past drains the
// groove a tenth; the fifth Miss empties it from its half: the song fails
// (a falling tone, the band stops) and the results say so, without a
// grade or a best.
import { playtest } from "@origingame/engine/playtest";
import { chart, game, PLAY, RESULTS, songStart, sorted, text } from "./lib.ts";

const notes = chart();
const t0 = songStart(1.0);
const fifth = notes[4]!.beat;

export default playtest({
  name: "five notes let pass empty the groove: the song fails and the results say so",
  commands: sorted([
    "1.0:play",
    `${(t0 + notes[2]!.beat * 0.5 + 0.2).toFixed(3)}:get:Game:three.json`,
    `${(t0 + fifth * 0.5 + 0.2).toFixed(3)}:get:Game:failed.json`,
    `${(t0 + fifth * 0.5 + 0.25).toFixed(3)}:dump_entity:stem-drums:drums.json`,
    `${(t0 + fifth * 0.5 + 2.2).toFixed(3)}:get:Game:results.json`,
    `${(t0 + fifth * 0.5 + 2.2).toFixed(3)}:dump_ui:results-ui.json`,
    `${(t0 + fifth * 0.5 + 2.25).toFixed(3)}:screenshot:failed.png`,
    `${(t0 + fifth * 0.5 + 2.3).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const three = game(run, "three.json");
    t.equal(three.miss, 3, "three notes passed: three Misses");
    t.ok(Math.abs(three.groove - 0.2) < 1e-4, `the groove fell from 0.5 to 0.2 (${three.groove.toFixed(3)})`);
    t.equal(three.phase, PLAY, "still playing");
    const f = game(run, "failed.json");
    t.equal(f.failed, true, "the fifth Miss emptied the groove: failed");
    t.equal(f.groove, 0, "the groove is empty");
    t.equal(run.json<{ components: { "og.AudioSource": { playing: boolean } } }>("drums.json").components["og.AudioSource"].playing, false, "the band stopped");
    const r = game(run, "results.json");
    t.equal(r.phase, RESULTS, "the results came up");
    t.equal(text(run, "results-ui.json", "Title", "Panel"), "The groove slipped away", "they say the groove slipped away");
    t.equal(text(run, "results-ui.json", "Letter"), "–", "with no grade");
    t.equal(r.newBest, false, "and no best");
  },
});
