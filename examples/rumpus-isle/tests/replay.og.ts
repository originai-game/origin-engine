// Determinism: a match by the keys alone from the title (Space is Play, two
// sit down and ready up, the launch count), the bump ring driven into a bump
// and a dash and ended by a knock, the scoreboard, the berry rush swept by
// Pip for berries and whistled, run twice in fresh processes, ends on the
// same world hash and the same tick each time: the critters' physics, the
// seeded berries and the UI replay exactly.
import { playtest } from "@origingame/engine/playtest";
import { game, hold, OVER, press, script, sweep } from "./lib.ts";

export default playtest({
  name: "a match by the keys from the title through the bump ring and the berry rush replays to the same hash",
  runs: 2,
  commands: script(
    press("Space", 0.5),
    press("Space", 1.0),
    press("Enter", 1.2),
    press("Space", 1.5),
    press("Enter", 1.7),
    "3.6:go",
    hold("KeyA", 3.8, 1),
    hold("ArrowRight", 3.8, 1),
    press("Space", 4.3),
    "4.9:get:Game:mid.json",
    "5.0:knock:1",
    "7.3:next",
    "7.4:go",
    ...sweep(7.6),
    "11.8:finish",
    "12.1:get:Game:end.json",
    "12.2:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(run.hashes.length, 2, "two runs");
    t.equal(run.hashes[1], run.hashes[0], `both runs end on hash ${run.hashes[0]}`);
    const mid = game(run, "mid.json");
    t.ok(mid.matches === 1 && mid.round === 0 && mid.bumps >= 1 && mid.dashes === 1, `the lobby launched the match and the bump ring saw a bump and a dash (${mid.bumps} bumps)`);
    const end = game(run, "end.json");
    t.ok(end.phase === OVER && end.round === 1, "the berry rush was whistled");
    t.ok(end.berriesTaken >= 1 && end.splashes === 1, `berries were taken (${end.berriesTaken}) after the one splash of the first round`);
    t.ok(end.points0 >= 5 && end.points1 >= 2, `the points carry (${end.points0}, ${end.points1})`);
  },
});
