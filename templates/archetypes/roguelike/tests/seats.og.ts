// The seats (src/seats.ts): the keys, a pad and the touch d-pad reach a
// seat's og.PlayerInput, and the Hero system steps from that alone. Seed
// 7 starts the lamp-bearer at (4, 15) under a corridor running up. Alone,
// seat 0 takes every device: an arrow, the pad's d-pad, then on a phone
// the HUD's d-pad each take one step up. Two on one device, seat 0 walks
// with WASD and pad 1 and seat 1 with the arrows and pad 2, each moving
// its own delver while the other's stays put, and the world's turn comes
// once both have acted.
import { playtest } from "@origingame/engine/playtest";
import { component, game, key, partner, press, script, tap, TWO } from "./lib.ts";

const single = [
  "0.3:dive:7",
  "0.5:get:Game:start.json",
  "0.6:key_down:ArrowUp",
  "0.64:dump_entity:hero:keys-input.json",
  "0.65:key_up:ArrowUp",
  "0.9:get:Game:keys.json",
  "1.0:gamepad:up",
  "1.04:dump_entity:hero:pad-input.json",
  "1.05:gamepad:off",
  "1.3:get:Game:pad.json",
  "1.4:viewport:phone",
  tap("Pad/Up", 1.6),
  "1.7:touch_move:10,400",
  "2.0:get:Game:touch.json",
  press("ArrowDown", 2.1),
  "2.4:get:Game:back.json",
  "2.5:quit",
];

// Seat 0 at (4, 13) up the corridor, seat 1 at (4, 15) below it: both step up.
const two = [
  "0.3:dive:7",
  "0.4:warp:4:13:0",
  "0.4:warp:4:15:1",
  "0.5:get:Game:start.json",
  "0.6:key_down:ArrowUp",
  "0.64:dump_entity:hero-1:arrow-input.json",
  "0.64:dump_entity:hero:arrow-input0.json",
  "0.65:key_up:ArrowUp",
  "0.9:get:Game:arrow.json",
  key("KeyW", 1.0),
  "1.3:get:Game:wasd.json",
  "1.4:gamepad:-@1:0,-1",
  "1.44:dump_entity:hero-1:pad2-input.json",
  "1.45:gamepad:off@1",
  "1.7:get:Game:pad2.json",
  "1.8:gamepad:up",
  "1.85:gamepad:off",
  "2.1:get:Game:pad1.json",
  "2.2:viewport:phone",
  tap("Seat0/Pad/Up", 2.4) + partner(2.4),
  "2.5:touch_move:10,400",
  "2.8:get:Game:touch.json",
  "2.9:quit",
];

export default playtest({
  name: "the keys, a pad and the touch d-pad reach each seat's og.PlayerInput and step its own delver",
  commands: script(TWO ? two : single),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const start = game(run, "start.json");
    if (!TWO) {
      t.equal(`${start.heroX},${start.heroY}`, "4,15", "seed 7 starts the lamp-bearer at (4, 15)");
      const keys = component<{ player: number; y: number }>(run, "keys-input.json", "og.PlayerInput");
      t.ok(keys.player === 0 && keys.y === 1, `ArrowUp held: seat 0's og.PlayerInput moves up (player ${keys.player}, y ${keys.y})`);
      t.equal(game(run, "keys.json").heroY, 14, "and the lamp-bearer stepped up");
      const pad = component<{ y: number }>(run, "pad-input.json", "og.PlayerInput");
      t.equal(pad.y, 1, "the pad's d-pad held: seat 0's og.PlayerInput moves up");
      t.equal(game(run, "pad.json").heroY, 13, "and it stepped up again");
      const touch = game(run, "touch.json");
      t.equal(touch.heroY, 12, "the phone HUD's d-pad stepped it up a third time");
      t.equal(touch.turns, 3, "three steps, three turns: alone, every step is the world's turn");
      t.equal(game(run, "back.json").heroY, 13, "and ArrowDown, still seat 0's on the phone, stepped it back");
      return;
    }
    t.equal(`${start.heroX},${start.heroY} ${start.hero2X},${start.hero2Y}`, "4,13 4,15", "seat 0 up the corridor, seat 1 below it");
    const arrowIn = component<{ player: number; y: number }>(run, "arrow-input.json", "og.PlayerInput");
    const arrowIn0 = component<{ y: number }>(run, "arrow-input0.json", "og.PlayerInput");
    t.ok(arrowIn.player === 1 && arrowIn.y === 1 && arrowIn0.y === 0, `ArrowUp held: seat 1's og.PlayerInput moves up, seat 0's does not (${arrowIn.y}, ${arrowIn0.y})`);
    const arrow = game(run, "arrow.json");
    t.equal(`${arrow.heroX},${arrow.heroY} ${arrow.hero2X},${arrow.hero2Y}`, "4,13 4,14", "the arrow stepped seat 1's delver; seat 0's stayed put");
    t.equal(arrow.turns, 0, "one delver acted: the world's turn waits for the other");
    const wasd = game(run, "wasd.json");
    t.equal(`${wasd.heroX},${wasd.heroY} ${wasd.hero2X},${wasd.hero2Y}`, "4,12 4,14", "W stepped seat 0's delver; seat 1's stayed put");
    t.equal(wasd.turns, 1, "both acted: the world's turn");
    t.equal(component<{ y: number }>(run, "pad2-input.json", "og.PlayerInput").y, 1, "pad 2's stick up: seat 1's og.PlayerInput moves up");
    const pad2 = game(run, "pad2.json");
    t.equal(`${pad2.heroX},${pad2.heroY} ${pad2.hero2X},${pad2.hero2Y}`, "4,12 4,13", "pad 2 stepped seat 1's delver; seat 0's stayed put");
    const pad1 = game(run, "pad1.json");
    t.equal(`${pad1.heroX},${pad1.heroY} ${pad1.hero2X},${pad1.hero2Y}`, "4,11 4,13", "pad 1 stepped seat 0's delver; seat 1's stayed put");
    t.equal(pad1.turns, 2, "the second world's turn");
    const touch = game(run, "touch.json");
    t.equal(`${touch.heroX},${touch.heroY} ${touch.hero2X},${touch.hero2Y}`, "4,10 4,13", "seat 0's touch d-pad stepped its delver; seat 1 waited");
    t.equal(touch.turns, 3, "the third world's turn");
  },
});
