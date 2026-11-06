// The runner's smoke playtest: the title, a tap starts a round, a placed
// coin is taken mid-run ("+10" rising from the popping coin) and a placed
// crate ends it inside the crash's hit stop.
//
// Properties first: `invariants` hold after every tick (the first failing
// tick and its value are the failure), beside the engine's defaults (no
// NaN, every field in its range, no dangling entity) and the game's
// og.assert.always calls (src/systems.ts); `require` names the og.assert
// sites the run must reach. Two frames are kept for what only a frame
// shows, the look of the title and of the crash, compared with
// references/<tier>/frames-<file> under og-tests' tolerance (tests/frames.ts;
// `OG_FX_CAPTURE=1 og test .` records them). The coin's frame went: the
// pickup is now proven by the run, not by its picture.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { OVER, RUNNING, runOf, tap } from "./lib.ts";

const FILES = ["title.png", "crash.png"] as const;

export default playtest({
  name: "a round: title, a coin's pickup, the crash (two reference frames)",
  invariants: [
    // The runner never sinks through the street nor leaves the view.
    "entity:runner:og.Transform.position.y>=-0.01",
    "entity:runner:og.Transform.position.y<8",
    "entity:runner:Run.coins<=1",
  ],
  require: ["the runner takes a coin", "the runner crashes", "a round starts from each phase[from the title]"],
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    "0.3:screenshot:title.png",
    tap(0.5),
    "1.0:place:coin:-2",
    "1.6:dump_entity:runner:coin.json",
    "2.0:place:crate:1",
    // The crate reaches the runner in tick 173; 2.95 s is three ticks into the eight-tick hit stop.
    "2.95:screenshot:crash.png",
    "2.95:dump_entity:runner:crash.json",
    "3.0:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const coin = runOf(run, "coin.json");
    t.ok(coin.phase === RUNNING && coin.coins === 1, "mid-run, just after the pickup");
    const crash = runOf(run, "crash.json");
    t.ok(crash.phase === OVER && crash.phaseTicks < 8, `the crash frame is inside the hit stop (${crash.phaseTicks} ticks after)`);
    checkFrames(run, t, FILES, "frames-");
  },
});
