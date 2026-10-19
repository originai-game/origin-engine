// A gamepad alone: the d-pad and A start from the title, the d-pad walks
// (a held direction repeats, into the rats of the corridor), A waits a
// turn, X drinks and, the flask empty, is refused; Start pauses and B
// resumes.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY } from "./lib.ts";

/** A pad button pressed at t and let go 0.05 s later (the pad set aside, so the release is no input of its own). */
const pad = (button: string, t: number) => `${t}:gamepad:${button};${(t + 0.05).toFixed(2)}:gamepad:off`;

export default playtest({
  name: "the d-pad, A, X, Start and B play a dive",
  commands: [
    pad("down", 0.3),
    pad("a", 0.4),
    "1.0:get:Game:started.json",
    "1.1:dive:7",
    pad("up", 1.3),
    pad("up", 1.6),
    "1.9:get:Game:stepped.json",
    "2.0:gamepad:up",
    "3.0:gamepad:off",
    "3.3:get:Game:held.json",
    pad("a", 3.4),
    "3.7:get:Game:waited.json",
    pad("x", 3.8),
    "4.1:get:Game:drank.json",
    pad("x", 4.2),
    "4.4:get:Game:refused.json",
    pad("start", 4.5),
    "4.7:get:Game:paused.json",
    pad("b", 4.8),
    "5.1:get:Game:resumed.json",
    "5.2:gamepad:off",
    "5.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "started.json").phase, PLAY, "the pad started a dive from the title");
    const stepped = game(run, "stepped.json");
    t.equal(`${stepped.heroX},${stepped.heroY}`, "4,13", "two d-pad presses walked two cells up");
    const held = game(run, "held.json");
    t.ok(held.turns >= 4, `a d-pad held for a second kept walking (${held.turns} turns)`);
    const waited = game(run, "waited.json");
    t.equal(waited.turns, held.turns + 1, "A waited a turn");
    t.equal(`${waited.heroX},${waited.heroY}`, `${held.heroX},${held.heroY}`, "in place");
    const drank = game(run, "drank.json");
    t.ok(drank.potions === 0 && drank.turns === waited.turns + 1, "X drank the draught, a turn");
    const refused = game(run, "refused.json");
    t.equal(refused.turns, drank.turns, "X with the flask empty was refused: no turn spent");
    t.equal(game(run, "paused.json").paused, true, "Start paused");
    t.equal(game(run, "resumed.json").paused, false, "B resumed");
  },
});
