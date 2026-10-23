// The depth wave's views station (S8), far from the others along x, built
// when first shown:
//
//   views  render targets and secondary cameras. A security camera in the
//          room's corner (og.CameraView into an og.render.renderTarget,
//          drawn every second tick) shown on a monitor on the back wall by
//          an unlit material's map; it sees a magenta crate on layer 1 that
//          the main camera (layer 0) does not draw. A mirror on the left
//          wall (og.Mirror into an hdr target, shown as the glass's
//          emissiveMap) reflecting the walker and the room. A second
//          player's view from above in a viewport rect at the bottom right
//          (split-screen). Leaving the station turns every view off.

import { og, defineComponent, f64, u32 } from "@origingame/engine";
import type { Asset, ClipAsset, Entity, Geometry, Material, SkeletonAsset, Vector3 as Vec3, World } from "@origingame/engine";

export const VW_STATIONS = ["views"] as const;
export type VWStation = (typeof VW_STATIONS)[number];

/** Where the station lives along x. */
export const VW_ORIGIN: Record<VWStation, number> = { views: 3400 };

/** The station's views, on the entity "vw" (to turn them off when leaving). */
export const VW = defineComponent("VW", {
  monitor: f64(0),
  mirror: f64(0),
  split: f64(0),
  /** The monitor's and the mirror's targets (RenderTarget handles). */
  monitorTarget: f64(0),
  mirrorTarget: f64(0),
  /** 1 while shown. */
  active: u32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export interface VWKit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
  walker: Asset;
  rig: { skeleton: SkeletonAsset; clips: ReadonlyArray<{ clip: ClipAsset; name: string }> };
}

let kit: VWKit;

/** Prepares the station; it is built when first shown (showVW). */
export function buildVW(world: World, k: VWKit): void {
  kit = k;
  world.insert(must(world.spawn({ name: "vw" })), VW);
}

/** The room: the floor, three walls, the furniture, the walker, the crate on layer 1. */
function build(world: World): void {
  const k = kit;
  const x0 = VW_ORIGIN.views;
  k.ground(x0, { x: 0.36, y: 0.34, z: 0.32 });
  const at = (p: Vec3): Vec3 => ({ x: x0 + p.x, y: p.y, z: p.z });
  const box = (name: string, m: Material, p: Vec3, size: Vec3, layers = 1, castShadow = true): Entity => {
    const e = must(world.spawn({ name, position: at(p) }));
    world.set(e, og.Transform, { scale: size });
    world.insert(e, og.Mesh, { geometry: k.box, material: m, castShadow, receiveShadow: true, layers });
    return e;
  };
  const wall = k.mat({ color: { r: 0.62, g: 0.6, b: 0.56 }, roughness: 0.9 });
  box("vw-back", wall, { x: 0, y: 2, z: -6.1 }, { x: 12.4, y: 4, z: 0.2 });
  box("vw-left", wall, { x: -6.1, y: 2, z: 0 }, { x: 0.2, y: 4, z: 12.4 });
  box("vw-right", wall, { x: 6.1, y: 2, z: 0 }, { x: 0.2, y: 4, z: 12.4 });
  box("vw-pillar", k.mat({ color: { r: 0.2, g: 0.7, b: 0.3 }, roughness: 0.6 }), { x: -2.4, y: 1.2, z: -2.5 }, { x: 0.7, y: 2.4, z: 0.7 });
  box("vw-crate", k.mat({ color: { r: 0.85, g: 0.25, b: 0.15 }, roughness: 0.6 }), { x: 2.2, y: 0.5, z: -1.2 }, { x: 1, y: 1, z: 1 });
  const ball = must(world.spawn({ name: "vw-ball", position: at({ x: 0.6, y: 0.6, z: -3.4 }) }));
  world.set(ball, og.Transform, { scale: { x: 1.2, y: 1.2, z: 1.2 } });
  world.insert(ball, og.Mesh, { geometry: k.sphere, material: k.mat({ color: { r: 0.2, g: 0.35, b: 0.95 }, roughness: 0.3 }), castShadow: true, receiveShadow: true });
  // On layer 1 only (mask 2): the security camera (layers 3) sees it, the
  // main camera does not. Shadow maps take every caster whatever its
  // layers, so it casts none (it would leave a shadow of nothing).
  box("vw-ghost", k.mat({ color: { r: 0.95, g: 0.1, b: 0.85 }, roughness: 0.5 }), { x: -1.2, y: 0.4, z: 1.4 }, { x: 0.8, y: 0.8, z: 0.8 }, 2, false);

  // The walker, walking on the spot, for the mirror to reflect.
  const walk = k.rig.clips.find((c) => c.name === "walk") ?? k.rig.clips[0]!;
  const walker = must(world.instantiate(k.walker, { name: "vw-walker", position: at({ x: -3.4, y: 0, z: 0.6 }), quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, -1.2) }));
  world.insert(walker, og.Skeleton, { asset: k.rig.skeleton, rootMotion: "off" });
  world.insert(walker, og.AnimationPlayer, { clip: walk.clip, speed: 1, loop: true });

  // The security monitor: a target the corner camera draws into, on the back wall.
  const monitorTarget = must(og.render.renderTarget(world, { width: 480, height: 270 }));
  const screen = must(og.render.planeGeometry(world, { width: 3.2, height: 1.8 }));
  const shown = must(world.spawn({ name: "vw-screen", position: at({ x: 1.6, y: 2.3, z: -5.95 }) }));
  world.insert(shown, og.Mesh, { geometry: screen, material: k.mat({ map: monitorTarget.texture, unlit: true }) });
  box("vw-bezel", k.mat({ color: { r: 0.04, g: 0.04, b: 0.05 }, roughness: 0.5 }), { x: 1.6, y: 2.3, z: -6.0 }, { x: 3.4, y: 2.0, z: 0.04 });
  const eye = at({ x: 5.4, y: 3.4, z: 5.4 });
  const monitor = must(world.spawn({ name: "vw-camera", position: eye, quaternion: og.transform.lookAt(eye, at({ x: -1, y: 0.6, z: -1 })) }));
  world.insert(monitor, og.CameraView, { target: monitorTarget.target, fov: 60, layers: 3, every: 2, exposure: 0.8 });

  // The mirror on the left wall, facing +x: an hdr target as emitted light.
  const mirrorTarget = must(og.render.renderTarget(world, { width: 512, height: 384, format: "hdr" }));
  const glass = must(og.render.planeGeometry(world, { width: 4, height: 2.4 }));
  const mirror = must(world.spawn({ name: "vw-mirror", position: at({ x: -5.98, y: 1.5, z: 0.4 }), quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, Math.PI / 2) }));
  world.insert(mirror, og.Mesh, { geometry: glass, material: k.mat({ color: { r: 0, g: 0, b: 0 }, emissive: { r: 1, g: 1, b: 1 }, emissiveMap: mirrorTarget.texture, emissiveIntensity: 0.92 }) });
  world.insert(mirror, og.Mirror, { target: mirrorTarget.target, width: 4, height: 2.4 });

  // Split-screen: a second player's view from above, bottom right.
  const top = at({ x: 0, y: 12, z: 3 });
  const split = must(world.spawn({ name: "vw-split", position: top, quaternion: og.transform.lookAt(top, at({ x: 0, y: 0, z: -0.8 })) }));
  world.insert(split, og.CameraView, { viewportX: 0.7, viewportY: 0.64, viewportWidth: 0.28, viewportHeight: 0.32, fov: 50, exposure: 0.8, order: 1 });

  world.set(world.find("vw")!, VW, {
    monitor, mirror, split,
    monitorTarget: monitorTarget.target as unknown as number,
    mirrorTarget: mirrorTarget.target as unknown as number,
  });
}

