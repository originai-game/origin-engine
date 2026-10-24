// Round 3's procedural modelling station (geom-2), far from the others
// along x, built when first shown:
//
//   models  models/clockwork.ogt, a `type model`: og-geom's parts written
//           as text (an extruded gear with a hole and a bevel on an axle
//           pivot, three bolts placed by `at` blocks and drawn as one
//           instanced mesh, a lathed vase, a horn swept along a path,
//           tapering and twisting, a dice cut by CSG, a fin and its
//           mirror, a plinth with uvRepeat), cooked into the model blob a
//           glTF cooks to and loaded with og.load.gltf. og.render.raycast
//           finds the gear's face and passes through its hole, from the
//           BVH the cook built.

import { og, defineComponent, defineSystem, f32, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const MODEL_STATIONS = ["models"] as const;
export type ModelStation = (typeof MODEL_STATIONS)[number];

/** Where the station lives along x (beyond the islands' sea). */
export const MODEL_ORIGIN = 7000;

/** What the playtest reads, on the entity "models". */
export const Models = defineComponent("Models", {
  /** og.load.gltfInfo's geometries: one per distinct mesh (the bolts are one). */
  geometries: u32(0),
  /** A ray along -z at the gear's root ring: hits on the model, and the nearest's z. */
  ringHits: u32(0),
  ringZ: f32(0),
  /** The same ray through the gear's hole: hits on the model. */
  holeHits: u32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The axle pivot's height in the model (models/clockwork.ogt). */
const AXLE = 1.25;

function build(world: World): void {
  const x = MODEL_ORIGIN;
  const floor = must(og.render.planeGeometry(world, { width: 40, height: 40 }));
  const ground = must(og.render.standardMaterial(world, { color: { r: 0.5, g: 0.52, b: 0.55 }, roughness: 0.95 }));
  const f = must(world.spawn({ name: "models-floor", position: { x, y: 0, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2) }));
  world.insert(f, og.Mesh, { geometry: floor, material: ground, receiveShadow: true });
  const model = must(og.load.gltf(world, "models/clockwork.ogt"));
  must(world.instantiate(model, { name: "clockwork", position: { x, y: 0, z: 0 } }));
  const info = must(og.load.gltfInfo(world, model));
  const e = must(world.spawn({ name: "models" }));
  world.insert(e, Models, { geometries: info.geometries.length });
}

/**
 * Each tick, once the station is built: rays along -z at the gear's root
 * ring and through its hole (og.render.raycast reads the meshes as the
 * tick started; the cooked BVH answers it). Nothing to do before.
 */
export const ModelsProbe = defineSystem("ModelsProbe", { writes: [Models] }, (q, _time, world) => {
  const ray = (dy: number) =>
    must(og.render.raycast(world, { origin: { x: MODEL_ORIGIN, y: AXLE + dy, z: 5 }, direction: { x: 0, y: 0, z: -1 } }));
  for (const a of q.archetypes()) {
    if (a.count === 0) continue;
    const ring = ray(0.6);
    const hole = ray(0);
    const c = a.column(Models);
    for (let i = 0; i < a.count; i++) {
      c.ringHits[i] = ring.length;
      c.ringZ[i] = ring.length > 0 ? ring[0]!.point.z : 0;
      c.holeHits[i] = hole.length;
    }
  }
});

/** Turns the station off (every station starts from it). */
export function resetModels(_world: World): void {}

/** Shows the station: its world (built the first time), camera and light. */
export function showModels(world: World, _station: ModelStation): void {
  if (world.find("models") === null) build(world);
  const x = MODEL_ORIGIN;
  const cam = must(world.resource(og.Camera));
  const eye = { x: x + 0.6, y: 2.6, z: 6.2 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x, y: 0.95, z: 0 });
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.environmentIntensity = 0.6;
  settings.bloom = false;
  cam.exposure = 0.8;
  const sun = world.find("sun");
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.75, z: -0.55 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
}
