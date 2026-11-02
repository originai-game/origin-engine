// @bun
// tests/evidence/g2-3/sample/src/main.ts
import { og, defineComponent, defineSystem, u32, SHAPE_BOX, SHAPE_SPHERE, JOINT_BALL } from "/home/user/workspace/repo/packages/engine/src/index.ts";
var must = (r) => {
  if (!r.ok)
    throw new Error(r.reason);
  return r.value;
};
var EYE = { x: 0, y: 7.5, z: 17 };
var LOOK = { x: 0, y: 2, z: 0 };
var Bench = defineComponent("Bench", {
  asleep: u32(0),
  slept: u32(0),
  woke: u32(0),
  broken: u32(0)
});
var crates = [];
var topLink = null;
var awake = null;
var sleeping = null;
var snapped = null;
var Watch = defineSystem("Watch", { reads: [og.RigidBody, og.BodySlept, og.BodyWoke, og.JointBroken], writes: [Bench, og.Mesh] }, (_q, _time, world) => {
  const bench = world.find("bench");
  const b = must(world.get(bench, Bench));
  let asleep = 0;
  for (const e of crates) {
    const now = must(world.get(e, og.RigidBody)).sleeping;
    if (now)
      asleep += 1;
    world.set(e, og.Mesh, { material: now ? sleeping : awake });
  }
  const broken = world.events(og.JointBroken).length;
  if (broken > 0 && topLink !== null)
    world.set(topLink, og.Mesh, { material: snapped });
  world.set(bench, Bench, {
    asleep,
    slept: b.slept + world.events(og.BodySlept).length,
    woke: b.woke + world.events(og.BodyWoke).length,
    broken: b.broken + broken
  });
});
function mesh(world, e, geometry, material, scale) {
  world.set(e, og.Transform, { scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
}
var main_default = og.game({
  tick: 60,
  seed: 1,
  systems: [Watch],
  start(world) {
    must(world.resource(og.Physics)).sleepTicks = 30;
    const cam = must(world.resource(og.Camera));
    cam.position = EYE;
    cam.quaternion = og.transform.lookAt(EYE, LOOK);
    cam.fov = 50;
    cam.background = { r: 0.05, g: 0.06, b: 0.09 };
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.45 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });
    const box = must(og.render.boxGeometry(world, {}));
    const ball = must(og.render.sphereGeometry(world, { radius: 0.5 }));
    const mat = (r, g, b, roughness = 0.7) => must(og.render.standardMaterial(world, { color: { r, g, b }, roughness }));
    awake = mat(0.8, 0.55, 0.25);
    sleeping = mat(0.2, 0.45, 0.95);
    snapped = mat(0.95, 0.15, 0.1);
    const steel = mat(0.6, 0.62, 0.66, 0.35);
    const plank = mat(0.45, 0.33, 0.2);
    const floor = must(world.spawn({ name: "floor", position: { x: 0, y: -0.5, z: 0 } }));
    world.insert(floor, og.Collider, { shape: SHAPE_BOX });
    mesh(world, floor, box, mat(0.3, 0.29, 0.27, 0.95), { x: 40, y: 1, z: 20 });
    const bell = must(world.spawn({ name: "dumbbell", position: { x: -7, y: 4, z: 0 }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 0, z: 1 }, 0.6) }));
    world.insert(bell, og.RigidBody);
    world.insert(bell, og.Velocity, { linear: { x: 2, y: 0, z: 0 }, angular: { x: 1.5, y: 0, z: 4 } });
    for (const [name, x] of [["bell-left", -1.1], ["bell-right", 1.1]]) {
      const end = must(world.spawn({ name, parent: bell, position: { x, y: 0, z: 0 } }));
      world.insert(end, og.Collider, { shape: SHAPE_SPHERE, radius: 0.5 });
      mesh(world, end, ball, steel, { x: 1, y: 1, z: 1 });
    }
    const bar = must(world.spawn({ name: "bell-bar", parent: bell }));
    world.insert(bar, og.Collider, { shape: SHAPE_BOX });
    mesh(world, bar, box, steel, { x: 1.6, y: 0.2, z: 0.2 });
    crates = [];
    for (let layer = 0;layer < 3; layer++) {
      for (let i = 0;i < 3; i++) {
        const along = (i - 1) * 1.05;
        const position = layer % 2 === 0 ? { x: along, y: 0.5 + layer * 1.02, z: 0 } : { x: 0.05 * layer, y: 0.5 + layer * 1.02, z: along };
        const e = must(world.spawn({ name: `crate-${layer * 3 + i}`, position }));
        world.insert(e, og.RigidBody);
        world.insert(e, og.Collider, { shape: SHAPE_BOX });
        mesh(world, e, box, awake, { x: 1, y: 1, z: 1 });
        crates.push(e);
      }
    }
    const HOOK = { x: 7, y: 7, z: 0 };
    let above = null;
    for (let i = 0;i < 3; i++) {
      const link = must(world.spawn({ name: `link-${i}`, position: { x: HOOK.x, y: HOOK.y - 0.5 - i, z: 0 } }));
      world.insert(link, og.RigidBody, { mass: 1 });
      world.insert(link, og.Collider, { shape: SHAPE_BOX });
      mesh(world, link, box, steel, { x: 0.15, y: 0.9, z: 0.15 });
      if (above === null) {
        world.insert(link, og.Joint, { kind: JOINT_BALL, anchor: { x: 0, y: 0.5, z: 0 }, otherAnchor: HOOK, breakForce: 300 });
        topLink = link;
      } else {
        const j = must(world.spawn({ name: `link-joint-${i}` }));
        world.insert(j, og.Joint, { kind: JOINT_BALL, body: link, other: above, anchor: { x: 0, y: 0.5, z: 0 }, otherAnchor: { x: 0, y: -0.5, z: 0 } });
      }
      above = link;
    }
    const platform = must(world.spawn({ name: "platform", position: { x: HOOK.x, y: HOOK.y - 3.6, z: 0 } }));
    world.insert(platform, og.RigidBody, { mass: 5 });
    world.insert(platform, og.Collider, { shape: SHAPE_BOX });
    mesh(world, platform, box, plank, { x: 2, y: 0.2, z: 1.2 });
    const pj = must(world.spawn({ name: "platform-joint" }));
    world.insert(pj, og.Joint, { kind: JOINT_BALL, body: platform, other: above, anchor: { x: 0, y: 0.1, z: 0 }, otherAnchor: { x: 0, y: -0.5, z: 0 } });
    const load = must(world.spawn({ name: "load", position: { x: HOOK.x + 0.6, y: HOOK.y - 1.6, z: 0 } }));
    world.insert(load, og.RigidBody, { mass: 40 });
    world.insert(load, og.Collider, { shape: SHAPE_BOX });
    mesh(world, load, box, plank, { x: 0.8, y: 0.8, z: 0.8 });
    world.insert(must(world.spawn({ name: "bench" })), Bench);
  }
});
export {
  Watch,
  main_default as default
};

//# debugId=E15B2E80B4E226D564756E2164756E21
//# sourceMappingURL=game.native.js.map
