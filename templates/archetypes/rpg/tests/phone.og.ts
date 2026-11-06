// The phone layout (390×844 at 3×, a notch and a home bar): the title's
// menu inside the safe area with the touch footer; the HUD in it too (the
// party, the purse and Pause along the top, the stick and Attack, Talk and
// Potion along the bottom, all touch-sized). The stick walks the hero,
// Talk by Maren starts her conversation, a tap on a reply picks it and a
// tap on her line moves it on.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, LOCAL, PLAY, position, script, tap, texts } from "./lib.ts";

/** Seat 0's touch controls (src/ui.ts touchControls): alone, the stick and the buttons apart; two on one screen, its corner's column. */
const MOVE = LOCAL ? "Hands0/Move" : "Move0/Move";
const button = (name: string) => (LOCAL ? `Hands0/Buttons/${name}` : `Hands0/Small/${name}`);

const reply = (text: string, t: number) => `${t}:move_to_text:${text};${(t + 0.017).toFixed(3)}:touch_start;${(t + 0.1).toFixed(3)}:touch_end`;

export default playtest({
  name: "the phone layout: a HUD inside the safe area, a stick that walks, Talk, and replies by tap",
  size: "1170x2532",
  commands: script(
    "0.1:viewport:phone",
    "0.9:dump_ui:title.json",
    tap("Play", 1.0),
    "1.6:get:Game:started.json",
    "1.6:get:hero.og.Transform:from.json",
    "1.7:dump_ui:hud.json",
    `1.8:touch_move_to:${MOVE}`,
    "1.82:touch_start",
    "1.9:touch_move:84,600",
    "2.4:touch_end",
    "2.4:get:hero.og.Transform:to.json",
    "2.6:warp:maren",
    `2.8:touch_move_to:${button("Talk")}`,
    "2.82:touch_start",
    "2.9:touch_end",
    "3.2:get:Game:talking.json",
    "3.2:dump_ui:talk.json",
    reply("What happened here?", 3.5),
    "3.9:dump_ui:story.json",
    tap("Hit", 4.2),
    "4.6:dump_ui:story2.json",
    "4.65:screenshot:talk.png",
    "4.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const title = elements(run, "title.json").filter((e) => e.path.startsWith("main-menu/"));
    t.ok(title.some((e) => e.id === "Play") && title.every((e) => e.y + e.height <= 844 - 34 && e.y >= 47), "the title's menu sits inside the safe area");
    t.ok(texts(run, "title.json").includes("The stick moves · Attack strikes · Talk speaks and opens · Potion heals · tap a reply"), "with the touch controls in its footer");
    t.equal(game(run, "started.json").phase, PLAY, "a tap on Enter the keep started a run");
    const ui = run.json<{ surface: { width: number } }>("hud.json");
    const hud = elements(run, "hud.json");
    const el = (path: string) => hud.find((e) => e.path === path);
    t.equal(ui.surface.width, 390, "laid out for a 390 px wide phone");
    for (const p of ["PartyHud/Party", "Purse/Pause", "Purse/Gold"]) t.ok((el(p)?.y ?? 0) >= 47, `${p} sits below the notch`);
    for (const p of [MOVE, button("Attack"), button("Talk"), button("Potion")]) {
      const e = el(p);
      t.ok(e !== undefined && e.y + e.height <= 844 - 34, `${p} sits above the home bar`);
      t.ok(e !== undefined && e.width >= 44 && e.height >= 44, `${p} is at least 44 px square (${e?.width}×${e?.height})`);
    }
    t.ok(hud.every((e) => e.x >= 0 && e.x + e.width <= 390), "nothing runs off the sides");
    const box = (p: string) => el(p)!;
    const overlap = (a: { x: number; y: number; width: number; height: number }, b: typeof a) => a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height;
    t.ok(!overlap(box(MOVE), box(button("Attack"))) && !overlap(box("PartyHud/Party"), box("Purse/Gold")), "the stick and the buttons, the party and the purse, stand apart");
    const [x0, , z0] = position(run, "from.json");
    const [x1, , z1] = position(run, "to.json");
    t.ok(Math.hypot(x1 - x0, z1 - z0) > 1, `the stick walked the hero (${Math.hypot(x1 - x0, z1 - z0).toFixed(2)} m)`);
    t.ok(game(run, "talking.json").conversation !== 0, "Talk by Maren started her conversation");
    t.ok(texts(run, "talk.json").includes("Who are you?"), "her replies show");
    t.ok(texts(run, "story.json").some((s) => s.startsWith("The old warden sealed the crypt")), "a tap on a reply picked it");
    t.ok(texts(run, "story2.json").some((s) => s.startsWith("Put down what walks in my yard")), "a tap on her line moved it on");
    t.frame("talk.png", () => t.equal(run.png("talk.png").width, 1170, "talk.png captured at the phone's device pixels"));
  },
});
