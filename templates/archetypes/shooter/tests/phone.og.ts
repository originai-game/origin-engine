// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area with the touch footer; the HUD in it too, the
// wave and score along the top, Pause in its corner, and the stick, Fire,
// Jump and Reload along the bottom, all touch-sized. The stick walks the
// trooper, a finger dragged on the open screen turns the view, holding
// Fire fires, Jump jumps and Reload reloads; the frame is compared with
// references/<tier>/<view>-phone-play.png.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { elements, game, LOCAL, PLAY, position, tap, texts, value, VARIANT, type Hero } from "./lib.ts";

/** Seat 0's touch buttons: in their own cluster alone, over the stick with two seats. */
const HANDS = LOCAL ? "TouchMove0/Hands" : "TouchHands0";
/** A point on the open screen in seat 0's part (the right side alone, the left half with two). */
const LOOK = LOCAL ? [110, 300] : [290, 340];

export default playtest({
  name: "the phone layout: a HUD inside the safe area, a stick that walks, drag to look, Fire, Jump and Reload",
  size: "1170x2532",
  commands: [
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("Play", 1.0),
    "1.7:get:Game:started.json",
    "1.7:warp:2:9",
    "1.8:get:hero.og.Transform:from.json",
    "1.8:dump_ui:hud.json",
    "1.9:touch:1:down:TouchMove0/Stick",
    "1.95:touch:1:move:86,560",
    "2.6:touch:1:up:86,560",
    "2.7:get:hero.og.Transform:to.json",
    "2.7:get:hero.SeatLook:look-a.json",
    `2.8:touch:2:down:${LOOK[0]},${LOOK[1]}`,
    `2.85:touch:2:move:${LOOK[0]! - 40},${LOOK[1]}`,
    `2.9:touch:2:move:${LOOK[0]! - 80},${LOOK[1]}`,
    `3.0:touch:2:up:${LOOK[0]! - 80},${LOOK[1]}`,
    "3.1:get:hero.SeatLook:look-b.json",
    `3.2:touch:3:down:${HANDS}/Fire`,
    `3.7:touch:3:up:${HANDS}/Fire`,
    "3.8:get:hero.Hero:fired.json",
    tap(`${HANDS}/Small/Jump`, 3.9),
    "4.05:get:hero.Hero:jumped.json",
    tap(`${HANDS}/Small/Reload`, 4.6),
    "4.75:get:hero.Hero:reloading.json",
    "4.75:dump_ui:reload-ui.json",
    "4.8:screenshot:play.png",
    "4.9:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json").filter((e) => e.path.startsWith("main-menu/"));
    t.ok(title.some((e) => e.id === "Play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.ok(texts(run, "title.json").includes("The stick moves · drag to aim · hold Fire · Reload · Jump"), "with the touch controls in its footer");
    t.equal(game(run, "started.json").phase, PLAY, "a tap on Hold the line started a run");
    const ui = run.json<{ surface: { width: number } }>("hud.json");
    const hud = elements(run, "hud.json");
    const el = (path: string) => hud.find((e) => e.path === path || e.id === path);
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Pause", "Run"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    for (const path of ["TouchMove0/Stick", `${HANDS}/Fire`, `${HANDS}/Small/Jump`, `${HANDS}/Small/Reload`]) {
      const e = el(path);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `${path} sits above the home bar`);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${path} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    t.ok(hud.every((e) => e.x >= 0 && e.x + e.width <= 390), "nothing runs off the sides");
    const over = (a: { x: number; y: number; width: number; height: number }, b: typeof a) => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
    const centre = { x: 195 - 40, y: 422 - 40, width: 80, height: 80 };
    for (const id of ["Ammo", "Health"]) {
      const e = el(id);
      t.ok(e !== undefined && !over(e, centre), `${id} keeps clear of the crosshair`);
      t.ok(e !== undefined && !over(e, el("Run")!) && !over(e, el("Pause")!), `${id} keeps clear of the run's line and Pause`);
    }
    const vitals = hud.filter((e) => e.id === "Health" || e.id === "Ammo");
    t.ok(vitals.every((a, i) => vitals.every((b, j) => i === j || !over(a, b))), `the seats' health and rounds never overlap (${vitals.length} panels)`);
    const [x0, , z0] = position(run, "from.json");
    const [x1, , z1] = position(run, "to.json");
    t.ok(Math.hypot(x1 - x0, z1 - z0) > 1.5, `the stick pushed up walks the trooper (${Math.hypot(x1 - x0, z1 - z0).toFixed(2)} m)`);
    const la = value<{ yaw: number }>(run, "look-a.json").yaw;
    const lb = value<{ yaw: number }>(run, "look-b.json").yaw;
    t.ok(lb - la > 0.2, `a finger dragged left turns the view left (yaw ${la.toFixed(2)} → ${lb.toFixed(2)})`);
    const fired = value<Hero>(run, "fired.json");
    t.ok(fired.mag < 24 && fired.mag >= 18, `holding Fire fires (${24 - fired.mag} rounds)`);
    t.ok(value<Hero>(run, "jumped.json").jumps === 1, "Jump jumps");
    t.ok(value<Hero>(run, "reloading.json").reload > 0, "Reload reloads");
    // Reloading, seat 0's rounds still say so and keep the weapon's line, inside its own part of the screen.
    const reload = elements(run, "reload-ui.json");
    const word = reload.find((e) => e.path.startsWith("Vitals") && e.id === "ReloadLabel");
    t.ok(word !== undefined && word.text !== "", `the rounds panel says it is reloading (${word?.text})`);
    t.ok(reload.some((e) => e.path.startsWith("Vitals") && e.id === "Name" && e.text !== ""), "and keeps the weapon's name");
    const ammo0 = reload.find((e) => e.path === (LOCAL ? "Vitals0/Ammo" : "Vitals/Ammo"));
    const seam = reload.find((e) => e.path === "Divider/Line");
    t.ok(ammo0 !== undefined && (!LOCAL || (seam !== undefined && ammo0.x + ammo0.width <= seam.x)), `which stays in seat 0's half (ends at ${ammo0 ? Math.round(ammo0.x + ammo0.width) : "?"}, the seam at ${seam ? Math.round(seam.x) : "none"})`);
    t.ok(LOCAL === (seam !== undefined), LOCAL ? "a dark seam divides the halves" : "alone, no seam");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
    // The phone frame: first person alone and split (references/<tier>/fp-phone-play.png, fp-local-phone-play.png).
    checkFrames(run, t, ["play.png"], `${VARIANT}-phone-`, VARIANT === "fp");
  },
});
