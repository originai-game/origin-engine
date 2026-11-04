// Destructible props (og.destruction): a brick wall, a concrete pillar, a
// wooden crate and a glass pane, each a `type model` with a `fracture` block
// (assets/props/), broken from the command line so a run can frame each
// moment:
//
//   og run examples/demolition-yard --commands "1:boom;1.6:screenshot:wall.png;2:strike;3:drop;4:throw;6:screenshot:yard.png;6.05:quit"
//
// boom: an explosion at the wall's foot; strike: a blow at the pillar's
// middle; drop: the crate falls from 4 m and splinters where it lands;
// throw: a heavy ball through the pane.
import { og, BODY_DYNAMIC, BODY_STATIC, SHAPE_BOX, SHAPE_SPHERE } from "@origingame/engine";
import type { Entity, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const WALL = { x: -3, y: 0, z: 0 };
const PILLAR = { x: 1.2, y: 0, z: -0.4 };
const CRATE = { x: 3.2, y: 4, z: 0.6 };
const PANE = { x: 5.4, y: 0.4, z: 0 };

function prop(world: World, model: string, name: string, at: { x: number; y: number; z: number }, d: Record<string, number | boolean>): Entity {
  const e = must(world.instantiate(must(og.load.gltf(world, model)), { name, position: at }));
  must(world.insert(e, og.Destructible, d));
  return e;
}

function build(world: World): void {
  const dust = must(og.load.emitter(world, "fx/dust.ogt"));
  const sparks = must(og.load.emitter(world, "fx/sparks.ogt"));
  const chips = must(og.load.emitter(world, "fx/chips.ogt"));

  // The yard: a concrete slab the chunks land on.
  const box = must(og.render.boxGeometry(world, {}));
  const slab = must(og.render.standardMaterial(world, { color: { r: 0.36, g: 0.35, b: 0.33 }, roughness: 0.95 }));
  const ground = must(world.spawn({ name: "ground", position: { x: 1, y: -0.25, z: 0 } }));
  world.set(ground, og.Transform, { scale: { x: 24, y: 0.5, z: 14 } });
  world.insert(ground, og.Mesh, { geometry: box, material: slab, receiveShadow: true });
  world.insert(ground, og.RigidBody, { kind: BODY_STATIC });
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 }, friction: 0.8 });
  // A stand for the pane.
  const frame = must(world.spawn({ name: "stand", position: { x: PANE.x, y: 0.2, z: 0 } }));
  world.set(frame, og.Transform, { scale: { x: 1.8, y: 0.4, z: 0.3 } });
  world.insert(frame, og.Mesh, { geometry: box, material: slab, castShadow: true });
  world.insert(frame, og.RigidBody, { kind: BODY_STATIC });
  world.insert(frame, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 } });

  // Brick: tough bonds, a stress limit low enough that a hole at its foot
  // brings down what it held; fine pieces are GPU debris.
  prop(world, "props/wall.ogt", "wall", WALL, { toughness: 800, strength: 6000, density: 1900, dust, sparks, debris: chips });
  prop(world, "props/pillar.ogt", "pillar", PILLAR, { toughness: 2000, strength: 20000, density: 2400, dust, sparks, debris: chips });
  // Wood: a dynamic crate, broken by its own landing.
  const crate = prop(world, "props/crate.ogt", "crate", CRATE, { toughness: 120, density: 600, impactThreshold: 40, impactRadius: 0.9, anchored: false, dust });
  world.insert(crate, og.RigidBody, { kind: BODY_DYNAMIC, gravityScale: 0 });
  world.insert(crate, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 0.8, z: 0.8 }, density: 600 });
  // Glass: weak, light, shattering at a touch.
  prop(world, "props/pane.ogt", "pane", PANE, { toughness: 50, strength: 0, density: 2500, impactThreshold: 2, impactRadius: 0.5, sparks });

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.4, y: -0.75, z: -0.5 }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3.2, castShadow: true });
  const cam = must(world.resource(og.Camera));
  const eye = { x: 1, y: 3.2, z: 9.5 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: 1, y: 1, z: 0 });
  cam.fov = 55;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
}

function find(world: World, name: string): Entity {
  const e = world.find(name);
  if (e === null) throw new Error(`no ${name}`);
  return e;
}

export default og.game({
  tick: 60,
  seed: 1,
  start: build,
  commands: {
    boom: (world) => {
      must(og.destruction.explode(world, { point: { x: WALL.x, y: 0.4, z: 0.4 }, radius: 1.6, innerRadius: 0.5, damage: 6000, impulse: 400 }));
    },
    strike: (world) => {
      must(og.destruction.damage(world, find(world, "pillar"), { point: { x: PILLAR.x, y: 1.4, z: PILLAR.z + 0.25 }, radius: 0.5, innerRadius: 0.25, amount: 8000 }));
    },
    drop: (world) => {
      must(world.set(find(world, "crate"), og.RigidBody, { gravityScale: 1 }));
    },
    throw: (world) => {
      const ball = must(world.spawn({ name: "ball", position: { x: PANE.x, y: 1.2, z: 3 } }));
      must(world.insert(ball, og.RigidBody, { kind: BODY_DYNAMIC, mass: 6, ccd: 1 }));
      must(world.insert(ball, og.Collider, { shape: SHAPE_SPHERE, radius: 0.15 }));
      must(world.insert(ball, og.Velocity, { linear: { x: 0, y: 1, z: -14 } }));
      const sphere = must(og.render.sphereGeometry(world, { radius: 0.15 }));
      const iron = must(og.render.standardMaterial(world, { color: { r: 0.2, g: 0.2, b: 0.22 }, metalness: 1, roughness: 0.4 }));
      must(world.insert(ball, og.Mesh, { geometry: sphere, material: iron, castShadow: true }));
    },
  },
});
