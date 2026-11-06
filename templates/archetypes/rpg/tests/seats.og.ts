// The seats (src/seats.ts): each hero carries its seat's og.PlayerInput,
// the only input the game reads. Alone (and online before a room runs)
// every device reaches seat 0: 1 enters the keep (choice1's bit), W walks
// the knight the view's way (on the isometric view up the screen, on the
// others where its look points), the first pad's stick walks it, Space
// swings (Attack's bit), and on a phone the touch move stick walks it, a
// finger dragged across the right half turns its look (on the isometric view, the hero itself), and
// the touch Attack swings. Two on one device (src/creation.ts players
// local-multiplayer): seat 1's Numpad1 enters, its arrows, Enter and pad 2
// play the barbarian while the knight stands, and then seat 0's WASD walks
// only the knight. Code that read the keys or the pad itself, past the
// seats, would move the wrong hero or both.
import { playtest } from "@origingame/engine/playtest";
import { game, ISOMETRIC, LOCAL, PLAY, position, script, SEATS, value, type Hero } from "./lib.ts";

interface Input { player: number; x: number; y: number; aimX: number; aimY: number; buttons: number }
type V3 = [number, number, number];

/** og.PlayerInput.buttons, in src/creation.ts's control order. */
const ATTACK = 1;
const CHOICE1 = 8;

const dist = (a: V3, b: V3) => Math.hypot(a[0] - b[0], a[2] - b[2]);
/** The ground direction W walks at the start of a run: up the screen (isometric, 45°), else the start look (src/heroes.ts START_YAW). */
const yaw0 = ISOMETRIC ? Math.PI / 4 : -0.75;
const forward = { x: -Math.sin(yaw0), z: -Math.cos(yaw0) };
const right = { x: Math.cos(yaw0), z: -Math.sin(yaw0) };
const along = (a: V3, b: V3, d: { x: number; z: number }) => (b[0] - a[0]) * d.x + (b[2] - a[2]) * d.z;
const turned = (a: number, b: number) => Math.abs(Math.atan2(Math.sin(a - b), Math.cos(a - b)));

const alone = playtest({
  name: "1, W, the pad, Space and the touch sticks and Attack reach seat 0's og.PlayerInput and play the knight",
  size: "1280x720",
  commands: script(
    "0.3:key_down:Digit1",
    "0.35:get:hero.og.PlayerInput:start-input.json",
    "0.4:key_up:Digit1",
    "0.5:get:Game:started.json",
    "0.6:get:hero.og.Transform:a.json",
    ...(SEATS > 1 ? ["0.6:get:hero-1.og.Transform:other-a.json"] : []),
    "0.7:key_down:KeyW",
    "0.9:get:hero.og.PlayerInput:w.json",
    "1.1:key_up:KeyW",
    "1.3:get:hero.og.Transform:b.json",
    "1.4:gamepad:-@0:1,0",
    "1.7:get:hero.og.PlayerInput:pad.json",
    "1.9:gamepad:-@0:0,0",
    "2.0:get:hero.og.Transform:c.json",
    "2.1:key_down:Space",
    "2.15:get:hero.og.PlayerInput:space.json",
    "2.2:key_up:Space",
    "2.4:get:hero.Hero:swung.json",
    "2.9:viewport:phone",
    "3.1:get:hero.og.Transform:d.json",
    "3.2:touch:1:down:Move0/Move",
    "3.25:touch:1:move:82,600",
    "3.6:get:hero.og.PlayerInput:touch.json",
    "3.8:touch:1:up:82,600",
    "3.9:get:hero.og.Transform:e.json",
    "3.9:get:hero.SeatLook:look-a.json",
    "3.9:get:hero.Hero:face-a.json",
    "4.0:touch:2:down:240,330",
    "4.05:touch:2:move:250,330",
    "4.1:touch:2:move:280,330",
    "4.15:touch:2:move:300,330",
    "4.5:touch:2:up:300,330",
    "4.6:get:hero.SeatLook:look-b.json",
    "4.6:get:hero.Hero:face-b.json",
    "4.7:touch_move_to:Hands0/Small/Attack",
    "4.72:touch_start",
    "4.8:get:hero.og.PlayerInput:tap.json",
    "4.84:touch_end",
    "5.0:get:hero.Hero:tapped.json",
    ...(SEATS > 1 ? ["5.0:get:hero-1.og.Transform:other-b.json"] : []),
    "5.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = value<Input>(run, "start-input.json");
    t.equal(start.player, 0, "the knight carries seat 0's og.PlayerInput");
    t.ok((start.buttons & CHOICE1) !== 0, `1 held is choice1's bit (buttons ${start.buttons})`);
    t.equal(game(run, "started.json").phase, PLAY, "and the run started from it");
    const w = value<Input>(run, "w.json");
    t.ok(Math.abs(w.y - 1) < 1e-3 && Math.abs(w.x) < 1e-3, `W reaches it as forward (${w.x}, ${w.y})`);
    const a = position(run, "a.json");
    const b = position(run, "b.json");
    t.ok(along(a, b, forward) > 0.8, `and walks the knight ${ISOMETRIC ? "up the screen" : "where it looks"} (${along(a, b, forward).toFixed(2)} m)`);
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9 && Math.abs(pad.y) < 0.1, `the first pad's stick reaches it (${pad.x.toFixed(2)}, ${pad.y.toFixed(2)})`);
    const c = position(run, "c.json");
    t.ok(along(b, c, right) > 0.6, `and walks the knight to the right (${along(b, c, right).toFixed(2)} m)`);
    t.ok((value<Input>(run, "space.json").buttons & ATTACK) !== 0, "Space is Attack's bit");
    t.equal(value<Hero>(run, "swung.json").swings, 1, "and the knight swings");
    const touch = value<Input>(run, "touch.json");
    t.ok(touch.y > 0.9, `the touch move stick pushed up reaches it as forward (${touch.x.toFixed(2)}, ${touch.y.toFixed(2)})`);
    const d = position(run, "d.json");
    const e = position(run, "e.json");
    t.ok(dist(d, e) > 0.8, `and walks the knight (${dist(d, e).toFixed(2)} m)`);
    const yawA = value<{ yaw: number }>(run, "look-a.json").yaw;
    const yawB = value<{ yaw: number }>(run, "look-b.json").yaw;
    t.ok(turned(yawA, yawB) > 0.3, `a finger dragged across the right half turns the knight's look through its og.PlayerInput (ViewRig reads nothing else)${ISOMETRIC ? ", and with it the knight" : ""} (yaw ${yawA.toFixed(2)} → ${yawB.toFixed(2)})`);
    if (ISOMETRIC) {
      const fa = value<Hero>(run, "face-a.json").facing;
      const fb = value<Hero>(run, "face-b.json").facing;
      t.ok(turned(fa, fb) > 0.3, `on the isometric view the look turns the hero itself (facing ${fa.toFixed(2)} → ${fb.toFixed(2)})`);
    }
    t.ok((value<Input>(run, "tap.json").buttons & ATTACK) !== 0, "the touch Attack is Attack's bit");
    t.equal(value<Hero>(run, "tapped.json").swings, 2, "and swings again");
    if (SEATS > 1) {
      t.ok(dist(position(run, "other-a.json"), position(run, "other-b.json")) < 0.05, "the barbarian stood still: this device drives seat 0 alone");
    }
  },
});

