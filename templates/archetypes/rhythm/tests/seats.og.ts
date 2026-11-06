// The seats: the game reads og.PlayerInput alone (src/seats.ts).
//
// Alone, on a phone layout, bar 1 and bar 2 are hit through every device
// seat 0 takes: J K L (seat 1's keys in a two-seat game), the arrows, the
// pad's d-pad and fingers on the lanes (the game's own touch controls); a
// held key shows in the seat-0 entity's og.PlayerInput, and every press is
// still judged at its own time (all Perfect).
//
// In versus on one device (misses drain nothing here, so both seats stay
// in): bars 1 and 2 are seat 0's alone, A S D, pad 1 and fingers on the
// left half; bars 3 and 4 seat 1's alone, J K L, the arrows, pad 2 and
// fingers on the right half. Each seat hits only its own bars' notes: the
// other's devices never reach it. A held K is seat 1's bit, not seat 0's.
//
// Online, outside a room, this device drives seat 0 alone (seat 1 is the
// other peer's): every key set and pad, the left half's fingers; the right
// half's fingers press nothing (seat 1's highway), and seat 1 hits nothing.
import { playtest } from "@origingame/engine/playtest";
import { BEAT, chart, game, LOCAL, ONLINE, phoneLaneX, press, rival, songStart, sorted } from "./lib.ts";

const all = chart();
const notes = all.filter((n) => n.beat < 8);
const later = all.filter((n) => n.beat >= 8 && n.beat < 16);
const t0 = songStart(1.0);
const at = (beat: number) => (t0 + beat * BEAT).toFixed(3);
const finger = (lane: number, t: number, seat = 0) => `${t.toFixed(4)}:touch_move:${phoneLaneX(lane, seat).toFixed(1)},690;${(t + 0.0001).toFixed(4)}:touch_start;${(t + 0.05).toFixed(4)}:touch_end`;
/** A pad's button at t, let go 0.05 s later: pad 1, or pad 2 (`@1`). */
const pad = (button: string, t: number, second = false) => `${t.toFixed(4)}:gamepad:${button}${second ? "@1" : ""};${(t + 0.05).toFixed(4)}:gamepad:off${second ? "@1" : ""}`;
const LEFT = ["KeyA", "KeyS", "KeyD"];
const RIGHT = ["KeyJ", "KeyK", "KeyL"];
const ARROWS = ["ArrowLeft", "ArrowDown", "ArrowRight"];
const DPAD = ["left", "down", "right"];
const time = (n: { beat: number }) => t0 + n.beat * BEAT;
/** Alone: two notes by J K L, two by the arrows, two by the pad, two by fingers. */
const alone = (i: number, lane: number, t: number) => (i < 2 ? press(RIGHT[lane]!, t) : i < 4 ? press(ARROWS[lane]!, t) : i < 6 ? pad(DPAD[lane]!, t) : finger(lane, t));
/** Seat 0 in versus: A S D, pad 1, the left half's fingers (online also J K L and the arrows, this device's keys all seat 0's). */
const seat0 = (i: number, lane: number, t: number) =>
  ONLINE && i >= 6 ? press((i === 6 ? RIGHT : ARROWS)[lane]!, t) : i < 3 ? press(LEFT[lane]!, t) : i < 5 ? pad(DPAD[lane]!, t) : finger(lane, t, 0);
/** Seat 1 on this device: J K L, the arrows, pad 2, the right half's fingers, a quarter of its notes each. */
const seat1 = (i: number, count: number, lane: number, t: number) => {
  const q = Math.floor((4 * i) / count);
  return q === 0 ? press(RIGHT[lane]!, t) : q === 1 ? press(ARROWS[lane]!, t) : q === 2 ? pad(DPAD[lane]!, t, true) : finger(lane, t, 1);
};

interface Dump { components: { "og.PlayerInput": { player: number; buttons: number } } }

