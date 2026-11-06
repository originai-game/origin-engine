// The seats (src/seats.ts): each hero carries its seat's og.PlayerInput,
// the only input the game reads. Alone (and online before a room runs)
// every device reaches seat 0: S starts the run (the start control's bit),
// W walks the trickster forward, an arrow key (seat 1's, which a lone
// player also holds) sidesteps it, Shift runs it, the first pad's stick
// walks it and its A jumps it (east, away from the guard's post), and on a phone the touch move stick walks it
// and the look stick turns its look. Two on one device (src/creation.ts
// players local-multiplayer): seat 1's Enter starts, its arrows, Enter and
// pad 2 walk and jump the second walker while the trickster stands, and
// then seat 0's WASD walks only the trickster.
import { playtest } from "@origingame/engine/playtest";
import { component, LOCAL, position, SEATS, value } from "./lib.ts";
import type { Mission, V3 } from "./lib.ts";

interface Input {
  player: number;
  x: number;
  y: number;
  aimX: number;
  aimY: number;
  buttons: number;
}

/** og.PlayerInput.buttons, in src/creation.ts's control order. */
const JUMP = 1;
const RUN = 2;
const START = 4;

const dist = (a: V3, b: V3) => Math.hypot(a[0] - b[0], a[2] - b[2]);

const alone = playtest({
  name: "S, W, an arrow, Shift, the pad and the touch sticks reach seat 0's og.PlayerInput and play the trickster",
  size: "1280x720",
  commands: [
    "0.3:key_down:KeyS",
    "0.35:get:trickster.og.PlayerInput:start-input.json",
    "0.4:key_up:KeyS",
    "0.5:get:dropship.Mission:started.json",
    "0.6:dump_entity:trickster:a.json",
    ...(SEATS > 1 ? ["0.6:dump_entity:trickster-1:other-a.json"] : []),
    "0.7:key_down:KeyW",
    "0.9:get:trickster.og.PlayerInput:w.json",
    "1.1:key_up:KeyW",
    "1.3:dump_entity:trickster:b.json",
    "1.4:key_down:ArrowRight",
    "1.9:key_up:ArrowRight",
    "2.0:dump_entity:trickster:c.json",
    "2.1:key_down:ShiftLeft",
    "2.1:key_down:ArrowRight",
    "2.5:get:trickster.og.PlayerInput:run-input.json",
    "2.5:get:trickster.Hero:running.json",
    "2.6:key_up:ArrowRight",
    "2.6:key_up:ShiftLeft",
    "2.8:dump_entity:trickster:d.json",
    "2.9:gamepad:-@0:1,0",
    "3.2:get:trickster.og.PlayerInput:pad.json",
    "3.4:gamepad:-@0:0,0",
    "3.5:dump_entity:trickster:e.json",
    "3.6:gamepad:a@0",
    "3.65:get:trickster.og.PlayerInput:pad-a.json",
    "3.8:dump_entity:trickster:jump.json",
    "3.9:gamepad:off@0",
    "4.6:viewport:phone",
    "4.8:dump_entity:trickster:f.json",
    "4.9:touch:1:down:TouchMove0/Move",
    "4.95:touch:1:move:82,600",
    "5.3:get:trickster.og.PlayerInput:touch.json",
    "5.5:touch:1:up:82,600",
    "5.6:dump_entity:trickster:g.json",
    "5.6:get:trickster.SeatLook:look-a.json",
    "5.7:touch:2:down:TouchActions0/Look",
    "5.75:touch:2:move:370,728",
    "6.0:get:trickster.og.PlayerInput:look.json",
    "6.2:touch:2:up:370,728",
    "6.3:get:trickster.SeatLook:look-b.json",
    ...(SEATS > 1 ? ["6.3:dump_entity:trickster-1:other-b.json"] : []),
    "6.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = value<Input>(run, "start-input.json");
    t.equal(start.player, 0, "the trickster carries seat 0's og.PlayerInput");
    t.ok((start.buttons & START) !== 0, `S held is the start control's bit (buttons ${start.buttons})`);
    t.equal(value<Mission>(run, "started.json").phase, 1, "and the run started from it");
    const w = value<Input>(run, "w.json");
    t.ok(Math.abs(w.y - 1) < 1e-3 && Math.abs(w.x) < 1e-3, `W reaches it as forward (${w.x}, ${w.y})`);
    const a = position(run, "a.json");
    const b = position(run, "b.json");
    t.ok(dist(a, b) > 0.8, `and walks the trickster (${dist(a, b).toFixed(2)} m)`);
    const c = position(run, "c.json");
    t.ok(dist(b, c) > 0.6, `the arrow key, seat 1's, held by the lone player too, walks it (${dist(b, c).toFixed(2)} m)`);
    const runIn = value<Input>(run, "run-input.json");
    t.ok((runIn.buttons & RUN) !== 0, `Shift is the run control's bit (buttons ${runIn.buttons})`);
    const speed = value<{ speed: number }>(run, "running.json").speed;
    t.ok(speed > 4.5, `and runs it (${speed.toFixed(2)} m/s)`);
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9 && Math.abs(pad.y) < 0.1, `the first pad's stick reaches it (${pad.x.toFixed(2)}, ${pad.y.toFixed(2)})`);
    const d = position(run, "d.json");
    const e = position(run, "e.json");
    t.ok(dist(d, e) > 0.6, `and walks the trickster (${dist(d, e).toFixed(2)} m)`);
    t.ok((value<Input>(run, "pad-a.json").buttons & JUMP) !== 0, "the pad's A is the jump control's bit");
    t.ok(position(run, "jump.json")[1] > 0.3, `and jumps it (feet at ${position(run, "jump.json")[1].toFixed(2)} m)`);
    const touch = value<Input>(run, "touch.json");
    t.ok(touch.y > 0.9, `the touch move stick pushed up reaches it as forward (${touch.x.toFixed(2)}, ${touch.y.toFixed(2)})`);
    const f = position(run, "f.json");
    const g = position(run, "g.json");
    t.ok(dist(f, g) > 0.8, `and walks the trickster (${dist(f, g).toFixed(2)} m)`);
    const look = value<Input>(run, "look.json");
    t.ok(look.aimX > 0.5, `the touch look stick pushed right reaches it as a turn to the right (${look.aimX.toFixed(2)}°)`);
    const yawA = value<{ yaw: number }>(run, "look-a.json").yaw;
    const yawB = value<{ yaw: number }>(run, "look-b.json").yaw;
    t.ok(yawA - yawB > 0.3, `and turns the trickster's look right (yaw ${yawA.toFixed(2)} → ${yawB.toFixed(2)})`);
    if (SEATS > 1) {
      t.equal(component<{ player: number }>(run, "other-a.json", "og.PlayerInput").player, 1, "the second walker carries seat 1's og.PlayerInput");
      t.ok(dist(position(run, "other-a.json"), position(run, "other-b.json")) < 0.05, "and stood still: this device drives seat 0 alone");
    }
  },
});

