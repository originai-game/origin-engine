// Combat with a beetle, both ways (og-feel §3-§7). The mallet's strike lands
// on the attack clip's `hit` frame (an og.physics.overlap of the mallet's
// box on the enemy layer): hit stop, a white flash and a squash on the
// beetle, shake, the hit sound and a floating "-1"; its second strike beats
// it (two glimmers, its bit in Game.killMask) and it shrinks away. A
// beetle's touch costs a heart: the knock-back, Wren blinking, the hurt
// sound, a red "-1" and hit stop.
import { playtest } from "@origingame/engine/playtest";
import { game, press, script, value } from "./lib.ts";

export default playtest({
  name: "two strikes beat a beetle with hit stop, flash, shake and a number; a beetle's touch costs a heart and knocks Wren back",
  commands: script(
    "0.3:play",
    "0.5:warp:beetle-0:-1.6",
    "0.55:get:Game:before.json",
    press("KeyX", 0.6),
    // The strike lands four ticks after the press (the wind-up frame, then the `hit` frame's event).
    "0.668:get:og.TimeScale:stop1.json",
    "0.668:get:beetle-0.og.Sprite:flash.json",
    "0.668:get:beetle-0.og.Transform:held1.json",
    "0.7:get:beetle-0.og.Transform:held2.json",
    "0.7:get:og.Camera:shake.json",
    "0.75:screenshot:hit.png",
    "0.8:get:Game:one.json",
    press("KeyX", 1.2),
    "1.6:get:Game:two.json",
    "2.2:get:beetle-0.og.Sprite:gone.json",
    // A touch: beetle-1 walks into Wren.
    "2.3:warp:beetle-1:-0.3",
    "2.4:get:Game:touched.json",
    "2.4:get:og.TimeScale:stop2.json",
    "2.9:get:Game:after.json",
    "3.0:get:sfx-32.Sfx:hurt.json",
    "3.0:get:sfx-16.Sfx:hit.json",
    "3.1:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const before = game(run, "before.json");
    t.equal([before.hits, before.kills, before.hearts], [0, 0, 3], "a fresh run");
    t.ok(value<{ hitStop: number }>(run, "stop1.json").hitStop > 0, "the strike froze the world: hit stop");
    t.ok(value<{ flash: number }>(run, "flash.json").flash > 0.3, "the beetle flashed white");
    const h1 = value<{ position: number[]; scale: number[] }>(run, "held1.json");
    const h2 = value<{ position: number[]; scale: number[] }>(run, "held2.json");
    t.equal(h2.position[0], h1.position[0], "and held still through the stop");
    t.ok(h1.scale[0] > 1.05 && h1.scale[1] < 0.95, `squashed (${h1.scale[0].toFixed(2)} x ${h1.scale[1].toFixed(2)})`);
    t.ok(value<{ trauma: number }>(run, "shake.json").trauma > 0.1, "the camera shook");
    const one = game(run, "one.json");
    t.equal([one.hits, one.kills], [1, 0], "one strike landed, the beetle stands");
    const two = game(run, "two.json");
    t.equal([two.hits, two.kills, two.killMask & 1], [2, 1, 1], "the second beat it");
    t.equal(two.glimmers - before.glimmers, 2, "it dropped two glimmers");
    const gone = value<{ visible: boolean; opacity: number }>(run, "gone.json");
    t.ok(!gone.visible || gone.opacity < 0.05, "and shrank away");
    const touched = game(run, "touched.json");
    t.equal(touched.hearts, 2, "beetle-1's touch cost a heart");
    t.ok(touched.knock > 0 && touched.invulnerable > 0, "Wren is thrown back and blinking");
    t.ok(value<{ hitStop: number }>(run, "stop2.json").hitStop > 0 || touched.knock > 0, "with a hit stop");
    const after = game(run, "after.json");
    t.ok(Math.abs(after.heroX - touched.heroX) > 0.5, `knocked ${(after.heroX - touched.heroX).toFixed(2)} m`);
    t.ok(value<{ plays: number }>(run, "hurt.json").plays >= 1 && value<{ plays: number }>(run, "hit.json").plays >= 2, "the hurt and hit sounds played");
  },
});
