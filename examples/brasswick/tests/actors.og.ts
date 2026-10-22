// What Wick meets: a gear taken by touch (the count, a "+1", the sparks'
// emitter restarted), a tick stomped from above (a hit stop, a shake, the
// bounce, the squashed clip), a tick met from the side (the hurt: a hit stop
// and a shake, then Wick back at the last lamp after Tuning.hurtTicks), the
// spikes (the same), and the lamp (the checkpoint moves there).
import { playtest } from "@origingame/engine/playtest";
import { game, script, value } from "./lib.ts";

export default playtest({
  name: "gears are taken, a stomp stops a tick with a hit stop, a tick's side and the spikes hurt and bring Wick back to the lamp",
  commands: script(
    "0.5:play:1",
    "1.0:warp:l1-gear-0:0:-0.4",
    "1.2:get:Game:gear.json",
    "1.2:get:l1-gear-0.og.Sprite:gear-sprite.json",
    "1.2:get:fx-1.og.ParticleSystem:gear-fx.json",
    // The spikes in the first pit: a hurt, then the spawn (no lamp lit yet).
    "1.5:warp:12.5:-17",
    "2.0:get:Game:spiked.json",
    "2.9:get:Game:spike-back.json",
    // The second level: a stomp, then a tick met from the side after the lamp.
    "3.0:play:2",
    "3.5:warp:l2-tick-0:0:2.5",
    // The fall meets the tick at tick 222: the stop and the shake two ticks later, the bounce once the stop is over.
    "3.733:get:og.Time:stop.json",
    "3.733:get:og.Camera.trauma:trauma.json",
    "3.95:get:Game:stomp.json",
    "3.95:get:l2-tick-0.og.Flipbook:squashed.json",
    "4.2:warp:l2-lamp-0:0:0",
    "4.5:get:Game:lamp.json",
    "4.6:warp:l2-tick-1:0:0",
    "4.7:get:Game:touched.json",
    "5.6:get:Game:respawned.json",
    "5.7:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = game(run, "gear.json");
    t.ok(g.gears === 1 && (g.gearMask & 1) === 1, "touching a gear takes it");
    t.equal(value<{ visible: boolean }>(run, "gear-sprite.json").visible, false, "and hides it");
    const fx = value<{ startTick: number; visible: boolean }>(run, "gear-fx.json");
    t.ok(fx.visible && fx.startTick > 0, `the gear's sparks restarted there (startTick ${fx.startTick})`);
    const spiked = game(run, "spiked.json");
    t.ok(spiked.hurt > 0 && spiked.deaths === 1, "the spikes hurt Wick");
    const back = game(run, "spike-back.json");
    t.ok(back.hurt === 0 && Math.abs(back.heroX - 3.5) < 0.3 && back.safe > 0, `and he comes back at the spawn, blinking (${back.heroX.toFixed(2)})`);
    const stop = value<{ hitStop: number }>(run, "stop.json");
    t.ok(stop.hitStop > 0, `a stomp holds the world in a hit stop (${stop.hitStop} ticks left)`);
    t.ok(value<number>(run, "trauma.json") > 0, "and shakes the camera");
    const stomp = game(run, "stomp.json");
    t.ok(stomp.stomps === 1 && stomp.heroVY > 0, `the tick is stomped and Wick bounces off it (vy ${stomp.heroVY.toFixed(2)})`);
    t.equal(value<{ clip: number }>(run, "squashed.json").clip, 1, "the tick shows its squashed clip");
    const lamp = game(run, "lamp.json");
    t.ok(lamp.checkpoint === 0 && Math.abs(lamp.checkX - 128.5) < 0.05, "the lamp became the checkpoint");
    const touched = game(run, "touched.json");
    t.ok(touched.hurt > 0 && touched.deaths === 1, `a tick met from the side hurts (falls in this level ${touched.deaths})`);
    const res = game(run, "respawned.json");
    t.ok(Math.abs(res.heroX - 128.5) < 0.3 && res.hurt === 0, `Wick comes back at the lamp (${res.heroX.toFixed(2)})`);
  },
});
