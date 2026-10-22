// The foes' minds (assets/ai/foe.ogt on og.ai): a rat that sees the lamp
// hunts it along og.nav's path, a cell a turn, and strikes once beside it;
// the lamp-bearer's strikes land on a rat (a hit stop, the number) and the
// second fells it. Seed 7's first floor has a rat at (6, 7).
import { playtest } from "@origingame/engine/playtest";
import { foe, game, press, presses } from "./lib.ts";

export default playtest({
  name: "a rat hunts and strikes; two strikes fell it",
  commands: [
    "0.3:dive:7",
    "0.5:warp:3:7",
    "0.6:get:Foes:before.json",
    presses(["Space", "Space"], 0.7),
    "1.3:get:Foes:closer.json",
    presses(["Space", "Space"], 1.4),
    "2.0:get:Game:struck.json",
    "2.2:dive:7",
    "2.4:warp:5:7",
    press("ArrowRight", 2.6),
    "2.75:get:Game:hit.json",
    "2.75:dump_entity:foe-0:hit-foe.json",
    presses(["ArrowRight", "ArrowRight"], 2.9),
    "3.6:get:Game:felled.json",
    "3.6:get:Foes:felled-foes.json",
    "3.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const [x0, y0] = foe(run, "before.json", 0);
    t.equal(`${x0},${y0}`, "6,7", "the rat stands at (6, 7)");
    const [x1] = foe(run, "closer.json", 0);
    t.ok(x1 < x0, `two turns waited, it came along the row toward the lamp (x ${x0} → ${x1})`);
    const struck = game(run, "struck.json");
    t.ok(struck.hurts >= 1 && struck.hp < struck.hpMax, `beside the lamp-bearer it struck (${struck.hurts} strikes, health ${struck.hp})`);
    const hit = game(run, "hit.json");
    t.equal(hit.hits, 1, "walking into it struck it");
    const rat = run.json<{ components: { Foe: { hp: number; hpMax: number; active: boolean } } }>("hit-foe.json").components.Foe;
    t.ok(rat.active && rat.hp < rat.hpMax, `the strike took its health (${rat.hp}/${rat.hpMax})`);
    const felled = game(run, "felled.json");
    t.equal(felled.kills, 1, "the second strike felled it");
    t.equal(foe(run, "felled-foes.json", 0).join(","), "-1,-1", "and it left the board");
    t.equal(`${felled.heroX},${felled.heroY}`, "6,7", "the third press stepped where it stood");
  },
});
