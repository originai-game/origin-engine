// The reference scene of docs/design.md §7, the one the frame-time budget
// names: 500 instanced objects (two meshes in four PBR materials, every one
// casting and receiving shadows, each turning at its own rate), two shadowed
// lights (the sun's cascades and a spot light), 20 skinned characters (the
// CC0 walker of templates/game, 12 joints, each on its own clip, speed and
// phase), and post (bloom, ambient occlusion, ACES, TAA). Nothing depends on
// the wall clock, so a run of N ticks hashes the same on every host.
//
// packages/engine/bench/lanes.ts drives it natively (lanes off and on) and in
// Chrome over WebGPU (isolated with lanes off and on, not isolated);
// `og run crates/og-tests/reference-scene` shows it.

import { og, defineComponent, defineSystem, f32 } from "@origingame/engine";
import type { World } from "@origingame/engine";

/** Turns an entity about +Y at `speed` rad/s. */
export const Spin = defineComponent("Spin", { speed: f32(0) });

export const Turn = defineSystem("Turn", { reads: [Spin], writes: [og.Transform] }, (q, time) => {
  for (const a of q.archetypes()) {
    const r = a.column(og.Transform).quaternion;
    const s = a.column(Spin).speed;
    for (let i = 0; i < a.count; i++) {
      const half = (s[i]! * time.dt) / 2;
      const sy = half - (half * half * half) / 6;
      const cw = 1 - (half * half) / 2;
      const x = r[i * 4]!, y = r[i * 4 + 1]!, z = r[i * 4 + 2]!, w = r[i * 4 + 3]!;
      const nx = x * cw - z * sy, ny = y * cw + w * sy, nz = z * cw + x * sy, nw = w * cw - y * sy;
      const n = Math.sqrt(nx * nx + ny * ny + nz * nz + nw * nw);
      r[i * 4] = nx / n;
      r[i * 4 + 1] = ny / n;
      r[i * 4 + 2] = nz / n;
      r[i * 4 + 3] = nw / n;
    }
  }
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The objects' grid: 25 columns by 20 rows, 1.6 m apart. */
export const GRID = { columns: 25, rows: 20, spacing: 1.6 } as const;
export const CHARACTERS = 20;

export function build(world: World): void {
  const camera = must(world.resource(og.Camera));
  const eye = { x: 0, y: 15, z: 27 };
  camera.position = eye;
  camera.quaternion = og.transform.lookAt(eye, { x: 0, y: 0, z: -1 });
  camera.fov = 50;
  camera.far = 200;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.toneMapped = true;
  const post = must(world.resource(og.PostEffects));
  post.ao = true;

  const plane = must(og.render.planeGeometry(world, { width: 80, height: 80 }));
  const floorMat = must(og.render.standardMaterial(world, { color: { r: 0.32, g: 0.33, b: 0.3 }, roughness: 0.92 }));
  const floor = must(world.spawn({ name: "floor", quaternion: { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 } }));
  world.insert(floor, og.Mesh, { geometry: plane, material: floorMat, receiveShadow: true });

  const box = must(og.render.boxGeometry(world, { width: 0.8, height: 0.8, depth: 0.8 }));
  const sphere = must(og.render.sphereGeometry(world, { radius: 0.45 }));
  const materials = [
    must(og.render.standardMaterial(world, { color: { r: 1, g: 0.77, b: 0.34 }, metalness: 1, roughness: 0.3 })),
    must(og.render.standardMaterial(world, { color: { r: 0.8, g: 0.12, b: 0.1 }, metalness: 0, roughness: 0.45 })),
    must(og.render.standardMaterial(world, { color: { r: 0.55, g: 0.56, b: 0.58 }, metalness: 0, roughness: 0.85 })),
    must(og.render.standardMaterial(world, { color: { r: 0.95, g: 0.95, b: 0.97 }, metalness: 1, roughness: 0.08 })),
  ];
  const { columns, rows, spacing } = GRID;
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < columns; c++) {
      const i = r * columns + c;
      const position = { x: (c - (columns - 1) / 2) * spacing, y: 0.45, z: -(r * spacing) + 6 };
      const e = must(world.spawn({ name: `object-${i}`, position }));
      world.insert(e, og.Mesh, { geometry: i % 2 === 0 ? box : sphere, material: materials[i % 4]!, castShadow: true, receiveShadow: true });
      world.insert(e, Spin, { speed: 0.4 + (i % 7) * 0.15 });
    }
  }

  // The characters: a ring in front of the grid, each on its own clip and phase.
  const model = must(og.load.gltf(world, "models/walker.glb"));
  const anim = must(og.load.animation(world, "models/walker.glb"));
  const clips = ["walk", "run", "idle", "wave"].map((n) => anim.clips.find((c) => c.name === n)!.clip);
  for (let i = 0; i < CHARACTERS; i++) {
    const a = (i / CHARACTERS) * Math.PI * 2;
    const position = { x: Math.cos(a) * 9, y: 0, z: Math.sin(a) * 4 + 10 };
    const e = must(world.instantiate(model, { name: `character-${i}`, position, quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -Math.sin(a), y: 0, z: Math.cos(a) }) }));
    world.insert(e, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
    world.insert(e, og.AnimationPlayer, { clip: clips[i % clips.length]!, speed: 0.8 + (i % 5) * 0.1, time: (i * 0.137) % 1, loop: true, fade: 0 });
  }

  // Two shadowed lights: the sun and a spot over the middle of the grid.
  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.45, y: -0.75, z: -0.5 }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 2.6, castShadow: true, shadowDistance: 60 });
  const spotAt = { x: 2, y: 9, z: -2 };
  const spot = must(world.spawn({ name: "spot", position: spotAt, quaternion: og.transform.lookAt(spotAt, { x: 0, y: 0, z: -6 }) }));
  world.insert(spot, og.SpotLight, { color: { r: 1, g: 0.85, b: 0.6 }, intensity: 120, distance: 30, angle: 0.6, penumbra: 0.3, castShadow: true });
}

export default og.game({
  seed: 1,
  systems: [Turn],
  render: { antialias: "taa" },
  start: build,
});
