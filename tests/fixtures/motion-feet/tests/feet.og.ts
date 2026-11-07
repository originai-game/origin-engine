// og.motion's drawn feet over the course, measured from where they are
// drawn (dump_motion's `footing`, never a contact channel): the
// thresholds are the animation guide's (`og api show animation`, §2.1).
import { expect, test } from "bun:test";
import { runPlaytest } from "@origingame/engine/playtest";

interface Slides { samples: number; over0_1: number; over0_3: number; maxMps: number }
interface Agent {
  entity: number;
  footing: { segments: Record<string, Slides>; held: { frames: number; floating: number; maxM: number } };
  plantedSlide: { frames: number };
}

test("planted feet stay put through walks, a run, jumps, a crouching turn and held poses", async () => {
  const run = await runPlaytest({ size: "640x360", commands: "14.2:screenshot:course.png;14.2:dump_motion:feet.json;14.25:quit" });
  // The model runs only with a renderer: without one there is no
  // generated motion to measure.
  run.frame("course.png", () => {
    const dump = run.json<{ model: { agents: Agent[] } }>("feet.json").model;
    expect(dump.agents, "both agents drawn").toHaveLength(2);
    for (const a of dump.agents) {
      const segments = Object.entries(a.footing.segments).filter(([s]) => s !== "air");
      const sum = (f: (s: Slides) => number) => segments.reduce((n, [, s]) => n + f(s), 0);
      const samples = sum((s) => s.samples);
      const slow = sum((s) => s.over0_1);
      const fast = segments.filter(([s]) => s !== "takeoff").reduce((n, [, s]) => n + s.over0_3, 0);
      const what = `agent ${a.entity}: ${JSON.stringify(a.footing)}`;
      expect(samples, `${what}: feet on the ground in at least 300 samples`).toBeGreaterThanOrEqual(300);
      expect(slow, `${what}: at most 5% of samples on the ground faster than 0.1 m/s`).toBeLessThanOrEqual(0.05 * samples);
      expect(fast, `${what}: no foot on the ground faster than 0.3 m/s outside a takeoff`).toBe(0);
      expect(a.footing.held.floating, `${what}: a held pose never holds a foot over 3 cm`).toBe(0);
    }
  });
});
