// The Bump Ring: two critters start on their circle facing the middle (3.4 m
// out); driven at each other they meet and bump (a shove each, a thud), a
// dash on Space counts; a critter knocked into the sea splashes, is out with
// the last place, and with one standing the round is over: the banner names
// Pip, the HUD shows Juno under a drop with her 2nd, and the scoreboard
// gives +3 and +2.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, OVER, PLAY, press, SCORES, script, texts, twoPlayers, value } from "./lib.ts";

export default playtest({
  name: "the bump ring: a start on the circle, a bump and a dash, a knock into the sea ends the round with places, points and a banner",
  commands: script(
    ...twoPlayers(0.2),
    "0.7:get:Game:start.json",
    hold("KeyA", 0.8, 1),
    hold("ArrowRight", 0.8, 1),
    press("Space", 1.3),
    "1.9:get:Game:met.json",
    "1.9:get:Sfx:sfx.json",
    "1.9:get:voice-bump.Voice:thud.json",
    "2.0:knock:1",
    "2.3:get:Game:out.json",
    "2.3:dump_ui:over-ui.json",
    "4.5:get:Game:scores.json",
    "4.5:dump_ui:scores-ui.json",
    "4.6:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    const start = game(run, "start.json");
    t.ok(start.phase === PLAY && start.round === 0 && start.standing === 2, "the first round is in play with two standing");
    t.ok(Math.abs(start.x0 - 2.4) < 0.1 && Math.abs(start.z0 - 2.4) < 0.1 && Math.abs(start.x1 + 2.4) < 0.1, `Pip and Juno start on the circle (${start.x0.toFixed(2)}, ${start.z0.toFixed(2)}) and (${start.x1.toFixed(2)}, ${start.z1.toFixed(2)})`);
    t.ok(start.left > 38 && start.left <= 40, `the clock shows the 40 s round (${start.left.toFixed(1)} s left)`);
    const met = game(run, "met.json");
    t.ok(met.bumps >= 1, `driven at each other they bumped (${met.bumps} bumps)`);
    t.equal(met.dashes, 1, "Space dashed once");
    t.ok(met.x0 < 1 && met.x1 > -2.4 && met.x0 > met.x1, `Pip's dash shoved Juno back towards her start and neither passed through the other (x ${met.x0.toFixed(2)} and ${met.x1.toFixed(2)})`);
    t.ok(value<{ plays: number }>(run, "sfx.json").plays >= 3, "the go, the bump and the dash were heard");
    t.ok(value<{ plays: number }>(run, "thud.json").plays >= 1, "the bump's thud voice played");
    const out = game(run, "out.json");
    t.ok(out.phase === OVER && out.standing === 1 && out.splashes === 1 && !out.alive1 && out.alive0, "Juno knocked into the sea splashed and the round is over with Pip standing");
    t.ok(out.place0 === 1 && out.place1 === 2 && out.points0 === 3 && out.points1 === 2, `Pip 1st for 3 points, Juno 2nd for 2 (places ${out.place0}, ${out.place1}; points ${out.points0}, ${out.points1})`);
    t.ok(Math.abs(out.lasted1 - out.played / 60) < 0.02, "Juno's time on the island is the round's clock at her splash");
    const over = texts(run, "over-ui.json");
    t.ok(over.includes("Pip takes the round!"), "the banner names Pip");
    t.ok(over.includes("2nd") && over.includes("drop"), "the HUD shows Juno's 2nd under a drop");
    t.equal(game(run, "scores.json").phase, SCORES, "the scoreboard follows the beat");
    const scores = texts(run, "scores-ui.json");
    for (const w of ["Round 1: Bump Ring", "1st", "P1 · Pip", "+3", "2nd", "P2 · Juno", "+2", "Next round"]) t.ok(scores.includes(w), `the scoreboard says "${w}"`);
  },
});
