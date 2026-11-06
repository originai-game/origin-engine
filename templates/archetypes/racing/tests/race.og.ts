// A race from the grid to the results, and the next race:
// - the countdown holds the field: Throttle held through it moves nothing,
//   and the car goes at GO;
// - the checkpoints count in order (the second sector's mark, then the
//   third's), and crossing the line after them times a lap (its time and
//   the best lap); a car that misses a checkpoint crosses the line for no lap;
// - on the last lap the seats cross the line first: each finishes (seat 0
//   first), the race closes once every seat has finished, the results
//   show the win, the record (a race, a win, the best lap) goes to
//   og.platform's store, and Throttle on the results races again.
// The rivals are moved up the road first so the warped seats meet no one.
import { playtest } from "@origingame/engine/playtest";
import { car, COUNTDOWN_S, game, hold, LOCAL, press, RACE, RESULTS, SEATS, sorted, texts, type PlatformDump } from "./lib.ts";

const others = Array.from({ length: 6 - SEATS }, (_, k) => `3.65:warp_car:${SEATS + k}:${620 + 25 * k}:${k % 2 ? 3 : -3}`);
const both = (cmd: (seat: number) => string) => Array.from({ length: SEATS }, (_, s) => cmd(s));

export default playtest({
  name: "the countdown, checkpoints in order, a timed lap, the finish, the results and the next race",
  commands: sorted([
    "0.3:race",
    hold("KeyW", 0.4, 3.2),
    "1.5:get:Game:countdown.json",
    "1.5:get:car-0.Car:held.json",
    "3.5:get:car-0.Car:go.json",
    ...others,
    ...(SEATS > 1 ? ["3.65:warp_seat:1:700:3"] : []),
    // A missed checkpoint: past the first sector, then straight to the line.
    "3.7:warp:100",
    "3.8:warp:-15:0:30",
    "4.4:get:car-0.Car:skipped.json",
    // The sectors in order, then the line.
    "4.5:warp:260:0:30",
    "4.9:get:car-0.Car:sector1.json",
    "5.0:warp:530:0:30",
    "5.4:get:car-0.Car:sector2.json",
    "5.5:warp:-15:0:30",
    "6.2:get:car-0.Car:lap.json",
    // The last lap's line.
    ...both((s) => `6.3:warp_seat:${s}:300`),
    ...both((s) => `6.4:lap:${s}:2`),
    ...both((s) => `6.4:warp_seat:${s}:${-15 - 12 * s}:${s ? 3 : -3}:30`),
    ...both((s) => `7.4:get:car-${s}.Car:finish${s}.json`),
    "7.4:get:Game:closed.json",
    "10.2:get:Game:results.json",
    "10.2:dump_ui:results-ui.json",
    "10.2:dump_platform:platform.json",
    press("KeyW", 10.9),
    "11.2:get:Game:again.json",
    "11.2:get:car-0.Car:again-car.json",
    "11.3:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const cd = game(run, "countdown.json");
    t.ok(cd.phase === RACE && cd.clock < COUNTDOWN_S * 60, `the countdown runs (clock ${cd.clock})`);
    t.ok(Math.abs(car(run, "held.json").speed) < 0.01, "Throttle through the countdown moves nothing");
    t.ok(car(run, "go.json").speed > 1.5, `at GO the car goes (${car(run, "go.json").speed.toFixed(1)} m/s)`);
    const skipped = car(run, "skipped.json");
    t.ok(skipped.laps === 0 && skipped.lastLap === 0, `crossing the line with a checkpoint missed times no lap (laps ${skipped.laps}, last ${skipped.lastLap})`);
    t.equal(car(run, "sector1.json").checkpoint, 1, "the first checkpoint counts");
    t.equal(car(run, "sector2.json").checkpoint, 2, "then the second");
    const lap = car(run, "lap.json");
    t.ok(lap.laps === 1 && lap.lastLap > 0 && lap.bestLap === lap.lastLap && lap.checkpoint === 0, `then the line times lap 1 (${lap.lastLap} ticks, best ${lap.bestLap})`);
    for (let s = 0; s < SEATS; s++) {
      const f = car(run, `finish${s}.json`);
      t.ok(f.finish > 0 && f.laps === 3 && !f.out, `seat ${s} finished the third lap`);
      t.equal(f.place, s + 1, `in place ${s + 1}`);
    }
    t.ok(game(run, "closed.json").closed, "the race closed once every seat had finished");
    t.equal(game(run, "results.json").phase, RESULTS, "and the results followed");
    const words = texts(run, "results-ui.json");
    const title = LOCAL ? "P1 wins!" : "Victory!";
    t.ok(words.includes(title), `the results say ${title}`);
    const p = run.json<PlatformDump>("platform.json");
    const rec = p.calls.filter((c) => c.method === "store.set" && c.args["key"] === "record").at(-1)?.args["value"] as { races: number; wins: number; bestLap: number } | undefined;
    t.ok(rec !== undefined && rec.races === 1 && rec.wins === 1 && rec.bestLap === lap.bestLap, `the record went to the store (${JSON.stringify(rec)})`);
    const again = game(run, "again.json");
    t.ok(again.phase === RACE && again.clock < 60 && !again.closed, "Throttle on the results starts the next race from the countdown");
    t.ok(car(run, "again-car.json").laps === -1 && car(run, "again-car.json").finish === 0, "with the car back on the grid");
  },
});
