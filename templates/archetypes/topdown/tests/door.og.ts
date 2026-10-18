// The door: without the key the doorway's wall cells block the hero; the
// key is picked up by walking over it; with it the door opens on approach
// (og.tilemap.setTile clears the wall terrain's cells, re-tiling their
// neighbours and dropping their colliders), the key is spent, and the hero
// walks through into the east room.
import { playtest } from "@origingame/engine/playtest";
import { hold, MAP, position, report } from "./lib.ts";

export default playtest({
  name: "the doorway blocks until the key, then opens and leads into room 2",
  commands: [
    "0.1:slime:35:3",
    "0.2:place:17.5:6",
    hold("KeyD", 0.2, 1.0),
    "1.1:dump_entity:hero:locked.json",
    // The key at (16.5, 2.5).
    "1.2:place:15.2:2.3",
    hold("KeyD", 1.2, 1.6),
    "1.7:dump_entity:hero:key.json",
    "1.8:place:17.5:6",
    hold("KeyD", 1.8, 3.2),
    "3.3:dump_entity:hero:through.json",
    "3.3:screenshot:door.png",
    "3.4:quit",
  ].join(";"),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const locked = report(run, "locked.json");
    t.equal(locked.doorOpen, false, "the door stays shut without the key");
    t.equal(locked.doorWall, 1, "the doorway's cell is filled wall terrain");
    t.near(position(run, "locked.json").x, 19 - 0.3, 0.03, "the doorway's wall stops the hero");
    t.equal(locked.room, 1, "still in room 1");
    t.equal(report(run, "key.json").hasKey, true, "walking over the key picks it up");
    const through = report(run, "through.json");
    t.equal(through.doorOpen, true, "the key opened the door");
    t.equal(through.hasKey, false, "the key was spent");
    t.equal(through.doorWall, 0, "the doorway's wall terrain cell is cleared");
    t.ok(position(run, "through.json").x > MAP.split + 1, `the hero walked through (x ${position(run, "through.json").x.toFixed(2)})`);
    t.equal(through.room, 2, "the hero is in room 2");
  },
});
