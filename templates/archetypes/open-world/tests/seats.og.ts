// The seats (src/seats.ts): the keys, a pad and the touch HUD reach a
// seat's og.PlayerInput, and the explorer moves from that alone, along its
// own view (src/view.ts: behind it, through its eyes, or the fixed
// isometric angle). Alone (and online, this device's seat 0): W, the pad's
// stick and the touch stick each walk seat 0's explorer forward, the right
// stick turns its view (in isometric, the explorer), and Gather held at a
// tree fells it. Two on one device: WASD and pad 1 are seat 0's, the
// arrows, `/` and pad 2 seat 1's, each moving its own explorer while the
// other's stays put, and each half of a phone has its own stick.
import { playtest } from "@origingame/engine/playtest";
import { forward, game, LOCAL, position, script, value, VIEW } from "./lib.ts";

interface Input { player: number; x: number; y: number; aimX: number; buttons: number }

const along = (a: [number, number, number], b: [number, number, number], f: [number, number]) => (b[0] - a[0]) * f[0] + (b[2] - a[2]) * f[1];
const moved = (a: [number, number, number], b: [number, number, number]) => Math.hypot(b[0] - a[0], b[2] - a[2]);
const ISO = VIEW === "isometric-2.5d";

const single = [
  "0.3:run",
  "0.6:get:hero.SeatLook:look-a.json",
  "0.6:get:hero.og.Transform:a.json",
  "0.7:key_down:KeyW",
  "1.1:get:hero.og.PlayerInput:keys.json",
  "1.5:key_up:KeyW",
  "1.7:get:hero.og.Transform:b.json",
  "1.7:get:hero.SeatLook:look-b.json",
  "1.8:gamepad:-@0:0,-1",
  "2.2:get:hero.og.PlayerInput:pad.json",
  "2.6:gamepad:-@0:0,0",
  "2.8:get:hero.og.Transform:c.json",
  "2.8:get:hero.SeatLook:look-c.json",
  "2.8:get:hero.Hero:face-c.json",
  "2.9:gamepad:-@0:0,0,1,0",
  "3.4:gamepad:-@0:0,0,0,0",
  "3.5:get:hero.SeatLook:look-d.json",
  "3.5:get:hero.Hero:face-d.json",
  "3.6:viewport:phone",
  "3.8:get:hero.SeatLook:look-e.json",
  "3.8:get:hero.og.Transform:e.json",
  "3.9:touch_move_to:Stick",
  "3.92:touch_start",
  "3.95:touch_move:86,664",
  "4.3:get:hero.og.PlayerInput:touch.json",
  "4.6:touch_end",
  "4.8:get:hero.og.Transform:f.json",
  "4.9:warp:tree",
  "5.1:touch_move_to:Gather",
  "5.12:touch_start",
  "6.6:touch_end",
  "6.7:get:Game:felled.json",
  "6.8:quit",
];

const two = [
  "0.3:run",
  "0.6:get:hero.SeatLook:look0.json",
  "0.6:get:hero-1.SeatLook:look1.json",
  "0.6:get:hero.og.Transform:a0.json",
  "0.6:get:hero-1.og.Transform:a1.json",
  "0.7:key_down:ArrowUp",
  "1.1:get:hero-1.og.PlayerInput:arrow1.json",
  "1.1:get:hero.og.PlayerInput:arrow0.json",
  "1.5:key_up:ArrowUp",
  "1.7:get:hero.og.Transform:b0.json",
  "1.7:get:hero-1.og.Transform:b1.json",
  "1.8:key_down:KeyW",
  "2.6:key_up:KeyW",
  "2.8:get:hero.og.Transform:c0.json",
  "2.8:get:hero-1.og.Transform:c1.json",
  "2.8:get:hero-1.SeatLook:look1c.json",
  "2.9:gamepad:-@1:0,-1",
  "3.3:get:hero-1.og.PlayerInput:pad2.json",
  "3.7:gamepad:-@1:0,0",
  "3.9:get:hero.og.Transform:d0.json",
  "3.9:get:hero-1.og.Transform:d1.json",
  "4.0:warp_seat:1:tree",
  "4.2:key_down:Slash",
  "5.7:key_up:Slash",
  "5.8:get:Game:felled.json",
  "5.9:viewport:phone",
  "6.1:get:hero-1.og.Transform:e1.json",
  "6.1:get:hero.og.Transform:e0.json",
  "6.2:touch_move_to:TouchSeat1/Stick",
  "6.22:touch_start",
  "6.25:touch_move:308,668",
  "6.6:get:hero-1.og.PlayerInput:touch1.json",
  "6.9:touch_end",
  "7.1:get:hero-1.og.Transform:f1.json",
  "7.1:get:hero.og.Transform:f0.json",
  "7.2:quit",
];

