// The seats (src/seats.ts), in either view: each courier is a seat's,
// carrying og.PlayerInput, and every control reaches it through that alone.
// Seat 0's keys (D) and its pad's stick move seat 0's courier right and
// back; the arrows and the second pad move seat 1's courier beside a second
// seat on one device (src/creation.ts's local-multiplayer) and the one
// courier otherwise, while the other stays put; the touch controls' first
// button (Jump side-on, Dash from above) reaches seat 0's buttons.
import { playtest } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";

const first: string = creation.controls.buttons[0]!.name;
const players: string = creation.players;
/** Seat 0's first touch button: in its own column beside a second seat (src/seats.ts), bottom right alone. */
const button = players === "local-multiplayer" ? `TouchMove0/Actions/${first}` : `TouchActions0/${first}`;

interface Input {
  player: number;
  x: number;
  y: number;
  buttons: number;
}

interface Game {
  heroX: number;
  hero2X: number;
}

interface Got<T> {
  value: T;
}

export default playtest({
  name: "each seat's keys, pad and touch controls reach its og.PlayerInput and drive its own courier",
  commands: [
    "0.1:viewport:phone",
    "0.2:get:og.Lockstep:lock.json",
    "0.25:get:Game:g0.json",
    "0.3:key_down:KeyD",
    "0.5:get:hero.og.PlayerInput:keys.json",
    "0.8:key_up:KeyD",
    "0.9:get:Game:g1.json",
    "1.0:gamepad:-:-1,0",
    "1.2:get:hero.og.PlayerInput:pad.json",
    "1.5:gamepad:-:0,0",
    "1.8:get:Game:g2.json",
    "1.85:key_down:ArrowRight",
    "2.2:key_up:ArrowRight",
    "2.3:get:Game:g3.json",
    "2.4:gamepad:-@1:1,0",
    "2.9:gamepad:-@1:0,0",
    "3.0:get:Game:g4.json",
    `3.1:touch:1:down:${button}`,
    "3.15:get:hero.og.PlayerInput:touch.json",
    `3.2:touch:1:up:${button}`,
    "3.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = (i: number) => run.json<Got<Game>>(`g${i}.json`).value;
    const x = (i: number, seat: number) => (seat === 0 ? g(i).heroX : g(i).hero2X);
    const input = (f: string) => run.json<Got<Input>>(f).value;
    const seats: number = creation.seats;
    const local = seats === 2 && !run.json<Got<{ enabled: boolean }>>("lock.json").value.enabled;
    const keys = input("keys.json");
    t.equal(keys.player, 0, "the courier is seat 0's og.PlayerInput");
    t.ok(keys.x > 0.9, `D reaches it as right (${keys.x})`);
    t.ok(x(1, 0) > x(0, 0) + 1, `and moves seat 0's courier right (x ${x(0, 0).toFixed(2)} to ${x(1, 0).toFixed(2)})`);
    t.ok(input("pad.json").x < -0.9, `seat 0's pad's stick reaches it as left (${input("pad.json").x})`);
    t.ok(x(2, 0) < x(1, 0) - 1, `and moves the courier back (x ${x(1, 0).toFixed(2)} to ${x(2, 0).toFixed(2)})`);
    // The arrows and the second pad: seat 1's courier with two on the device, the one courier otherwise.
    const mover = local ? 1 : 0;
    t.ok(x(3, mover) > x(2, mover) + 1, `the arrows moved seat ${mover}'s courier right (x ${x(2, mover).toFixed(2)} to ${x(3, mover).toFixed(2)})`);
    t.ok(x(4, mover) > x(3, mover) + 1, `the second pad moved seat ${mover}'s courier right (x ${x(3, mover).toFixed(2)} to ${x(4, mover).toFixed(2)})`);
    if (local) t.ok(Math.abs(x(4, 0) - x(2, 0)) < 0.01, `seat 0's courier stayed put meanwhile (x ${x(2, 0).toFixed(2)}, ${x(4, 0).toFixed(2)})`);
    t.ok((input("touch.json").buttons & 1) !== 0, `the touch controls' ${first} reaches seat 0's buttons (${input("touch.json").buttons})`);
  },
});