const single = sorted([
  "0.1:viewport:phone",
  "0.5:key_down:KeyK",
  "0.55:dump_entity:seat-0:held.json",
  "0.6:key_up:KeyK",
  "1.0:play",
  ...notes.map((n, i) => alone(i, n.lane, time(n))),
  `${at(7.9)}:get:Game:hit.json`,
  `${at(8)}:quit`,
]);

const versus = sorted([
  "0.1:viewport:phone",
  "0.2:set:Tuning.grooveLoss=0",
  "0.5:key_down:KeyK",
  "0.55:dump_entity:seat-0:held0.json",
  "0.55:dump_entity:seat-1:held1.json",
  "0.6:key_up:KeyK",
  "1.0:play",
  ...notes.map((n, i) => seat0(i, n.lane, time(n))),
  `${at(7.9)}:get:Game:hit.json`,
  `${at(7.9)}:get:Rival:rival.json`,
  ...(ONLINE ? later.map((n) => finger(n.lane, time(n), 1)) : later.map((n, i) => seat1(i, later.length, n.lane, time(n)))),
  `${at(15.9)}:get:Game:hit2.json`,
  `${at(15.9)}:get:Rival:rival2.json`,
  `${at(16)}:quit`,
]);

export default playtest({
  name: "each seat's og.PlayerInput carries its own keys, pad and fingers to its own lanes",
  size: "1170x2532",
  commands: LOCAL || ONLINE ? versus : single,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(notes.length, 8, "bars 1 and 2 hold eight notes");
    if (!LOCAL && !ONLINE) {
      const held = run.json<Dump>("held.json").components["og.PlayerInput"];
      t.equal(held.player, 0, "the seat-0 entity carries seat 0's input");
      t.equal(held.buttons, 2, "K held is the middle lane's bit (lane1) in og.PlayerInput");
      const h = game(run, "hit.json");
      t.equal(h.hits, 8, "J K L, the arrows, the pad and fingers hit all eight");
      t.equal(h.perfect, 8, "each judged at its own time: all Perfect");
      t.equal(h.miss, 0, "none missed");
      return;
    }
    t.ok(later.length >= 4, `bars 3 and 4 hold ${later.length} notes`);
    const held0 = run.json<Dump>("held0.json").components["og.PlayerInput"];
    const held1 = run.json<Dump>("held1.json").components["og.PlayerInput"];
    t.equal(held0.player, 0, "the seat-0 entity carries seat 0's input");
    t.equal(held1.player, 1, "the seat-1 entity carries seat 1's input");
    if (LOCAL) {
      t.equal(held1.buttons, 2, "K held is seat 1's middle lane bit");
      t.equal(held0.buttons, 0, "and nothing of seat 0's");
    } else {
      t.equal(held0.buttons, 2, "online, this device's K is seat 0's middle lane bit");
      t.equal(held1.buttons, 0, "and seat 1's input is the other peer's: empty");
    }
    const h = game(run, "hit.json");
    const r = rival(run, "rival.json");
    t.equal(h.hits, 8, ONLINE ? "seat 0's keys, pad and left-half fingers hit bars 1 and 2" : "A S D, pad 1 and the left half's fingers hit seat 0's bars 1 and 2");
    t.equal(h.perfect, 8, "each judged at its own time: all Perfect");
    t.equal(r.hits, 0, "none of it reached seat 1");
    t.equal(r.miss, 8, "seat 1 let bars 1 and 2 pass");
    const h2 = game(run, "hit2.json");
    const r2 = rival(run, "rival2.json");
    t.equal(h2.hits, 8, ONLINE ? "the right half's fingers pressed nothing of seat 0's" : "J K L, the arrows, pad 2 and the right half's fingers did not reach seat 0");
    t.equal(h2.miss, later.length, "seat 0 let bars 3 and 4 pass");
    if (LOCAL) {
      t.equal(r2.hits, later.length, "J K L, the arrows, pad 2 and the right half's fingers hit seat 1's bars 3 and 4");
      t.equal(r2.perfect, later.length, "all Perfect");
    } else {
      t.equal(r2.hits, 0, "online, nothing on this device drives seat 1");
    }
  },
});
