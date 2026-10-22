// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area; the HUD in it too, with health, the floor and
// Pause along the top and the draught, Wait and a d-pad of touch-sized
// buttons along the bottom; the d-pad steps and Wait waits.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, tap } from "./lib.ts";

interface Element { id: string; parent: string; kind: string; x: number; y: number; width: number; height: number }

export default playtest({
  name: "the phone layout: a HUD inside the safe area and a d-pad that steps",
  size: "1170x2532",
  commands: [
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("play", 1.0),
    "1.1:touch_move:10,400",
    "1.7:get:Game:started.json",
    "1.8:dump_ui:hud.json",
    tap("Pad/Up", 1.9),
    "2.0:touch_move:10,400",
    "2.3:get:Game:up.json",
    tap("Wait", 2.4),
    "2.5:touch_move:10,400",
    "2.8:get:Game:waited.json",
    "2.9:screenshot:play.png",
    "3.0:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "a tap on Dive started a dive");
    const ui = run.json<{ surface: { width: number; height: number }; elements: Element[] }>("hud.json");
    const el = (id: string) => ui.elements.find((e) => e.id === id);
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Health", "Floor", "Pause"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    for (const id of ["Drink", "Wait", "Down"]) {
      const e = el(id);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `${id} sits above the home bar`);
    }
    for (const id of ["Pause", "Drink", "Wait", "Up", "Down", "Left", "Right"]) {
      const e = el(id);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${id} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    t.ok(ui.elements.every((e) => e.x >= 0 && e.x + e.width <= 390), "nothing runs off the sides");
    const up = game(run, "up.json");
    t.equal(up.turns, started.turns + 1, "the d-pad's Up took a turn");
    t.equal(up.heroY, started.heroY - 1, "a step up");
    t.equal(game(run, "waited.json").turns, up.turns + 1, "Wait waited a turn");
    const title = run.json<{ elements: Element[] }>("title.json").elements;
    t.ok(title.some((e) => e.id === "play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
  },
});
