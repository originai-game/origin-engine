// The fight: by a risen, Space swings and the blow lands on the attack
// clip's `hit` (a hit stop, the risen's health down, its frame at the top);
// three blows put a minion down: it falls, the quest counts it, the knight
// gains its experience, and it drops gold, which walking over picks up
// into the party's bag. A second down brings level 2 (more health, healed
// whole). Q drinks a draught: health back, one fewer in the bag. A risen
// that sees the knight hunts it and winds a blow up (og.ai's `strike`).
import { playtest } from "@origingame/engine/playtest";
import { game, press, Q_CLEAR, script, value, type Foe, type Hero } from "./lib.ts";

/** Each swing from beside the risen, facing it (warp turns the hero and its look): the blow's aim is the player's, whatever the view. */
const swings = [1.6, 2.2, 2.8, 3.4].flatMap((t) => [`${(t - 0.05).toFixed(2)}:warp:foe1`, press("Space", t)]);

export default playtest({
  name: "swings land and put a risen down: the quest counts it, gold drops, a level comes, a draught heals",
  commands: script(
    "0.3:run",
    "0.5:quest:clear",
    "1.4:get:foe-1.Foe:before.json",
    ...swings,
    "1.85:get:og.TimeScale:stop.json",
    "1.95:get:foe-1.Foe:struck.json",
    "1.95:get:Game:struck-game.json",
    "4.0:get:foe-1.Foe:down.json",
    "4.0:get:Game:down-game.json",
    "4.0:get:hero.Hero:xp.json",
    "4.0:get:loot-0.Loot:loot.json",
    "4.0:get:loot-0.og.Transform:loot-at.json",
    "4.1:warp:loot0",
    "4.6:get:Game:picked.json",
    "4.7:slay:0",
    "4.8:get:hero.Hero:level.json",
    "4.8:get:Game:level-game.json",
    "5.0:hp:0:40",
    "5.1:get:Game:before-drink.json",
    press("KeyQ", 5.2),
    "5.4:get:hero.Hero:drunk.json",
    "5.4:get:Game:drunk-game.json",
    "5.5:warp:foe3",
    "7.5:get:foe-3.Foe:hunter.json",
    "7.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = value<Foe>(run, "before.json");
    t.ok(before.hp === before.maxHp && before.hp > 0, `the risen stands whole (${before.hp})`);
    const struck = value<Foe>(run, "struck.json");
    t.ok(struck.hp < before.hp, `Space swung and the blow landed (health ${before.hp} → ${struck.hp})`);
    t.ok(value<{ hitStop: number }>(run, "stop.json").hitStop > 0 || game(run, "struck-game.json").hits > 0, "with a hit stop");
    t.ok(game(run, "struck-game.json").target !== 0, "and its frame shows");
    const down = value<Foe>(run, "down.json");
    t.ok(down.hp === 0 && down.dying > 0, "three blows put it down");
    const g = game(run, "down-game.json");
    t.ok(g.slain === 1 && g.quest === Q_CLEAR, "the quest counts it");
    const xp = value<Hero>(run, "xp.json");
    t.ok(xp.xp === 30 && xp.kills === 1, `the knight gains its experience (${xp.xp}, ${xp.kills} down)`);
    const loot = value<{ active: boolean; kind: number; amount: number }>(run, "loot.json");
    t.ok(loot.kind === 0 && loot.amount > 0, `it dropped gold (${loot.amount})`);
    t.ok(game(run, "picked.json").gold >= loot.amount, `walking over it picks it up (${game(run, "picked.json").gold} gold)`);
    const level = value<Hero>(run, "level.json");
    t.ok(level.level === 2 && level.maxHp > 100 && level.hp === level.maxHp, `a second down brings level 2, healed whole (${level.hp}/${level.maxHp})`);
    t.ok(game(run, "level-game.json").levelUps >= 1, "a level-up counted (each standing hero levels on the party's experience)");
    const drunk = value<Hero>(run, "drunk.json");
    const bag = game(run, "before-drink.json").potions;
    t.ok(drunk.hp > 40, `Q drinks: health back (${drunk.hp})`);
    t.ok(game(run, "drunk-game.json").potions === bag - 1 && game(run, "drunk-game.json").drunk === 1, "one fewer draught in the bag");
    const hunter = value<Foe>(run, "hunter.json");
    t.ok(hunter.strikes > 0, `by another risen: it sees the knight, hunts it and winds up a blow (${hunter.strikes} strikes)`);
  },
});
