// The song on og.Music's clock: the count-in's four clicks are started at
// the four beats before bar 1, the drums on bar 1's line, and every stem
// after that on its own bar line from og.MusicBeat (the bass and the chords
// at bar 5, the lead at bar 9), each to the sample (AudioSource.startAt
// is the bar's exact world time); the notes come onto the highway beats
// ahead of their time.
import { playtest } from "@origingame/engine/playtest";
import { game, songStart, sorted } from "./lib.ts";

const t0 = songStart(1.0);
const bar = (n: number) => t0 + (n - 1) * 2;

interface Src { components: { "og.AudioSource": { startAt: number; playing: boolean }; Stem: { starts: number } } }

export default playtest({
  name: "the clicks, the drums and every stem start on their beats and bar lines",
  commands: sorted([
    // Nothing is pressed here, so the groove must not run out.
    "0.9:set:Tuning.grooveLoss=0",
    "1.0:play",
    "1.15:dump_entity:stem-click0:click0.json",
    "1.15:dump_entity:stem-click3:click3.json",
    "1.15:dump_entity:stem-drums:drums1.json",
    `${(bar(5) + 0.1).toFixed(3)}:dump_entity:stem-bass0:bass.json`,
    `${(bar(5) + 0.1).toFixed(3)}:dump_entity:stem-chords0:chords.json`,
    `${(bar(5) + 0.1).toFixed(3)}:dump_entity:stem-drums:drums5.json`,
    `${(bar(9) + 0.1).toFixed(3)}:dump_entity:stem-leadA:lead.json`,
    `${(bar(9) + 0.1).toFixed(3)}:get:Game:bar9.json`,
    `${(bar(9) + 0.2).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const src = (f: string) => run.json<Src>(f).components["og.AudioSource"];
    const near = (a: number, b: number) => Math.abs(a - b) < 1e-6;
    t.ok(near(src("click0.json").startAt, t0 - 2), `the first click at ${(t0 - 2).toFixed(3)} s, four beats before bar 1`);
    t.ok(near(src("click3.json").startAt, t0 - 0.5), "the fourth a beat before it");
    t.ok(near(src("drums1.json").startAt, t0), `the drums on bar 1's line (${t0.toFixed(3)} s)`);
    t.ok(near(src("bass.json").startAt, bar(5)) && src("bass.json").playing, `the bass on bar 5's line (${bar(5).toFixed(3)} s)`);
    t.ok(near(src("chords.json").startAt, bar(5)), "the chords with it");
    t.ok(near(src("drums5.json").startAt, bar(5)), "the drums restarted on bar 5's line too");
    t.equal(run.json<Src>("drums5.json").components.Stem.starts, 5, "five bars, five starts of the one-bar drums");
    t.ok(near(src("lead.json").startAt, bar(9)), `the lead on bar 9's line (${bar(9).toFixed(3)} s)`);
    t.equal(game(run, "bar9.json").miss, game(run, "bar9.json").judged, "(nothing pressed: every note judged so far a Miss)");
  },
});
