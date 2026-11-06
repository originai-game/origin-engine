// A run's first turns, by taps and keys: Begin and the first fight by
// taps; the hand is five cards from a ten-card deck and three energy; the
// d-pad walks the hand, the play key plays the focused card (its cost
// spent, the card on the discard pile); End turn hands the turn to the
// foes, who do what their intents showed, and the heroes' next turn deals
// a fresh hand, the last one discarded, the energy back.
import { playtest } from "@origingame/engine/playtest";
import { costOf } from "../src/cards.ts";
import { BATTLE, beginRun, endTurn, game, hero, MAP, press, script, TWO } from "./lib.ts";

export default playtest({
  name: "Begin, a hand of five, the d-pad and the play key, End turn and the foes' turn",
  commands: script([
    "0.5:get:Game:title.json",
    beginRun(0.6),
    "1.15:get:Game:map.json",
    "2.4:get:Game:start.json",
    "2.4:dump_entity:hero:start-hero.json",
    press("ArrowRight", 2.5),
    "2.65:dump_entity:hero:focus.json",
    press("Enter", 2.7),
    "2.9:get:Game:played.json",
    "2.9:dump_entity:hero:played-hero.json",
    endTurn(3.0),
    "3.2:get:Game:foes.json",
    "5.6:get:Game:next.json",
    "5.6:dump_entity:hero:next-hero.json",
    "5.7:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "title.json").phase, 0, "the title first");
    t.equal(game(run, "map.json").phase, MAP, "Begin opened the map");
    const start = game(run, "start.json");
    t.equal(start.phase, BATTLE, "the first fight by a tap on the map");
    t.equal(start.party, TWO ? 2 : 1, `${TWO ? "two heroes" : "one hero"} in the run`);
    t.equal(start.turn, 1, "turn 1");
    const h0 = hero(run, "start-hero.json");
    t.equal(h0.deck.length, 10, "a ten-card starting deck");
    t.equal(h0.hand.length, 5, "five cards drawn");
    t.equal(h0.draw.length, 5, "five left in the draw pile");
    t.equal(h0.energy, 3, "three energy");
    t.equal(hero(run, "focus.json").focusCard, 1, "the d-pad's right moved the focus to the second card");
    const played = game(run, "played.json");
    const h1 = hero(run, "played-hero.json");
    t.equal(played.played, 1, "the play key played a card");
    t.equal(h1.hand.length, 4, "it left the hand");
    t.ok(h1.discard.includes(h0.hand[1]!), "the focused card went to the discard pile");
    const cost = costOf(h0.deck[h0.hand[1]!]!);
    t.equal(h1.energy, 3 - cost, `its cost (${cost}) was spent`);
    t.equal(game(run, "foes.json").side, 1, "End turn: the foes' turn");
    const next = game(run, "next.json");
    const h2 = hero(run, "next-hero.json");
    t.equal(next.turn, 2, "and then the heroes' second turn");
    t.equal(next.side, 0, "the heroes act again");
    t.ok(next.taken > 0, `the foes struck as their intents showed (the imp 6, the hound 3 × 2; taken ${next.taken})`);
    t.equal(h2.energy, 3, "the energy back");
    t.equal(h2.hand.length, 5, "a fresh hand of five");
    t.equal(h2.draw.length + h2.hand.length + h2.discard.length, 10, "every card in a pile or the hand");
    t.equal(h2.discard.length, 5, "the last hand and the played card discarded");
  },
});
