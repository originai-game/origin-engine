// The effects showcase: every og-render effect a game declares from
// TypeScript, on five stations. `station:<name>` (or `station:<1-5>`) moves
// the camera and switches the station's look; every station keeps the
// screen-space HUD (og.render.sprite/text with `screen: true`, and an og.ui
// badge) over the finished frame.
//
//   post      ambient occlusion, outlines, god rays, bloom, a grading LUT,
//             vignette, chromatic aberration, film grain
//   lens      depth of field and per-object motion blur
//   fx        particle systems (fire, smoke, sparks with collision), decals,
//             a trail, a jittering beam, a ribbon
//   2d        2D lighting (Lighting2D, Light2D, occluders and sprites that
//             cast shadows), sprite flash, glow and additive blend, 2D embers
//   stylised  toon, rim, dissolve, hit flash and outline materials, the
//             skinned walker with a blendshape, screen flash and camera shake
//
// The E5 round adds eight stations (src/e5.ts): sky, weather, water,
// shaders, controller, joints, rope, fluid; phase 2 adds terrain (src/p2.ts),
// the animation and physics round four (src/anim-physics.ts): retarget,
// ragdoll2d, cloth, fluids, and the render round four (src/render.ts):
// lights, moon, materials, skinned, and the 3D fluid station (src/sph.ts):
// fluids3d; P4 the physical materials station (src/physical.ts): physical,
// and the image-based lighting station (src/ibl.ts): environment; the
// depth wave the geometry station (src/geometry.ts): geometry, the views
// station (src/views.ts): views (render targets, a mirror, split-screen), and
// the light stations (src/light.ts): baked, volumetric (`baked:off` draws unbaked);
// round 3 the generated terrain station (src/islands.ts): islands, and
// the procedural modelling station (src/models.ts): models.
//
// Commands: station:<name> (or station:<1-25>). `dump_entity:stage` reads
// the Stage component (the station, the frame counter of the pulses);
// `dump_entity:e5` the E5 component.

import { og, defineComponent, defineSystem, f32, f64, u32, Vector3 } from "@origingame/engine";
import type { Entity, Font, Geometry, Material, Quaternion, Texture, Vector3 as Vec3, World } from "@origingame/engine";
import { E5, E5_STATIONS, E5Show, PlatformMove, buildE5, resetE5, showE5, type E5Station } from "./e5.ts";
import { P2, P2_STATIONS, P2Show, resetP2, showP2, type P2Station } from "./p2.ts";
import { AP_STATIONS, APShow, buildAP, resetAP, showAP, type APStation } from "./anim-physics.ts";
import { RD_STATIONS, buildRD, resetRD, showRD, type RDStation } from "./render.ts";
import { SPH_STATIONS, SPHShow, buildSPH, showSPH, type SPHStation } from "./sph.ts";
import { PM_STATIONS, buildPM, showPM, type PMStation } from "./physical.ts";
import { IBL_STATIONS, buildIBL, resetIBL, showIBL, type IBLStation } from "./ibl.ts";
import { GEO_STATIONS, GEOShow, buildGEO, resetGEO, showGEO, type GEOStation } from "./geometry.ts";
import { DEPTH_STATIONS, buildDepth, resetDepth, showDepth, type DepthStation } from "./depth.ts";
import { VW_STATIONS, buildVW, resetVW, showVW, type VWStation } from "./views.ts";
import { LIGHT_STATIONS, LightShow, buildLight, resetLight, setBaked, showLight, type LightStation } from "./light.ts";
import { GEN_STATIONS, resetGen, showGen, type GenStation } from "./islands.ts";
import { MODEL_STATIONS, ModelsProbe, resetModels, showModels, type ModelStation } from "./models.ts";
import { LOOK_PRESETS, LOOK_STATIONS, applyLookPreset, buildLooks, resetLooks, showLooks, type LookStation } from "./looks.ts";

/** The first five stations, then the E5 round's (src/e5.ts) and phase 2's (src/p2.ts, src/anim-physics.ts, src/render.ts, src/sph.ts). */
export const STATIONS = ["post", "lens", "fx", "2d", "stylised", ...E5_STATIONS, ...P2_STATIONS, ...AP_STATIONS, ...RD_STATIONS, ...SPH_STATIONS, ...PM_STATIONS, ...IBL_STATIONS, ...LOOK_STATIONS, ...GEO_STATIONS, ...DEPTH_STATIONS, ...VW_STATIONS, ...LIGHT_STATIONS, ...GEN_STATIONS, ...MODEL_STATIONS] as const;
export type Station = (typeof STATIONS)[number];

