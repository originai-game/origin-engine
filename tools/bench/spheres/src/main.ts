// The per-object half of tools/bench/threejs-compare.ts: 20,000 low-poly
// spheres (16 x 12 segments, as the spike and three-spheres.html use), one
// shared geometry and material, each bobbing on its own phase and speed and
// so moved by the Bob system on every tick (one Transform write per sphere
// per tick, as three.js writes one matrix per object per frame). One
// directional light, no shadows, no post, MSAA 4x, ACES.
import { og, defineComponent, defineSystem, f32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

export const COUNT = 20_000;
export const COLUMNS = 200;
export const SPACING = 1.2;

/** A sphere's rest height, phase (rad) and angular speed (rad/s). */
export const Bob = defineComponent("Bob", { base: f32(0), phase: f32(0), speed: f32(1) });

export const Move = defineSystem("Move", { reads: [Bob], writes: [og.Transform] }, (q, time) => {
  const t = time.tick * time.dt;
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const b = a.column(Bob);
    for (let i = 0; i < a.count; i++) p[i * 3 + 1] = b.base[i]! + Math.sin(t * b.speed[i]! + b.phase[i]!) * 0.5;
  }
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function build(world: World): void {
  const camera = must(world.resource(og.Camera));
  const eye = { x: 0, y: 70, z: 95 };
  camera.position = eye;
  camera.quaternion = og.transform.lookAt(eye, { x: 0, y: 0, z: 0 });
  camera.fov = 50;
  camera.far = 400;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = false;
  settings.bloom = false;
  settings.toneMapped = true;
  const sphere = must(og.render.sphereGeometry(world, { radius: 0.5, widthSegments: 16, heightSegments: 12 }));
  const material = must(og.render.standardMaterial(world, { color: { r: 0.85, g: 0.35, b: 0.2 }, roughness: 0.45, metalness: 0.1 }));
  const rows = COUNT / COLUMNS;
  for (let i = 0; i < COUNT; i++) {
    const x = ((i % COLUMNS) - (COLUMNS - 1) / 2) * SPACING;
    const z = (Math.floor(i / COLUMNS) - (rows - 1) / 2) * SPACING;
    const e = must(world.spawn({ position: { x, y: 0, z } }));
    world.insert(e, og.Mesh, { geometry: sphere, material });
    world.insert(e, Bob, { base: 0, phase: (i * 0.37) % (Math.PI * 2), speed: 1 + (i % 7) * 0.25 });
  }
  const sunDir = { x: -0.45, y: -0.8, z: -0.35 };
  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, sunDir) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: false });
}

export default og.game({ seed: 1, systems: [Move], render: { antialias: "msaa4" }, start: build });
