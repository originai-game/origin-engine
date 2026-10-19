// A gamepad alone: the d-pad and A play from the title; bar 1 and bar 2
// are hit with the d-pad (left, down, right) and the face buttons (X, A,
// B); Start pauses, the d-pad moves the menu's focus and A on Resume
// resumes.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, PLAY, songStart, sorted } from "./lib.ts";

const notes = chart();
/** A pad button pressed at t and let go 0.05 s later (the pad set aside, so the release is no input of its own). */
const pad = (button: string, t: number) => `${t.toFixed(4)}:gamepad:${button};${(t + 0.05).toFixed(4)}:gamepad:off`;
const t0 = songStart(1.1);
const DPAD = ["left", "down", "right"];
const FACE = ["x", "a", "b"];

export default playtest({
  name: "the d-pad and the face buttons hit the lanes; Start pauses, A resumes",
  commands: sorted([
    pad("down", 1.0),
    pad("a", 1.1),
    "1.4:get:Game:started.json",
    ...notes.filter((n) => n.beat < 8).map((n) => pad(n.beat < 4 ? DPAD[n.lane]! : FACE[n.lane]!, t0 + n.beat * BEAT)),
    `${(t0 + 7.9 * BEAT).toFixed(3)}:get:Game:hit.json`,
    pad("start", t0 + 8.2 * BEAT),
    `${(t0 + 8.8 * BEAT).toFixed(3)}:get:Game:paused.json`,
    pad("down", t0 + 9.0 * BEAT),
    pad("up", t0 + 9.2 * BEAT),
    pad("a", t0 + 9.4 * BEAT),
    `${(t0 + 10 * BEAT).toFixed(3)}:get:Game:resumed.json`,
    `${(t0 + 10.1 * BEAT).toFixed(3)}:gamepad:off`,
    `${(t0 + 10.2 * BEAT).toFixed(3)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "started.json").phase, PLAY, "the pad started the song from the title");
    const h = game(run, "hit.json");
    t.equal(h.hits, 8, "the d-pad hit bar 1's notes and the face buttons bar 2's");
    t.equal(h.miss, 0, "none missed");
    t.equal(game(run, "paused.json").paused, true, "Start paused");
    t.equal(game(run, "resumed.json").paused, false, "A on Resume resumed");
  },
});
