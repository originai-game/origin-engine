// og.motion's drawn feet over the course, measured from where they are
// drawn (dump_motion's `footing`, never a contact channel): the
// thresholds are the animation guide's (`og api show animation`, §2.1).
import { playtest } from "@origingame/engine/playtest";

interface Slides { samples: number; over0_1: number; over0_3: number; maxMps: number }
interface Agent {
  entity: number;
  footing: { segments: Record<string, Slides>; held: { frames: number; floating: number; maxM: number } };
  plantedSlide: { frames: number };
}

export default playtest({
  name: "planted feet stay put through walks, a run, jumps, a crouching turn and held poses",
  size: "640x360",
  commands: "14.2:screenshot:course.png;14.2:dump_motion:feet.json;14.25:quit",
  check(run, t) {
    // The model runs only with a renderer: without one there is no
    // generated motion to measure.
    t.frame("course.png", () => {
      const dump = run.json<{ model: { agents: Agent[] } }>("feet.json").model;
      t.equal(dump.agents.length, 2, "both agents drawn");
      for (const a of dump.agents) {
        const segments = Object.entries(a.footing.segments).filter(([s]) => s !== "air");
        const sum = (f: (s: Slides) => number) => segments.reduce((n, [, s]) => n + f(s), 0);
        const samples = sum((s) => s.samples);
        const slow = sum((s) => s.over0_1);
        const fast = segments.filter(([s]) => s !== "takeoff").reduce((n, [, s]) => n + s.over0_3, 0);
        const what = `agent ${a.entity}: ${JSON.stringify(a.footing)}`;
        t.ok(samples >= 300, `${what}: feet on the ground in at least 300 samples`);
        t.ok(slow <= 0.05 * samples, `${what}: at most 5% of samples on the ground faster than 0.1 m/s`);
        t.ok(fast <= 0.01 * samples, `${what}: at most 1% faster than 0.3 m/s outside a takeoff`);
        t.ok(a.footing.held.floating <= 0.02 * a.footing.held.frames, `${what}: a held pose holds no foot over 3 cm in 98% of its frames`);
      }
    });
  },
});
