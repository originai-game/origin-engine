// The strider plays from its seat (src/seats.ts): the keys, the first pad
// and the touch HUD's stick each reach seat 0's og.PlayerInput, and the
// game moves the strider from that input alone. W walks it north, the
// pad's stick pushed right walks it east, the touch stick pushed all the
// way up runs it (its Run bit), and E turns its view right (the view's
// heading is the strider's own state, Hero.yaw).
import { playtest } from "@origingame/engine/playtest";
import { position, value } from "./lib.ts";

interface Input { player: number; x: number; y: number; buttons: number }
interface Hero { seat: number; yaw: number; speed: number }

export default playtest({
  name: "keys, a pad and the touch stick reach seat 0's og.PlayerInput and move the strider",
  commands: [
    "0.1:viewport:phone",
    "0.3:run",
    "1.0:get:hero.og.Transform:a.json",
    "1.1:key_down:KeyW",
    "1.5:get:hero.og.PlayerInput:keys.json",
    "1.7:key_up:KeyW",
    "1.8:get:hero.og.Transform:b.json",
    "1.9:gamepad:-@0:1,0",
    "2.3:get:hero.og.PlayerInput:pad.json",
    "2.5:gamepad:-@0:0,0",
    "2.6:get:hero.og.Transform:c.json",
    "2.7:touch:1:down:Stick",
    "2.75:touch:1:move:86,600",
    "3.2:get:hero.og.PlayerInput:touch.json",
    "3.2:get:hero.Hero:running.json",
    "3.4:touch:1:up:86,600",
    "3.5:get:hero.og.Transform:d.json",
    "3.5:get:hero.Hero:before-turn.json",
    "3.6:key_down:KeyE",
    "4.1:key_up:KeyE",
    "4.2:get:hero.Hero:turned.json",
    "4.3:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const keys = value<Input>(run, "keys.json");
    t.equal(keys.player, 0, "the strider carries seat 0's og.PlayerInput");
    t.ok(Math.abs(keys.y - 1) < 1e-3 && Math.abs(keys.x) < 1e-3, `W reaches it as forward (${keys.x}, ${keys.y})`);
    const [ax, , az] = position(run, "a.json");
    const [bx, , bz] = position(run, "b.json");
    t.ok(az - bz > 1 && Math.abs(bx - ax) < 0.2, `and walks the strider north (z ${az.toFixed(2)} → ${bz.toFixed(2)})`);
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9 && Math.abs(pad.y) < 0.1, `the first pad's stick reaches it (${pad.x.toFixed(2)}, ${pad.y.toFixed(2)})`);
    const [cx, , cz] = position(run, "c.json");
    t.ok(cx - bx > 0.8 && Math.abs(cz - bz) < 0.5, `and walks the strider east (x ${bx.toFixed(2)} → ${cx.toFixed(2)})`);
    const touch = value<Input>(run, "touch.json");
    t.ok(touch.y > 0.9 && (touch.buttons & 2) !== 0, `the touch stick pushed all the way reaches it as forward with Run (${touch.y.toFixed(2)}, buttons ${touch.buttons})`);
    t.ok(Math.abs(value<Hero>(run, "running.json").speed - 6) < 0.1, "and runs the strider");
    const [dx, , dz] = position(run, "d.json");
    t.ok(Math.hypot(dx - cx, dz - cz) > 1.5, `which moved on (${Math.hypot(dx - cx, dz - cz).toFixed(2)} m)`);
    const before = value<Hero>(run, "before-turn.json").yaw;
    const turned = value<Hero>(run, "turned.json").yaw;
    t.ok(before - turned > 0.8, `E turns the strider's view right (yaw ${before.toFixed(2)} → ${turned.toFixed(2)})`);
  },
});
