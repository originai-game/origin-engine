// The bag (og.ui.inventoryScreen): Tab opens it over the held world; the
// letter, the sigils carried and the hillwort sit in its slots; an item
// dragged to another slot moves there (the game keeps the slots in
// Game.bag); a selected item shows its name and line; Use eats a hillwort
// (strength back, its sound, a "+40"), Escape closes it. H eats one from
// the stair too; with none left it is refused with the error sound. The
// pause menu opens the bag as well.
import { playtest } from "@origingame/engine/playtest";
import { game, press, script, tap, texts, value } from "./lib.ts";

export default playtest({
  name: "the bag opens with Tab, drags an item to another slot, shows its details and eats a hillwort with Use; H eats one; pause opens it",
  commands: script(
    "0.3:play:0:quick",
    "0.35:set:Tuning.sight=0",
    "0.4:give:herbs:2",
    "0.45:give:sigils:1",
    "0.5:hp:40",
    press("Tab", 0.8),
    "1.2:get:Game:open.json",
    "1.2:dump_ui:open-ui.json",
    "1.3:touch_move_to:Slots/Cell#4",
    "1.317:touch_start",
    "1.4:touch_move_to:Slots/Cell#9",
    "1.5:touch_move_to:Slots/Cell#9",
    "1.55:touch_end",
    "1.8:get:Game:moved.json",
    tap("Slots/Cell#9", 1.9),
    "2.3:dump_ui:detail.json",
    "2.35:screenshot:bag.png",
    tap("Use", 2.5),
    "2.8:get:Game:used.json",
    "2.8:get:sfx-512.Sfx:heal.json",
    press("Escape", 3.0),
    "3.3:get:Game:closed.json",
    press("KeyH", 3.5),
    "3.8:get:Game:eaten.json",
    press("KeyH", 4.0),
    "4.3:get:Game:none.json",
    "4.3:get:sfx-262144.Sfx:refused.json",
    press("Escape", 4.5),
    tap("bag", 4.9),
    "5.3:get:Game:from-pause.json",
    "5.4:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const open = game(run, "open.json");
    t.ok(open.bagOpen && open.held, "Tab opened the bag and held the world");
    const shown = texts(run, "open-ui.json");
    t.ok(shown.includes("Bag") && shown.includes("map") && shown.includes("gem") && shown.includes("leaf"), "the letter, a sigil and the hillwort are in it");
    const moved = game(run, "moved.json");
    t.equal(JSON.parse(moved.bag)[9], "herb", "the hillwort was dragged to slot 9");
    t.equal(moved.bagMoves, 1, "one move");
    const detail = texts(run, "detail.json");
    t.ok(detail.includes("Hillwort") && detail.includes("Bitter leaves. Restores 40 strength."), "selecting it shows its name and line");
    const used = game(run, "used.json");
    t.equal([used.hp, used.herbs, used.herbsUsed], [80, 1, 1], "Use ate one: strength back by 40");
    t.ok(value<{ plays: number }>(run, "heal.json").plays >= 1, "with the heal's sound");
    t.ok(!game(run, "closed.json").bagOpen, "Escape closed it");
    t.equal([game(run, "eaten.json").hp, game(run, "eaten.json").herbs], [100, 0], "H ate the other");
    t.equal(game(run, "none.json").herbsUsed, 2, "with none left (and strength full), H eats nothing");
    t.ok(value<{ plays: number }>(run, "refused.json").plays >= 1, "and is refused with the error sound");
    t.ok(game(run, "from-pause.json").bagOpen, "the pause menu's Bag opens it");
  },
});
