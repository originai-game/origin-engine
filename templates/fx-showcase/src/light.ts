// The depth wave's light stations (S7), far from the others along x,
// built when first shown:
//
//   baked       a Cornell box (scenes/cornell.ogt): the box is a static
//               model with a baked lightmap (`og bake lightmaps`), so the
//               white walls and blocks take red and green from the walls
//               beside them; its LightVolume's probes light a ball rolling
//               through it (red on its left, green on its right). The
//               ceiling spot's direct light and shadows are drawn every
//               frame; the ceiling is lit by bounces alone. `baked:off`
//               draws it unbaked (the sky's ambient on every surface, no
//               bleeding), `baked:on` again.
//   volumetric  a room with a window in fog lit as a medium (og.Fog
//               volumetric): the sun of an og.DayNight cycle throws shafts
//               through the window's panes that sweep across the floor as
//               the hours pass. Inside, a rect light panel under the
//               ceiling (og.RectLight, its highlight the rectangle's shape
//               on the glossy boards), a spot with a lattice cookie and a
//               point lantern with a cube cookie, each lighting the fog too
//               on the full tier.

import { og, defineComponent, defineSystem, f64, u32 } from "@origingame/engine";
import type { Material, Texture, World } from "@origingame/engine";

export const LIGHT_STATIONS = ["baked", "volumetric"] as const;
export type LightStation = (typeof LIGHT_STATIONS)[number];

/** Where each station lives along x. */
export const LIGHT_ORIGIN: Record<LightStation, number> = { baked: 4000, volumetric: 4100 };

/** The stations' state, on the entity "light". */
export const LightStage = defineComponent("LightStage", {
  /** 1 baked, 2 volumetric, 0 neither (the ball rolls only on 1). */
  active: u32(0),
  /** Ticks since it was shown. */
  shown: u32(0),
  /** Whether the baked lighting is drawn (the `baked` command). */
  baked: u32(1),
  /** The rolling ball's entity. */
  ball: f64(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** The ball rolls across the Cornell box, side to side. */
export const LightShow = defineSystem("LightShow", { writes: [LightStage] }, (_q, _time, world) => {
  const e = world.find("light");
  if (e === null) return;
  const s = must(world.get(e, LightStage));
  if (s.active === 0) return;
  world.set(e, LightStage, { shown: s.shown + 1 });
  if (s.active !== 1 || s.ball === 0) return;
  const t = s.shown / 60;
  const ball = s.ball as unknown as Parameters<typeof world.set>[0];
  world.set(ball, og.Transform, { position: { x: LIGHT_ORIGIN.baked + 0.62 * Math.sin(t * 0.7), y: 0.2, z: 0.62 } });
});

let mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;

/** Prepares the stations; each is built when first shown (showLight). */
export function buildLight(world: World, k: { mat: typeof mat }): void {
  mat = k.mat;
  world.insert(must(world.spawn({ name: "light" })), LightStage);
}

function buildBaked(world: World): void {
  const scene = must(og.load.scene(world, "scenes/cornell.ogt"));
  must(world.instantiate(scene));
  const sphere = must(og.render.sphereGeometry(world, { radius: 0.2 }));
  const ball = must(world.spawn({ name: "cornell-ball", position: { x: LIGHT_ORIGIN.baked, y: 0.2, z: 0.62 } }));
  world.insert(ball, og.Mesh, { geometry: sphere, material: mat({ color: { r: 0.85, g: 0.85, b: 0.85 }, roughness: 0.7 }), castShadow: true, receiveShadow: true });
  world.set(world.find("light")!, LightStage, { ball: ball as unknown as number });
}

function buildVolumetric(world: World): void {
  const x0 = LIGHT_ORIGIN.volumetric;
  const room = must(og.load.gltf(world, "models/window-room.glb"));
  must(world.instantiate(room, { name: "window-room", position: { x: x0, y: 0, z: 0 } }));
  const tex = (p: string) => must(og.load.texture(world, p)) as Texture;
  // A rect light panel under the ceiling by the left wall, facing the
  // floor, its fixture an emissive slab: the boards (roughness 0.35) take
  // its highlight in the rectangle's shape, the wall a soft wash.
  const down = og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0, y: -1, z: 0.0001 });
  const panel = must(world.spawn({ name: "rect-panel", position: { x: x0 + 0.6, y: 3.2, z: -3.2 }, quaternion: down }));
  world.insert(panel, og.RectLight, { color: { r: 0.55, g: 0.75, b: 1 }, intensity: 8, width: 1.6, height: 0.5, twoSided: false });
  const slab = must(og.render.boxGeometry(world, { width: 1.6, height: 0.5, depth: 0.04 }));
  const fixture = must(world.spawn({ name: "rect-fixture", position: { x: x0 + 0.6, y: 3.22, z: -3.2 }, quaternion: down }));
  world.insert(fixture, og.Mesh, { geometry: slab, material: mat({ color: { r: 0.1, g: 0.1, b: 0.1 }, emissive: { r: 0.55, g: 0.75, b: 1 }, emissiveIntensity: 4 }), castShadow: false });
  // A spot under the ceiling with a lattice cookie, aimed at the floor.
  const spot = must(world.spawn({ name: "lattice-spot", position: { x: x0 - 1.5, y: 3.8, z: 1.8 }, quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.2, y: -1, z: -0.1 }) }));
  world.insert(spot, og.SpotLight, { color: { r: 1, g: 0.85, b: 0.6 }, intensity: 30, distance: 9, angle: 0.5, penumbra: 0.15, castShadow: true, cookie: tex("textures/lattice.png") });
  // A lantern with a cube cookie, hanging in the room's far corner.
  const lantern = must(world.spawn({ name: "lantern", position: { x: x0 - 2.6, y: 2.4, z: -2.4 } }));
  world.insert(lantern, og.PointLight, { color: { r: 1, g: 0.75, b: 0.45 }, intensity: 4, distance: 6, castShadow: false, cookie: tex("textures/lantern.png") });
}

