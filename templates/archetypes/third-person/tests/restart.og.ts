// A new run calms the sentinels: one chasing the hero when the run
// restarts goes back to its round in `patrol` (its machine started afresh)
// instead of searching where it last saw the hero.
import { playtest } from "@origingame/engine/playtest";
import { value } from "./lib.ts";

interface Agent { state: number }
interface Sentinel { alert: boolean }

export default playtest({
  name: "a sentinel chasing when the run restarts is back on patrol",
  commands: [
    "0.3:run",
    "2.6:warp:-28.5:3",
    "3.25:get:sentinel-0.og.AiAgent:chasing.json",
    "3.3:run",
    "3.6:get:sentinel-0.og.AiAgent:restarted.json",
    "3.6:get:sentinel-0.Sentinel:calm.json",
    "3.7:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const chasing = value<Agent>(run, "chasing.json").state;
    t.ok(chasing === 1 || chasing === 2, `the west sentinel saw the hero and left its patrol (state ${chasing})`);
    t.equal(value<Agent>(run, "restarted.json").state, 0, "after the restart it patrols (state 0), not searching");
    t.equal(value<Sentinel>(run, "calm.json").alert, false, "and its eye is calm");
  },
});
