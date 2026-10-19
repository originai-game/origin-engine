// The core loop through real input: Dive on the title starts a dive on a
// fresh seed; then seed 7's first floor (the same start as Dive, with a
// known seed) is walked with the arrows: up the corridor (each step a
// turn, the lamp lighting what it reaches), a bump into the stone (no
// turn), then on up into the rats coming to meet the lamp (walking into a
// foe strikes it), and Q drinks the flask.
import { playtest } from "@origingame/engine/playtest";
import { component, game, PLAY, press, presses, tap, TITLE } from "./lib.ts";

export default playtest({
  name: "a dive from the title: steps, a bump, the lamp's light, strikes and a draught",
  commands: [
    "0.3:get:Game:title.json",
    "0.9:screenshot:title.png",
    tap("play", 1.0),
    "1.4:get:Game:started.json",
    "1.45:dive:7",
    "1.5:get:Game:start7.json",
    presses(["ArrowUp", "ArrowUp", "ArrowUp", "ArrowUp"], 1.6),
    press("ArrowLeft", 2.7),
    "3.0:get:Game:walked.json",
    presses(Array(12).fill("ArrowUp"), 3.1),
    "6.2:get:Game:fought.json",
    "6.25:screenshot:fought.png",
    press("KeyQ", 6.3),
    "6.7:get:Game:drank.json",
    "6.7:dump_entity:hero:hero.json",
    "6.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "title.json").phase, TITLE, "the game opens on the title");
    const started = game(run, "started.json");
    t.equal(started.phase, PLAY, "Dive started a dive");
    t.ok(started.seed !== 0 && started.seed !== 1, `on a fresh seed (${started.seed})`);
    t.ok(started.floor === 1 && started.hp === 20 && started.potions === 1 && started.dives === 1, "on floor 1 with full health and one draught");
    t.ok(started.lit > 0, `the lamp lights the first room (${started.lit} cells)`);
    const s7 = game(run, "start7.json");
    t.equal(`${s7.heroX},${s7.heroY}`, "4,15", "seed 7 starts the lamp-bearer in its first room");
    const walked = game(run, "walked.json");
    t.equal(`${walked.heroX},${walked.heroY}`, "4,11", "four presses walked four cells up the corridor");
    t.equal(walked.turns, 4, "four steps, four turns; the bump into the stone took none");
    t.ok(walked.seen > s7.seen, `the lamp lit new cells on the way (${s7.seen} → ${walked.seen})`);
    t.equal(component<{ bumps: number }>(run, "hero.json", "Hero").bumps >= 1, true, "the stone refused the step to the left");
    const fought = game(run, "fought.json");
    t.ok(fought.hits >= 2 && fought.kills >= 1, `walking into the rats struck them (${fought.hits} strikes, ${fought.kills} felled)`);
    t.ok(fought.hurts >= 1 && fought.hp < 20, `the rats struck back (${fought.hurts} times, health ${fought.hp})`);
    t.ok(fought.heroY < 11, "and the lamp-bearer pressed on up");
    const drank = game(run, "drank.json");
    t.equal(drank.potions, fought.potions - 1, "Q drank the draught");
    t.ok(drank.hp > fought.hp, `and it healed (${fought.hp} → ${drank.hp})`);
  },
});