/** The showcase's state, on one entity named "stage". */
export const Stage = defineComponent("Stage", {
  station: u32(0),
  // Asset and material handles exceed 32 bits: f64 keeps them exact.
  font: f64(0),
  crate: f64(0),
  tile: f64(0),
  spark: f64(0),
  dissolve: f64(0),
  flash: f64(0),
  /** Ticks since the station was shown (drives the pulses). */
  shown: u32(0),
  /** The look preset shown on the looks station: 0 none, k + 1 LOOK_PRESETS[k] (the HUD names it). */
  look: u32(0),
});

/** Moves an entity on a circle about `centre` (radius, turns per second, phase in turns). */
export const Orbit = defineComponent("Orbit", {
  centre: Vector3(),
  radius: f32(1),
  speed: f32(0.25),
  phase: f32(0),
  /** The circle's plane: 0 horizontal (xz), 1 vertical (xy). */
  plane: u32(0),
});

const TAU = Math.PI * 2;
/** Where each station lives in the world. */
const ORIGIN = { post: 0, lens: 0, fx: 100, "2d": 1000, stylised: 200 } as const;

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const axisAngle = (axis: Vec3, angle: number): Quaternion => og.transform.fromAxisAngle(axis, angle);
// A function, not a constant: the web Worker binds the engine host after this module loads.
const floor = (): Quaternion => axisAngle({ x: 1, y: 0, z: 0 }, -Math.PI / 2);

// ── systems ────────────────────────────────────────────────────────────────

/** Orbiting entities: the motion-blurred box, the trail's wisp, a 2D light. */
export const Orbiter = defineSystem("Orbiter", { reads: [Orbit], writes: [og.Transform] }, (q, time) => {
  const t = time.tick / 60;
  for (const a of q.archetypes()) {
    const o = a.column(Orbit);
    const p = a.column(og.Transform).position;
    for (let i = 0; i < a.count; i++) {
      const ang = (o.phase[i]! + t * o.speed[i]!) * TAU;
      const r = o.radius[i]!;
      const c = [o.centre[i * 3]!, o.centre[i * 3 + 1]!, o.centre[i * 3 + 2]!];
      if (o.plane[i] === 1) {
        p[i * 3] = c[0]! + Math.cos(ang) * r;
        p[i * 3 + 1] = c[1]! + Math.sin(ang) * r;
        p[i * 3 + 2] = c[2]!;
      } else {
        p[i * 3] = c[0]! + Math.cos(ang) * r;
        p[i * 3 + 1] = c[1]!;
        p[i * 3 + 2] = c[2]! + Math.sin(ang) * r;
      }
    }
  }
});

/** The walker's first blendshape (a blink) breathes with the tick. */
export const Blink = defineSystem("Blink", { writes: [og.Blendshapes] }, (q, time) => {
  for (const a of q.archetypes()) {
    const w = a.column(og.Blendshapes).w0;
    for (let i = 0; i < a.count; i++) w[i] = 0.5 + 0.5 * Math.sin((time.tick / 60) * TAU * 0.8);
  }
});

/**
 * Per tick: the stylised station's pulses (dissolve, hit flash, screen flash,
 * shake), the 2D station's sprites, the fx station's ribbon, and the HUD.
 */
export const Show = defineSystem("Show", { writes: [Stage, og.PostEffects] }, (q, time, world) => {
  let s: { station: number; look: number; font: Font; crate: Texture; tile: Texture; spark: Texture; dissolve: Material; flash: Material } | null = null;
  for (const a of q.archetypes()) {
    if (a.count === 0) continue;
    const c = a.column(Stage);
    c.shown[0] = c.shown[0]! + 1;
    // The handles are kept as f64 in Stage; they are the engine's branded handles.
    const h = <T>(v: number) => v as unknown as T;
    s = { station: c.station[0]!, look: c.look[0]!, font: h<Font>(c.font[0]!), crate: h<Texture>(c.crate[0]!), tile: h<Texture>(c.tile[0]!), spark: h<Texture>(c.spark[0]!), dissolve: h<Material>(c.dissolve[0]!), flash: h<Material>(c.flash[0]!) };
  }
  if (s === null) return;
  const station = STATIONS[s.station] ?? "post";
  const t = time.tick / 60;
  const post = must(world.resource(og.PostEffects));
  if (station === "stylised") {
    // Dissolve ping-pongs over 4 s; the hit flash and the screen flash
    // strike every 2 s and decay; the shake follows the hit.
    const phase = (t % 4) / 4;
    og.render.updateMaterial(world, s.dissolve, { dissolve: phase < 0.5 ? phase * 2 : 2 - phase * 2 });
    const since = t % 2;
    const hit = Math.max(0, 1 - since * 4);
    og.render.updateMaterial(world, s.flash, { flash: hit });
    post.flash = since < 0.12 ? 0.35 * (1 - since / 0.12) : 0;
    post.shakeTrauma = Math.max(0, 0.8 - since * 1.5);
  }
  if (station === "2d") draw2d(world, s, t);
  if (station === "fx") {
    const points = [];
    for (let k = 0; k <= 16; k++) {
      const u = k / 16;
      const hue = (u + t * 0.25) % 1;
      points.push({
        position: { x: 100 - 2.4 + 4.8 * u, y: 2.6 + 0.5 * Math.sin(u * TAU + t * 2), z: -1.2 },
        width: 0.12 + 0.1 * Math.sin(u * Math.PI),
        color: { r: 2 * (0.5 + 0.5 * Math.cos(hue * TAU)), g: 2 * (0.5 + 0.5 * Math.cos((hue - 1 / 3) * TAU)), b: 2 * (0.5 + 0.5 * Math.cos((hue - 2 / 3) * TAU)) },
        opacity: 0.9,
      });
    }
    must(og.render.ribbon(world, { points, texture: s.spark, blend: "additive" }));
  }
  hud(world, s.font, station, s.look);
});

