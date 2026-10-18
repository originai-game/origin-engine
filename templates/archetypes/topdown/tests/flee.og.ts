// The slime's senses and its fear (assets/ai/slime.ogt, og.Perception):
// the pillar in the west room hides the hero 4.5 m away (sight rays stop at
// the map's wall colliders), stepping out from behind it shows the hero,
// and a slime under 30 hp keeps away instead of chasing.
import { playtest } from "@origingame/engine/playtest";
import { position } from "./lib.ts";

interface AiDump {
  path: string;
  sees: boolean;
}

export default playtest({
  name: "the pillar hides the hero from the slime, and a hurt slime flees",
  commands: [
    // Behind the pillar (cells 6–7, rows 7–8): the slime east of it, the hero west.
    "0.1:slime:9.5:8",
    "0.1:place:5:8",
    "0.3:dump_ai:slime:hidden.json",
    // The hero steps out below the pillar.
    "0.4:slime:9.5:8",
    "0.4:place:7:5",
    "0.6:dump_ai:slime:visible.json",
    // Hurt: the slime keeps away from a hero 2 m off.
    "1.0:set:slime.Slime.hp=20",
    "1.0:slime:13.5:5",
    "1.0:place:11.5:5",
    "1.3:dump_ai:slime:fear.json",
    "2.2:dump_entity:slime:fled.json",
    "2.2:dump_entity:hero:hero.json",
    "2.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const hidden = run.json<AiDump>("hidden.json");
    t.equal(hidden.sees, false, "behind the pillar the hero is hidden, 4.5 m away (inside the slime's 5 m sight)");
    t.ok(hidden.path.includes("patrol"), `so the slime patrols on (${hidden.path})`);
    const visible = run.json<AiDump>("visible.json");
    t.equal(visible.sees, true, "below the pillar the slime sees the hero");
    t.ok(visible.path.includes("chase"), `and chases (${visible.path})`);

    const fear = run.json<AiDump>("fear.json");
    t.ok(fear.path.includes("flee"), `under 30 hp it flees (${fear.path})`);
    const s = position(run, "fled.json");
    const h = position(run, "hero.json");
    const d = Math.hypot(s.x - h.x, s.y - h.y);
    t.ok(d > 3.5, `and keeps away from the hero (${d.toFixed(2)} m, from 2)`);
  },
});
