// Seats (src/seats.ts): the courier reads its seat's og.PlayerInput alone,
// so every device bound to seat 0's `jump` plays it. Space starts the
// round; the pad's A and W each hold a full jump. (Enter and Up are seat
// 1's keys with two on one device: alone they are seat 0's too.)
import { playtest } from "@origingame/engine/playtest";
import { RUNNING, positionOf, runOf, tap } from "./lib.ts";

interface EntityDump {
  components: Record<string, Record<string, unknown>>;
}

const JUMPS = [
  ["pad.json", "the pad's A"],
  ["w.json", "W"],
] as const;

export default playtest({
  name: "seat 0's jump: Space, the pad's A and W reach og.PlayerInput and jump the courier",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    tap(0.3),
    "0.5:dump_entity:runner:start.json",
    "1.0:gamepad:a",
    "1.05:dump_entity:runner:pad-input.json",
    "1.25:dump_entity:runner:pad.json",
    "1.4:gamepad:-",
    "2.0:key_down:KeyW",
    "2.25:dump_entity:runner:w.json",
    "2.4:key_up:KeyW",
    "2.9:dump_entity:runner:landed.json",
    "3.0:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = run.json<EntityDump>("start.json");
    t.equal(start.components["og.PlayerInput"]?.["player"], 0, "the courier carries seat 0's og.PlayerInput");
    t.equal(runOf(run, "start.json").phase, RUNNING, "Space started the round");
    const held = run.json<EntityDump>("pad-input.json").components["og.PlayerInput"]?.["buttons"];
    t.equal(held, 1, "the pad's A is the jump bit of og.PlayerInput");
    for (const [file, what] of JUMPS) {
      const y = positionOf(run, file).y;
      t.ok(y > 0.8, `${what} jumps the courier (feet at ${y.toFixed(2)} m)`);
    }
    t.ok(positionOf(run, "landed.json").y < 0.05, "and it lands between the jumps");
    t.equal(runOf(run, "landed.json").phase, RUNNING, "nothing ended the round");
  },
});
