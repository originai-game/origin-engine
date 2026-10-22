// The starter's playtest, run by `og test` through `og run --headless
// --commands`: the courier runs right over two coins, jumps up through the
// one-way ledge above it and lands on top, Escape opens the pause menu and
// holds time, Escape again resumes, and a fall into the pit costs a life
// and puts the courier back at the spawn.
import { playtest } from "@origingame/engine/playtest";

interface Got<T> {
  value: T;
}

interface Game {
  coins: number;
  lives: number;
  paused: boolean;
  heroX: number;
  heroY: number;
}

interface EntityDump {
  components: Record<string, Record<string, number | boolean | string | number[]>>;
}

interface UiDump {
  elements: Array<{ id: string; text: string; width: number; height: number }>;
}

export default playtest({
  name: "The courier runs over two coins, jumps up through a one-way ledge onto it, pauses and resumes, and the pit costs a life",
  commands:
    "0.5:key_down:ArrowRight;2.0:key_up:ArrowRight;" +
    "2.3:key_down:Space;2.9:key_up:Space;" +
    "3.5:screenshot:ledge.png;3.55:dump_entity:hero:hero.json;3.6:get:Game:ledge.json;3.65:dump_ui:ui.json;" +
    "3.7:key_down:Escape;3.75:key_up:Escape;4.2:get:Game:paused.json;" +
    "4.3:key_down:Escape;4.35:key_up:Escape;4.5:get:Game:resumed.json;" +
    "4.6:warp:21:-1;5.0:get:Game:fell.json;5.1:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    t.frame("ledge.png", () => {
      const frame = run.png("ledge.png");
      t.ok(frame.width > 0 && frame.height > 0, `a ${frame.width}x${frame.height} frame was captured`);
    });
    const ledge = run.json<Got<Game>>("ledge.json").value;
    t.ok(ledge.coins === 2, `the run took the two coins on the roof (${ledge.coins})`);
    // The ledge (cells 11-15 of row 9) has its top at y 5: the courier passed up through it and stands on it.
    t.ok(ledge.heroX > 11 && ledge.heroX < 16, `the courier is under the ledge's span (x ${ledge.heroX.toFixed(2)})`);
    t.ok(Math.abs(ledge.heroY - 5) < 0.05, `and stands on the one-way ledge (y ${ledge.heroY.toFixed(3)})`);
    const hero = run.json<EntityDump>("hero.json");
    t.ok(hero.components["og.PlatformerController"]!["grounded"] === true, "the controller is grounded on the ledge");
    const ui = run.json<UiDump>("ui.json");
    const count = ui.elements.find((e) => e.id === "Count");
    t.ok(count !== undefined && count.text === "2", `the platformer HUD counts the coins ("${count?.text}")`);
    const paused = run.json<Got<Game>>("paused.json").value;
    t.ok(paused.paused, "Escape opened the pause menu");
    t.ok(!run.json<Got<Game>>("resumed.json").value.paused, "Escape again resumed");
    const fell = run.json<Got<Game>>("fell.json").value;
    t.ok(fell.lives === 2, `the pit cost a life (${fell.lives} left)`);
    t.ok(Math.abs(fell.heroX - 2.5) < 0.5 && fell.heroY < 2.5, `the courier is back at the spawn (${fell.heroX.toFixed(2)}, ${fell.heroY.toFixed(2)})`);
  },
});
