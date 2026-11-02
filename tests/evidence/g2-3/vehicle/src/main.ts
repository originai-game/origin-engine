// The G2-3 vehicle sample: a 1,500 kg truck with VEHICLE_DRIVETRAIN on six
// og.VehicleWheel children (front pair steering, rear four driven, the
// last pair on the handbrake). It pulls away at full throttle and the
// automatic gearbox shifts up; at 4 s it turns at full lock with the
// handbrake pulled for half a second, and the tail steps out. Dark marks
// are laid where a rear tyre slides sideways faster than 0.8 m/s.
// tests/evidence/g2-3/NOTES.md has the commands and what each frame shows.

import { og, defineComponent, defineSystem, u32, f32, quat, vec3, SHAPE_BOX, VEHICLE_DRIVETRAIN } from "@origingame/engine";
import type { Entity, Geometry, Material, World } from "@origingame/engine";

const must = <T>(r: { ok: true; value: T } | { ok: false; reason: string }): T => {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
};

/** What must replay: on the entity "drive". */
const Drive = defineComponent("Drive", {
  /** Ticks since the start. */
  tick: u32(0),
  /** Skid marks laid. */
  marks: u32(0),
  /** The highest gear reached. */
  topGear: u32(0),
  /** The largest rear-tyre sideways speed seen (m/s). */
  slide: f32(0),
});

const MAX_MARKS = 200;
let wheelMeshes: Entity[] = [];
/** Marks spawned hidden at the start (a spawn inside a system is deferred), shown as they are laid. */
let markPool: Entity[] = [];

export const Driver = defineSystem("Driver", { writes: [Drive, og.Vehicle, og.Transform, og.Mesh, og.Camera] }, (_q, _time, world) => {
  const drive = world.find("drive")!;
  const truck = world.find("truck")!;
  const d = must(world.get(drive, Drive));
  const t = d.tick;
  // The script: wait, full throttle, a handbrake turn, then drive on in a bend.
  const input =
    t < 30 ? { throttle: 0, steer: 0, handbrake: 0 }
    : t < 240 ? { throttle: 1, steer: 0, handbrake: 0 }
    : t < 270 ? { throttle: 0.3, steer: 1, handbrake: 1 }
    : { throttle: 1, steer: 0.4, handbrake: 0 };
  must(world.set(truck, og.Vehicle, input));
  const v = must(world.get(truck, og.Vehicle));

  // The wheels drawn where the last step left them (a cylinder is upright: turn it onto its axle).
  const polled = og.physics.vehicleWheels(world, truck);
  // None before the first step.
  const wheels = polled.ok ? polled.value : [];
  const onAxle = og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2);
  let marks = d.marks;
  let slide = d.slide;
  for (let i = 0; i < wheels.length && i < wheelMeshes.length; i++) {
    const w = wheels[i]!;
    must(world.set(wheelMeshes[i]!, og.Transform, { position: w.position, quaternion: quat.multiply(w.rotation, onAxle) }));
    const rear = i >= 2;
    if (rear) slide = Math.max(slide, w.sideSpeed);
    if (rear && w.contact && w.sideSpeed > 0.8 && t % 2 === 0 && marks < MAX_MARKS) {
      const m = markPool[marks]!;
      must(world.set(m, og.Transform, { position: { x: w.position.x, y: 0.012, z: w.position.z } }));
      must(world.set(m, og.Mesh, { visible: true }));
      marks += 1;
    }
  }

  // A chase camera behind and above the truck.
  const p = must(world.get(truck, og.Transform));
  const back = vec3.applyQuaternion({ x: 0, y: 0, z: -1 }, p.quaternion);
  const cam = must(world.resource(og.Camera));
  const eye = { x: p.position.x + back.x * 14 + 6, y: 9, z: p.position.z + back.z * 14 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, p.position);

  must(world.set(drive, Drive, { tick: t + 1, marks, topGear: Math.max(d.topGear, v.gear), slide }));
});

