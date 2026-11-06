// The seats (src/seats.ts): each hero is a seat's, carrying og.PlayerInput,
// and every control reaches it through that alone. Alone (and in a room
// before it runs) seat 0 takes both seats' keys and any pad: D and the
// arrows, pad 1's and pad 2's sticks, Enter and ShiftRight all drive the one
// hero. Two on one device (src/creation.ts's local-multiplayer), WASD, the
// left keys and pad 1 are seat 0's, the arrows, the right keys and pad 2
// seat 1's: each moves its own hero while the other stays put. The camera's
// view stands on the heroes' middle.
import { playtest } from "@origingame/engine/playtest";
import { MAP, component, hold, position, press, report } from "./lib.ts";

interface Game {
  value: { heroX: number[]; heroY: number[] };
}

export default playtest({
  name: "each seat's keys and pad reach its og.PlayerInput and drive its own hero",
  commands: [
    "0.1:slime:35:3",
    "0.1:get:og.Lockstep:lock.json",
    "0.1:get:Game:g0.json",
    hold("KeyD", 0.2, 0.7),
    "0.8:get:Game:g1.json",
    "0.8:dump_entity:hero:keys.json",
    "0.8:gamepad:-:-1,0",
    "1.3:gamepad:-:0,0",
    "1.4:get:Game:g2.json",
    hold("ArrowDown", 1.4, 1.9),
    "2.0:get:Game:g3.json",
    "2.0:gamepad:-@1:1,0",
    "2.5:gamepad:-@1:0,0",
    "2.6:get:Game:g4.json",
    press("Enter", 2.6),
    press("ShiftRight", 3.1),
    "3.5:get:Game:g5.json",
    "3.5:dump_entity:hero:end.json",
    "3.5:dump_entity:view:view.json",
    "3.6:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const g = (i: number) => run.json<Game>(`g${i}.json`).value;
    const at = (i: number, seat: number) => ({ x: g(i).heroX[seat]!, y: g(i).heroY[seat]! });
    const seats = g(0).heroX.length;
    const local = seats === 2 && run.json<{ value: { enabled: boolean } }>("lock.json").value.enabled === false;
    t.equal(component<{ player: number }>(run, "keys.json", "og.PlayerInput").player, 0, "the hero is seat 0's og.PlayerInput");
    t.near(at(0, 0).x, MAP.spawn.x, 0.01, "seat 0's hero starts at the spawn");
    t.ok(at(1, 0).x > at(0, 0).x + 1.5, `D walked seat 0's hero right (x ${at(1, 0).x.toFixed(2)})`);
    t.ok(at(2, 0).x < at(1, 0).x - 1.5, `pad 1's stick walked it back (x ${at(2, 0).x.toFixed(2)})`);
    // The arrows and pad 2: seat 1's hero with two on the device, the one hero otherwise.
    const mover = local ? 1 : 0;
    const still = local ? 0 : 1;
    t.ok(at(3, mover).y < at(2, mover).y - 1.5, `the arrows walked seat ${mover}'s hero down (y ${at(2, mover).y.toFixed(2)} to ${at(3, mover).y.toFixed(2)})`);
    t.ok(at(4, mover).x > at(3, mover).x + 1.5, `pad 2's stick walked seat ${mover}'s hero right (x ${at(3, mover).x.toFixed(2)} to ${at(4, mover).x.toFixed(2)})`);
    if (seats === 2) {
      t.near(at(4, still).x, at(2, still).x, 0.01, `seat ${still}'s hero stayed put in x meanwhile`);
      t.near(at(4, still).y, at(2, still).y, 0.01, `and in y`);
    }
    t.equal(report(run, "end.json").swings, 1, "Enter swung a sword");
    const dashes = component<{ dashes: number }>(run, "end.json", "Hero").dashes;
    t.equal(dashes, local ? 0 : 1, local ? "ShiftRight is seat 1's dash: seat 0's hero did not dash" : "ShiftRight dashed the hero");
    if (local) t.ok(at(5, 1).x > at(4, 1).x + 1, `seat 1's hero dashed (x ${at(4, 1).x.toFixed(2)} to ${at(5, 1).x.toFixed(2)})`);
    const view = position(run, "view.json");
    let mx = 0;
    let my = 0;
    for (let s = 0; s < seats; s++) {
      mx += at(5, s).x / seats;
      my += at(5, s).y / seats;
    }
    t.ok(Math.abs(view.x - mx) < 0.3 && Math.abs(view.y - my) < 0.3, `the camera's view stands on the heroes' middle (${view.x.toFixed(2)},${view.y.toFixed(2)} for ${mx.toFixed(2)},${my.toFixed(2)})`);
  },
});