function draw2d(world: World, s: { crate: Texture; tile: Texture; spark: Texture }, t: number): void {
  const x0 = ORIGIN["2d"];
  // The floor: 18 x 10 world units of tiles on layer 0, lit.
  for (let ty = -5; ty < 5; ty++) {
    for (let tx = -9; tx < 9; tx++) {
      og.render.sprite(world, { texture: s.tile, position: { x: x0 + tx + 0.5, y: ty + 0.5, z: 0 }, width: 1, height: 1, layer: 0 });
    }
  }
  // Crates cast shadows from the lights; one flashes white when "hit".
  const crates: Array<[number, number]> = [[-4, 1], [-1.5, -1.5], [2, 1.5], [4.5, -1], [0.5, 3]];
  crates.forEach(([cx, cy], i) => {
    og.render.sprite(world, {
      texture: s.crate,
      position: { x: x0 + cx, y: cy, z: 0 },
      width: 1.1,
      height: 1.1,
      layer: 1,
      castShadow: true,
      flash: i === 2 ? Math.max(0, 1 - (t % 1.5) * 3) : 0,
      flashColor: { r: 1, g: 1, b: 1 },
    });
  });
  // An additive glowing orb above the lit layers.
  og.render.sprite(world, { texture: s.spark, position: { x: x0 - 1, y: 1.5, z: 0 }, width: 1.6, height: 1.6, layer: 3, color: { r: 0.4, g: 0.8, b: 1.6 }, blend: "additive", glow: 1.5 });
}

/**
 * The screen-space HUD: a panel, the title and the station (on the looks
 * station, the preset shown: each frame names its look), and a og.ui badge.
 */
function hud(world: World, font: Font, station: Station, look: number): void {
  const looks = station === "looks";
  og.render.sprite(world, { screen: true, position: { x: 16, y: 16, z: 0 }, width: looks ? 480 : 300, height: 76, anchor: { x: 0, y: 0 }, color: { r: 0.02, g: 0.02, b: 0.03 }, opacity: 0.72, layer: 0 });
  og.render.sprite(world, { screen: true, position: { x: 16, y: 16, z: 0 }, width: 6, height: 76, anchor: { x: 0, y: 0 }, color: { r: 1, g: 0.62, b: 0.08 }, layer: 1 });
  og.render.text(world, { font, text: "Origin Engine effects", position: { x: 34, y: 46, z: 0 }, size: 24, screen: true, layer: 2 });
  const i = STATIONS.indexOf(station);
  const e5 = i - 5;
  const p2 = e5 - E5_STATIONS.length;
  // Phase 2's stations count within their own round: terrain 1/1, the
  // animation and physics round's 1/4 to 4/4; the render round's and the
  // 3D fluid station carry no count.
  const ap = p2 - P2_STATIONS.length;
  const rd = ap - AP_STATIONS.length;
  const pm = rd - RD_STATIONS.length - SPH_STATIONS.length;
  const label = i < 5 ? `station ${i + 1}/5  ${station}` : p2 < 0 ? `E5 station ${e5 + 1}/${E5_STATIONS.length}  ${station}` : ap < 0 ? `P2 station ${p2 + 1}/${P2_STATIONS.length}  ${station}` : rd < 0 ? `P2 station ${ap + 1}/${AP_STATIONS.length}  ${station}` : pm < 0 ? `phase 2 station  ${station}` : (GEO_STATIONS as readonly string[]).includes(station) || (DEPTH_STATIONS as readonly string[]).includes(station) || (VW_STATIONS as readonly string[]).includes(station) || (LIGHT_STATIONS as readonly string[]).includes(station) ? `depth station  ${station}` : (IBL_STATIONS as readonly string[]).includes(station) ? `phase 4 station  ${station}` : (GEN_STATIONS as readonly string[]).includes(station) || (MODEL_STATIONS as readonly string[]).includes(station) ? `round 3 station  ${station}` : `P4 station  ${station}`;
  const line = looks ? `${label}  ${LOOK_PRESETS[look - 1] ?? "none"}` : label;
  og.render.text(world, { font, text: line, position: { x: 34, y: 76, z: 0 }, size: 18, color: { r: 1, g: 0.72, b: 0.3 }, screen: true, layer: 2 });
  og.ui.anchor("Badge", { anchor: "bottomRight" }, () => og.ui.panel("BadgePanel", { padding: { left: 12, top: 8, right: 12, bottom: 8 } }, () => og.ui.label("BadgeText", "og.ui over the frame", {})));
}

