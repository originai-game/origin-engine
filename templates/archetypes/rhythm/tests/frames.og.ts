// Smoke: two representative frames at the existing tolerance.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { BEAT, chart, play, songStart, sorted } from "./lib.ts";

const FILES = ["title.png", "highway.png"];
const notes = chart();
const t0 = songStart(1.2, 7);
const t1 = songStart(9.0, 23);
const last = notes[notes.length - 1]!.beat;

export default playtest({
  name: "smoke: boots, plays and matches two representative frames",
  size: "1280x720",
  commands: sorted([
    "1.0:screenshot:title.png",
    "1.2:play:7",
    ...play(notes, t0, 24, 33.5),
    `${(t0 + 33.6 * BEAT).toFixed(3)}:screenshot:highway.png`,
    "9.0:play:23",
    ...play(notes, t1, 88, last + 1),
    `${(t1 + last * BEAT + 2.5).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
