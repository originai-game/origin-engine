// A gamepad alone: A on the title plays, A taps the orb, the d-pad's right
// chooses the next generator, X buys the chosen one (refused when too
// dear), Y asks for the supernova, Start pauses and B resumes.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY } from "./lib.ts";

/** A pad button pressed at t and let go 0.05 s later (the pad set aside, so the release is no input of its own). */
const pad = (button: string, t: number) => `${t}:gamepad:${button};${(t + 0.05).toFixed(2)}:gamepad:off`;

export default playtest({
  name: "A, the d-pad, X, Y, Start and B play the orb",
  commands: [
    // The title's first item holds the focus when the menu opens (og-ui §4), so A plays at once; no d-pad press first.
    pad("a", 1.1),
    "1.9:get:Game:started.json",
    pad("a", 2.0),
    pad("a", 2.1),
    pad("a", 2.2),
    "2.4:get:Game:tapped.json",
    "2.5:sparks:200",
    pad("right", 2.6),
    pad("x", 2.7),
    "2.9:get:Game:bought.json",
    pad("right", 3.0),
    pad("right", 3.1),
    pad("x", 3.2),
    "3.4:get:Game:refused.json",
    "3.5:sparks:300000",
    pad("y", 3.6),
    "3.8:get:Game:asked.json",
    pad("b", 3.9),
    "4.1:get:Game:declined.json",
    pad("start", 4.2),
    "4.4:get:Game:paused.json",
    pad("b", 4.5),
    "4.8:get:Game:resumed.json",
    "4.9:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "started.json").phase, PLAY, "the pad played from the title");
    t.equal(game(run, "tapped.json").taps, 3, "A tapped the orb three times");
    const bought = game(run, "bought.json");
    t.equal(bought.selected, 1, "right chose the Storm");
    t.equal(bought.owned1, 1, "X bought it");
    t.equal(bought.sparks < 200 - 109, true, "for 110");
    const refused = game(run, "refused.json");
    t.equal(refused.selected, 3, "right twice more chose the Star");
    t.ok(refused.refused === 1 && refused.owned3 === 0, "X was refused: a Star costs 13,000");
    t.equal(game(run, "asked.json").confirmNova, true, "Y asked for the supernova");
    t.equal(game(run, "declined.json").confirmNova, false, "B declined it");
    t.equal(game(run, "paused.json").paused, true, "Start paused");
    t.equal(game(run, "resumed.json").paused, false, "B resumed");
  },
});