export function resetLight(world: World): void {
  const e = world.find("light");
  if (e !== null) world.set(e, LightStage, { active: 0 });
  const fog = world.resource(og.Fog);
  if (fog.ok) fog.value.volumetric = false;
  const d = world.resource(og.DayNight);
  if (d.ok) d.value.enabled = false;
  const settings = world.resource(og.RenderSettings);
  if (settings.ok) settings.value.bakedLighting = true;
}

/** `baked:on|off`: the baked lighting drawn, or every scene as if unbaked. */
export function setBaked(world: World, on: boolean): void {
  must(world.resource(og.RenderSettings)).bakedLighting = on;
  const e = world.find("light");
  if (e !== null) world.set(e, LightStage, { baked: on ? 1 : 0 });
}

/** Shows a station: builds it the first time, then its camera and light. */
export function showLight(world: World, station: LightStation): void {
  if (world.find(`light-${station}`) === null) {
    if (station === "baked") buildBaked(world);
    else buildVolumetric(world);
    must(world.spawn({ name: `light-${station}` }));
  }
  const e = world.find("light")!;
  world.set(e, LightStage, { active: station === "baked" ? 1 : 2, shown: 0 });
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const fog = must(world.resource(og.Fog));
  const sun = world.find("sun");
  const x = LIGHT_ORIGIN[station];
  const look = (eye: { x: number; y: number; z: number }, at: { x: number; y: number; z: number }) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, at);
  };
  settings.bloom = false;
  settings.bakedLighting = must(world.get(e, LightStage)).baked === 1;
  if (station === "baked") {
    look({ x, y: 1, z: 3.7 }, { x, y: 0.95, z: 0 });
    cam.fov = 40;
    cam.exposure = 0.85;
    settings.sky = false;
    settings.environmentIntensity = 0.35;
    fog.enabled = false;
    // The sun stays out of the box: its light is the ceiling spot's.
    if (sun !== null) world.set(sun, og.DirectionalLight, { intensity: 0 });
    return;
  }
  look({ x: x - 3.4, y: 1.5, z: 3.6 }, { x: x + 2, y: 1.4, z: -1.2 });
  cam.fov = 60;
  cam.exposure = 0.8;
  settings.sky = true;
  settings.environmentIntensity = 0.25;
  // The sun of a day-night cycle, an hour every four seconds: the shafts sweep.
  const d = must(world.resource(og.DayNight));
  Object.assign(d, { enabled: true, start: 8.5, speed: 0.25, tilt: 30, azimuth: 0, lightIntensity: 1.4, tintFog: false, step: 10 });
  Object.assign(fog, {
    enabled: true,
    volumetric: true,
    density: 0.1,
    heightDensity: 0.03,
    height: 0,
    falloff: 0.4,
    color: { r: 0.9, g: 0.9, b: 0.92 },
    scattering: 0.9,
    anisotropy: 0.55,
    volumetricDistance: 30,
    sky: 0,
  });
}
