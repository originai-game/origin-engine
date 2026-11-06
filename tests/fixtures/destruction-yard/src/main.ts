// The destruction yard: og.destruction on three props on a static slab,
// broken on fixed beats (README.md). A brick wall blown at its foot keeps two
// piers, a concrete pillar struck at its middle falls onto its stump, and a
// glass pane hit by an iron ball is all shards.
//
//   og run tests/fixtures/destruction-yard --commands "1:boom;1.6:strike;2:throw;4:quit"
//
// The beats are exported (BEATS) so that tests/support/yard-piers.ts plays
// the same ones on the same ticks as `og run` and the playtest.
import { og, defineResource, defineSystem, u32, BODY_DYNAMIC, BODY_STATIC, MATERIAL_BRICK, MATERIAL_CONCRETE, MATERIAL_GLASS, SHAPE_BOX, SHAPE_SPHERE } from "@origingame/engine";
import type { Destructible, Entity, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export const WALL = { x: -3, y: 0, z: 0 };
export const PILLAR = { x: 1.2, y: 0, z: -0.4 };
export const PANE = { x: 5.4, y: 0.4, z: 0 };
/** The slab: its centre's x, its width and depth, its top at y = 0. */
const SLAB = { x: 1, w: 24, d: 14 };
/** Each beat's command and the tick it runs on (60 Hz). */
export const BEATS: ReadonlyArray<readonly [number, string]> = [
  [60, "boom"],
  [96, "strike"],
  [120, "throw"],
];

/** What the run has seen: the og.Fractured events and the chunks they report. */
const Yard = defineResource("Yard", { fractured: u32(), chunks: u32() });

function find(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`no ${name}`);
  return e;
}

function build(world: World): void {
  const dust = must(og.load.emitter(world, "fx/dust.ogt"));
  const material = (kind: number) => must(og.destruction.material(world, kind));
  const prop = (model: string, name: string, at: { x: number; y: number; z: number }, d: Partial<Destructible>) => {
    const e = must(world.instantiate(must(og.load.gltf(world, model)), { name, position: at }));
    must(world.insert(e, og.Destructible, d));
  };
  const box = must(og.render.boxGeometry(world, {}));
  const solid = (name: string, at: { x: number; y: number; z: number }, size: { x: number; y: number; z: number }, color: { r: number; g: number; b: number }) => {
    const e = must(world.spawn({ name, position: at }));
    must(world.set(e, og.Transform, { scale: size }));
    must(world.insert(e, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color, roughness: 0.9 })), castShadow: true }));
    must(world.insert(e, og.RigidBody, { kind: BODY_STATIC }));
    must(world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 }, friction: 0.8 }));
  };
  // The slab the chunks land on, its top at y = 0, and the pane's plinth.
  solid("ground", { x: SLAB.x, y: -0.25, z: 0 }, { x: SLAB.w, y: 0.5, z: SLAB.d }, { r: 0.2, g: 0.19, b: 0.18 });
  solid("plinth", { x: PANE.x, y: 0.2, z: 0 }, { x: 1.8, y: 0.4, z: 0.3 }, { r: 0.3, g: 0.3, b: 0.32 });
  prop("props/wall.ogt", "wall", WALL, { ...material(MATERIAL_BRICK), breakFx: dust });
  prop("props/pillar.ogt", "pillar", PILLAR, { ...material(MATERIAL_CONCRETE), breakFx: dust });
  prop("props/pane.ogt", "pane", PANE, material(MATERIAL_GLASS));

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.4, y: -0.75, z: -0.5 }) }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 2.4, castShadow: true }));
  const cam = must(world.resource(og.Camera));
  cam.position = { x: 1, y: 2.6, z: 8 };
  cam.quaternion = og.transform.lookAt(cam.position, { x: 1, y: 1, z: 0 });
  cam.fov = 55;
  must(world.resource(og.RenderSettings)).sky = true;
}

/** The properties every tick must keep, and the moments the run must reach (og.assert). */
const Watch = defineSystem("Watch", { reads: [og.Fractured, og.Chunk, og.MatrixWorld], writes: [Yard] }, (q, _t, world) => {
  const yard = must(world.resource(Yard));
  for (const f of world.events(og.Fractured)) {
    yard.fractured += 1;
    yard.chunks += f.chunks;
    if (f.entity === world.find("wall")) og.assert.reachable("the wall breaks");
    if (f.entity === world.find("pillar")) og.assert.reachable("the pillar breaks");
    if (f.entity === world.find("pane")) og.assert.reachable("the pane breaks");
  }
  // Nothing falls through the slab: no chunk over its 24 x 14 m footprint
  // ends a tick under its top (a shard thrown past its edge may fall; each
  // chunk is read where it is in the world, og.MatrixWorld's translation).
  let lowest = Infinity;
  for (const a of q.archetypes()) {
    const m = a.column(og.MatrixWorld).matrix;
    for (let i = 0; i < a.count; i++) {
      const [x, y, z] = [m[i * 16 + 12]!, m[i * 16 + 13]!, m[i * 16 + 14]!];
      if (Math.abs(x - SLAB.x) < SLAB.w / 2 && Math.abs(z) < SLAB.d / 2) lowest = Math.min(lowest, y);
    }
  }
  og.assert.always(lowest > -0.1, "no chunk falls through the slab", { lowest });
});

function ball(world: World): void {
  const e = must(world.spawn({ name: "ball", position: { x: PANE.x, y: 1.2, z: 3 } }));
  must(world.insert(e, og.RigidBody, { kind: BODY_DYNAMIC, mass: 6, ccd: 1 }));
  must(world.insert(e, og.Collider, { shape: SHAPE_SPHERE, radius: 0.15 }));
  must(world.insert(e, og.Velocity, { linear: { x: 0, y: 1, z: -14 } }));
  const sphere = must(og.render.sphereGeometry(world, { radius: 0.15 }));
  const iron = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.2, b: 0.22 }, metalness: 1, roughness: 0.4 }));
  must(world.insert(e, og.Mesh, { geometry: sphere, material: iron, castShadow: true }));
}

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Yard],
  systems: [Watch],
  start: build,
  commands: {
    // An explosion at the wall's foot: the stress solver drops what hung over the hole.
    boom: (world) => must(og.destruction.explode(world, { point: { x: WALL.x, y: 0.3, z: 0.15 }, radius: 1.5, innerRadius: 0.8, damage: 6000, impulse: 2500 })),
    // A blow at the pillar's middle, pushing away from the camera.
    strike: (world) => must(og.destruction.damage(world, find(world, "pillar"), { point: { x: PILLAR.x, y: 1.4, z: PILLAR.z + 0.25 }, radius: 0.5, innerRadius: 0.25, amount: 8000, impulse: { x: 0, y: 0, z: -600 } })),
    // An iron ball through the pane.
    throw: (world) => ball(world),
  },
  commandHelp: {
    boom: { usage: "", doc: "Blow a hole at the wall's foot." },
    strike: { usage: "", doc: "Strike the pillar at its middle." },
    throw: { usage: "", doc: "Throw an iron ball through the pane." },
  },
  facts: (world) => {
    const yard = world.resource(Yard);
    const info = (name: string) => {
      const e = world.find(name);
      const r = e === null ? null : og.destruction.info(world, e);
      return r && r.ok ? r.value : null;
    };
    const [wall, pillar, pane] = [info("wall"), info("pillar"), info("pane")];
    return {
      fractured: yard.ok ? yard.value.fractured : 0,
      wall: wall ? wall.health : -1,
      pillar: pillar ? pillar.health : -1,
      paneIntact: pane ? pane.intact : -1,
    };
  },
});
