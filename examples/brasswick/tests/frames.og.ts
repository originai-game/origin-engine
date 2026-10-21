// Nine frames at 1280x720 compared with references/<tier>/: the title over
// the first level, the level select (two levels open, the third locked),
// the first level's start (Wick, the HUD), the crate on its plate and the
// door slid up into the ceiling, a tick stomped in the second level (the
// sparks in the hit stop), the dark third level with its gate slid open on
// its joint and a lamp lit, the pause menu, the settings in Spanish, and
// the results. `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { game, hold, press, script, tap } from "./lib.ts";

const FILES = ["title.png", "levels.png", "start.png", "door.png", "stomp.png", "gate.png", "pause.png", "settings-es.png", "results.png"];

export default playtest({
  name: "the title, the level select, a start, a door held open, a stomp, the jointed gate in the dark, pause, the Spanish settings and the results match their reference frames",
  size: "1280x720",
  commands: script(
    "0.2:unlock:2",
    "1.0:screenshot:title.png",
    tap("levels", 1.1),
    "2.0:screenshot:levels.png",
    press("Escape", 2.1),
    tap("new", 2.6),
    "3.8:screenshot:start.png",
    "3.9:warp:26.5:-16",
    hold("ArrowRight", 4.3, 1.3),
    "6.4:get:Game:door.json",
    "6.4:screenshot:door.png",
    "6.5:play:2",
    "7.6:warp:l2-tick-0:0:2.5",
    "7.95:get:Game:stomp.json",
    "7.95:screenshot:stomp.png",
    "8.5:play:3",
    "9.5:warp:22.5:-20",
    hold("ArrowRight", 9.6, 2.6),
    "12.6:warp:l3-lamp-1:-1.2:0",
    hold("ArrowRight", 12.8, 0.25),
    "13.6:get:Game:gate.json",
    "13.6:get:l3-gate-j.og.Joint:joint.json",
    "13.6:screenshot:gate.png",
    press("Escape", 13.8),
    "14.5:screenshot:pause.png",
    "14.6:language:es",
    tap("settings", 14.7),
    "15.5:screenshot:settings-es.png",
    "15.6:language:en",
    tap("Done", 15.7),
    tap("resume", 16.2),
    "16.6:finish",
    "18.2:screenshot:results.png",
    "18.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.equal(game(run, "door.json").switches, 1, "the crate holds the first level's plate: its door is up");
    t.equal(game(run, "stomp.json").stomps, 1, "the tick was stomped as the stomp frame is taken");
    const gate = game(run, "gate.json");
    t.ok(gate.switches === 8 && gate.checkpoint === 1, `the third level's crate holds the gate's plate and its second lamp is lit (switches ${gate.switches}, checkpoint ${gate.checkpoint})`);
    t.ok(run.json<{ value: { value: number } }>("joint.json").value.value > 2.9, "the gate's prismatic joint has slid its whole travel");
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
