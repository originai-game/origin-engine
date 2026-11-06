// The seats (src/seats.ts): the keys, a pad and the touch hand reach a
// seat's og.PlayerInput, and the rules (src/run.ts) act on that alone.
// Alone, seat 0 takes every device: an arrow and the pad's d-pad walk the
// hand, the pad's A and Space play the focused card, two taps on a card of
// the phone's hand play it and a tap on End turn ends the turn. Two on one
// device, seat 0 plays with WASD, Space and pad 1 and seat 1 with the
// arrows, Enter and pad 2, each moving its own hand while the other's
// stays put. Last, scripted input given to the seats' og.PlayerInput with
// no device touched (bot) plays cards too: a game reading the keys itself
// would not. The whole script runs on a phone's layout (390 × 844), as a
// phone in hand would: its keys and pads still reach the same seats.
import { playtest } from "@origingame/engine/playtest";
import { component, game, hero, key, script, tap, TWO } from "./lib.ts";

const single = [
  "0.1:viewport:phone",
  "0.3:goto:0",
  "0.5:energy:9",
  "0.6:key_down:ArrowRight",
  "0.64:dump_entity:hero:keys-input.json",
  "0.65:key_up:ArrowRight",
  "0.8:dump_entity:hero:keys.json",
  "1.0:gamepad:right",
  "1.04:dump_entity:hero:pad-input.json",
  "1.05:gamepad:off",
  "1.2:dump_entity:hero:pad.json",
  "1.3:gamepad:a",
  "1.35:gamepad:off",
  "1.6:get:Game:pad-played.json",
  key("Space", 1.7),
  "1.9:get:Game:space-played.json",
  "2.0:give:defend",
  tap("Hands/Seat0/Hand0/Fan/0-10", 2.4),
  tap("Hands/Seat0/Hand0/Fan/0-10", 2.8),
  "3.2:get:Game:tap-played.json",
  "3.2:dump_entity:hero:tapped.json",
  tap("Hands/Seat0/Bar/End", 3.3),
  "3.5:get:Game:ended.json",
  "5.4:get:Game:bot-before.json",
  "5.4:bot:1",
  "7.4:bot:0",
  "7.5:get:Game:bot.json",
  "7.6:quit",
];

const two = [
  "0.1:viewport:phone",
  "0.3:goto:0",
  "0.5:energy:9",
  "0.5:energy:9:1",
  "0.6:key_down:ArrowRight",
  "0.64:dump_entity:hero-1:arrow-input.json",
  "0.64:dump_entity:hero:arrow-input0.json",
  "0.65:key_up:ArrowRight",
  "0.8:dump_entity:hero:arrow0.json",
  "0.8:dump_entity:hero-1:arrow1.json",
  key("KeyD", 0.9),
  "1.1:dump_entity:hero:wasd0.json",
  "1.2:gamepad:-@1:1,0",
  "1.24:dump_entity:hero-1:pad2-input.json",
  "1.25:gamepad:off@1",
  "1.4:dump_entity:hero-1:pad2.json",
  "1.4:dump_entity:hero:pad2-0.json",
  key("Enter", 1.5),
  "1.7:get:Game:enter.json",
  "1.7:dump_entity:hero-1:enter1.json",
  "1.7:dump_entity:hero:enter0.json",
  key("Space", 1.8),
  "2.0:get:Game:space.json",
  "2.1:give:defend:1",
  tap("Hands/Seat1/Hand1/Fan/1-10", 2.4),
  tap("Hands/Seat1/Hand1/Fan/1-10", 2.8),
  "3.2:get:Game:tap.json",
  tap("Hands/Seat0/Bar/End", 3.3),
  "3.5:get:Game:one-ended.json",
  tap("Hands/Seat1/Bar/End", 3.6),
  "3.8:get:Game:ended.json",
  "5.4:get:Game:bot-before.json",
  "5.4:bot:1",
  "7.4:bot:0",
  "7.5:get:Game:bot.json",
  "7.6:quit",
];

export default playtest({
  name: "the keys, a pad and the touch hand reach each seat's og.PlayerInput and play its own cards",
  commands: script(TWO ? two : single),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    if (!TWO) {
      const keys = component<{ player: number; x: number }>(run, "keys-input.json", "og.PlayerInput");
      t.ok(keys.player === 0 && keys.x === 1, `ArrowRight held: seat 0's og.PlayerInput moves right (player ${keys.player}, x ${keys.x})`);
      t.equal(hero(run, "keys.json").focusCard, 1, "and the focus moved to the second card");
      t.equal(component<{ x: number }>(run, "pad-input.json", "og.PlayerInput").x, 1, "the pad's d-pad right: seat 0's og.PlayerInput moves right");
      t.equal(hero(run, "pad.json").focusCard, 2, "and the focus moved again");
      t.equal(game(run, "pad-played.json").played, 1, "the pad's A played the focused card");
      t.equal(game(run, "space-played.json").played, 2, "Space played the next");
      t.equal(game(run, "tap-played.json").played, 3, "two taps on a card of the phone's hand played it");
      t.ok(!hero(run, "tapped.json").hand.includes(10), "the tapped Defend left the hand");
      t.equal(game(run, "ended.json").side, 1, "a tap on End turn ended the turn: the foes act");
    } else {
      const a1 = component<{ player: number; x: number }>(run, "arrow-input.json", "og.PlayerInput");
      const a0 = component<{ x: number }>(run, "arrow-input0.json", "og.PlayerInput");
      t.ok(a1.player === 1 && a1.x === 1 && a0.x === 0, `ArrowRight held: seat 1's og.PlayerInput moves right, seat 0's does not (${a1.x}, ${a0.x})`);
      t.equal(`${hero(run, "arrow0.json").focusCard} ${hero(run, "arrow1.json").focusCard}`, "0 1", "the Hunter's focus moved; the Warden's stayed");
      t.equal(hero(run, "wasd0.json").focusCard, 1, "D moved the Warden's focus");
      t.equal(component<{ x: number }>(run, "pad2-input.json", "og.PlayerInput").x, 1, "pad 2's stick right: seat 1's og.PlayerInput moves right");
      t.equal(`${hero(run, "pad2-0.json").focusCard} ${hero(run, "pad2.json").focusCard}`, "1 2", "pad 2 moved the Hunter's focus; the Warden's stayed");
      t.equal(game(run, "enter.json").played, 1, "Enter played a card");
      t.equal(`${hero(run, "enter0.json").hand.length} ${hero(run, "enter1.json").hand.length}`, "5 4", "from the Hunter's hand");
      t.equal(game(run, "space.json").played, 2, "Space played one of the Warden's");
      t.equal(game(run, "tap.json").played, 3, "two taps on the Hunter's phone hand played its card");
      t.equal(game(run, "one-ended.json").side, 0, "the Warden's End turn: the foes wait for the Hunter");
      t.equal(game(run, "ended.json").side, 1, "the Hunter's End turn too: the foes act");
    }
    const before = game(run, "bot-before.json");
    const after = game(run, "bot.json");
    t.ok(after.played > before.played, `scripted input on the seats' og.PlayerInput alone played cards (${before.played} → ${after.played})`);
  },
});
