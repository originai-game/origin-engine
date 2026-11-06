// The handling: Brake held with Throttle through a turn at speed drifts
// (the car slides, its tail out) and the drift fills the boost meter;
// Boost spends the meter for more speed than Throttle alone reaches; a
// barrier stops the car at the road's edge (a contact, the car still on
// the road); and a car driven into another's back pushes it on, both
// cars counting the contact and neither passing through the other.
import { playtest } from "@origingame/engine/playtest";
import { car, carName, game, hold, SEATS, sorted } from "./lib.ts";

/** How far from the centre line the barriers let a car's middle go (src/world.ts REACH: the barrier's face, 7.7 m, less the car's half-width). */
const REACH = 6.76;
const AHEAD = SEATS; // the first rival's grid slot

export default playtest({
  name: "a drift fills the boost, Boost is quicker, barriers and cars collide",
  commands: sorted([
    "0.3:race:go",
    // The field moved well up the road, out of the way.
    ...Array.from({ length: 6 - SEATS }, (_, k) => `0.4:warp_car:${SEATS + k}:${250 + 20 * k}:${k % 2 ? 3 : -3}`),
    ...(SEATS > 1 ? ["0.4:warp_seat:1:380:4"] : []),
    // A drift down the start straight.
    "0.5:warp:-150:-3:30",
    "0.55:get:car-0.Car:before.json",
    hold("KeyW", 0.6, 2.4),
    hold("KeyS", 0.7, 0.6),
    hold("KeyD", 0.7, 0.3),
    hold("KeyA", 1.0, 0.3),
    "0.9:get:car-0.Car:drift.json",
    "1.3:get:car-0.Car:drifted.json",
    // Boost against Throttle alone, from the same speed in the same place.
    "1.5:warp:-150:0:30",
    "2.3:get:car-0.Car:plain.json",
    "2.4:warp:-150:0:30",
    hold("Space", 2.45, 0.7),
    "2.6:get:car-0.Car:boost-on.json",
    "3.2:get:car-0.Car:boosted.json",
    // A barrier: across the road at speed, held hard right.
    "3.4:get:Game:bumps-before.json",
    "3.4:warp:-150:3:30",
    hold("KeyD", 3.45, 0.8),
    hold("KeyW", 3.45, 0.8),
    "3.6:get:car-0.Car:wall-a.json",
    "3.8:get:car-0.Car:wall-b.json",
    "4.0:get:car-0.Car:wall-c.json",
    "4.3:get:car-0.Car:wall.json",
    "4.3:get:Game:bumps-after.json",
    // Into the back of a standing car.
    `4.4:warp_car:${AHEAD}:-130:0:0`,
    "4.4:warp:-150:0:25",
    `4.4:get:${carName(AHEAD)}.Car:hit-before.json`,
    `5.4:get:${carName(AHEAD)}.Car:hit.json`,
    `5.4:get:${carName(AHEAD)}.og.Transform:hit-at.json`,
    "5.4:get:car-0.Car:hitter.json",
    "5.4:get:car-0.og.Transform:hitter-at.json",
    "5.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = car(run, "before.json");
    const drift = car(run, "drift.json");
    t.ok(drift.drifting && Math.abs(drift.slip) > 2, `Brake with Throttle and a turn drifts (slip ${drift.slip.toFixed(1)} m/s)`);
    const drifted = car(run, "drifted.json");
    t.ok(drifted.boost > before.boost + 0.05, `and the drift filled the boost (${before.boost.toFixed(2)} → ${drifted.boost.toFixed(2)})`);
    const plain = car(run, "plain.json");
    const on = car(run, "boost-on.json");
    const boosted = car(run, "boosted.json");
    t.ok(on.boosting, "Boost lights the boost");
    t.ok(boosted.boost < on.boost, `and spends the meter (${on.boost.toFixed(2)} → ${boosted.boost.toFixed(2)})`);
    t.ok(boosted.speed > plain.speed + 1, `for more speed than Throttle alone (${plain.speed.toFixed(1)} → ${boosted.speed.toFixed(1)} m/s)`);
    const walls = ["wall-a.json", "wall-b.json", "wall-c.json", "wall.json"].map((f) => car(run, f));
    t.ok(walls.every((c) => Math.abs(c.across) <= REACH), `the barrier keeps the car on the road (across ${walls.map((c) => c.across.toFixed(1)).join(", ")})`);
    t.ok(game(run, "bumps-after.json").bumps > game(run, "bumps-before.json").bumps, "the race counts the bump");
    t.ok(walls[3]!.hits > 0, `and counts the contact (${walls[3]!.hits})`);
    const hit = car(run, "hit.json");
    const hitter = car(run, "hitter.json");
    t.ok(hit.hits > car(run, "hit-before.json").hits && hitter.hits > walls[3]!.hits, "the car driven into another's back and the car hit both count the contact");
    t.ok(hit.speed > 3, `and the car hit is pushed on (${hit.speed.toFixed(1)} m/s)`);
    const a = run.json<{ value: { position: [number, number, number] } }>("hit-at.json").value.position;
    const b = run.json<{ value: { position: [number, number, number] } }>("hitter-at.json").value.position;
    const gap = Math.sqrt((a[0] - b[0]) ** 2 + (a[2] - b[2]) ** 2);
    t.ok(gap > 1.6, `neither passes through the other (${gap.toFixed(2)} m apart)`);
  },
});