const two = playtest({
  name: "two seats on one device: seat 1's Numpad1, arrows, Enter and pad 2 play the barbarian, seat 0's WASD the knight",
  size: "1280x720",
  commands: script(
    "0.3:key_down:Numpad1",
    "0.35:get:hero-1.og.PlayerInput:start-input.json",
    "0.4:key_up:Numpad1",
    "0.5:get:Game:started.json",
    "0.6:get:hero.og.Transform:a0.json",
    "0.6:get:hero-1.og.Transform:a1.json",
    "0.7:key_down:ArrowUp",
    "1.0:get:hero-1.og.PlayerInput:up.json",
    "1.2:key_up:ArrowUp",
    "1.3:get:hero.og.Transform:b0.json",
    "1.3:get:hero-1.og.Transform:b1.json",
    "1.4:gamepad:-@1:1,0",
    "1.7:get:hero-1.og.PlayerInput:pad.json",
    "1.9:gamepad:-@1:0,0",
    "2.0:get:hero-1.og.Transform:c1.json",
    "2.1:key_down:Enter",
    "2.15:get:hero-1.og.PlayerInput:enter.json",
    "2.2:key_up:Enter",
    "2.4:get:hero-1.Hero:swung1.json",
    "2.4:get:hero.Hero:still0.json",
    "2.8:get:hero.og.Transform:c0.json",
    "2.8:get:hero-1.og.Transform:d1.json",
    "2.9:key_down:KeyW",
    "3.2:get:hero.og.PlayerInput:w.json",
    "3.4:key_up:KeyW",
    "3.5:get:hero.og.Transform:d0.json",
    "3.5:get:hero-1.og.Transform:e1.json",
    "3.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = value<Input>(run, "start-input.json");
    t.equal(start.player, 1, "the barbarian carries seat 1's og.PlayerInput");
    t.ok((start.buttons & CHOICE1) !== 0, `seat 1's Numpad1 is its choice1 bit (buttons ${start.buttons})`);
    t.equal(game(run, "started.json").phase, PLAY, "and either seat enters the keep");
    const up = value<Input>(run, "up.json");
    t.ok(up.y > 0.99, `seat 1's ArrowUp reaches seat 1 as forward (${up.y})`);
    t.ok(dist(position(run, "a1.json"), position(run, "b1.json")) > 0.8, "and walks the barbarian");
    t.ok(dist(position(run, "a0.json"), position(run, "b0.json")) < 0.05, "while the knight stands");
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9, `pad 2's stick reaches seat 1 (${pad.x.toFixed(2)})`);
    t.ok(dist(position(run, "b1.json"), position(run, "c1.json")) > 0.6, "and walks the barbarian");
    t.ok((value<Input>(run, "enter.json").buttons & ATTACK) !== 0, "seat 1's Enter is its Attack bit");
    t.equal(value<Hero>(run, "swung1.json").swings, 1, "and the barbarian swings");
    t.equal(value<Hero>(run, "still0.json").swings, 0, "the knight does not");
    t.ok(dist(position(run, "b0.json"), position(run, "c0.json")) < 0.05, "and still stands");
    const w = value<Input>(run, "w.json");
    t.ok(w.player === 0 && w.y > 0.99, "seat 0's W reaches seat 0 as forward");
    t.ok(dist(position(run, "c0.json"), position(run, "d0.json")) > 0.8, "and walks the knight");
    t.ok(dist(position(run, "d1.json"), position(run, "e1.json")) < 0.05, "while the barbarian stands");
  },
});

export default LOCAL ? two : alone;
