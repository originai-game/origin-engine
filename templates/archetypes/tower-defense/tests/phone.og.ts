// A phone in portrait (390 × 844 CSS px at 3×, a notch and a home
// indicator): the title and the HUD lay out for it, the tray runs along
// the bottom inside the safe area with touch-sized buttons, and a tap on
// a tray button picks its plant.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY, play, tap } from "./lib.ts";

interface Ui {
  surface: { width: number; height: number; dpr: number } | null;
  elements: Array<{ id: string; parent: string; kind?: string; text: string; x: number; y: number; width: number; height: number }>;
}

export default playtest({
  name: "the phone layout: title, HUD and a bottom tray of touch-sized buttons inside the safe area",
  size: "1170x2532",
  commands: [
    "0.05:viewport:phone",
    "0.4:screenshot:title.png",
    play(0.5),
    "1.3:plant:thorn:3:5",
    tap("Plant#2/Pick", 1.4),
    // A finger lifts away: nothing hovers on a touch screen.
    "1.5:touch_move:195,400",
    "1.7:get:Game:picked.json",
    "1.8:screenshot:play.png",
    "1.8:dump_ui:ui.json",
    "1.9:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const picked = game(run, "picked.json");
    t.equal(picked.phase, PLAY, "a tap on Play started the round");
    t.equal(picked.selected, 2, "a tap on the tray picked the hedge");
    const ui = run.json<Ui>("ui.json");
    t.equal(ui.surface?.width, 390, "the UI lays out at 390 CSS px");
    const picks = ui.elements.filter((e) => e.id === "Pick");
    t.equal(picks.length, 3, "three tray buttons");
    for (const p of picks) {
      t.ok(p.height >= 44, `${p.text}: ${p.height} px tall, a touch target`);
      t.ok(p.y + p.height <= 844 - 34, `${p.text} sits above the home indicator`);
      t.ok(p.y > 844 / 2, `${p.text} is in the bottom half, the thumb's`);
    }
    const pause = ui.elements.find((e) => e.id === "Pause");
    t.ok(pause !== undefined && pause.y >= 47, "the pause button sits below the notch");
    for (const f of ["title.png", "play.png"]) t.equal(run.png(f).height, 2532, `${f} is a phone frame`);
  },
});
