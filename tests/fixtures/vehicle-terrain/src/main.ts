// The vehicle on terrain: one og.Vehicle (a raycast car on a box chassis)
// driven over cooked hills (og.terrain.load, og.Terrain and a
// SHAPE_HEIGHTFIELD collider), with a HUD speed readout (og.ui), a chase
// camera that eases after the car, tyre smoke while the rear slides
// (og.ParticleSystem) and skid marks (a ring of og.Decal), and an engine
// tone whose pitch follows the speed (og.audio.createSound). README.md says
// which tests read it.
//
//   og run tests/fixtures/vehicle-terrain --commands "0.2:key_down:KeyW;2:key_down:KeyD;3:key_up:KeyD;3.5:quit"
import { og, defineComponent, defineResource, defineSystem, f32, u32, BODY_DYNAMIC, SHAPE_BOX, SHAPE_HEIGHTFIELD } from "@origingame/engine";
import type { Entity, Quaternion, Vector3, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Skid marks kept at once: the oldest is reused. */
const MARKS = 24;
/** Where the car starts, on the hills' middle. */
const START = { x: 0, z: -40 };
/** The road: its width and where it runs along z, from behind the start. */
const ROAD = { w: 14, z0: -46, z1: -6 };

/** The run as the HUD, the camera and the playtest read it. */
export const Drive = defineResource("Drive", { kmh: f32(), slip: f32(), marks: u32(), smoking: u32(), lowest: f32(99) });
/** A skid mark's place in the ring. */
const Mark = defineComponent("Mark", { slot: u32() });

const GROW = { kind: "grow" } as const;

/** v turned by the unit quaternion q. */
function rotate(v: Vector3, q: Quaternion): Vector3 {
  const tx = 2 * (q.y * v.z - q.z * v.y), ty = 2 * (q.z * v.x - q.x * v.z), tz = 2 * (q.x * v.y - q.y * v.x);
  return { x: v.x + q.w * tx + (q.y * tz - q.z * ty), y: v.y + q.w * ty + (q.z * tx - q.x * tz), z: v.z + q.w * tz + (q.x * ty - q.y * tx) };
}

/** The product a * b (b first, then a). */
function mul(a: Quaternion, b: Quaternion): Quaternion {
  return {
    x: a.w * b.x + a.x * b.w + a.y * b.z - a.z * b.y,
    y: a.w * b.y - a.x * b.z + a.y * b.w + a.z * b.x,
    z: a.w * b.z + a.x * b.y - a.y * b.x + a.z * b.w,
    w: a.w * b.w - a.x * b.x - a.y * b.y - a.z * b.z,
  };
}

function find(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`no ${name}`);
  return e;
}

/** A sine with its first overtones, one second looped: the engine's hum, pitched by og.AudioSource.pitch. */
function engineTone(world: World) {
  const rate = 48000;
  const samples = new Float32Array(rate);
  for (let i = 0; i < rate; i++) {
    const t = (i / rate) * Math.PI * 2 * 55;
    samples[i] = 0.3 * Math.sin(t) + 0.15 * Math.sin(2 * t) + 0.08 * Math.sin(3 * t);
  }
  return must(og.audio.createSound(world, { samples, rate, channels: 1 }));
}

