// The gloam's hall. Entering it holds the camera on the hall (an
// og.CameraZone smaller than the view, centred) and wakes the gloam (a
// toast, a small shake). From a plank out of its reach Wren watches it walk
// under, wind up for half a second flashing red (the telegraph, og-feel
// §2) and charge. Down, then the mallet: its fall is the heaviest moment of
// the run (a long hit stop, slow motion for 40 real ticks and back to 1, a
// big shake, a burst, eight glimmers) and the third chime appears where it
// fell.
import { playtest } from "@origingame/engine/playtest";
import { game, press, script, value } from "./lib.ts";

const STRIKES = [3.9, 4.3, 4.7, 5.1, 5.5, 5.9, 6.3];
const SAMPLES = Array.from({ length: 30 }, (_, i) => 3.9 + i * 0.1);

export default playtest({
  name: "the hall holds the camera, the gloam telegraphs and charges, falls in slow motion and leaves the third chime",
  commands: script(
    "0.2:set:Tuning.gloamHp=3",
    "0.3:play",
    "0.4:warp:44.5:6.3",
    "1.2:get:og.Camera:zone.json",
    "1.2:get:Game:awake.json",
    "2.55:get:gloam.Enemy:wind.json",
    "2.55:get:gloam.og.Sprite:telegraph.json",
    "2.617:get:gloam.og.Sprite:telegraph2.json",
    "3.0:get:gloam.Enemy:dash.json",
    "3.6:warp:gloam:-1.8",
    ...STRIKES.map((t) => press("KeyX", t)),
    ...SAMPLES.map((t, i) => `${t.toFixed(2)}:get:og.TimeScale:scale-${i}.json`),
    "7.0:get:Game:down.json",
    "7.0:get:og.TimeScale:after.json",
    "7.1:warp:chime-2:0:-1",
    "7.6:get:Game:taken.json",
    "7.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const zone = value<{ zoneActive: boolean; zoneMin: [number, number] }>(run, "zone.json");
    t.ok(zone.zoneActive && Math.abs(zone.zoneMin[0] - 40) < 0.01, "the hall's camera zone holds the view");
    const awake = game(run, "awake.json");
    t.ok(awake.arena && awake.bossAwake, "Wren in the hall woke the gloam");
    t.equal(value<{ state: number }>(run, "wind.json").state, 1, "it winds up (State.Wind)");
    const flash = Math.max(value<{ flash: number }>(run, "telegraph.json").flash, value<{ flash: number }>(run, "telegraph2.json").flash);
    t.ok(flash > 0.3, `flashing as it does (${flash.toFixed(2)})`);
    t.equal(value<{ state: number }>(run, "dash.json").state, 2, "then charges (State.Dash)");
    const scales = SAMPLES.map((_, i) => value<{ scale: number }>(run, `scale-${i}.json`).scale);
    t.ok(scales.some((s) => Math.abs(s - 0.3) < 1e-6), `its fall ran in slow motion (${scales.map((s) => s.toFixed(1)).join(" ")})`);
    t.equal(value<{ scale: number }>(run, "after.json").scale, 1, "and the time scale is back to 1");
    const down = game(run, "down.json");
    t.ok(down.bossDown && down.kills === 1 && down.glimmers >= 8, `the gloam is beaten (${down.kills} kill, ${down.glimmers} glimmers)`);
    const taken = game(run, "taken.json");
    t.equal([taken.chimes, taken.chimeMask & 4], [1, 4], "its chime, taken");
  },
});
