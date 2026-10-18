// The slime's behaviour tree (assets/ai/slime.ogt): with the hero out of
// sight it patrols its beat on the map's grid; the hero stepping within
// Tuning.slimeSight halts the patrol at once (the selector is re-checked
// every tick), the slime chases at Tuning.slimeSpeed and bites in reach
// through the game's `bite` command, which costs a heart and bounces it back.
import { playtest } from "@origingame/engine/playtest";
import { component, position, report } from "./lib.ts";

interface AiDump {
  behaviour: string;
  node: string;
  path: string;
  sees: boolean;
  status: string;
  perception: { sees: boolean; sinceSeen: number };
}

export default playtest({
  name: "the slime patrols its beat, chases the hero on sight and bites in reach",
  commands: [
    // The hero waits at the spawn, 9 m away: out of sight.
    "0.5:dump_ai:slime:patrol-a.json",
    "0.5:dump_entity:slime:patrol-a-at.json",
    "2.5:dump_ai:slime:patrol-b.json",
    "2.5:dump_entity:slime:patrol-b-at.json",
    "2.5:dump_nav:slime:patrol-nav.json",
    // The hero steps 3 m from it, in the open.
    "3.0:slime:13.5:6.5",
    "3.0:place:10.5:6.5",
    "3.05:dump_ai:slime:seen.json",
    "3.6:dump_entity:slime:chase-at.json",
    "3.6:dump_ai:slime:chase.json",
    "5.5:dump_entity:hero:bitten.json",
    "5.5:dump_entity:slime:after.json",
    "5.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const a = run.json<AiDump>("patrol-a.json");
    t.equal(a.behaviour, "slime", "the slime runs assets/ai/slime.ogt");
    t.ok(a.path.includes("patrol beat"), `out of sight it patrols (${a.path})`);
    t.equal(a.sees, false, "it does not see the hero at the spawn");
    const pa = position(run, "patrol-a-at.json");
    const pb = position(run, "patrol-b-at.json");
    t.ok(Math.hypot(pb.x - pa.x, pb.y - pa.y) > 0.5, `the patrol moves it (${pa.x.toFixed(2)},${pa.y.toFixed(2)} to ${pb.x.toFixed(2)},${pb.y.toFixed(2)})`);
    t.ok(pb.x >= 13.3 && pb.x <= 17.7 && pb.y >= 3.3 && pb.y <= 6.7, "and keeps it on its beat");
    const nav = run.json<{ status: string; corners: unknown[] }>("patrol-nav.json");
    t.ok(nav.status === "moving" && nav.corners.length >= 2, `og.nav walks it along a path to the next beat point (${nav.status})`);

    const seen = run.json<AiDump>("seen.json");
    t.equal(seen.sees, true, "three ticks after the hero steps into sight, the slime sees it");
    t.ok(seen.path.includes("chase"), `and the patrol gave way to the chase (${seen.path})`);
    const c = position(run, "chase-at.json");
    t.ok(c.x < 13.0, `it closes in on the hero (x ${c.x.toFixed(2)})`);
    const speed = (13.5 - c.x) / 0.55;
    t.ok(speed > 1.0 && speed < 1.75, `at about Tuning.slimeSpeed, 1.6 m/s (${speed.toFixed(2)})`);
    t.ok(run.json<AiDump>("chase.json").path.includes("chase"), "still chasing");

    const bitten = report(run, "bitten.json");
    t.ok(bitten.hurts >= 1, `the bite landed (${bitten.hurts} hurt)`);
    t.ok(bitten.hp < 3, "and cost a heart");
    t.ok(component<{ bites: number }>(run, "after.json", "Slime").bites >= 1, "through the slime's bite command");
  },
});
