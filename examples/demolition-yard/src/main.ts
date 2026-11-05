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
  // Each material its own fx: what breaks loose (breakFx), what a hit
  // knocks off (hitFx) and the fine debris the GPU draws (debris).
  const fx = (id: string) => must(og.load.emitter(world, `fx/${id}.ogt`));
  const brick = { breakFx: fx("brick-dust"), hitFx: fx("brick-chips"), debris: fx("chips") };
  const concrete = { breakFx: fx("stone-dust"), hitFx: fx("stone-grit"), debris: fx("grit") };
  const wood = { breakFx: fx("wood-dust"), hitFx: fx("splinters") };
  const shards = fx("glass-shards");
  const glass = { breakFx: shards, hitFx: shards };

  // The yard: a concrete slab the chunks land on.
  const box = must(og.render.boxGeometry(world, {}));
  const slab = must(og.render.standardMaterial(world, { color: { r: 0.36, g: 0.35, b: 0.33 }, roughness: 0.95 }));
  const ground = must(world.instantiate(must(og.load.gltf(world, "props/ground.ogt")), { name: "ground", position: { x: 1, y: 0, z: 0 } }));
  world.insert(ground, og.RigidBody, { kind: BODY_STATIC });
  world.insert(ground, og.Collider, { shape: SHAPE_BOX, size: { x: 24, y: 0.5, z: 14 }, offset: { x: 0, y: -0.25, z: 0 }, friction: 0.8 });
  // A stand for the pane.
  const frame = must(world.spawn({ name: "stand", position: { x: PANE.x, y: 0.2, z: 0 } }));
  world.set(frame, og.Transform, { scale: { x: 1.8, y: 0.4, z: 0.3 } });
  world.insert(frame, og.Mesh, { geometry: box, material: slab, castShadow: true });
  world.insert(frame, og.RigidBody, { kind: BODY_STATIC });
  world.insert(frame, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 } });

  // Brick: tough bonds, weak mortar (strength) held by friction where it is
  // pressed (the default friction) and next to nothing pulled apart
  // (tension): a hole at its foot drops what hung over it, steps the piers
  // back course by course and leaves no brick hanging over air; fine pieces
  // are GPU debris.
  prop(world, "props/wall.ogt", "wall", WALL, { toughness: 800, strength: 3000, tension: 0.1, density: 1900, impactThreshold: 400, ...brick });
  prop(world, "props/pillar.ogt", "pillar", PILLAR, { toughness: 2000, strength: 20000, density: 2400, impactThreshold: 150, ...concrete });
  // Wood: a dynamic crate, broken by its own landing.
  const crate = prop(world, "props/crate.ogt", "crate", CRATE, { toughness: 120, density: 600, impactThreshold: 40, impactRadius: 0.9, anchored: false, ...wood });
  world.insert(crate, og.RigidBody, { kind: BODY_DYNAMIC, gravityScale: 0 });
  world.insert(crate, og.Collider, { shape: SHAPE_BOX, size: { x: 0.8, y: 0.8, z: 0.8 }, density: 600 });
  // A slim steel frame round the pane (posts and a head rail), standing
  // when the glass goes: it says "a pane" where clear glass alone is hard
  // to see at a glance.
  const steel = must(og.render.standardMaterial(world, { color: { r: 0.06, g: 0.065, b: 0.07 }, metalness: 0.85, roughness: 0.38 }));
  const bar = (name: string, x: number, y: number, sx: number, sy: number) => {
    const e = must(world.spawn({ name, position: { x, y, z: PANE.z } }));
    world.set(e, og.Transform, { scale: { x: sx, y: sy, z: 0.06 } });
    world.insert(e, og.Mesh, { geometry: box, material: steel, castShadow: true });
    world.insert(e, og.RigidBody, { kind: BODY_STATIC });
    world.insert(e, og.Collider, { shape: SHAPE_BOX, size: { x: 1, y: 1, z: 1 } });
  };
  bar("post-left", PANE.x - 0.83, PANE.y + 0.62, 0.05, 1.24);
  bar("post-right", PANE.x + 0.83, PANE.y + 0.62, 0.05, 1.24);
  bar("rail", PANE.x, PANE.y + 1.225, 1.71, 0.05);
  // Glass: weak, light, shattering at a touch.
  prop(world, "props/pane.ogt", "pane", PANE, { toughness: 25, strength: 0, density: 2500, impactThreshold: 2, impactRadius: 0.8, ...glass });

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
      must(og.destruction.explode(world, { point: { x: WALL.x, y: 0.3, z: 0.15 }, radius: 1.8, innerRadius: 1.0, damage: 6000, impulse: 300 }));
    },
    strike: (world) => {
      must(og.destruction.damage(world, find(world, "pillar"), { point: { x: PILLAR.x, y: 1.4, z: PILLAR.z + 0.25 }, radius: 0.5, innerRadius: 0.25, amount: 8000, impulse: { x: 0, y: 0, z: -600 } }));
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
