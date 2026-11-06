// The seats: the cars are driven by og.PlayerInput alone (src/seats.ts).
//
// Each round puts the seats' cars at a standstill on clear road, then
// drives through one device for 0.8 s: the car it reaches must pick up
// speed (and turn, with the steer held), and the other seat's car must
// not move.
//   alone   every device is seat 0's: W with D, the arrows, either pad
//           (A with the stick right), the touch Throttle with Right and
//           Boost; S reaches it as Brake.
//   local   W and D, pad 1 and the left set of touch controls are seat
//           0's; the arrows, pad 2 and the right set seat 1's.
//   online  outside a room this device drives seat 0 alone (seat 1 is
//           the other peer's): every key set, pad and touch control is
//           seat 0's, and seat 1's car stands.
// A game reading the keys or the pointer itself instead of each seat's
// og.PlayerInput fails here: seat 1's devices would move seat 0's car, or
// the input entity would not hold the press.
import { playtest } from "@origingame/engine/playtest";
import { car, carName, finger, hold, LOCAL, SEATS, sorted, value } from "./lib.ts";

const THROTTLE = 1, BRAKE = 2;
interface Input { player: number; x: number; buttons: number }

/** Which seat each round's device drives on this scaffold. */
const seatOf = (localSeat: number) => (LOCAL ? localSeat : 0);
const touchPath = (seat: number, id: string) => (LOCAL ? `Touch${seat}/Pedals/${id === "Brake" ? "Brake" : `Right/${id}`}` : `Pedals/Pedals/${id === "Brake" ? "Brake" : `Right/${id}`}`);
const steerPath = (seat: number) => (LOCAL ? `Touch${seat}/Steer/Right` : "Steering/Steer/Right");

interface Round { name: string; drives: number; steps: (t: number) => string[] }
const rounds: Round[] = [
  { name: "W with D", drives: seatOf(0), steps: (t) => [hold("KeyW", t, 0.8), hold("KeyD", t + 0.4, 0.4), `${(t + 0.3).toFixed(3)}:get:car-0.og.PlayerInput:in-keys.json`] },
  { name: "the arrows", drives: seatOf(1), steps: (t) => [hold("ArrowUp", t, 0.8), hold("ArrowRight", t + 0.4, 0.4)] },
  { name: "pad 1", drives: seatOf(0), steps: (t) => [`${t}:gamepad:a:1,0`, `${(t + 0.8).toFixed(3)}:gamepad:-`] },
  { name: "pad 2", drives: seatOf(1), steps: (t) => [`${t}:gamepad:a@1:1,0`, `${(t + 0.8).toFixed(3)}:gamepad:-@1`] },
  {
    name: "the left touch controls",
    drives: seatOf(0),
    steps: (t) => [finger(touchPath(0, "Throttle"), t, 0.8, 1), finger(steerPath(0), t + 0.4, 0.4, 2), finger(touchPath(0, "Boost"), t + 0.2, 0.2, 3), `${(t + 0.3).toFixed(3)}:get:${carName(seatOf(0))}.Car:boosting.json`],
  },
  ...(LOCAL ? [{ name: "the right touch controls", drives: 1, steps: (t: number) => [finger(touchPath(1, "Throttle"), t, 0.8, 1), finger(steerPath(1), t + 0.4, 0.4, 2)] }] : []),
];

const START = 0.6;
const STEP = 1.1;
const steps: string[] = ["0.1:viewport:landscape", "0.3:race:go", hold("KeyS", 0.4, 0.1), "0.45:get:car-0.og.PlayerInput:in-brake.json"];
rounds.forEach((r, i) => {
  const t = START + i * STEP;
  steps.push(`${(t - 0.1).toFixed(3)}:warp_seat:0:450`);
  if (SEATS > 1) steps.push(`${(t - 0.1).toFixed(3)}:warp_seat:1:650`);
  steps.push(...r.steps(t));
  for (let s = 0; s < SEATS; s++) steps.push(`${(t + 0.85).toFixed(3)}:get:car-${s}.Car:r${i}-car${s}.json`);
});
steps.push(`${(START + rounds.length * STEP).toFixed(3)}:quit`);

export default playtest({
  name: "each seat's og.PlayerInput carries its own keys, pads and touch controls to its own car",
  size: "2532x1170",
  commands: sorted(steps),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const keys = value<Input>(run, "in-keys.json");
    t.equal(keys.player, 0, "car-0 carries seat 0's og.PlayerInput");
    t.ok((keys.buttons & THROTTLE) !== 0, `W reaches it as Throttle (buttons ${keys.buttons})`);
    t.ok(((value<Input>(run, "in-brake.json").buttons) & BRAKE) !== 0, "S reaches it as Brake");
    t.ok(car(run, "boosting.json").boosting, "the touch Boost lights the boost");
    rounds.forEach((r, i) => {
      for (let s = 0; s < SEATS; s++) {
        const c = car(run, `r${i}-car${s}.json`);
        if (s === r.drives) {
          t.ok(c.speed > 5, `${r.name} drive seat ${s}'s car (${c.speed.toFixed(1)} m/s)`);
          t.ok(c.steer > 0.3, `and steer it right (${c.steer.toFixed(2)})`);
        } else t.ok(Math.abs(c.speed) < 0.5, `${r.name} leave seat ${s}'s car standing (${c.speed.toFixed(2)} m/s)`);
      }
    });
  },
});