export default playtest({
  name: "the keys, a pad and the touch HUD reach each seat's og.PlayerInput and move its own explorer along its view",
  size: "1280x720",
  commands: script(LOCAL ? two : single),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    if (!LOCAL) {
      const keys = value<Input>(run, "keys.json");
      t.ok(keys.player === 0 && Math.abs(keys.y - 1) < 1e-3, `W held: seat 0's og.PlayerInput moves forward (player ${keys.player}, y ${keys.y})`);
      const ab = along(position(run, "a.json"), position(run, "b.json"), forward(run, "look-a.json"));
      t.ok(ab > 1.5, `and the explorer walked ${ab.toFixed(2)} m forward along its view (${VIEW})`);
      const pad = value<Input>(run, "pad.json");
      t.ok(pad.y > 0.9, `the pad's stick pushed up reaches it as forward (${pad.y.toFixed(2)})`);
      const bc = along(position(run, "b.json"), position(run, "c.json"), forward(run, "look-b.json"));
      t.ok(bc > 1.5, `and walked the explorer ${bc.toFixed(2)} m forward`);
      if (ISO) {
        const before = value<{ facing: number }>(run, "face-c.json").facing;
        const after = value<{ facing: number }>(run, "face-d.json").facing;
        t.ok(Math.abs(after - before) > 0.3, `the right stick turns the explorer (the isometric angle stays): facing ${before.toFixed(2)} → ${after.toFixed(2)}`);
      } else {
        const before = value<{ yaw: number }>(run, "look-c.json").yaw;
        const after = value<{ yaw: number }>(run, "look-d.json").yaw;
        t.ok(before - after > 0.3, `the right stick turns the view right (yaw ${before.toFixed(2)} → ${after.toFixed(2)})`);
      }
      const touch = value<Input>(run, "touch.json");
      t.ok(touch.player === 0 && touch.y > 0.5, `the touch stick pushed up reaches seat 0 as forward (${touch.y.toFixed(2)})`);
      const ef = along(position(run, "e.json"), position(run, "f.json"), forward(run, "look-e.json"));
      t.ok(ef > 1, `and walked the explorer ${ef.toFixed(2)} m forward`);
      const felled = game(run, "felled.json");
      t.ok(felled.felled === 1 && felled.wood === 2, `the touch HUD's Gather held at a tree felled it (${felled.felled}, wood ${felled.wood})`);
      return;
    }
    const arrow1 = value<Input>(run, "arrow1.json");
    const arrow0 = value<Input>(run, "arrow0.json");
    t.ok(arrow1.player === 1 && arrow1.y === 1 && arrow0.y === 0, `ArrowUp held: seat 1's og.PlayerInput moves forward, seat 0's does not (${arrow1.y}, ${arrow0.y})`);
    t.ok(along(position(run, "a1.json"), position(run, "b1.json"), forward(run, "look1.json")) > 1.5, "the arrows walk seat 1's explorer forward along its view");
    t.ok(moved(position(run, "a0.json"), position(run, "b0.json")) < 0.05, "and seat 0's stays put");
    t.ok(along(position(run, "b0.json"), position(run, "c0.json"), forward(run, "look0.json")) > 1.5, "W walks seat 0's explorer forward");
    t.ok(moved(position(run, "b1.json"), position(run, "c1.json")) < 0.05, "and seat 1's stays put");
    t.ok(value<Input>(run, "pad2.json").y > 0.9, "pad 2's stick pushed up reaches seat 1's og.PlayerInput");
    t.ok(along(position(run, "c1.json"), position(run, "d1.json"), forward(run, "look1c.json")) > 1.5, "and walks seat 1's explorer");
    t.ok(moved(position(run, "c0.json"), position(run, "d0.json")) < 0.05, "while seat 0's stays put");
    const felled = game(run, "felled.json");
    t.ok(felled.felled === 1 && felled.wood === 2, `seat 1's / held at a tree felled it into the shared bag (${felled.felled}, wood ${felled.wood})`);
    const touch1 = value<Input>(run, "touch1.json");
    t.ok(touch1.player === 1 && touch1.y > 0.5, `seat 1's own touch stick, in its corner of the phone, reaches seat 1 (${touch1.y.toFixed(2)})`);
    t.ok(moved(position(run, "e1.json"), position(run, "f1.json")) > 1, "and moves seat 1's explorer");
    t.ok(moved(position(run, "e0.json"), position(run, "f0.json")) < 0.05, "and not seat 0's");
  },
});