function build(world: World): void {
  og.ui.configure(world, {
    font: must(og.load.font(world, "fonts/InstrumentSans-Regular.ttf")),
    monoFont: must(og.load.font(world, "fonts/PTMono-Regular.ttf")),
  });
  const info = must(og.terrain.load(world, "terrain/hills.ogt"));
  const hills = must(world.spawn({ name: "hills" }));
  must(world.insert(hills, og.Terrain, { data: info.data }));
  must(world.insert(hills, og.Collider, { shape: SHAPE_HEIGHTFIELD, mesh: info.geometry, friction: 1 }));
  const ground = must(og.terrain.sample(world, { data: info.data, origin: { x: 0, y: 0, z: 0 }, x: START.x, z: START.z })).height;
  // A flattened track in the terrain itself: its heightfield carries the
  // car and its surface receives the skid decals, with no covering mesh.
  for (let z = ROAD.z0; z <= ROAD.z1; z += 5) {
    must(og.terrain.edit(world, info.data, { mode: "flatten", center: { x: START.x, y: ground + 0.1, z }, radius: ROAD.w, strength: 1, falloff: 0.4 }));
  }

  const car = must(world.spawn({ name: "car", position: { x: START.x, y: ground + 1, z: START.z } }));
  must(world.insert(car, og.RigidBody, { kind: BODY_DYNAMIC, mass: 1000, angularDamping: 0.4, ccd: 1 }));
  must(world.insert(car, og.Collider, { shape: SHAPE_BOX, size: { x: 1.7, y: 0.25, z: 4 }, friction: 0.05 }));
  must(world.insert(car, og.Velocity, {}));
  must(world.insert(car, og.Vehicle, { wheelBase: 2.6, track: 1.7, wheelRadius: 0.35, mountHeight: -0.3, suspensionRest: 0.3, suspensionTravel: 0.18, suspensionStiffness: 50, suspensionDamping: 4.5, maxSteer: 0.5, rollInfluence: 0.1, engineForce: 6000, maxSpeed: 30, grip: 1.2, driftGrip: 0.6 }));
  const box = must(og.render.boxGeometry(world, {}));
  const paint = must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.15, b: 0.1 }, metalness: 0.3, roughness: 0.4 }));
  const body = must(world.spawn({ name: "body", parent: car, position: { x: 0, y: 0.25, z: 0 }, scale: { x: 1.7, y: 0.6, z: 4 } }));
  must(world.insert(body, og.Mesh, { geometry: box, material: paint, castShadow: true }));
  // Four wheels drawn at the chassis' corners (the raycast wheels have no mesh of their own).
  const tyre = must(og.render.cylinderGeometry(world, { radiusTop: 0.35, radiusBottom: 0.35, height: 0.25, radialSegments: 16 }));
  const rubber = must(og.render.standardMaterial(world, { color: { r: 0.06, g: 0.06, b: 0.06 }, roughness: 0.8 }));
  const across = og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, Math.PI / 2);
  for (const [x, z] of [[-0.85, 1.3], [0.85, 1.3], [-0.85, -1.3], [0.85, -1.3]] as const) {
    const w = must(world.spawn({ name: `wheel-${x < 0 ? "l" : "r"}${z > 0 ? "f" : "b"}`, parent: car, position: { x, y: -0.3, z }, quaternion: across }));
    must(world.insert(w, og.Mesh, { geometry: tyre, material: rubber, castShadow: true }));
  }
  must(world.insert(car, og.AudioSource, { sound: engineTone(world), bus: "sfx", loop: true, volume: 0.4, pitch: 0.6 }));

  const smoke = must(og.load.emitter(world, "fx/smoke.ogt"));
  for (const side of [-1, 1]) {
    const e = must(world.spawn({ name: `smoke-${side < 0 ? "l" : "r"}`, parent: car, position: { x: side * 0.85, y: -0.4, z: -1.3 } }));
    // Not emitting until a slide starts (stopTick before startTick).
    must(world.insert(e, og.ParticleSystem, { emitter: smoke, seed: side < 0 ? 11 : 12, startTick: 1, stopTick: 0 }));
  }
  for (let i = 0; i < MARKS; i++) {
    const e = must(world.spawn({ name: `mark-${i}`, position: { x: 0, y: -60, z: 0 }, scale: { x: 0.3, y: 1.2, z: 1 } }));
    must(world.insert(e, og.Decal, { color: { r: 0.05, g: 0.05, b: 0.05 }, opacity: 0, angleFade: 80 }));
    must(world.insert(e, Mark, { slot: i }));
  }

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.3, y: -0.8, z: 0.4 }) }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 2.6, castShadow: true }));
  must(world.resource(og.RenderSettings)).sky = true;
  const cam = must(world.resource(og.Camera));
  cam.position = { x: START.x, y: ground + 4, z: START.z - 8 };
  cam.quaternion = og.transform.lookAt(cam.position, { x: START.x, y: ground + 1, z: START.z });
}

/** W/S throttle and brake, A/D steer, Space the handbrake; the speed and slip read back. */
const Driver = defineSystem("Driver", { reads: [og.Input], writes: [og.Vehicle, Drive] }, (_q, _t, world) => {
  const input = must(world.resource(og.Input));
  const car = find(world, "car");
  const steer = (input.down("KeyD") ? 1 : 0) - (input.down("KeyA") ? 1 : 0);
  must(world.set(car, og.Vehicle, { throttle: input.down("KeyW") ? 1 : 0, brake: input.down("KeyS") ? 1 : 0, handbrake: input.down("Space") ? 1 : 0, steer }));
  const v = must(world.get(car, og.Vehicle));
  const d = must(world.resource(Drive));
  d.kmh = Math.abs(v.speed) * 3.6;
  d.slip = v.slip;
  og.assert.always(Number.isFinite(d.kmh) && d.kmh < 250, "the car's speed stays finite and plausible", { kmh: d.kmh });
  og.assert.sometimes(d.kmh > 20, "the car drives over 20 km/h");
});

/** The car never sinks under the hills (their lowest point is 0). */
const Ground = defineSystem("Ground", { reads: [og.Transform], writes: [Drive], after: ["Driver"] }, (_q, _t, world) => {
  const y = must(world.get(find(world, "car"), og.Transform)).position.y;
  const d = must(world.resource(Drive));
  d.lowest = Math.min(d.lowest, y);
  og.assert.always(y > -0.5, "the car never falls through the terrain", { y });
});