function setActive(world: World, on: boolean): void {
  const e = world.find("vw");
  if (e === null) return;
  const s = must(world.get(e, VW));
  if (s.monitor === 0) return;
  world.set(s.monitor as Entity, og.CameraView, { active: on });
  world.set(s.split as Entity, og.CameraView, { active: on });
  world.set(s.mirror as Entity, og.Mirror, { active: on });
  world.set(e, VW, { active: on ? 1 : 0 });
}

/** Leaves the station: its views stop drawing (the split view would cover every station). */
export function resetVW(world: World): void {
  setActive(world, false);
}

/** Shows the station: builds it the first time, then its camera and light. */
export function showVW(world: World, station: VWStation): void {
  if (world.find(`vw-${station}`) === null) {
    build(world);
    must(world.spawn({ name: `vw-${station}` }));
  }
  setActive(world, true);
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = VW_ORIGIN[station];
  const eye = { x: x + 3.6, y: 2.6, z: 7.4 };
  cam.position = eye;
  cam.quaternion = og.transform.lookAt(eye, { x: x - 1.2, y: 1.4, z: -1.6 });
  cam.exposure = 0.8;
  if (sun !== null) {
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.35, y: -0.85, z: -0.3 }) });
    world.set(sun, og.DirectionalLight, { intensity: 3 });
  }
  settings.sky = true;
  settings.environmentIntensity = 0.5;
  settings.bloom = false;
}
