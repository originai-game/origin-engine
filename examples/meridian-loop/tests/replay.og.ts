// Determinism: a session of keys and the autopilot (a standing start, a
// slide into the wall, the jump, a fall into the gap and its reset, R,
// pause and resume, a record and its ghost on the second lap) run twice in
// fresh processes ends on one world hash.
import { playtest } from "@origingame/engine/playtest";
import { hold, press, race, script } from "./lib.ts";

export default playtest({
  name: "a session with slides, the jump, resets, pause and the ghost replays to the same world hash in two processes",
  runs: 2,
  commands: script(
    "0.5:race",
    hold("KeyW", 3.6, 2.5),
    hold("KeyD", 4.6, 0.8),
    "6.5:warp:150:30",
    hold("KeyW", 6.55, 1.2),
    hold("KeyA", 6.55, 1.0),
    "8.5:warp:205:36",
    hold("KeyW", 8.55, 2),
    "12:warp:229:8",
    press("KeyR", 16),
    press("Escape", 16.5),
    "17:get:Race:paused.json",
    press("Escape", 17.5),
    "18:drive:auto",
    "90:get:Race:end.json",
    "90.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(race(run, "paused.json").paused, "Escape paused the race");
    const end = race(run, "end.json");
    t.ok(end.resets >= 2 && end.landings >= 1 && end.skidTicks > 0 && end.bumps > 0, `the session slid (${end.skidTicks} ticks), hit the wall (${end.bumps}), landed (${end.landings}) and was reset (${end.resets})`);
    t.ok(end.laps >= 2 && end.record > 0 && end.bestLen > 0, `laps were driven (${end.laps}), a clean one the record with its ghost`);
    t.equal(run.hashes.length, 2, "two processes ran");
    t.equal(run.hashes[1], run.hashes[0], `both ended on one world hash (${run.hashes[0]})`);
  },
});