function mesh(world: World, e: Entity, geometry: Geometry, material: Material, scale: { x: number; y: number; z: number }): void {
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Driver],
  start(world: World) {
    const cam = must(world.resource(og.Camera));
    cam.fov = 55;
    cam.background = { r: 0.05, g: 0.06, b: 0.09 };
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.45 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });

    const box = must(og.render.boxGeometry(world, {}));
    const tyre = must(og.render.cylinderGeometry(world, { radiusTop: 0.4, radiusBottom: 0.4, height: 0.3 }));
    const mat = (r: number, g: number, b: number, roughness = 0.7) => must(og.render.standardMaterial(world, { color: { r, g, b }, roughness }));
    const markMaterial = mat(0.06, 0.06, 0.07, 1);
    markPool = [];
    for (let i = 0; i < MAX_MARKS; i++) {
      const m = must(world.spawn({ name: `mark-${i}`, scale: { x: 0.35, y: 0.01, z: 0.35 } }));
      world.insert(m, og.Mesh, { geometry: box, material: markMaterial, visible: false, receiveShadow: true });
      markPool.push(m);
    }

    // A 400 m road with a stripe every 10 m across it, for the speed to read.
    const road = must(world.spawn({ name: "road", position: { x: 0, y: -0.5, z: 150 } }));
    world.insert(road, og.Collider, { shape: SHAPE_BOX });
    mesh(world, road, box, mat(0.3, 0.29, 0.27, 0.95), { x: 400, y: 1, z: 400 });
    const stripe = mat(0.8, 0.78, 0.7, 0.9);
    for (let i = 0; i < 30; i++) {
      const s = must(world.spawn({ name: `stripe-${i}`, position: { x: 0, y: 0.005, z: i * 10 } }));
      mesh(world, s, box, stripe, { x: 30, y: 0.01, z: 0.3 });
    }

    // The truck: a 1,500 kg chassis facing +z, and six wheel children.
    const truck = must(world.spawn({ name: "truck", position: { x: 0, y: 1.2, z: 0 } }));
    world.insert(truck, og.RigidBody, { mass: 1500 });
    // Its scale stays 1: a parent's scale scales its children's offsets, the wheels' mounts too.
    world.insert(truck, og.Collider, { shape: SHAPE_BOX, size: { x: 1.8, y: 0.6, z: 5 } });
    const body = must(world.spawn({ name: "body", parent: truck }));
    mesh(world, body, box, mat(0.85, 0.3, 0.12, 0.5), { x: 1.8, y: 0.6, z: 5 });
    // A short-shifting automatic: up at 3,000 rpm, down at 1,500.
    world.insert(truck, og.Vehicle, { model: VEHICLE_DRIVETRAIN, shiftUpRpm: 3000, shiftDownRpm: 1500 });
    const cab = must(world.spawn({ name: "cab", parent: truck, position: { x: 0, y: 0.6, z: 1.6 } }));
    mesh(world, cab, box, mat(0.2, 0.22, 0.26, 0.3), { x: 1.7, y: 0.7, z: 1.4 });
    const black = mat(0.05, 0.05, 0.05, 0.9);
    wheelMeshes = [];
    let n = 0;
    for (const z of [1.9, -0.6, -1.9]) {
      for (const x of [0.85, -0.85]) {
        const wheel = must(world.spawn({ name: `wheel-${n}`, parent: truck, position: { x, y: -0.2, z } }));
        world.insert(wheel, og.VehicleWheel, { steer: z > 0 ? 1 : 0, drive: z < 0, handbrake: z < -1 });
        const drawn = must(world.spawn({ name: `tyre-${n}` }));
        world.insert(drawn, og.Mesh, { geometry: tyre, material: black, castShadow: true, receiveShadow: true });
        wheelMeshes.push(drawn);
        n += 1;
      }
    }

    world.insert(must(world.spawn({ name: "drive" })), Drive);
  },
});