/** Smoke while the rear slides; the engine's pitch follows the speed. */
const Effects = defineSystem("Effects", { writes: [Drive, og.ParticleSystem, og.AudioSource], after: ["Ground"] }, (_q, time, world) => {
  const d = must(world.resource(Drive));
  const car = find(world, "car");
  const sliding = Math.abs(d.slip) > 0.15 && d.kmh > 10;
  for (const name of ["smoke-l", "smoke-r"]) {
    const e = find(world, name);
    const ps = must(world.get(e, og.ParticleSystem));
    const on = ps.startTick > ps.stopTick;
    if (sliding && !on) must(world.set(e, og.ParticleSystem, { startTick: time.tick, stopTick: 0 }));
    if (!sliding && on) must(world.set(e, og.ParticleSystem, { stopTick: time.tick }));
  }
  if (sliding) {
    d.smoking += 1;
    og.assert.reachable("the rear slides");
  }
  const a = must(world.get(car, og.AudioSource));
  must(world.set(car, og.AudioSource, { ...a, pitch: 0.6 + Math.min(1.4, d.kmh / 60) }));
});

/** Skid marks while the rear slides: a ring of og.Decal (a system of its own, since a query matches the entities holding every component it names). */
const Marks = defineSystem("Marks", { writes: [Drive, Mark, og.Decal, og.Transform], after: ["Effects"] }, (q, time, world) => {
  // A mark every sixth tick of a slide, under the car's rear, along its heading.
  const d = must(world.resource(Drive));
  if (!(Math.abs(d.slip) > 0.15 && d.kmh > 10) || time.tick % 6 !== 0) return;
  const car = find(world, "car");
  const t = must(world.get(car, og.Transform));
  const slot = d.marks % MARKS;
  d.marks += 1;
  const back: Vector3 = rotate({ x: 0, y: -0.3, z: -1.3 }, t.quaternion);
  // The decal projects along its local -z: turned to face down, its length along the car.
  const down = mul(t.quaternion, og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2));
  for (const arch of q.archetypes()) {
    const m = arch.column(Mark).slot;
    const p = arch.column(og.Transform).position;
    const r = arch.column(og.Transform).quaternion;
    const decal = arch.column(og.Decal).opacity;
    for (let i = 0; i < arch.count; i++) {
      if (m[i] !== slot) continue;
      p[i * 3] = t.position.x + back.x;
      p[i * 3 + 1] = t.position.y + back.y;
      p[i * 3 + 2] = t.position.z + back.z;
      r.set([down.x, down.y, down.z, down.w], i * 4);
      decal[i] = 0.8;
    }
  }
});

/** The chase camera: 8 m behind and 3.5 m over the car, easing there (frame-rate independent). */
const Chase = defineSystem("Chase", { reads: [og.Transform], writes: [og.Camera], after: ["Marks"] }, (_q, time, world) => {
  const t = must(world.get(find(world, "car"), og.Transform));
  const behind = rotate({ x: 0, y: 3.5, z: -8 }, t.quaternion);
  const goal = { x: t.position.x + behind.x, y: t.position.y + behind.y, z: t.position.z + behind.z };
  const cam = must(world.resource(og.Camera));
  const k = 1 - Math.exp(-4 * time.dt);
  const p = cam.position;
  cam.position = { x: p.x + (goal.x - p.x) * k, y: p.y + (goal.y - p.y) * k, z: p.z + (goal.z - p.z) * k };
  cam.quaternion = og.transform.lookAt(cam.position, { x: t.position.x, y: t.position.y + 1, z: t.position.z });
});

/** The HUD: the speed, bottom right. */
const Hud = defineSystem("Hud", { reads: [Drive] }, (_q, _t, world) => {
  const d = must(world.resource(Drive));
  og.ui.column("Hud", { width: GROW, height: GROW, padding: { left: 24, top: 24, right: 24, bottom: 24 }, align: "end", cross: "end" }, () =>
    og.ui.panel("Speed", { padding: { left: 16, top: 10, right: 16, bottom: 10 } }, () => {
      og.ui.label("Value", `${Math.round(d.kmh)}`, { step: "displaySm", font: "mono", nowrap: true });
      og.ui.label("Unit", "km/h", { step: "meta", color: { token: "inkMuted" } });
    }),
  );
});

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/drive.ogt",
  resources: [Drive],
  systems: [Driver, Ground, Effects, Marks, Chase, Hud],
  start: build,
  facts: (world) => {
    const d = world.resource(Drive);
    if (!d.ok) return {};
    return { kmh: d.value.kmh, slip: d.value.slip, marks: d.value.marks, smoking: d.value.smoking, lowest: d.value.lowest };
  },
});
