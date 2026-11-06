// The trooper plays from its seat (src/seats.ts): the keys, the first pad,
// the mouse and the touch HUD each reach seat 0's og.PlayerInput, and the
// game moves, turns and fires from that input alone. W walks the trooper
// (north, or up the screen in the isometric view), the pad's left stick
// pushed right walks it the other way round, its right stick and the mouse
// turn the look (the mouse while the pointer is locked), F fires a round from the magazine, and the touch stick
// pushed up walks it too.
import { playtest } from "@origingame/engine/playtest";
import { ISOMETRIC, position, value } from "./lib.ts";

interface Input { player: number; x: number; y: number; buttons: number }
interface Look { yaw: number; pitch: number }
interface Trooper { mag: number }

export default playtest({
  name: "keys, a pad, the mouse and the touch stick reach seat 0's og.PlayerInput and move, turn and fire the trooper",
  commands: [
    "0.1:viewport:phone",
    "0.3:run",
    "0.4:warp:-3:9",
    "1.0:get:hero.og.Transform:a.json",
    "1.1:key_down:KeyW",
    "1.5:get:hero.og.PlayerInput:keys.json",
    "1.7:key_up:KeyW",
    "1.8:get:hero.og.Transform:b.json",
    "1.9:gamepad:-@0:1,0",
    "2.3:get:hero.og.PlayerInput:pad.json",
    "2.5:gamepad:-@0:0,0",
    "2.6:get:hero.og.Transform:c.json",
    "2.6:get:hero.SeatLook:look-a.json",
    "2.7:gamepad:-@0:0,0,1,0",
    "3.1:gamepad:-@0:0,0,0,0",
    "3.2:get:hero.SeatLook:look-b.json",
    "3.25:pointer_lock:on",
    "3.3:mouse_move:-240,0",
    "3.35:pointer_lock:off",
    "3.4:get:hero.SeatLook:look-c.json",
    "3.5:get:hero.Hero:full.json",
    "3.5:key_down:KeyF",
    "3.6:get:hero.og.PlayerInput:fire.json",
    "3.8:key_up:KeyF",
    "3.9:get:hero.Hero:fired.json",
    "4.0:touch:1:down:TouchMove0/Stick",
    "4.05:touch:1:move:84,560",
    "4.4:get:hero.og.PlayerInput:touch.json",
    "4.6:touch:1:up:84,560",
    "4.7:get:hero.og.Transform:d.json",
    "4.8:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const keys = value<Input>(run, "keys.json");
    t.equal(keys.player, 0, "the trooper carries seat 0's og.PlayerInput");
    t.ok(Math.abs(keys.y - 1) < 1e-3 && Math.abs(keys.x) < 1e-3, `W reaches it as forward (${keys.x}, ${keys.y})`);
    const [ax, , az] = position(run, "a.json");
    const [bx, , bz] = position(run, "b.json");
    t.ok(Math.hypot(bx - ax, bz - az) > 1, `and walks the trooper (${Math.hypot(bx - ax, bz - az).toFixed(2)} m)`);
    if (ISOMETRIC) t.ok(bx < ax - 0.5 && bz < az - 0.5, `up the screen, north-west in the arena (${(bx - ax).toFixed(2)}, ${(bz - az).toFixed(2)})`);
    else t.ok(az - bz > 1 && Math.abs(bx - ax) < 0.2, `north, the way it looks (z ${az.toFixed(2)} → ${bz.toFixed(2)})`);
    const pad = value<Input>(run, "pad.json");
    t.ok(pad.x > 0.9 && Math.abs(pad.y) < 0.1, `the first pad's stick reaches it (${pad.x.toFixed(2)}, ${pad.y.toFixed(2)})`);
    const [cx, , cz] = position(run, "c.json");
    t.ok(Math.hypot(cx - bx, cz - bz) > 0.8, `and walks the trooper (${Math.hypot(cx - bx, cz - bz).toFixed(2)} m)`);
    if (!ISOMETRIC) t.ok(cx - bx > 0.8, `east, to its right (x ${bx.toFixed(2)} → ${cx.toFixed(2)})`);
    const la = value<Look>(run, "look-a.json").yaw;
    const lb = value<Look>(run, "look-b.json").yaw;
    const lc = value<Look>(run, "look-c.json").yaw;
    t.ok(Math.abs(lb - la) > 0.3, `the pad's right stick turns the look (yaw ${la.toFixed(2)} → ${lb.toFixed(2)})`);
    t.ok(Math.abs(lc - lb) > 0.3, `and so does the mouse (yaw ${lb.toFixed(2)} → ${lc.toFixed(2)})`);
    t.ok((value<Input>(run, "fire.json").buttons & 1) !== 0, "F reaches it as Fire");
    const full = value<Trooper>(run, "full.json").mag;
    const fired = value<Trooper>(run, "fired.json").mag;
    t.ok(fired < full && fired >= full - 4, `and fires rounds from the magazine (${full} → ${fired})`);
    const touch = value<Input>(run, "touch.json");
    t.ok(touch.y > 0.9, `the touch stick pushed up reaches it as forward (${touch.x.toFixed(2)}, ${touch.y.toFixed(2)})`);
    const [dx, , dz] = position(run, "d.json");
    t.ok(Math.hypot(dx - cx, dz - cz) > 1, `and walks the trooper on (${Math.hypot(dx - cx, dz - cz).toFixed(2)} m)`);
  },
});
