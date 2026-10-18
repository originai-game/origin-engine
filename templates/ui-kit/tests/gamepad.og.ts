// A real gamepad (og.Gamepad through the `gamepad` core action, as the web
// shell delivers a pad's state): the d-pad pressed four times focuses the
// fourth row, A opens it, B closes it, and the left stick past half-way
// steps focus as the d-pad does (once per push, one intent per edge). No `pad:` game command and no mapped keys.
import { playtest } from "@origingame/engine/playtest";
import { SCREEN, demo } from "./lib.ts";

const press = (t: number, button: string): string => [`${t.toFixed(2)}:gamepad:${button}`, `${(t + 0.05).toFixed(2)}:gamepad:-`].join(";");

export default playtest({
  name: "a gamepad's d-pad, stick, A and B drive the gallery through og.Gamepad",
  commands: [
    press(0.5, "down"),
    press(0.6, "down"),
    press(0.7, "down"),
    press(0.8, "down"),
    press(0.9, "a"),
    "1.3:dump_entity:demo:opened.json",
    press(1.4, "b"),
    "1.9:dump_entity:demo:closed.json",
    // Back in the gallery nothing is focused: the stick down past half-way
    // twice, released between, steps to the first row, then the second.
    "2.0:gamepad:-:0,0.9",
    "2.05:gamepad:-:0,0",
    "2.1:gamepad:-:0,0.9",
    "2.15:gamepad:-:0,0",
    press(2.2, "a"),
    "2.6:dump_entity:demo:stick.json",
    "2.7:gamepad:off",
    "2.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(demo(run, "opened.json").screen, SCREEN.settings, "four d-pad downs and A opened the fourth row (settings)");
    t.equal(demo(run, "closed.json").screen, SCREEN.gallery, "B closed it");
    t.equal(demo(run, "stick.json").screen, SCREEN.mainMenu, "two stick pushes stepped to the second row and A opened it");
  },
});
