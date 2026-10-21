// Old Burrow's talk and wares. Up beside the mole opens the talk
// (og.ui.cutscene): the world holds (time scale 0) and Enter walks the four
// lines, the bell-keeper's name in the last. Then the shop
// (og.ui.shopScreen): a Buy short of the price is refused (the button
// shakes with the error sound), the wing cloak bought with enough glimmers
// takes its price and gives Wren an air jump; Escape closes the shop and
// the world moves again.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, hold, press, script, tap, texts, value } from "./lib.ts";

const ENTERS = [1.2, 1.45, 1.7, 1.95, 2.2, 2.45, 2.7, 2.95, 3.2];

export default playtest({
  name: "the mole's talk walked by Enter, a refused and a bought ware, the cloak's air jump",
  commands: script(
    "0.2:name:Ada",
    "0.3:play",
    "0.4:warp:mole",
    press("ArrowUp", 0.9),
    "1.1:get:og.TimeScale:held.json",
    "1.1:dump_ui:talk1.json",
    ...ENTERS.map((t) => press("Enter", t)),
    "2.1:dump_ui:talk4.json",
    "3.6:get:Game:shopping.json",
    "3.6:dump_ui:shop.json",
    tap("Buy", 3.7),
    "3.9:get:Game:refused.json",
    "3.9:get:sfx-256.Sfx:buy0.json",
    "4.0:give:glimmers:20",
    tap("Cell#2", 4.1),
    tap("Buy", 4.4),
    "4.7:get:Game:bought.json",
    "4.7:get:sfx-256.Sfx:buy1.json",
    "4.7:dump_ui:owned.json",
    press("Escape", 4.9),
    "5.3:get:Game:closed.json",
    "5.3:get:og.TimeScale:moving.json",
    "5.4:get:hero.og.PlatformerController:pc.json",
    hold("Space", 5.6, 0.25),
    hold("Space", 6.0, 0.3),
    "6.8:get:Game:air.json",
    "6.9:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(value<{ scale: number }>(run, "held.json").scale, 0, "the talk holds the world");
    t.ok(texts(run, "talk1.json").includes("Old Burrow"), "Old Burrow speaks");
    const shop = game(run, "shopping.json");
    t.ok(!shop.talking && shop.shopping && shop.talks === 1, "Enter walked the four lines into the shop");
    t.ok(texts(run, "shop.json").includes("Old Burrow's wares"), "the shop is his wares");
    const refused = game(run, "refused.json");
    t.equal([refused.bought, refused.glimmers], [0, 0], "Buy with nothing in the purse bought nothing");
    t.equal(value<{ plays: number }>(run, "buy0.json").plays, 0, "and made no sale's sound");
    const bought = game(run, "bought.json");
    t.ok(bought.cloak && bought.bought === 1 && bought.glimmers === 20 - 16, `the wing cloak, bought for 16 (${bought.glimmers} left)`);
    t.equal(value<{ plays: number }>(run, "buy1.json").plays, 1, "with the sale's sound");
    t.ok(elements(run, "owned.json").some((e) => e.text === "Wing cloak"), "the shop still lists it");
    const closed = game(run, "closed.json");
    t.ok(!closed.shopping && !closed.held, "Escape closed the shop");
    t.equal(value<{ scale: number }>(run, "moving.json").scale, 1, "and the world moves again");
    t.equal(value<{ airJumps: number }>(run, "pc.json").airJumps, 1, "the cloak gives the controller an air jump");
    t.equal(game(run, "air.json").jumps, 2, "a jump, then another in the air");
  },
});
