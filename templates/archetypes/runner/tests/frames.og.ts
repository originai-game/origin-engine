// The look, frame by frame: the title over the scrolling city, a coin's
// pickup mid-run ("+10" rising from the popping coin, dust at the feet) and
// the crash's hit stop (the courier flashing red on the crate, the view
// shaking). Each is compared with references/<tier>/frames-<file> under
// og-tests' tolerance (tests/frames.ts); `OG_FX_CAPTURE=1 og test .` records
// them. The script has no dice in it: patterns are off, the coin and the
// crate are placed.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { OVER, RUNNING, runOf, tap } from "./lib.ts";

const FILES = ["title.png", "coin.png", "crash.png"] as const;

export default playtest({
  name: "reference frames: title, a coin's pickup, the crash",
  commands: [
    "0.1:set:runner.Tuning.spawnInterval=60",
    "0.3:screenshot:title.png",
    tap(0.5),
    "1.0:place:coin:-2",
    "1.6:screenshot:coin.png",
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
    t.ok(coin.phase === RUNNING && coin.coins === 1, "the coin frame is mid-run, just after the pickup");
    const crash = runOf(run, "crash.json");
    t.ok(crash.phase === OVER && crash.phaseTicks < 8, `the crash frame is inside the hit stop (${crash.phaseTicks} ticks after)`);
    checkFrames(run, t, FILES, "frames-");
  },
});
