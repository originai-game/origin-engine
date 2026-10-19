// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area with the touch footer; the HUD in it too, the
// hearts, the beacons, the clock and Pause along the top and the joystick,
// Kindle and Jump along the bottom, all touch-sized. The joystick walks
// the hero, Jump jumps, and holding Kindle by a beacon lights it.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, PLAY, position, tap, texts } from "./lib.ts";

export default playtest({
  name: "the phone layout: a HUD inside the safe area, a joystick that walks, Jump and Kindle",
  size: "1170x2532",
  commands: [
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("play", 1.0),
    "1.1:touch_move:200,400",
    "1.7:get:Game:started.json",
    "1.7:get:hero.og.Transform:from.json",
    "1.8:dump_ui:hud.json",
    "1.9:touch_move_to:Stick",
    "1.92:touch_start",
    "2.0:touch_move:86,664",
    "2.5:get:Game:walking.json",
    "3.0:touch_end",
    "3.0:get:hero.og.Transform:to.json",
    tap("Jump", 3.3),
    "3.5:get:Game:jumped.json",
    "4.4:warp:0",
    "4.6:touch_move_to:Kindle",
    "4.62:touch_start",
    "6.6:touch_end",
    "6.7:get:Game:lit.json",
    "6.75:screenshot:play.png",
    "6.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json");
    t.ok(title.some((e) => e.id === "play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.ok(texts(run, "title.json").includes("The stick moves · Jump jumps · hold Kindle by a beacon · the view swings behind you"), "with the touch controls in its footer");
    t.equal(game(run, "started.json").phase, PLAY, "a tap on Set out started a run");
    const ui = run.json<{ surface: { width: number } }>("hud.json");
    const hud = elements(run, "hud.json");
    const el = (id: string) => hud.find((e) => e.id === id);
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Hearts", "Beacons", "Clock", "Pause"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    for (const id of ["Stick", "Kindle", "Jump"]) {
      const e = el(id);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `${id} sits above the home bar`);
    }
    for (const id of ["Pause", "Stick", "Kindle", "Jump"]) {
      const e = el(id);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${id} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    t.ok(hud.every((e) => e.x >= 0 && e.x + e.width <= 390), "nothing runs off the sides");
    t.ok(game(run, "walking.json").speed > 2, "the joystick pushed up walks the hero");
    const [, , z0] = position(run, "from.json");
    const [, , z1] = position(run, "to.json");
    t.ok(z0 - z1 > 1.5, `north, away from the camera (${(z0 - z1).toFixed(2)} m)`);
    const jumped = game(run, "jumped.json");
    t.ok(jumped.jumps === 1 && !jumped.grounded, "Jump jumped");
    t.equal(game(run, "lit.json").lit, 1, "holding Kindle by a beacon lit it");
    t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels");
  },
});