// ── stations ───────────────────────────────────────────────────────────────

/** What a station shows: the camera, the light and the post stack. */
function show(world: World, station: Station): void {
  const stage = world.find("stage");
  if (stage !== null) world.set(stage, Stage, { station: STATIONS.indexOf(station), shown: 0, look: 0 });
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const post = must(world.resource(og.PostEffects));
  const l2d = must(world.resource(og.Lighting2D));
  const sun = world.find("sun");
  const lut = must(og.load.lut(world, "luts/teal-orange.cube"));
  // Every station starts from the defaults: everything off.
  Object.assign(post, {
    ao: false, outline: false, depthOfField: false, motionBlur: false, godRays: false, glow: false,
    lut: 0, vignette: false, chromaticAberration: 0, filmGrain: 0, flash: 0, shakeTrauma: 0,
    custom: [],
  });
  l2d.enabled = false;
  resetE5(world);
  resetP2(world);
  resetAP(world);
  resetRD(world);
  resetIBL(world);
  resetLooks(world);
  resetGEO(world);
  resetDepth(world);
  resetVW(world);
  resetLight(world);
  resetGen(world);
  resetModels(world);
  cam.orthographic = false;
  cam.fov = 50;
  settings.toneMapped = true;
  settings.autoExposure = false;
  cam.exposure = 1;
  const look = (eye: Vec3, target: Vec3) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, target);
  };
  const sunAt = (target: Vec3, intensity: number) => {
    if (sun === null) return;
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, target) });
    world.set(sun, og.DirectionalLight, { intensity });
  };
  switch (station) {
    case "post":
      look({ x: 0, y: 2.4, z: 9 }, { x: 0, y: 1.3, z: -2 });
      sunAt({ x: 0.3, y: -0.13, z: 1 }, 3.5);
      cam.exposure = 0.7;
      settings.sky = true;
      settings.environmentIntensity = 0.55;
      settings.bloom = true;
      settings.bloomIntensity = 0.5;
      Object.assign(post, {
        ao: true, aoRadius: 0.7, aoIntensity: 1.2,
        outline: true, outlineWidth: 2,
        godRays: true, godRaysIntensity: 0.3,
        lut, lutIntensity: 0.85,
        vignette: true, vignetteIntensity: 0.45,
        chromaticAberration: 2.5, filmGrain: 0.035,
      });
      break;
    case "lens":
      look({ x: 2.6, y: 1.4, z: 4.2 }, { x: -0.6, y: 0.7, z: 0 });
      sunAt({ x: -0.5, y: -0.7, z: -0.4 }, 3);
      settings.sky = true;
      settings.bloom = false;
      Object.assign(post, {
        depthOfField: true, dofFocusDistance: 4.6, dofFocusRange: 0.6, dofMaxBlur: 0.014, dofQuality: "high",
        motionBlur: true, motionBlurShutter: 2, motionBlurMaxPixels: 90, motionBlurSamples: 24,
      });
      break;
    case "fx":
      look({ x: 100, y: 2.6, z: 6.5 }, { x: 100, y: 1, z: -1 });
      sunAt({ x: -0.3, y: -0.8, z: -0.4 }, 0.15);
      settings.sky = false;
      settings.environmentIntensity = 0.12;
      settings.bloom = true;
      settings.bloomIntensity = 0.7;
      cam.background = { r: 0.004, g: 0.005, b: 0.012 };
      break;
    case "2d":
      cam.orthographic = true;
      cam.height = 10;
      cam.position = { x: ORIGIN["2d"], y: 0, z: 10 };
      cam.quaternion = { x: 0, y: 0, z: 0, w: 1 };
      cam.background = { r: 0, g: 0, b: 0 };
      settings.sky = false;
      settings.toneMapped = false;
      settings.bloom = false;
      l2d.enabled = true;
      l2d.ambient = { r: 0.07, g: 0.07, b: 0.1 };
      l2d.maxLayer = 2;
      post.glow = true;
      post.glowIntensity = 1.2;
      break;
    case "stylised":
      look({ x: 200, y: 2.2, z: 7 }, { x: 200, y: 1, z: 0 });
      sunAt({ x: -0.4, y: -0.8, z: -0.45 }, 3);
      settings.sky = true;
      settings.environmentIntensity = 0.4;
      settings.bloom = true;
      settings.bloomIntensity = 0.6;
      break;
    default:
      if ((P2_STATIONS as readonly string[]).includes(station)) showP2(world, station as P2Station);
      else if ((AP_STATIONS as readonly string[]).includes(station)) showAP(world, station as APStation);
      else if ((RD_STATIONS as readonly string[]).includes(station)) showRD(world, station as RDStation);
      else if ((SPH_STATIONS as readonly string[]).includes(station)) showSPH(world, station as SPHStation);
      else if ((PM_STATIONS as readonly string[]).includes(station)) showPM(world, station as PMStation);
      else if ((IBL_STATIONS as readonly string[]).includes(station)) showIBL(world, station as IBLStation);
      else if ((LOOK_STATIONS as readonly string[]).includes(station)) showLooks(world, station as LookStation);
      else if ((GEO_STATIONS as readonly string[]).includes(station)) showGEO(world, station as GEOStation);
      else if ((DEPTH_STATIONS as readonly string[]).includes(station)) showDepth(world, station as DepthStation);
      else if ((VW_STATIONS as readonly string[]).includes(station)) showVW(world, station as VWStation);
      else if ((LIGHT_STATIONS as readonly string[]).includes(station)) showLight(world, station as LightStation);
      else if ((GEN_STATIONS as readonly string[]).includes(station)) showGen(world, station as GenStation);
      else if ((MODEL_STATIONS as readonly string[]).includes(station)) showModels(world, station as ModelStation);
      else showE5(world, station as E5Station);
  }
}

