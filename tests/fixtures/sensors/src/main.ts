// A pressure plate (a sensor) and three crates walked across it on fixed
// paths (README.md). Every tick the Plate system checks
// og.physics.sensorContents against where the crates were stepped to and
// against what og.SensorEntered and og.SensorExited added up to.
//
//   og run tests/fixtures/sensors --headless --commands "2:dump_world;3:quit"
import { Entity, og, defineComponent, defineResource, defineSystem, u32 } from "@origingame/engine";
import type { Result, World } from "@origingame/engine";

function must<T>(r: Result<T>): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The plate's half width plus a crate's: a crate on the y = 0.25 line is on the plate while |x| is under it. */
const REACH = 1.25;

/** A crate's x on tick `t`: three walks across the plate, starting at different ticks, from both sides. */
export function path(index: number, tick: number): number {
  if (index === 0) return -3.1 + 0.2 * tick;
  if (index === 1) return 3.1 - 0.16 * Math.max(0, tick - 10);
  return -3.1 + 0.25 * Math.max(0, tick - 25);
}

const Crate = defineComponent("Crate", { index: u32() });

/** What the plate saw: `inside` from sensorContents, `fromEvents` added up from the events (bits by crate index). */
const Plate = defineResource("Plate", {
  plate: Entity(),
  inside: u32(),
  count: u32(),
  fromEvents: u32(),
  entered: u32(),
  exited: u32(),
  pressedTicks: u32(),
});

/** Teleports each kinematic crate to its path's x for this tick; the step at the end of the tick moves its collider. */
const Walk = defineSystem("Walk", { reads: [Crate], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const index = a.column(Crate).index;
    const position = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) position[i * 3] = path(index[i]!, time.tick);
  }
});

const bit = (world: World, e: Entity): number => {
  const c = world.get(e, Crate);
  return c.ok ? 1 << c.value.index : 0;
};

const Check = defineSystem(
  "Plate",
  { reads: [og.Contact, og.SensorEntered, og.SensorExited, Crate], writes: [Plate] },
  (_q, time, world) => {
    const p = must(world.resource(Plate));
    const plate = p.plate;
    for (const e of world.events(og.SensorEntered)) {
      og.assert.always(e.sensor === plate, "only the plate is a sensor");
      og.assert.always((p.fromEvents & bit(world, e.other)) === 0, "a crate enters once before it leaves");
      p.fromEvents |= bit(world, e.other);
      p.entered += 1;
    }
    for (const e of world.events(og.SensorExited)) {
      og.assert.always((p.fromEvents & bit(world, e.other)) !== 0, "a crate leaves only after entering");
      p.fromEvents &= ~bit(world, e.other);
      p.exited += 1;
    }
    const inside = must(og.physics.sensorContents(world, plate));
    let mask = 0;
    for (let i = 0; i < inside.length; i++) {
      og.assert.always(i === 0 || inside[i - 1]! < inside[i]!, "contents in entity order, each once", { inside });
      mask |= bit(world, inside[i]!);
    }
    // The contents are the last step's: the crates where Walk put them on the tick before.
    let expected = 0;
    for (const index of [0, 1, 2]) if (time.tick > 0 && Math.abs(path(index, time.tick - 1)) < REACH) expected |= 1 << index;
    og.assert.always(mask === expected, "the plate holds the crates over it", { tick: time.tick, mask, expected });
    og.assert.always(mask === p.fromEvents, "contents are what the events add up to", { mask, fromEvents: p.fromEvents });
    og.assert.sometimes(inside.length === 2, "two crates on the plate at once");
    if (inside.length > 0) p.pressedTicks += 1;
    p.inside = mask;
    p.count = inside.length;
  },
);

function build(world: World): void {
  const spawnCrate = (index: number, name: string) => {
    const e = must(world.spawn({ name, position: { x: path(index, 0), y: 0.25, z: 0 } }));
    must(world.insert(e, og.RigidBody, { kind: "kinematic" }));
    must(world.insert(e, og.Collider, { shape: "box", size: { x: 0.5, y: 0.5, z: 0.5 } }));
    must(world.insert(e, Crate, { index }));
  };
  // One crate spawned before the plate (a lower id than the sensor's), two after.
  spawnCrate(0, "crate-a");
  const plate = must(world.spawn({ name: "plate", position: { x: 0, y: 0.12, z: 0 } }));
  must(world.insert(plate, og.Collider, { shape: "box", size: { x: 2, y: 0.2, z: 2 }, sensor: true }));
  must(world.resource(Plate)).plate = plate;
  spawnCrate(1, "crate-b");
  spawnCrate(2, "crate-c");
  // The floor the crates slide over, just under the plate: solid, never inside it.
  const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
  must(world.insert(floor, og.Collider, { shape: "box", size: { x: 20, y: 1, z: 20 } }));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Plate],
  systems: [Walk, Check],
  start: build,
  facts: (world) => {
    const p = world.resource(Plate);
    return p.ok
      ? { inside: p.value.inside, count: p.value.count, entered: p.value.entered, exited: p.value.exited, pressedTicks: p.value.pressedTicks }
      : {};
  },
});
