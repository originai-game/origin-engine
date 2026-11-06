// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area with the touch footer; the HUD in it too, the
// clock, the landmarks, the bag, the map and Pause along the top, the stick,
// Gather, Jump and Run along the bottom, all touch-sized. The stick walks
// the explorer, a drag on the world turns the view, Jump jumps, Run runs,
// and Gather held by a tree fells it.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, LOCAL, PLAY, position, tap, texts, value, VIEW } from "./lib.ts";

const ISO = VIEW === "isometric-2.5d";
// Two on one phone: seat 0's controls are the left column's, and it drags the view in the left half (alone: the right half).
const DRAG = LOCAL ? 40 : 230;
const LOCAL_PATH: Record<string, string> = { Stick: "TouchSeat0/Stick", Gather: "TouchSeat0/Hands/Gather", Jump: "TouchSeat0/Hands/Pair/Jump", Run: "TouchSeat0/Hands/Pair/Run" };
const at = (id: string) => (LOCAL ? LOCAL_PATH[id] ?? id : id);

export default playtest({
  name: "the phone layout: a HUD inside the safe area, a stick that walks, drag to look, Jump, Run and Gather",
  size: "1170x2532",
  commands: [
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("Play", 1.0),
    "1.1:touch_move:200,400",
    "1.7:get:Game:started.json",
    "1.7:get:hero.og.Transform:from.json",
    "1.8:dump_ui:hud.json",
    `1.9:touch_move_to:${at("Stick")}`,
    "1.92:touch_start",
    "2.0:touch_move:86,664",
    "2.5:get:Game:walking.json",
    "3.0:touch_end",
    "3.0:get:hero.og.Transform:to.json",
    "3.1:get:hero.SeatLook:look-a.json",
    `3.2:touch_move:${DRAG},330`,
    "3.22:touch_start",
    `3.3:touch_move:${DRAG + 50},330`,
    `3.4:touch_move:${DRAG + 100},330`,
    "3.5:touch_end",
    "3.6:get:hero.SeatLook:look-b.json",
    tap(at("Jump"), 3.7),
    "3.9:get:Game:jumped.json",
    `4.4:touch_move_to:${at("Stick")}`,
    "4.42:touch_start",
    "4.5:touch_move:86,664",
    `4.6:touch:1:down:${at("Run")}`,
    "5.3:get:Game:running.json",
    `5.4:touch:1:up:${at("Run")}`,
    "5.4:touch_end",
    "5.6:warp:tree",
    `5.8:touch_move_to:${at("Gather")}`,
    "5.82:touch_start",
    "7.3:touch_end",
    "7.4:get:Game:felled.json",
    "7.45:screenshot:play.png",
    "7.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json").filter((e) => e.path.startsWith("main-menu/"));
    t.ok(title.some((e) => e.id === "Play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.ok(texts(run, "title.json").includes("The stick moves · drag to look · Run, Jump · hold Gather by a tree or a rock"), "with the touch controls in its footer");
    t.equal(game(run, "started.json").phase, PLAY, "a tap on Set out started a run");
    const ui = run.json<{ surface: { width: number } }>("hud.json");
    const hud = elements(run, "hud.json");
    const el = (id: string) => hud.find((e) => e.path === id || e.path.endsWith(`/${id}`));
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Status/Clock", "Pause"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    const controls = ["Stick", "Gather", "Jump", "Run"].map(at);
    for (const id of controls) {
      const e = el(id);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `${id} sits above the home bar`);
    }
    for (const id of ["Pause", ...controls]) {
      const e = el(id);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${id} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    t.ok(hud.every((e) => e.x >= 0 && e.x + e.width <= 390), "nothing runs off the sides");
    t.ok(game(run, "walking.json").speed > 2, "the stick pushed up walks the explorer");
    const [x0, , z0] = position(run, "from.json");
    const [x1, , z1] = position(run, "to.json");
    t.ok(Math.hypot(x1 - x0, z1 - z0) > 1.5, `across the ground (${Math.hypot(x1 - x0, z1 - z0).toFixed(2)} m)`);
    if (!ISO) {
      const a = value<{ yaw: number }>(run, "look-a.json").yaw;
      const b = value<{ yaw: number }>(run, "look-b.json").yaw;
      t.ok(Math.abs(b - a) > 0.1, `a drag across the world turns the view (yaw ${a.toFixed(2)} → ${b.toFixed(2)})`);
    }
    const jumped = game(run, "jumped.json");
    t.ok(jumped.jumps === 1 && !jumped.grounded, "Jump jumped");
    t.ok(game(run, "running.json").speed > 5, `Run held with the stick runs (${game(run, "running.json").speed.toFixed(2)} m/s)`);
    const felled = game(run, "felled.json");
    t.ok(felled.felled === 1 && felled.wood === 2, "holding Gather by a tree felled it into the bag");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
  },
});
