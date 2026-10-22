// The Crumble Tiles: the island sinks away and the grid of tiles is laid;
// the critters start nearer the middle (2.6 m out); nothing cracks during
// the grace; a tile stood on cracks (it shakes) and drops after
// Tuning.crackTicks; Pip, standing still, falls with his and is out with the
// last place while Juno, who stepped onto a whole tile just before, stands:
// the round is hers, Pip's seconds on the tiles are the round's clock at his
// splash, the longest sets the record the scoreboard announces and
// og.platform keeps.
import { playtest } from "@origingame/engine/playtest";
import { calls, game, hold, OVER, PLAY, position, script, texts, twoPlayers, value } from "./lib.ts";

interface TileV { crack: number; fall: number; down: boolean }

export default playtest({
  name: "the crumble tiles: the island sinks, a grace, a tile cracks and drops under a still critter, places, the lasted record",
  commands: script(
    ...twoPlayers(0.2, 3),
    "0.6:get:island.og.Transform:island.json",
    "0.6:get:tile-0-0.og.Transform:tile00.json",
    "0.6:get:Game:start.json",
    "1.3:get:Game:grace.json",
    "2.0:get:Game:cracked.json",
    "2.0:get:tile-4-4.Tile:cracking.json",
    "2.0:get:tile-4-4.og.Transform:shaking.json",
    // Juno keeps moving onto whole tiles (each one she stands on cracks in turn); Pip, still, falls with
    // his and splashes 2.73 s into the round.
    hold("ArrowUp", 2.15, 0.5),
    hold("ArrowRight", 2.65, 0.5),
    hold("ArrowDown", 3.15, 0.35),
    "2.9:get:tile-4-4.Tile:dropped.json",
    "3.4:get:Game:over.json",
    "3.4:dump_ui:over-ui.json",
    "5.4:dump_ui:scores.json",
    "5.4:dump_platform:calls.json",
    "5.5:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(position(run, "island.json")[1] < -20, "the island is sunk away under the tiles");
    t.ok(Math.abs(position(run, "tile00.json")[1]) < 0.01, "the tiles are laid at the sea's level");
    const start = game(run, "start.json");
    t.ok(start.phase === PLAY && start.round === 2 && start.standing === 2, "the third round is in play with two standing");
    t.ok(Math.abs(start.x0 - 1.84) < 0.1 && Math.abs(start.z0 - 1.84) < 0.1, `the critters start nearer the middle (${start.x0.toFixed(2)}, ${start.z0.toFixed(2)})`);
    t.ok(start.left > 43 && start.left <= 45, `the clock shows the 45 s round (${start.left.toFixed(1)} s left)`);
    const grace = game(run, "grace.json");
    t.ok(grace.played < 60 && grace.cracks === 0, `nothing cracks in the grace (${grace.played} ticks in)`);
    const cracked = game(run, "cracked.json");
    t.ok(cracked.played > 60 && cracked.cracks === 2, `after it the tile under each still critter cracked (${cracked.cracks} cracks at ${cracked.played} ticks)`);
    const cracking = value<TileV>(run, "cracking.json");
    t.ok(cracking.crack > 0 && cracking.crack < 42 && !cracking.down, `Pip's tile is cracking (${cracking.crack} ticks) and not yet down`);
    const [sx, sy] = position(run, "shaking.json");
    t.ok(sx !== 2.325 && Math.abs(sx - 2.325) < 0.1 && sy < 0, `it shakes about its home (${sx.toFixed(3)}, ${sy.toFixed(3)})`);
    const dropped = value<TileV>(run, "dropped.json");
    t.ok(dropped.down, "after crackTicks it dropped");
    const over = game(run, "over.json");
    t.ok(over.phase === OVER && over.standing === 1 && over.splashes === 1 && !over.alive0 && over.alive1, "Pip fell with it and splashed; Juno, who stepped onto a whole tile, stands: the round is over");
    t.ok(over.place0 === 2 && over.place1 === 1 && over.points0 === 2 && over.points1 === 3, `Juno 1st for 3 points, Pip 2nd for 2 (places ${over.place0}, ${over.place1})`);
    t.ok(Math.abs(over.lasted0 - over.played / 60) < 0.02 && over.lasted0 > 1.5, `Pip lasted the clock at his splash (${over.lasted0.toFixed(2)} s)`);
    t.ok(over.newRecord && Math.abs(over.bestCrumble - Math.round(over.lasted1 * 10) / 10) < 0.051, `the longest stood sets the isle's record (${over.bestCrumble} s)`);
    const banner = texts(run, "over-ui.json");
    t.ok(banner.includes("Juno takes the round!"), "the banner names Juno");
    t.ok(banner.includes("2nd") && banner.includes("drop"), "the HUD shows Pip's 2nd under a drop");
    const scores = texts(run, "scores.json");
    t.ok(scores.includes("Round 3: Crumble Tiles"), "the scoreboard names the round");
    const lasted = `${over.bestCrumble.toFixed(1)} s`;
    t.ok(scores.includes(lasted), `and the seconds each lasted (${lasted})`);
    t.ok(scores.includes(`New record: ${lasted} on the tiles!`), "and announces the record");
    t.ok(scores.includes("To the podium"), "the third round's button goes to the podium");
    const written = calls(run, "calls.json").find((c) => c.method === "store.set" && c.args.key === "records");
    // The record is an f32 field: the payload carries it widened (2.700000047683716 for 2.7).
    t.ok(written !== undefined && Math.abs((written.args.value?.crumble as number) - over.bestCrumble) < 1e-6 && written.result?.ok === true, "og.platform keeps the record under `records`");
  },
});
