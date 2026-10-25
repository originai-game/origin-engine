// The complex scene of tools/bench/threejs-compare.ts, the scene that judges
// docs/design.md §7's 1.5x budget: a product-scale level built the way a
// shipped game builds one. src/layout.js places everything (and
// tools/bench/three-complex.js places the same things from it):
//
// - the ground: one 320 m model, 131,072 triangles in four textured,
//   normal-mapped materials, with a lake (a blended water plane);
// - 5,261 props from 19 models in 60 materials, 3.0 million triangles (trees, rocks, bushes,
//   houses with blended windows, market stalls, crates, barrels, fences,
//   lamp posts with emissive glass, palms, islets, cars, the old stairs,
//   three dropships with blended cockpits), drawn as the engine draws any
//   model: instanced by (geometry, material); 24 translucent ghost carts
//   move round the ring road every tick;
// - 48 skinned characters on two models and 14 clips (24 walkers, 24
//   villagers on kestrel-stair's clip library), each on its own clip,
//   speed and phase;
// - the sun with three 2048 cascades over 120 m, two shadowed spot lights
//   and two shadowed point lights (cube shadows on both tiers);
// - 20 particle emitters (4 fires, 16 smokes), the sky and its IBL;
// - post: AO (GTAO on WebGPU, SSAO on WebGL2: the tier's), bloom, ACES,
//   and the tier's antialiasing (TAA on WebGPU, MSAA 4x on WebGL2).
//
// Every model is drawn at its full mesh (RenderSettings.lodBias 0), since
// three.js has no levels of detail. Nothing reads the wall clock: the
// camera, the ghosts and the clips advance with the tick.

import { og, defineComponent, defineSystem, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { LAKE, MODELS, SHADOW_DISTANCE, SUN, cameraAt, characters, emitters, ghostPose, lights, placements } from "./layout.js";

/** A ghost cart's index on the ring road. */
export const Ghost = defineComponent("Ghost", { index: u32(0) });

const yaw = (a: number) => ({ x: 0, y: Math.sin(a / 2), z: 0, w: Math.cos(a / 2) });

export const Drive = defineSystem("Drive", { reads: [Ghost], writes: [og.Transform] }, (q, time) => {
  const t = time.tick * time.dt;
  for (const a of q.archetypes()) {
    const p = a.column(og.Transform).position;
    const r = a.column(og.Transform).quaternion;
    const g = a.column(Ghost).index;
    for (let i = 0; i < a.count; i++) {
      const pose = ghostPose(g[i]!, t);
      p[i * 3] = pose.x;
      p[i * 3 + 1] = pose.y;
      p[i * 3 + 2] = pose.z;
      r[i * 4 + 1] = Math.sin(pose.yaw / 2);
      r[i * 4 + 3] = Math.cos(pose.yaw / 2);
    }
  }
});

export const View = defineSystem("View", { writes: [og.Camera] }, (_q, time, world) => {
  const cam = must(world.resource(og.Camera));
  const v = cameraAt(time.tick * time.dt);
  cam.position = v.eye;
  cam.quaternion = og.transform.lookAt(v.eye, v.at);
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export function build(world: World): void {
  const camera = must(world.resource(og.Camera));
  const v = cameraAt(0);
  camera.position = v.eye;
  camera.quaternion = og.transform.lookAt(v.eye, v.at);
  camera.fov = v.fov;
  camera.near = v.near;
  camera.far = v.far;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  settings.bloom = true;
  settings.toneMapped = true;
  settings.toneMapping = "aces";
  settings.lodBias = 0;
  const post = must(world.resource(og.PostEffects));
  post.ao = true;

  must(world.instantiate(must(og.load.gltf(world, "models/ground.glb", { castShadow: false, receiveShadow: true })), { name: "ground" }));
  const plane = must(og.render.planeGeometry(world, { width: LAKE.r * 2.4, height: LAKE.r * 2.4 }));
  const waterMat = must(og.render.standardMaterial(world, { color: { r: 0.12, g: 0.3, b: 0.38 }, roughness: 0.08, metalness: 0, opacity: 0.6, transparent: true }));
  const water = must(world.spawn({ name: "water", position: { x: LAKE.x, y: 0, z: LAKE.z }, quaternion: { x: -Math.SQRT1_2, y: 0, z: 0, w: Math.SQRT1_2 } }));
  world.insert(water, og.Mesh, { geometry: plane, material: waterMat, castShadow: false, receiveShadow: true });

  const where = placements();
  for (const m of MODELS) {
    const asset = must(og.load.gltf(world, `models/${m.file}`, { castShadow: true, receiveShadow: true }));
    where[m.file]!.forEach((p, i) => {
      const e = must(world.instantiate(asset, { name: `${m.file.replace(".glb", "")}-${i}`, position: { x: p.x, y: p.y, z: p.z }, quaternion: yaw(p.yaw), scale: { x: p.scale, y: p.scale, z: p.scale } }));
      if (m.moving) world.insert(e, Ghost, { index: i });
    });
  }

  const loadRig = (file: string) => ({ model: must(og.load.gltf(world, `models/${file}`)), anim: must(og.load.animation(world, `models/${file}`)) });
  const rigs = { "walker.glb": loadRig("walker.glb"), "villager.glb": loadRig("villager.glb") };
  characters().forEach((c, i) => {
    const rig = rigs[c.model as keyof typeof rigs];
    const clip = rig.anim.clips.find((k) => k.name === c.clip);
    if (!clip) throw new Error(`${c.model} has no clip ${c.clip}`);
    const e = must(world.instantiate(rig.model, { name: `character-${i}-${c.clip}`, position: { x: c.x, y: c.y, z: c.z }, quaternion: yaw(c.yaw) }));
    world.insert(e, og.Skeleton, { asset: rig.anim.skeleton, rootMotion: "off" });
    world.insert(e, og.AnimationPlayer, { clip: clip.clip, speed: c.speed, time: c.time, loop: true, fade: 0 });
  });

  const fx = { fire: must(og.load.emitter(world, "fx/fire.ogt")), smoke: must(og.load.emitter(world, "fx/smoke.ogt")) };
  emitters().forEach((em, i) => {
    const e = must(world.spawn({ name: `fx-${em.fx}-${i}`, position: { x: em.x, y: em.y, z: em.z } }));
    world.insert(e, og.ParticleSystem, { emitter: fx[em.fx as keyof typeof fx], seed: em.seed });
  });

  const n = Math.hypot(SUN.x, SUN.y, SUN.z);
  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: SUN.x / n, y: SUN.y / n, z: SUN.z / n }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 2.6, castShadow: true, shadowDistance: SHADOW_DISTANCE });
  lights().forEach((l, i) => {
    const color = { r: l.color[0]!, g: l.color[1]!, b: l.color[2]! };
    const at = { x: l.x, y: l.y, z: l.z };
    if (l.kind === "spot") {
      const e = must(world.spawn({ name: `spot-${i}`, position: at, quaternion: og.transform.lookAt(at, { x: l.tx!, y: l.ty!, z: l.tz! }) }));
      world.insert(e, og.SpotLight, { color, intensity: l.intensity, distance: l.distance, angle: l.angle!, penumbra: l.penumbra!, castShadow: true });
    } else {
      const e = must(world.spawn({ name: `point-${i}`, position: at }));
      world.insert(e, og.PointLight, { color, intensity: l.intensity, distance: l.distance, castShadow: true });
    }
  });
}

export default og.game({
  seed: 1,
  systems: [Drive, View],
  render: { antialias: "tier", pointShadows: "cube", shadowMapSize: 2048 },
  start: build,
});
