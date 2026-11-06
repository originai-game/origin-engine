// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area; the battle's HUD in it too, Pause and the map's
// step along the top, the hand, the energy and End turn along the bottom,
// each touch-sized; a tap on a card lifts it and a second plays it, and a
// tap on End turn ends the turn.
import { playtest } from "@origingame/engine/playtest";
import { BATTLE, game, hero, script, tap, TWO } from "./lib.ts";

interface Element { id: string; path: string; kind: string; x: number; y: number; width: number; height: number }

export default playtest({
  name: "the phone layout: the HUD and the hand inside the safe area, cards and End turn tapped",
  size: "1170x2532",
  commands: script([
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("Play", 1.0),
    tap("Path#0", 1.8),
    "2.6:get:Game:started.json",
    "2.6:dump_entity:hero:hand.json",
    "2.7:dump_ui:hud.json",
    "2.8:energy:9",
    "2.8:give:defend",
    tap("Hands/Seat0/Hand0/Fan/0-10", 3.0),
    tap("Hands/Seat0/Hand0/Fan/0-10", 3.4),
    "3.8:get:Game:played.json",
    tap("Hands/Seat0/Bar/End", 3.9),
    ...(TWO ? [tap("Hands/Seat1/Bar/End", 4.2)] : []),
    "4.5:get:Game:ended.json",
    "4.6:screenshot:play.png",
    "4.7:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "started.json").phase, BATTLE, "taps on Begin and the first fight started it");
    const ui = run.json<{ surface: { width: number; height: number }; elements: Element[] }>("hud.json");
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    const el = (path: string) => ui.elements.find((e) => e.path === path);
    for (const p of ["Corner/Pause", "Corner/Step"]) t.ok((el(p)?.y ?? 0) >= 47, `${p} sits below the notch`);
    const ends = ["Hands/Seat0/Bar/End", ...(TWO ? ["Hands/Seat1/Bar/End"] : [])];
    for (const p of ["Corner/Pause", ...ends]) {
      const e = el(p);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${p} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    const cards = hero(run, "hand.json").hand.map((i) => el(`Hands/Seat0/Hand0/Fan/0-${i}`));
    t.ok(cards.every((c) => c !== undefined && c.y + c.height <= 844 - 34 + 1 && c.x >= -1 && c.x + c.width <= 391), "every card of the hand sits inside the safe area");
    t.ok(ui.elements.every((e) => e.x >= -1 && e.x + e.width <= 391), "nothing runs off the sides");
    t.equal(game(run, "played.json").played, 1, "two taps on a card played it");
    t.equal(game(run, "ended.json").side, 1, "End turn tapped: the foes act");
    const title = run.json<{ elements: Element[] }>("title.json").elements.filter((e) => e.path.startsWith("main-menu/"));
    t.ok(title.some((e) => e.id === "Play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.frame("play.png", () => t.equal(run.png("play.png").width, 1170, "play.png captured at the phone's device pixels"));
  },
});
