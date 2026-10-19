// Three frames at 1280×720 compared with references/<tier>/: the title,
// the highway in bar 9 (the lead's notes on three lanes, a Perfect, the
// combo) and the results of a song cleared from bar 23. `OG_FX_CAPTURE=1
// og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { BEAT, chart, play, songStart, sorted } from "./lib.ts";

const FILES = ["title.png", "highway.png", "results.png"];
const notes = chart();
const t0 = songStart(1.2, 7);
const t1 = songStart(9.0, 23);
const last = notes[notes.length - 1]!.beat;

export default playtest({
  name: "the title, the highway and the results match their reference frames",
  size: "1280x720",
  commands: sorted([
    "1.0:screenshot:title.png",
    "1.2:play:7",
    ...play(notes, t0, 24, 33.5),
    `${(t0 + 33.6 * BEAT).toFixed(3)}:screenshot:highway.png`,
    "9.0:play:23",
    ...play(notes, t1, 88, last + 1),
    `${(t1 + last * BEAT + 2.4).toFixed(3)}:screenshot:results.png`,
    `${(t1 + last * BEAT + 2.5).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    checkFrames(run, t, FILES, "");
  },
});
