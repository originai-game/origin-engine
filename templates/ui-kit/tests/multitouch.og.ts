// Two fingers at once (og.Pointers): the primary touch holds the stick to
// the right while a second touch taps Hit; the stick keeps its deflection
// through the tap and Hit answers once.
import { playtest } from "@origingame/engine/playtest";
import { demo } from "./lib.ts";

export default playtest({
  name: "a stick held by one touch and a button tapped by another work together",
  commands: [
    "0.0:show:widgets",
    "0.5:touch_move_to:Stick",
    "0.55:touch_start",
    "0.6:touch_move:1030,637",
    "0.7:dump_entity:demo:held.json",
    "0.75:touch:1:down:Hit",
    "0.8:dump_entity:demo:both.json",
    "0.85:touch:1:up:Hit",
    "0.9:dump_entity:demo:after.json",
    "0.95:touch_end",
    "1.1:dump_entity:demo:released.json",
    "1.2:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const held = demo(run, "held.json");
    const both = demo(run, "both.json");
    const after = demo(run, "after.json");
    t.ok(held.stickX > 0.5, `the first touch pushes the stick right (x ${held.stickX})`);
    t.ok(both.stickX > 0.5, `the stick keeps its deflection while the second touch is down (x ${both.stickX})`);
    t.equal(both.hp, held.hp, "Hit has not fired while the second touch is still down");
    t.equal(after.hp, held.hp - 15, "the second touch lifting over Hit hits once");
    t.ok(after.stickX > 0.5, `the stick is still held after the tap (x ${after.stickX})`);
    t.equal(demo(run, "released.json").stickX, 0, "the stick centres when the first touch lifts");
  },
});
