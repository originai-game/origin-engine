// The template's playtest, run by `og test` through `og run --headless
// --commands`: a tap on the menu's Start button (found by its UI id, no key
// fallback) starts the trickster spinning and drops the crate onto the
// ground; the frame, the dumps and the UI tree are written.
import { playtest } from "@origingame/engine/playtest";

interface EntityDump {
  components: Record<string, Record<string, number | boolean | string | number[]>>;
}

interface UiDump {
  elements: Array<{ id: string; parent: string; text: string; x: number; y: number; width: number; height: number }>;
}

export default playtest({
  name: "A tap on Start spins the trickster and drops the crate; the run writes its frame, dumps and UI tree",
  commands:
    "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;" +
    "2.0:screenshot:smoke.png;2.1:dump_entity:trickster:trickster.json;2.15:dump_entity:crate:crate.json;" +
    "2.2:dump_world:world.json;2.3:dump_ui:ui.json;2.5:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    // Frames are read inside t.frame: without a renderer (og test in a sandbox
    // with no GPU) the frame is reported unchecked and every other check runs.
    t.frame("smoke.png", () => {
      const frame = run.png("smoke.png");
      t.ok(frame.width > 0 && frame.height > 0, `a ${frame.width}x${frame.height} frame was captured`);
    });
    const world = run.json<{ entities: number; systems: Array<{ name: string; matched: number | null }> }>("world.json");
    t.ok(world.entities >= 5, `the scene spawned with the crate (${world.entities} entities)`);
    t.ok(world.systems.some((s) => s.name === "Rotate" && s.matched === 1), "Rotate matches the trickster");
    const ui = run.json<UiDump>("ui.json");
    const start = ui.elements.find((e) => e.id === "Start");
    t.ok(start !== undefined && start.width > 0 && start.height > 0, "the UI tree has the Start button with bounds");
    t.ok(start?.text === "Started", `the button reads "${start?.text}" after the tap`);
    const trickster = run.json<EntityDump>("trickster.json");
    const spin = trickster.components["Spin"]!;
    t.ok(Number(spin["speed"]) > 0, `the trickster spins (${spin["speed"]} rad/s)`);
    const crate = run.json<EntityDump>("crate.json");
    const y = (crate.components["og.Transform"]!["position"] as number[])[1]!;
    t.ok(y > 0.4 && y < 0.6, `the crate fell from 4 m and rests on the ground (y ${y.toFixed(3)})`);
  },
});
