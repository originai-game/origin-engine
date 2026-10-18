// The platformer HUD on a phone in portrait (390×844 at 3×, a 47 px notch, a
// 34 px home indicator): the corners sit inside the safe area, every touch
// control is at least 44 CSS px, and Jump answers a touch.
import { playtest } from "@origingame/engine/playtest";
import { demo, tap, ui } from "./lib.ts";

export default playtest({
  name: "on a phone the HUD keeps to the safe area and its touch controls to 44 px",
  size: "1170x2532",
  commands: ["0.0:viewport:phone", "0.1:show:platformer", "0.6:dump_ui:hud.json", "0.6:dump_entity:demo:before.json", tap(0.7, "jump"), "1.0:dump_entity:demo:after.json", "1.1:quit"].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const els = ui(run, "hud.json");
    const find = (id: string) => els.find((e) => e.id === id);
    for (const id of ["Up", "Down", "Left", "Right", "jump", "Pause"]) {
      const e = find(id);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${id} is at least 44×44 (${e ? `${e.width}×${e.height}` : "missing"})`);
    }
    const top = find("Hud.left");
    t.ok(top !== undefined && top.y >= 47, `the top-left group is below the notch (y ${top?.y})`);
    const pad = find("TouchActions");
    t.ok(pad !== undefined && pad.y + pad.height <= 844 - 34, `the action buttons end above the home indicator (${pad ? pad.y + pad.height : "missing"})`);
    t.equal(demo(run, "after.json").coins - demo(run, "before.json").coins, 1, "a tap on Jump answered the jump action once");
  },
});
