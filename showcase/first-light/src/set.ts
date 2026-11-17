// What stands on the land: the oak, the shepherd and the dog on the knoll,
// the paddock's wall and sheep, the village, the jetty and the mill.
import { og, math } from "@origingame/engine";
import type { Asset, Entity, TerrainData, World } from "@origingame/engine";
import { MILL, PADDOCK, VILLAGE } from "./layout.ts";
import { groundAt, mul, must, rad } from "./util.ts";

/** Where the camera stands and what it looks along (a heading from +z towards +x, in degrees). */
export const VIEW = { x: 1.2, z: 9, eye: 2.25, heading: 167 };

/** A point `distance` metres from the camera, `bearing` degrees right of the middle of its view. */
export function inView(bearing: number, distance: number): { x: number; z: number } {
  const h = rad(VIEW.heading - bearing);
  return { x: VIEW.x + math.sin(h) * distance, z: VIEW.z + math.cos(h) * distance };
}

/** The jetty model's origin against the lake's surface: its deck stands just clear of the water. */
const JETTY_Y = -1.15;

export function buildSet(world: World, land: TerrainData): void {
  const models = new Map<string, Asset>();
  const model = (path: string) => {
    let a = models.get(path);
    if (a === undefined) {
      a = must(og.load.gltf(world, path));
      models.set(path, a);
    }
    return a;
  };
  /** A model standing on the ground at (x, z), its front turned to `heading` degrees. */
  const stand = (name: string, path: string, x: number, z: number, heading: number, opt: { scale?: number; lift?: number; sway?: number } = {}): Entity => {
    const s = opt.scale ?? 1;
    const e = must(world.spawn({ name, position: { x, y: groundAt(world, land, x, z) + (opt.lift ?? 0), z }, quaternion: og.transform.fromHeading(rad(heading)), scale: { x: s, y: s, z: s } }));
    must(world.insert(e, og.Model, { asset: model(path), sway: opt.sway ?? 0 }));
    return e;
  };

  // The knoll: the oak at the frame's left edge, the shepherd and the dog beside it.
  const oak = inView(-39, 6);
  stand("oak", "plants/oak-old.ogt", oak.x, oak.z, 40, { sway: 0.06, lift: -0.25 });
  const boy = inView(-20, 5.6);
  stand("shepherd", "models/shepherd.glb", boy.x, boy.z, VIEW.heading + 8);
  const crook = inView(-17.2, 5.85);
  const staff = stand("crook", "models/crook.ogt", crook.x, crook.z, VIEW.heading + 60);
  must(world.set(staff, og.Transform, { quaternion: mul(og.transform.fromHeading(rad(VIEW.heading + 60)), og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, rad(-4))) }));
  const dog = inView(-12.5, 5.9);
  stand("dog", "models/dog.glb", dog.x, dog.z, VIEW.heading + 6);
  // Low stones in the grass at the frame's foot: bearing, distance, scale, heading.
  const stones: [number, number, number, number][] = [[-33, 3, 0.55, 20], [14, 4.2, 0.6, 140], [30, 5.5, 0.5, 260], [-8, 8.5, 0.45, 80], [22, 11, 0.7, 200]];
  for (const [i, [bearing, distance, scale, heading]] of stones.entries()) {
    const p = inView(bearing, distance);
    stand(`boulder-${i}`, "models/boulder.glb", p.x, p.z, heading, { scale, lift: -0.35 * scale });
  }
  const caps = inView(-31, 5);
  stand("mushrooms", "models/mushrooms.glb", caps.x, caps.z, 30, { scale: 1.2 });

  // The paddock: a dry stone wall round it, a gate on the near side, sheep inside.
  const corners = PADDOCK.wall;
  let piece = 0;
  for (let c = 0; c < corners.length; c++) {
    const [ax, az] = corners[c]!, [bx, bz] = corners[(c + 1) % corners.length]!;
    const length = Math.sqrt((bx - ax) * (bx - ax) + (bz - az) * (bz - az));
    const count = Math.round(length / 3.9);
    for (let k = 0; k < count; k++) {
      const x = ax + ((bx - ax) * (k + 0.5)) / count, z = az + ((bz - az) * (k + 0.5)) / count;
      const gate = Math.sqrt((x - PADDOCK.gate.x) * (x - PADDOCK.gate.x) + (z - PADDOCK.gate.z) * (z - PADDOCK.gate.z)) < 2;
      const y0 = groundAt(world, land, ax + ((bx - ax) * k) / count, az + ((bz - az) * k) / count), y1 = groundAt(world, land, ax + ((bx - ax) * (k + 1)) / count, az + ((bz - az) * (k + 1)) / count);
      // The piece runs along its own x: turned to the wall's line, then tipped to the slope.
      const yaw = og.transform.fromHeading(math.atan2(bx - ax, bz - az) - Math.PI / 2);
      const tip = og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, math.atan2(y1 - y0, length / count));
      const e = must(world.spawn({ name: gate ? "gate" : `wall-${piece++}`, position: { x, y: (y0 + y1) / 2 - (gate ? 0.05 : 0.12), z }, quaternion: mul(yaw, tip), scale: gate ? { x: 1.5, y: 1, z: 1 } : { x: 1, y: 1.25, z: 1.25 } }));
      must(world.insert(e, og.Model, { asset: model(gate ? "models/gate.glb" : "models/wall.glb") }));
    }
  }
  for (const [i, [x, z]] of PADDOCK.flock.entries()) stand(`sheep-${i}`, i % 3 === 1 ? "models/sheep-stand.glb" : "models/sheep-graze.glb", x, z, (i * 137) % 360, { scale: 0.92 + 0.04 * (i % 4) });

  // The village, its jetty and boat.
  for (const [i, h] of VILLAGE.houses.entries()) stand(`house-${i}`, `models/${h.model}.glb`, h.x, h.z, h.turn, { lift: -0.25 });
  // The jetty runs out from where the ground meets the water.
  const out = { x: math.sin(rad(VILLAGE.jetty.heading)), z: math.cos(rad(VILLAGE.jetty.heading)) };
  let reach = 0;
  while (reach < 60 && groundAt(world, land, VILLAGE.jetty.x + out.x * reach, VILLAGE.jetty.z + out.z * reach) > 0.25) reach += 0.5;
  const along = (d: number, side: number) => ({ x: VILLAGE.jetty.x + out.x * (reach + d) - out.z * side, z: VILLAGE.jetty.z + out.z * (reach + d) + out.x * side });
  const mid = along(4.5, 0);
  const jetty = must(world.spawn({ name: "jetty", position: { x: mid.x, y: JETTY_Y, z: mid.z }, quaternion: og.transform.fromHeading(rad(VILLAGE.jetty.heading - 90)) }));
  must(world.insert(jetty, og.Model, { asset: model("models/jetty.glb") }));
  const moored = along(7, 3.2);
  const boat = must(world.spawn({ name: "boat", position: { x: moored.x, y: -0.22, z: moored.z }, quaternion: og.transform.fromHeading(rad(VILLAGE.jetty.heading + 70)) }));
  must(world.insert(boat, og.Model, { asset: model("models/boat.glb") }));

  // The mill on its knoll, facing the lake; its sails turn on the cap's axle.
  const tower = stand("mill", "models/windmill-tower.glb", MILL.x, MILL.z, 330, { lift: -0.4 });
  const sails = must(world.spawn({ name: "mill-sails", parent: tower, position: { x: 0, y: 10.6, z: 3.9 } }));
  must(world.insert(sails, og.Model, { asset: model("models/windmill-sails.glb") }));
}
