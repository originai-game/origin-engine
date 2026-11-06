// Between fights: a fight won offers each hero three cards (a tap takes
// one into its deck); the map's fork offers a fire or one more fight; at
// the fire, Temper offers cards of the deck to upgrade and one is
// upgraded; then the elite's step. With two on one device each hero
// chooses in turn.
import { playtest } from "@origingame/engine/playtest";
import { UPGRADED } from "../src/cards.ts";
import { game, hero, MAP, press, REST, REWARD, script, tap, TWO } from "./lib.ts";

export default playtest({
  name: "a won fight's card reward, the fork, the fire's Temper",
  commands: script([
    "0.3:goto:0",
    "0.4:foehp:0:1",
    "0.4:foehp:1:1",
    "0.4:give:cleave",
    press("ArrowLeft", 0.6),
    press("Enter", 0.8),
    "2.4:get:Game:reward.json",
    tap("Reward0/Cards/Options/r0", 2.6),
    ...(TWO ? [tap("Reward1/Cards/Options/r1", 3.4)] : []),
    "4.2:get:Game:map.json",
    "4.2:dump_entity:hero:rewarded.json",
    ...(TWO ? ["4.2:dump_entity:hero-1:rewarded-1.json"] : []),
    "4.3:goto:2",
    "4.6:get:Game:rest.json",
    tap("Rest0/Cards/Options/temper", 5.0),
    "5.6:get:Game:temper.json",
    tap("Temper0/Cards/Options/t0", 5.8),
    ...(TWO ? [tap("Rest1/Cards/Options/heal", 6.6)] : []),
    "7.4:get:Game:after.json",
    "7.4:dump_entity:hero:tempered.json",
    "7.5:quit",
  ]),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const reward = game(run, "reward.json");
    t.equal(reward.phase, REWARD, "Cleave felled both: the reward");
    t.equal(reward.rewards0.length, 3, "three cards offered");
    t.equal(new Set(reward.rewards0).size, 3, "three different cards");
    const map = game(run, "map.json");
    t.equal(map.phase, MAP, `the map once ${TWO ? "both heroes" : "the hero"} chose`);
    t.equal(map.step, 1, "at the next step");
    const h = hero(run, "rewarded.json");
    t.equal(h.deck.length, 12, "the deck holds the given Cleave and the reward");
    t.equal(h.deck[11], reward.rewards0[0], "the card tapped");
    if (TWO) t.equal(hero(run, "rewarded-1.json").deck[10], reward.rewards1[1], "the Hunter took its own reward");
    t.equal(game(run, "rest.json").phase, REST, "the fork's first road: the fire");
    const temper = game(run, "temper.json");
    t.equal(temper.tempers0.length, 3, "Temper offers three cards of the deck");
    const after = game(run, "after.json");
    t.equal(after.phase, MAP, "the map after the fire");
    t.equal(after.step, 3, "at the elite's step");
    const tempered = hero(run, "tempered.json");
    t.ok((tempered.deck[temper.tempers0[0]!]! & UPGRADED) !== 0, "the card chosen is upgraded");
  },
});