const two = playtest({
  name: "two seats on one device: seat 1's Enter, arrows and pad 2 play the second walker, seat 0's WASD the trickster",
  size: "1280x720",
  commands: [
    "0.3:key_down:Enter",
    "0.35:get:trickster-1.og.PlayerInput:start-input.json",
    "0.4:key_up:Enter",
    "0.5:get:dropship.Mission:started.json",
    "0.6:dump_entity:trickster:a0.json",
    "0.6:dump_entity:trickster-1:a1.json",
    "0.7:key_down:ArrowUp",
    "1.0:get:trickster-1.og.PlayerInput:up.json",
    "1.2:key_up:ArrowUp",
    "1.3:dump_entity:trickster:b0.json",
    "1.3:dump_entity:trickster-1:b1.json",
    "1.4:gamepad:-@1:1,0",
    "1.7:get:trickster-1.og.PlayerInput:pad.json",
    "1.9:gamepad:-@1:0,0",
    "2.0:dump_entity:trickster-1:c1.json",
    "2.1:key_down:Enter",
    "2.25:dump_entity:trickster-1:jump.json",
    "2.3:key_up:Enter",
    "2.8:dump_entity:trickster:c0.json",
    "2.8:dump_entity:trickster-1:d1.json",
    "2.9:key_down:KeyW",
    "3.2:get:trickster.og.PlayerInput:w.json",
    "3.4:key_up:KeyW",
    "3.5:dump_entity:trickster:d0.json",
    "3.5:dump_entity:trickster-1:e1.json",
    "3.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = value<Input>(run, "start-input.json");
    t.equal(start.player, 1, "the second walker carries seat 1's og.PlayerInput");
    t.ok((start.buttons & START) !== 0, `seat 1's Enter is its start bit (buttons ${start.buttons})`);
    t.equal(value<Mission>(run, "started.json").phase, 1, "and either seat starts the run");
    const up = value<Input>(run, "up.json");
    t.ok(up.y > 0.99, `seat 1's ArrowUp reaches seat 1 as forward (${up.y})`);
    t.ok(dist(position(run, "a1.json"), position(run, "b1.json")) > 0.8, "and walks the second walker");
    t.ok(dist(position(run, "a0.json"), position(run, "b0.json")) < 0.05, "while the trickster stands");
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9, `pad 2's stick reaches seat 1 (${pad.x.toFixed(2)})`);
    t.ok(dist(position(run, "b1.json"), position(run, "c1.json")) > 0.6, "and walks the second walker");
    t.ok(position(run, "jump.json")[1] > 0.3, `seat 1's Enter jumps it (feet at ${position(run, "jump.json")[1].toFixed(2)} m)`);
    t.ok(dist(position(run, "b0.json"), position(run, "c0.json")) < 0.05, "and the trickster still stands");
    const w = value<Input>(run, "w.json");
    t.ok(w.player === 0 && w.y > 0.99, "seat 0's W reaches seat 0 as forward");
    t.ok(dist(position(run, "c0.json"), position(run, "d0.json")) > 0.8, "and walks the trickster");
    t.ok(dist(position(run, "d1.json"), position(run, "e1.json")) < 0.05, "while the second walker stands");
  },
});

export default LOCAL ? two : alone;
