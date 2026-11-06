// The rivals: from the grid at GO they drive the racing line a whole lap
// on their own, inside the barriers, through both checkpoints in order,
// each lap timed, while the seats stand on the grid; and the field's
// places follow the distance run.
import { playtest } from "@origingame/engine/playtest";
import { car, carName, SEATS } from "./lib.ts";

const RIVALS = Array.from({ length: 6 - SEATS }, (_, k) => SEATS + k);
const SAMPLES = [10, 20, 30, 37];

export default playtest({
  name: "the rivals lap the circuit on the racing line, timed, inside the barriers",
  commands: [
    "0.3:race:go",
    ...SAMPLES.flatMap((s) => RIVALS.map((r) => `${s}:get:${carName(r)}.Car:r${r}-${s}.json`)),
    `${SAMPLES.at(-1)}:get:car-0.Car:seat.json`,
    `${SAMPLES.at(-1)! + 0.1}:quit`,
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    for (const r of RIVALS) {
      const seen = SAMPLES.map((s) => car(run, `r${r}-${s}.json`));
      t.ok(seen.every((c) => Math.abs(c.across) <= 6.76), `rival ${r} stays inside the barriers (across ${seen.map((c) => c.across.toFixed(1)).join(", ")})`);
      const last = seen.at(-1)!;
      t.ok(last.laps >= 1 && last.lastLap > 0, `and completes a timed lap (${(last.lastLap / 60).toFixed(2)} s)`);
      t.ok(last.lastLap > 20 * 60 && last.lastLap < 40 * 60, "at an arcade pace (20 to 40 s a lap)");
      t.ok(last.wrongWay === 0 && last.place <= 6 - SEATS, `never the wrong way, ahead of the seats standing (place ${last.place})`);
    }
    const seat = car(run, "seat.json");
    t.ok(seat.place > 6 - SEATS && seat.laps <= 0, `the standing seat is behind the field (place ${seat.place})`);
  },
});
