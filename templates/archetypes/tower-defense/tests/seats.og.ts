// The seats (src/seats.ts): each cursor entity carries its seat's
// og.PlayerInput, the only input the game reads. Alone (and online before a
// room runs) every device reaches seat 0: the keys (seat 2's too, which a
// lone player also holds) step the cursor and pick the plant, the pad's
// d-pad and A step it and plant, a tap on the lawn arrives as the `tap` bit
// at the aim point, and a tray card on a phone presses its plant's
// control. Two on one device (src/creation.ts players local-multiplayer):
// seat 2's arrows, Period, Enter and pad 2 move its own cursor and plant
// from its own pick, while seat 1's cursor stays put; then seat 1's WASD,
// 3, Space and a tap move only seat 1's.
import { playtest } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { type Game, game, press, tap, tapCell } from "./lib.ts";

interface Input {
  player: number;
  buttons: number;
  aimX: number;
  aimY: number;
}

interface Seats extends Game {
  selected2: number;
  cursor2X: number;
  cursor2Y: number;
}

const alone = playtest({
  name: "keys, the pad, a tap and a touch card reach seat 0's og.PlayerInput and play the cursor",
  commands: [
    "0.3:begin",
    "0.5:get:Game:start.json",
    "0.6:key_down:KeyQ",
    "0.65:get:cursor.og.PlayerInput:held.json",
    "0.7:key_up:KeyQ",
    press("ArrowRight", 0.8),
    press("KeyD", 0.9),
    press("Numpad3", 1.0),
    "1.1:get:Game:keys.json",
    "1.2:gamepad:up",
    "1.3:gamepad:a",
    "1.4:gamepad:off",
    "1.5:get:Game:pad.json",
    tapCell(2, 7, 1.6),
    "1.7:get:cursor.og.PlayerInput:aim.json",
    "1.8:get:Game:tap.json",
    "1.9:viewport:phone",
    tap("Plant#1/Pick", 2.0),
    "2.1:touch_move:195,400",
    "2.3:get:Game:card.json",
    "2.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const held = run.json<{ value: Input }>("held.json").value;
    t.equal(held.player, 0, "the cursor carries seat 0's og.PlayerInput");
    t.equal(held.buttons & 4, 4, "Q held is the cycle control's bit (src/creation.ts) in og.PlayerInput.buttons");
    const start = game(run, "start.json");
    const keys = game(run, "keys.json");
    t.ok(keys.cursorX === start.cursorX + 2 && keys.cursorY === start.cursorY, `the arrow and D stepped the cursor two cells right (${start.cursorX},${start.cursorY} to ${keys.cursorX},${keys.cursorY})`);
    t.equal(keys.selected, 2, "Q cycled to the frost lily and seat 2's Numpad3, held by the lone player too, picked the hedge");
    const pad = game(run, "pad.json");
    t.ok(pad.cursorX === keys.cursorX && pad.cursorY === keys.cursorY - 1, `the d-pad stepped the cursor up (${pad.cursorX},${pad.cursorY})`);
    t.equal(pad.placed, 1, "the pad's A set a plant under it");
    const aim = run.json<{ value: Input }>("aim.json").value;
    t.ok(Math.floor(aim.aimX) === 2 && 12 - Math.floor(aim.aimY) === 7, `the tap's aim point is on cell (2, 7) (${aim.aimX.toFixed(2)}, ${aim.aimY.toFixed(2)})`);
    const tapped = game(run, "tap.json");
    t.ok(tapped.cursorX === 2 && tapped.cursorY === 7, `the tap moved the cursor to (2, 7) (${tapped.cursorX},${tapped.cursorY})`);
    t.equal(tapped.placed, 2, "and set a hedge there");
    t.equal(game(run, "card.json").selected, 1, "a tap on the phone tray's frost lily card picked it through og.PlayerInput");
  },
});

const two = playtest({
  name: "two seats on one device: seat 2's keys and pad 2 play its own cursor and plant, seat 1's WASD, Space and a tap its own",
  commands: [
    "0.3:begin",
    "0.5:get:Game:start.json",
    "0.5:get:cursor-2.og.PlayerInput:seat2.json",
    press("ArrowRight", 0.8),
    press("ArrowDown", 0.9),
    press("Period", 1.0),
    press("Enter", 1.1),
    "1.2:get:Game:keys2.json",
    "1.3:gamepad:-@1:1,0",
    "1.4:gamepad:-@1:0,0",
    "1.5:gamepad:-@1:0,-1",
    "1.6:gamepad:a@1",
    "1.7:gamepad:off@1",
    "1.8:get:Game:pad2.json",
    press("KeyD", 1.9),
    press("Digit3", 1.95),
    press("Space", 2.0),
    "2.1:get:Game:keys1.json",
    tapCell(2, 7, 2.2),
    "2.4:get:Game:tap1.json",
    "2.5:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.json<{ value: Input }>("seat2.json").value.player, 1, "the second cursor carries seat 1's og.PlayerInput");
    const g = (f: string) => run.json<{ value: Seats }>(f).value;
    const start = g("start.json");
    const k2 = g("keys2.json");
    t.ok(k2.cursor2X === start.cursor2X + 1 && k2.cursor2Y === start.cursor2Y + 1, `seat 2's arrows stepped its cursor right and down (${start.cursor2X},${start.cursor2Y} to ${k2.cursor2X},${k2.cursor2Y})`);
    t.equal(k2.selected2, 1, "seat 2's Period cycled its plant to the frost lily");
    t.equal(k2.placed, 1, "seat 2's Enter set it");
    t.equal(k2.gold, 150 - 70, "a frost lily paid for from the shared purse");
    t.ok(k2.cursorX === start.cursorX && k2.cursorY === start.cursorY && k2.selected === 0, `seat 1's cursor and plant stayed put (${k2.cursorX},${k2.cursorY})`);
    const p2 = g("pad2.json");
    t.ok(p2.cursor2X === k2.cursor2X + 1 && p2.cursor2Y === k2.cursor2Y - 1, `pad 2's stick stepped seat 2's cursor right and up (${p2.cursor2X},${p2.cursor2Y})`);
    t.equal(p2.placed, 2, "pad 2's A set seat 2's frost lily");
    t.ok(p2.cursorX === start.cursorX && p2.cursorY === start.cursorY, `seat 1's cursor stayed put (${p2.cursorX},${p2.cursorY})`);
    const k1 = g("keys1.json");
    t.ok(k1.cursorX === start.cursorX + 1 && k1.cursorY === start.cursorY && k1.selected === 2, `seat 1's D and 3 stepped its cursor and picked the hedge (${k1.cursorX},${k1.cursorY})`);
    t.equal(k1.placed, 3, "seat 1's Space set the hedge");
    t.ok(k1.cursor2X === p2.cursor2X && k1.cursor2Y === p2.cursor2Y && k1.selected2 === 1, `seat 2's cursor and plant stayed put (${k1.cursor2X},${k1.cursor2Y})`);
    const t1 = g("tap1.json");
    t.ok(t1.cursorX === 2 && t1.cursorY === 7, `seat 1's tap moved its cursor to (2, 7) (${t1.cursorX},${t1.cursorY})`);
    t.ok(t1.cursor2X === p2.cursor2X && t1.cursor2Y === p2.cursor2Y, "and not seat 2's");
  },
});

export default (creation.players as string) === "local-multiplayer" ? two : alone;
