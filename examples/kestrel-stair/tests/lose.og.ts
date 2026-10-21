// Losing. Night: the clock reaching 19:30 (Tuning.duskSeconds of play) ends
// the climb; the lose sound, then after a beat the results say night fell,
// with no stars, and the board is not written. Strength gone: the imps'
// bites take the last of it and the courier falls (its die clip); the
// results say so; Title goes back to the title.
import { playtest } from "@origingame/engine/playtest";
import { FALLEN, game, NIGHT, OVER, script, tap, texts, TITLE, value } from "./lib.ts";

export default playtest({
  name: "night at 19:30 ends the climb; strength gone, the courier falls; the results say which, and Title goes back",
  commands: script(
    "0.3:play:0:quick",
    "0.4:dusk:299",
    "1.5:get:Game:dusk.json",
    "1.5:get:sfx-65536.Sfx:lose.json",
    "3.9:get:Game:night.json",
    "3.9:dump_ui:night-ui.json",
    "4.0:dump_platform:calls.json",
    "4.1:play:0:quick",
    "4.4:hp:10",
    "4.5:warp:-3:31",
    "6.9:get:Game:fallen.json",
    "6.9:get:hero-body.og.AnimationPlayer:die.json",
    "8.8:dump_ui:fallen-ui.json",
    tap("title", 9.0),
    "10.0:get:Game:title.json",
    "10.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const dusk = game(run, "dusk.json");
    t.equal(dusk.ending, NIGHT, `night fell at ${dusk.hour.toFixed(2)} h`);
    t.ok(value<{ plays: number }>(run, "lose.json").plays >= 1, "with the lose sound");
    const night = game(run, "night.json");
    t.equal(night.phase, OVER, "then the results");
    t.ok(texts(run, "night-ui.json").includes("Night falls"), "saying night fell");
    const calls = run.json<{ calls: Array<{ method: string }> }>("calls.json").calls;
    t.ok(!calls.some((c) => c.method === "leaderboard.submit"), "and nothing went on the board");
    const fallen = game(run, "fallen.json");
    t.equal([fallen.ending, fallen.hp], [FALLEN, 0], "the imp's bites took the last strength");
    t.equal(value<{ weight: number }>(run, "die.json").weight, 1, "the courier falls on its die clip");
    t.ok(texts(run, "fallen-ui.json").includes("Fallen on the stair"), "the results say so");
    t.equal(game(run, "title.json").phase, TITLE, "Title went back to the title");
  },
});
