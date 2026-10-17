// The template's playtest, run by `og test` through `og run --headless
// --commands`: the Start button (or, without a UI module, the S key) starts
// the trickster spinning; the frame and the world dump are written.
import { playtest } from "@origingame/engine/playtest";

interface EntityDump {
  components: Record<string, Record<string, number | Record<string, number>>>;
}

export default playtest({
  name: "Start sets the trickster spinning, and the run writes its frame and dump",
  commands:
    "0.5:touch_move_to:Start;0.6:touch_start;0.7:touch_end;0.8:type:s;" +
    "2.0:screenshot:smoke.png;2.1:dump_entity:trickster:trickster.json;2.2:dump_world:world.json;2.5:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    const frame = run.png("smoke.png");
    t.ok(frame.width > 0 && frame.height > 0, `a ${frame.width}x${frame.height} frame was captured`);
    const world = run.json<{ entities: number; systems: Array<{ name: string; matched: number | null }> }>("world.json");
    t.ok(world.entities >= 4, `the scene spawned (${world.entities} entities)`);
    t.ok(world.systems.some((s) => s.name === "Rotate" && s.matched === 1), "Rotate matches the trickster");
    const trickster = run.json<EntityDump>("trickster.json");
    const spin = trickster.components["Spin"]!;
    t.ok(Number(spin["speed"]) > 0, `the trickster spins (${spin["speed"]} rad/s)`);
  },
});
