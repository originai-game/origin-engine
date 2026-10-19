// A press is judged when it happened, not at the start of the tick that
// reads it: bars 1 and 2's notes pressed 44 ms late and early (a tick's
// start would read them 50 ms off: Great), 7 ms either way, 89 ms either
// way (Great, inside 90), 20 ms late and 30 ms early. Each judgement's
// error is the press's offset to a tenth of a millisecond: the key's
// og.KeyDown carries its time within the tick (the command step's here,
// the browser event's timeStamp in a page).
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, KEYS, press, songStart, sorted, tap } from "./lib.ts";

const notes = chart().filter((n) => n.beat < 8);
const t0 = songStart(1.05);
const offsets = [44, -44, 7, -7, 89, -89, 20, -30];
/** Perfect within 45 ms, Great within 90 (Judgement.* of src/components.ts: Perfect 0, Great 1). */
const expected = offsets.map((ms) => (Math.abs(ms) <= 45 ? 0 : 1));

export default playtest({
  name: "a press is judged at the millisecond it happened within its tick",
  commands: sorted([
    tap("play", 1.0),
    ...notes.flatMap((n, i) => {
      const at = t0 + n.beat * BEAT;
      return [press(KEYS[n.lane]!, at + offsets[i]! / 1000), `${(at + 0.2).toFixed(4)}:get:Game:note-${i}.json`];
    }),
    `${(t0 + 8 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(notes.length, 8, "bars 1 and 2 hold eight notes");
    for (let i = 0; i < notes.length; i++) {
      const g = game(run, `note-${i}.json`) as unknown as { lastOffsetMs: number; lastJudgement: number; judged: number };
      t.equal(g.judged, i + 1, `note ${i} was judged`);
      t.near(g.lastOffsetMs, offsets[i]!, 0.1, `note ${i}'s error is the press's ${offsets[i]} ms`);
      t.equal(g.lastJudgement, expected[i], `note ${i} at ${offsets[i]} ms is ${expected[i] === 0 ? "Perfect" : "Great"}`);
    }
  },
});
