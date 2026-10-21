// The ghost: the first lap (driven by the autopilot) sets the record, and
// from the start of the second lap the ghost drives it again: at the same
// time into its lap it is where the car was, to the poses' centimetre.
// The race starts at 0.5 s (`race:go`): GO is tick 31, the first lap is
// 2046 ticks (the autopilot's, asserted), so the second starts on tick
// 2077; the car is read 300, 900 and 1500 ticks into its first lap and the
// ghost as far into the second.
import { playtest } from "@origingame/engine/playtest";
import { dist, position, race, script } from "./lib.ts";

const GO = 31;
const LAP1 = 2046;
const AT = [300, 900, 1500];
const t = (tick: number) => (tick / 60).toFixed(3);

export default playtest({
  name: "the best lap's ghost drives where the car drove, at the same time into the lap",
  commands: script(
    "0.5:race:go",
    "0.6:drive:auto",
    "20:get:Race:lap1.json",
    ...AT.map((d, i) => `${t(GO + d)}:get:car.og.Transform:car-${i}.json`),
    ...AT.map((d, i) => `${t(GO + LAP1 + d)}:get:ghost.og.Transform:ghost-${i}.json`),
    `${t(GO + LAP1 + 600)}:get:Race:lap2.json`,
    `${t(GO + LAP1 + 600)}:screenshot:ghost.png`,
    "60:ghost:off",
    "60.2:get:ghost.og.Transform:hidden.json",
    "60.2:get:Race:off.json",
    "60.3:quit",
  ),
  check(run, ok) {
    ok.equal(run.warnings.join("; "), "", "the script ran without warnings");
    ok.ok(run.quit, "the script reached quit");
    ok.equal(race(run, "lap1.json").ghostActive, false, "no ghost on the first lap of a first session (no record yet)");
    const r = race(run, "lap2.json");
    ok.equal(r.lap1, LAP1, `the autopilot's first lap is ${LAP1} ticks (a change of handling or of the autopilot moves it: update LAP1)`);
    ok.equal(r.record, LAP1, "the first lap is the record");
    ok.ok(r.ghostActive, "the ghost drives the second lap");
    ok.equal(r.ghostTick, 600, "600 ticks into the lap, the ghost is 600 ticks into the record");
    AT.forEach((d, i) => {
      const e = dist(position(run, `car-${i}.json`), position(run, `ghost-${i}.json`));
      ok.ok(e < 0.02, `${d} ticks into the lap the ghost is where the car was (${(e * 100).toFixed(1)} cm apart)`);
    });
    ok.equal(race(run, "off.json").ghostActive, false, "the ghost switched off in the settings goes");
    ok.ok(position(run, "hidden.json")[1]! < -10, "and is put away under the world");
  },
});
