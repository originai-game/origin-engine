// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area with the touch footer; a tap on Race starts
// the countdown; the HUD sits inside the safe area too, every touch
// control at least 44 px square, nothing off the sides; and after GO the
// touch Throttle with Right drives the car and turns it.
import { playtest } from "@origingame/engine/playtest";
import { car, COUNTDOWN_S, elements, finger, game, LOCAL, RACE, sorted, tap, texts } from "./lib.ts";

const TOP = 47;
const BOTTOM = 844 - 34;
const path = (id: string) => (LOCAL ? `Touch0/${id}` : id.startsWith("Steer") ? `Steering/${id}` : `Pedals/${id}`);
const GO = 1.1 + COUNTDOWN_S;

export default playtest({
  name: "the phone layout: a HUD inside the safe area, touch controls that drive",
  size: "1170x2532",
  commands: sorted([
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("Play", 1.0),
    "1.6:get:Game:started.json",
    "2.0:dump_ui:hud.json",
    finger(path("Pedals/Right/Throttle"), GO + 0.1, 1.4, 1),
    finger(path("Steer/Right"), GO + 1.0, 0.4, 2),
    `${(GO + 1.4).toFixed(2)}:get:car-0.Car:driving.json`,
    `${(GO + 1.45).toFixed(2)}:screenshot:race.png`,
    `${(GO + 1.5).toFixed(2)}:quit`,
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json").filter((e) => e.path.startsWith("main-menu/"));
    t.ok(title.some((e) => e.id === "Play") && title.every((e) => e.y >= TOP && e.y + e.height <= BOTTOM), "the title's menu sits inside the safe area");
    t.ok(texts(run, "title.json").some((s) => s.startsWith("Left and Right steer")), "with the touch controls in its footer");
    const g = game(run, "started.json");
    t.ok(g.phase === RACE && g.clock < COUNTDOWN_S * 60, "a tap on Race started the countdown");
    const ui = run.json<{ surface: { width: number } }>("hud.json");
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    const hud = elements(run, "hud.json").filter((e) => e.kind !== "screen" && e.width > 0);
    const controls = hud.filter((e) => e.kind === "action");
    t.ok(controls.length >= (LOCAL ? 11 : 6), `the touch controls and Pause show (${controls.length})`);
    for (const e of controls) {
      t.ok(e.y >= TOP && e.y + e.height <= BOTTOM, `${e.path} sits inside the safe area`);
      t.ok(e.width >= 44 && e.height >= 44, `${e.path} is at least 44 px square (${e.width}×${e.height})`);
    }
    t.ok(hud.every((e) => e.x >= -0.5 && e.x + e.width <= 390.5), "nothing runs off the sides");
    const c = car(run, "driving.json");
    t.ok(c.speed > 8 && c.steer > 0.3, `the touch Throttle drives the car and Right turns it (${c.speed.toFixed(1)} m/s, steer ${c.steer.toFixed(2)})`);
    t.frame(["race.png"], () => t.equal(run.png("race.png").width, 1170, "race.png captured"));
  },
});
