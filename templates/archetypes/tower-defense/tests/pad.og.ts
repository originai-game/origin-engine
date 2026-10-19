// A gamepad alone: the d-pad and A start from the title, the d-pad walks
// the cursor, A sets a plant, X picks the next plant, Y calls the wave,
// Start pauses and B resumes.
import { playtest } from "@origingame/engine/playtest";
import { game, PLAY } from "./lib.ts";

export default playtest({
  name: "the d-pad, A, X, Y, Start and B play a round",
  commands: [
    "0.3:gamepad:down",
    "0.4:gamepad:a",
    "1.2:get:Game:started.json",
    "1.3:gamepad:up",
    "1.4:gamepad:left",
    "1.5:gamepad:a",
    "1.6:gamepad:x",
    "1.7:gamepad:right",
    "1.8:gamepad:a",
    "1.9:get:Game:planted.json",
    "2.0:gamepad:y",
    "2.2:get:Game:called.json",
    "2.3:gamepad:start",
    "2.5:get:Game:paused.json",
    "2.6:gamepad:b",
    "2.9:get:Game:resumed.json",
    "3.0:gamepad:off",
    "3.1:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.equal(game(run, "started.json").phase, PLAY, "the pad started a round from the title");
    const planted = game(run, "planted.json");
    t.equal(planted.placed, 2, "A set two plants");
    t.equal(planted.selected, 1, "X picked the frost lily");
    t.ok(planted.cursorX === 4 && planted.cursorY === 8, `the d-pad walked the cursor to (4, 8) (${planted.cursorX}, ${planted.cursorY})`);
    t.equal(planted.gold, 150 - 50 - 70, "a thorn and a frost lily paid for");
    t.equal(game(run, "called.json").wave, 1, "Y called the first wave");
    t.equal(game(run, "paused.json").paused, true, "Start paused");
    t.equal(game(run, "resumed.json").paused, false, "B resumed");
  },
});