// ── the world ──────────────────────────────────────────────────────────────

function spawnMesh(world: World, name: string, geometry: Geometry, material: Material, position: Vec3, extra?: { quaternion?: Quaternion; scale?: Vec3 }): Entity {
  const e = must(world.spawn({ name, position, ...(extra?.quaternion ? { quaternion: extra.quaternion } : {}) }));
  if (extra?.scale) world.set(e, og.Transform, { scale: extra.scale });
  world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
  return e;
}

function ground(world: World, x: number, color: Vec3): void {
  const plane = must(og.render.planeGeometry(world, { width: 60, height: 60 }));
  const m = must(og.render.standardMaterial(world, { color: { r: color.x, g: color.y, b: color.z }, roughness: 0.95 }));
  const e = must(world.spawn({ name: `ground-${x}`, position: { x, y: 0, z: 0 }, quaternion: floor() }));
  world.insert(e, og.Mesh, { geometry: plane, material: m, receiveShadow: true });
}

function build(world: World): void {
  const font = must(og.load.font(world, "fonts/NotoSansSC-Regular-subset.ttf"));
  og.ui.configure(world, { font });
  const tex = (p: string) => must(og.load.texture(world, p));
  const box = must(og.render.boxGeometry(world, {}));
  const sphere = must(og.render.sphereGeometry(world, { radius: 0.5 }));
  const mat = (d: Parameters<typeof og.render.standardMaterial>[1]) => must(og.render.standardMaterial(world, d));

  const sun = must(world.spawn({ name: "sun" }));
  world.insert(sun, og.DirectionalLight, { intensity: 3 });

  // Stations post and lens: blocks and spheres on a field, the trickster.
  ground(world, 0, { x: 0.32, y: 0.34, z: 0.36 });
  const stone = mat({ color: { r: 0.7, g: 0.68, b: 0.64 }, roughness: 0.8 });
  const amber = mat({ color: { r: 0.95, g: 0.55, b: 0.1 }, roughness: 0.5 });
  const teal = mat({ color: { r: 0.1, g: 0.55, b: 0.6 }, roughness: 0.35, metalness: 0.2 });
  const glow = mat({ color: { r: 0.1, g: 0.1, b: 0.1 }, emissive: { r: 1, g: 0.5, b: 0.15 }, emissiveIntensity: 6 });
  spawnMesh(world, "block-a", box, stone, { x: -3, y: 0.75, z: -1 }, { scale: { x: 1.5, y: 1.5, z: 1.5 } });
  spawnMesh(world, "block-b", box, amber, { x: -1.6, y: 0.4, z: 0.6 }, { scale: { x: 0.8, y: 0.8, z: 0.8 } });
  spawnMesh(world, "pillar", box, stone, { x: 2.6, y: 1.5, z: -2.5 }, { scale: { x: 0.6, y: 3, z: 0.6 } });
  spawnMesh(world, "ball-a", sphere, teal, { x: 1.2, y: 0.5, z: 0.4 });
  spawnMesh(world, "ball-b", sphere, glow, { x: 3.4, y: 0.35, z: 1.4 }, { scale: { x: 0.7, y: 0.7, z: 0.7 } });
  const trickster = must(og.load.gltf(world, "models/trickster.glb"));
  must(world.instantiate(trickster, { name: "trickster", position: { x: 0, y: 0, z: -3 } }));
  // The motion-blurred box: a fast orbit in front of the camera (lens station).
  const blur = spawnMesh(world, "blur", box, amber, { x: 0, y: 0.6, z: 1 }, { scale: { x: 0.5, y: 0.5, z: 0.5 } });
  world.insert(blur, Orbit, { centre: { x: 0, y: 2, z: 1.2 }, radius: 0.6, speed: 2, plane: 1, phase: 0.1 });

  // Station fx: a campfire with fire, smoke and sparks; decals; a trail; a beam.
  const fx = ORIGIN.fx;
  ground(world, fx, { x: 0.18, y: 0.17, z: 0.16 });
  const wallGeo = must(og.render.planeGeometry(world, { width: 14, height: 6 }));
  const wall = must(world.spawn({ name: "wall", position: { x: fx, y: 3, z: -3 } }));
  world.insert(wall, og.Mesh, { geometry: wallGeo, material: mat({ color: { r: 0.42, g: 0.4, b: 0.38 }, roughness: 0.9 }), castShadow: true, receiveShadow: true });
  const ring = must(og.render.boxGeometry(world, { width: 0.25, height: 0.18, depth: 0.25 }));
  const coal = mat({ color: { r: 0.15, g: 0.1, b: 0.08 }, emissive: { r: 1, g: 0.3, b: 0.05 }, emissiveIntensity: 1.5 });
  for (let k = 0; k < 8; k++) {
    const a = (k / 8) * TAU;
    spawnMesh(world, `stone-${k}`, ring, coal, { x: fx + Math.cos(a) * 0.55, y: 0.09, z: Math.sin(a) * 0.55 }, { quaternion: axisAngle({ x: 0, y: 1, z: 0 }, a) });
  }
  const emitter = (name: string, path: string, at: Vec3, seed: number) => {
    const e = must(world.spawn({ name, position: at }));
    world.insert(e, og.ParticleSystem, { emitter: must(og.load.emitter(world, path)), seed });
  };
  emitter("fire", "fx/fire.ogt", { x: fx, y: 0.1, z: 0 }, 1);
  emitter("smoke", "fx/smoke.ogt", { x: fx, y: 1.4, z: 0 }, 2);
  emitter("sparks", "fx/sparks.ogt", { x: fx, y: 0.3, z: 0 }, 3);
  const firelight = must(world.spawn({ name: "firelight", position: { x: fx, y: 1, z: 0.3 } }));
  world.insert(firelight, og.PointLight, { color: { r: 1, g: 0.55, b: 0.2 }, intensity: 22, distance: 9, castShadow: true });
  must(world.resource(og.RenderSettings)).decalAtlas = tex("textures/decals.png");
  const decal = (name: string, at: Vec3, size: Vec3, q: Quaternion, uv: [number, number, number, number], d: { color?: { r: number; g: number; b: number }; emissive?: { r: number; g: number; b: number } }) => {
    const e = must(world.spawn({ name, position: at, quaternion: q }));
    world.set(e, og.Transform, { scale: size });
    world.insert(e, og.Decal, { uvMin: { x: uv[0], y: uv[1] }, uvMax: { x: uv[2], y: uv[3] }, ...d });
  };
  decal("scorch", { x: fx, y: 0, z: 0 }, { x: 3.2, y: 3.2, z: 1 }, floor(), [0, 0, 0.5, 0.5], {});
  decal("rune", { x: fx - 2.6, y: 2.6, z: -3 }, { x: 2, y: 2, z: 1 }, { x: 0, y: 0, z: 0, w: 1 }, [0.5, 0, 1, 0.5], { color: { r: 0.2, g: 0.9, b: 1 }, emissive: { r: 0.4, g: 1.8, b: 2.4 } });
  decal("arrow", { x: fx + 2.8, y: 2.2, z: -3 }, { x: 1.8, y: 1.8, z: 1 }, { x: 0, y: 0, z: 0, w: 1 }, [0, 0.5, 0.5, 1], {});
  decal("splat", { x: fx + 1.8, y: 0, z: 1.6 }, { x: 1.2, y: 1.2, z: 1 }, floor(), [0.5, 0.5, 1, 1], {});
  const wisp = spawnMesh(world, "wisp", sphere, mat({ color: { r: 0.1, g: 0.1, b: 0.1 }, emissive: { r: 0.4, g: 1, b: 2 }, emissiveIntensity: 4 }), { x: fx + 1.8, y: 1.3, z: 0 }, { scale: { x: 0.18, y: 0.18, z: 0.18 } });
  world.insert(wisp, Orbit, { centre: { x: fx, y: 1.3, z: 0 }, radius: 1.8, speed: 0.35 });
  world.insert(wisp, og.Trail, { lifetime: 1.2, widthStart: 0.22, widthEnd: 0, color: { r: 0.6, g: 1.4, b: 3 }, colorEnd: { r: 0.2, g: 0.4, b: 1.2 }, opacity: 1, opacityEnd: 0, blend: "additive" });
  const pylon = mat({ color: { r: 0.25, g: 0.27, b: 0.3 }, metalness: 0.6, roughness: 0.4 });
  spawnMesh(world, "pylon-l", box, pylon, { x: fx - 4.2, y: 1.2, z: -1.6 }, { scale: { x: 0.3, y: 2.4, z: 0.3 } });
  spawnMesh(world, "pylon-r", box, pylon, { x: fx + 4.2, y: 1.2, z: -1.6 }, { scale: { x: 0.3, y: 2.4, z: 0.3 } });
  const arc = must(world.spawn({ name: "arc", position: { x: fx - 4.2, y: 2.45, z: -1.6 } }));
  world.insert(arc, og.Beam, { end: { x: fx + 4.2, y: 2.45, z: -1.6 }, width: 0.2, color: { r: 1.6, g: 1.4, b: 3.5 }, texture: tex("textures/beam.png"), tile: 0.6, scroll: 2, segments: 14, jitter: 0.12, seed: 7, blend: "additive" });

  // Station 2d: lights and occluders (the sprites are drawn per tick).
  const x2 = ORIGIN["2d"];
  const lamp = (name: string, at: Vec3, color: Vec3, radius: number) => {
    const e = must(world.spawn({ name, position: at }));
    world.insert(e, og.Light2D, { color: { r: color.x, g: color.y, b: color.z }, intensity: 1.6, radius, sourceRadius: 0.3 });
    return e;
  };
  lamp("lamp-warm", { x: x2 - 5.5, y: 3, z: 0 }, { x: 1, y: 0.7, z: 0.35 }, 9);
  const cold = lamp("lamp-cold", { x: x2 + 3, y: -2, z: 0 }, { x: 0.35, y: 0.6, z: 1 }, 7);
  world.insert(cold, Orbit, { centre: { x: x2 + 2.5, y: -1, z: 0 }, radius: 1.5, speed: 0.2, plane: 1 });
  const wallBlock = must(world.spawn({ name: "wall-2d", position: { x: x2 - 2.5, y: 3.5, z: 0 }, quaternion: axisAngle({ x: 0, y: 0, z: 1 }, 0.3) }));
  world.insert(wallBlock, og.Occluder2D, { width: 3, height: 0.4 });
  const embers = must(world.spawn({ name: "embers", position: { x: x2 + 0.5, y: -4.2, z: 0 } }));
  world.insert(embers, og.ParticleSystem, { emitter: must(og.load.emitter(world, "fx/embers2d.ogt")), seed: 4 });

  // Station stylised: toon, rim, dissolve, flash and outline materials, the walker.
  const st = ORIGIN.stylised;
  ground(world, st, { x: 0.3, y: 0.32, z: 0.36 });
  const big = { x: 1.4, y: 1.4, z: 1.4 };
  spawnMesh(world, "toon", sphere, mat({ color: { r: 0.95, g: 0.45, b: 0.3 }, toon: true, gradientMap: tex("textures/toon.png"), outlineWidth: 3, outlineColor: { r: 0.05, g: 0.02, b: 0.02 } }), { x: st - 3.2, y: 0.7, z: 0 }, { scale: big });
  spawnMesh(world, "rim", sphere, mat({ color: { r: 0.12, g: 0.14, b: 0.2 }, roughness: 0.6, rimIntensity: 2.5, rimPower: 2.5, rimColor: { r: 0.3, g: 0.9, b: 1.2 } }), { x: st - 1.4, y: 0.7, z: -0.4 }, { scale: big });
  const dissolve = mat({ color: { r: 0.8, g: 0.8, b: 0.85 }, dissolveEdgeColor: { r: 5, g: 1.6, b: 0.3 }, dissolveEdgeWidth: 0.08 });
  spawnMesh(world, "dissolve", box, dissolve, { x: st + 1.6, y: 0.7, z: -0.4 }, { scale: big, quaternion: axisAngle({ x: 0, y: 1, z: 0 }, 0.6) });
  const flash = mat({ color: { r: 0.3, g: 0.55, b: 0.25 }, flashColor: { r: 4, g: 4, b: 4 } });
  spawnMesh(world, "flash", box, flash, { x: st + 3.4, y: 0.55, z: 0.6 }, { scale: { x: 1.1, y: 1.1, z: 1.1 } });
  const walkerModel = must(og.load.gltf(world, "models/walker.glb"));
  const rig = must(og.load.animation(world, "models/walker.glb"));
  const walk = rig.clips.find((c) => c.name === "walk") ?? rig.clips[0]!;
  const walker = must(world.instantiate(walkerModel, { name: "walker", position: { x: st + 0.1, y: 0, z: 1.9 }, quaternion: axisAngle({ x: 0, y: 1, z: 0 }, 0.35) }));
  world.insert(walker, og.Skeleton, { asset: rig.skeleton, rootMotion: "off" });
  world.insert(walker, og.AnimationPlayer, { clip: walk.clip, speed: 1, loop: true });
  world.insert(walker, og.Blendshapes, { source: "component" });

  // The E5 round's stations.
  const e5 = must(world.spawn({ name: "e5" }));
  world.insert(e5, E5);
  world.insert(must(world.spawn({ name: "p2" })), P2);
  buildE5(world, { box, sphere, mat, ground: (x, c) => ground(world, x, c) }, walkerModel, rig, tex("textures/tile.png"));
  // The phase 2 animation and physics round's stations.
  buildAP(world, { box, sphere, mat, ground: (x, c) => ground(world, x, c) }, walkerModel);
  // The phase 2 render round's stations.
  buildRD({ box, sphere, mat, ground: (x, c) => ground(world, x, c), tex, walker: walkerModel, rig });
  // The 3D fluid station.
  buildSPH(world, { box, mat, ground: (x, c) => ground(world, x, c) });
  // The P4 physical materials station.
  buildPM({ box, sphere, mat, ground: (x, c) => ground(world, x, c), tex });
  // The image-based lighting station (phase 4).
  buildIBL({ box, sphere, mat });
  // The looks station (phase 4): the catalogue's presets over one diorama.
  buildLooks({ box, sphere, mat, tex });
  // The depth wave's geometry station: runtime geometry, lines, points, debug draw.
  buildGEO(world, { mat, ground: (x, c) => ground(world, x, c) });
  // The depth wave's render stations: levels of detail and reflections.
  buildDepth({ box, sphere, mat });
  // The depth wave's views station: render targets, a mirror, split-screen.
  buildVW(world, { box, sphere, mat, ground: (x, c) => ground(world, x, c), walker: walkerModel, rig });
  // The depth wave's light stations: baked lighting, volumetric fog, area lights, cookies.
  buildLight(world, { mat });

  const stage = must(world.spawn({ name: "stage" }));
  world.insert(stage, Stage, {
    font,
    crate: tex("textures/crate.png"),
    tile: tex("textures/tile.png"),
    spark: tex("textures/spark.png"),
    dissolve,
    flash,
  });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Orbiter, Blink, Show, E5Show, PlatformMove, P2Show, APShow, SPHShow, GEOShow, LightShow, ModelsProbe],
  render: { antialias: "taa" },
  start(world) {
    build(world);
    show(world, "post");
  },
  commands: {
    station: (world, args) => {
      const name = (args[0] ?? "post") as Station;
      show(world, STATIONS.includes(name) ? name : STATIONS[Number(name) - 1] ?? "post");
    },
    baked: (world, args) => setBaked(world, args[0] !== "off"),
    look: (world, args) => {
      const slug = args[0] ?? "none";
      applyLookPreset(world, slug);
      // The HUD names the preset shown (none for an unknown slug, as applied).
      const stage = world.find("stage");
      if (stage !== null) world.set(stage, Stage, { look: LOOK_PRESETS.indexOf(slug as (typeof LOOK_PRESETS)[number]) + 1 });
    },
  },
  commandHelp: { station: { usage: "<name|1-33>", doc: "Show a station: post, lens, fx, 2d, stylised, then the E5 round's eight, then phase 2's terrain, the animation and physics round's four, the render round's four (lights, moon, materials, skinned), the 3D fluid station (fluids3d), P4's physical materials (physical) and the image-based lighting station (environment), the looks station (looks), and the depth wave's geometry station (geometry: runtime geometry, lines, points, debug draw), lod (2,000 trees in levels of detail), reflections (a reflection probe and SSR on a wet floor), views (a security monitor, a mirror and split-screen, from render targets and secondary cameras), and its light stations (baked: a lightmapped Cornell box; volumetric: sun shafts through a window, a rect light and cookies in volumetric fog), and round 3's islands (a terrain the cook generates, its sea at the terrain's seaLevel) and models (a `type model` of og-geom parts the cook built: a gear, bolts, a vase, a horn, a dice)." }, baked: { usage: "<on|off>", doc: "RenderSettings.bakedLighting: off draws the baked station's Cornell box unbaked." }, look: { usage: "<slug|none>", doc: "On the looks station: apply one of the engine's look presets (templates/looks/<slug>.ogt) to the diorama, or none." } },
  facts: (world) => {
    const e = world.find("stage");
    const s = e === null ? null : world.get(e, Stage);
    return s?.ok ? { station: STATIONS[s.value.station] ?? "post", shown: s.value.shown } : {};
  },
});
