// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu, then the wallet and Pause below the notch, the four generators
// along the bottom above the home bar with touch-sized buttons, the orb
// above them taking a finger's tap, and a card's button buying.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, tap, tapAt, type UiElement, ui } from "./lib.ts";

/** The orb's centre on the phone (src/ui.ts: 7.4 m across, the orb at y 1.4 m). */
const ORB = { x: 195, y: 422 - 1.4 * (390 / 7.4) };

export default playtest({
  name: "the phone layout: a HUD inside the safe area, the orb tapped, a card bought",
  size: "1170x2532",
  commands: [
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("play", 1.0),
    "1.1:touch_move:10,400",
    "1.7:get:Game:started.json",
    tapAt(ORB.x, ORB.y, 1.8),
    tapAt(ORB.x + 20, ORB.y - 10, 1.9),
    "2.1:get:Game:tapped.json",
    "2.2:sparks:100",
    tap("Shop/Gen#0/Buy", 2.3),
    "2.4:touch_move:10,400",
    "2.6:get:Game:bought.json",
    "2.7:dump_ui:hud.json",
    "2.75:screenshot:play.png",
    "2.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = ui(run, "title.json").elements;
    t.ok(title.some((e) => e.id === "play") && title.every((e) => e.y >= 47 && e.y + e.height <= 844 - 34), "the title's menu sits inside the safe area");
    t.equal(game(run, "started.json").phase, PLAY, "a tap on Play showed the orb");
    t.equal(game(run, "tapped.json").taps, 2, "two taps on the orb");
    t.equal(game(run, "bought.json").owned0, 1, "the Ember card's button bought one");
    const hud = ui(run, "hud.json");
    const el = (id: string, parent?: string): UiElement | undefined => hud.elements.find((e) => e.id === id && (parent === undefined || e.parent === parent));
    t.equal(hud.surface.width, 390, "laid out for a 390 px wide phone");
    for (const id of ["Wallet", "Corner"]) t.ok((el(id)?.y ?? 0) >= 47, `${id} sits below the notch`);
    const shop = el("Shop");
    t.ok(shop !== undefined && shop.y + shop.height <= 844 - 34 && shop.x >= 0 && shop.x + shop.width <= 390, "the generators sit above the home bar, inside the width");
    const buys = hud.elements.filter((e) => e.id === "Buy");
    t.equal(buys.length, 4, "four generator buttons");
    t.ok(buys.every((e) => e.width >= 44 && e.height >= 44), "each at least 44 px square");
    t.ok(shop !== undefined && ORB.y < shop.y - 80, "the orb sits well above them");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
  },
});
