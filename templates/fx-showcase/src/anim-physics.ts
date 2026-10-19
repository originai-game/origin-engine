// The phase 2 animation and physics round's stations, far from the others
// along x:
//
//   retarget   one walk from an animation library on three characters of
//              other proportions and bone names: the walker (Blender names,
//              retargeted at run time with og.anim.retarget), the strider
//              (Mixamo names, retargeted by the cook from its import
//              settings' `animation { library }`) and the stout (Unreal
//              names, at run time); root motion carries each at its own
//              stride, the feet planted on the ground
//   ragdoll2d  a side-on puppet walking on a ledge in a 2D world
//              (og.Physics.profile 2) goes limp, knocked off the ledge: an
//              og.Ragdoll of rapier2d bones on hinges about z
//   cloth      a sheet lowered onto the floor while its bar sweeps back
//              and forth lays itself in folds, layer on layer, without
//              passing through itself (og.Cloth.thickness, friction)
//   fluids     eight og.Fluid2D in one frame: a heavy blue fluid released
//              beside a light orange one slides under it (a group, mixing
//              0), red poured on yellow blends into orange (a group,
//              mixing 1), and four cups of lone fluids

import { og, defineComponent, defineSystem, f32, f64, u32, PIN_TOP, PROFILE_2D, PROFILE_3D, SHAPE_BOX } from "@origingame/engine";
import type { Asset, Entity, Geometry, Material, Quaternion, Value, Vector3 as Vec3, World } from "@origingame/engine";

export const AP_STATIONS = ["retarget", "ragdoll2d", "cloth", "fluids"] as const;
export type APStation = (typeof AP_STATIONS)[number];

/** Where each station lives along x. */
export const AP_ORIGIN: Record<APStation, number> = {
  retarget: 1400,
  ragdoll2d: 1500,
  cloth: 1600,
  fluids: 1700,
};

/** The cloth station's sheet: points across and down, their spacing (and the thickness it keeps). */
const SHEET = { n: 20, spacing: 0.06 };
/**
 * The cloth station's sheet top row (and its bar): lowered from 1.6 m to
 * 0.12 m over two and a half seconds while it sweeps back and forth along
 * z, so the sheet lays itself on the floor in folds, layer on layer.
 */
function sheetAt(shown: number): Vec3 {
  const s = Math.min(shown / 150, 1);
  return { x: AP_ORIGIN.cloth - 0.57, y: 1.6 - 1.48 * s, z: 0.3 * Math.sin(shown * 0.07) };
}

/** The round's state, on the entity "anim-physics". */
export const AP = defineComponent("AnimPhysics", {
  /** Index into AP_STATIONS + 1 (0: another station). */
  station: u32(0),
  /** Ticks since the station was shown. */
  shown: u32(0),
  /** Humanoid bones og.anim.humanoid found on each retarget character. */
  walkerBones: u32(0),
  striderBones: u32(0),
  stoutBones: u32(0),
  /** The 2D puppet, the cloth, and the fluids of the layering tank. */
  puppet: f64(0),
  sheet: f64(0),
  heavy: f64(0),
  light: f64(0),
  /** Written by the station: the puppet's lowest bone and its bones' largest |z| while limp. */
  ragdollLow: f32(99),
  ragdollZ: f32(0),
  /** Written: the cloth's closest pair of points that are not near neighbours (× its thickness, over the run), and its points on the layers above the floor's. */
  clothClosest: f32(99),
  clothAbove: u32(0),
  /** Written: the mean heights of the layering tank's heavy and light fluids. */
  heavyY: f32(0),
  lightY: f32(0),
});

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

const axisAngle = (axis: Vec3, angle: number): Quaternion => og.transform.fromAxisAngle(axis, angle);

/** The closest two points of a sheet that are not near neighbours (rest distance at least `spacing`), over `spacing`. */
function closest(points: ReadonlyArray<Vec3>): number {
  let best = Infinity;
  const { n, spacing } = SHEET;
  for (let i = 0; i < points.length; i++) {
    const a = points[i]!;
    for (let j = i + 1; j < points.length; j++) {
      const dx = (i % n) - (j % n);
      const dy = Math.floor(i / n) - Math.floor(j / n);
      if (dx * dx + dy * dy < 1) continue;
      const b = points[j]!;
      const d = Math.hypot(a.x - b.x, a.y - b.y, a.z - b.z);
      if (d < best) best = d;
    }
  }
  return best / spacing;
}

/** Per tick on the shown station: the puppet's fall, the sheet's lowering, the fluids' heights. */
export const APShow = defineSystem("APShow", { writes: [AP, og.Ragdoll, og.Transform] }, (_q, _time, world) => {
  const e = world.find("anim-physics");
  if (e === null) return;
  const s = must(world.get(e, AP));
  const shown = s.shown + 1;
  world.set(e, AP, { shown });
  if (s.station === 0) return;
  switch (AP_STATIONS[s.station - 1]) {
    case "ragdoll2d": {
      // It walks for half a second, then goes limp, knocked off its ledge.
      const puppet = s.puppet as unknown as Entity;
      world.set(puppet, og.Ragdoll, { weight: shown > 30 ? 1 : 0 });
      const pose = og.physics.ragdollPose(world, puppet);
      if (pose.ok && shown > 30) {
        let low = s.ragdollLow;
        let z = s.ragdollZ;
        for (const b of pose.value) {
          low = Math.min(low, b.position.y);
          z = Math.max(z, Math.abs(b.position.z));
        }
        world.set(e, AP, { ragdollLow: low, ragdollZ: z });
      }
      break;
    }
    case "cloth": {
      const sheet = s.sheet as unknown as Entity;
      const top = sheetAt(shown);
      world.set(sheet, og.Transform, { position: top });
      const bar = world.find("sheet-bar");
      if (bar !== null) world.set(bar, og.Transform, { position: { x: AP_ORIGIN.cloth, y: top.y + 0.02, z: top.z } });
      const pts = og.physics.clothPoints(world, sheet);
      if (pts.ok) {
        const above = pts.value.filter((p) => p.y > 0.08 && p.y < 0.4).length;
        world.set(e, AP, { clothClosest: Math.min(s.clothClosest, closest(pts.value)), clothAbove: above });
      }
      break;
    }
    case "fluids": {
      const mean = (f: number) => {
        const p = og.physics.fluidParticles(world, f as unknown as Entity);
        return p.ok && p.value.length > 0 ? p.value.reduce((a, q) => a + q.y, 0) / p.value.length : 0;
      };
      world.set(e, AP, { heavyY: mean(s.heavy), lightY: mean(s.light) });
      break;
    }
  }
});

interface Kit {
  box: Geometry;
  sphere: Geometry;
  mat: (d: Parameters<typeof og.render.standardMaterial>[1]) => Material;
  ground: (x: number, color: Vec3) => void;
}

let builders: Partial<Record<APStation, (world: World) => void>> = {};

/** Marks a station of this round as not shown (another station is), back in the 3D profile. */
export function resetAP(world: World): void {
  const e = world.find("anim-physics");
  if (e !== null) world.set(e, AP, { station: 0, shown: 0 });
  const physics = must(world.resource(og.Physics));
  if (physics.profile !== PROFILE_3D) physics.profile = PROFILE_3D;
}

/** Shows a station of this round: built when first shown, then its camera and light. */
export function showAP(world: World, station: APStation): void {
  if (world.find(`p2-${station}`) === null) {
    builders[station]?.(world);
    must(world.spawn({ name: `p2-${station}` }));
  }
  const e = world.find("anim-physics");
  if (e !== null) world.set(e, AP, { station: AP_STATIONS.indexOf(station) + 1, shown: 0 });
  const cam = must(world.resource(og.Camera));
  const settings = must(world.resource(og.RenderSettings));
  const sun = world.find("sun");
  const x = AP_ORIGIN[station];
  const look = (eye: Vec3, target: Vec3) => {
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, target);
  };
  const sunAt = (target: Vec3, intensity: number) => {
    if (sun === null) return;
    world.set(sun, og.Transform, { quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, target) });
    world.set(sun, og.DirectionalLight, { intensity });
  };
  settings.sky = true;
  settings.bloom = false;
  settings.environmentIntensity = 0.55;
  switch (station) {
    case "retarget":
      look({ x: x - 1, y: 1.5, z: 8.5 }, { x: x - 1, y: 0.9, z: 0 });
      sunAt({ x: -0.35, y: -0.8, z: -0.5 }, 3);
      break;
    case "ragdoll2d":
      // A 2D world while it is shown: rapier2d in the xy plane.
      must(world.resource(og.Physics)).profile = PROFILE_2D;
      cam.orthographic = true;
      cam.height = 4.4;
      cam.position = { x: x - 0.2, y: 1.1, z: 10 };
      cam.quaternion = { x: 0, y: 0, z: 0, w: 1 };
      cam.background = { r: 0.16, g: 0.18, b: 0.22 };
      settings.sky = false;
      sunAt({ x: -0.3, y: -0.5, z: -0.8 }, 3);
      break;
    case "cloth":
      look({ x: x + 0.95, y: 0.8, z: 1.15 }, { x: x - 0.05, y: 0.06, z: 0 });
      sunAt({ x: -0.4, y: -0.8, z: -0.45 }, 3);
      break;
    case "fluids":
      cam.orthographic = true;
      cam.height = 7.4;
      cam.position = { x, y: 0.4, z: 10 };
      cam.quaternion = { x: 0, y: 0, z: 0, w: 1 };
      cam.background = { r: 0.07, g: 0.07, b: 0.09 };
      settings.sky = false;
      settings.toneMapped = false;
      break;
  }
}

/** The characters of the retarget station: name, model, row (z), colour of the lane under it. */
const WALKERS = [
  { name: "rt-strider", model: "models/strider.glb", z: -1.4 },
  { name: "rt-walker", model: "models/walker.glb", z: 0 },
  { name: "rt-stout", model: "models/stout.glb", z: 1.4 },
] as const;

/** Prepares this round's stations (each built when first shown). */
export function buildAP(world: World, k: Kit, walkerModel: Asset): void {
  const ap = must(world.spawn({ name: "anim-physics" }));
  world.insert(ap, AP);
  builders = {
    // retarget: a library's walk on three characters, walking along +x.
    retarget: (world: World) => {
      const x = AP_ORIGIN.retarget;
      k.ground(x, { x: 0.3, y: 0.31, z: 0.3 });
      const lane = k.mat({ color: { r: 0.55, g: 0.53, b: 0.48 }, roughness: 0.9 });
      const library = must(og.load.animation(world, "anims/humanoid-clips.glb"));
      const libWalk = library.clips.find((c) => c.name === "walk")!;
      const striderRig = must(og.load.rig(world, "rigs/strider.ogt"));
      const bones: number[] = [];
      for (const w of WALKERS) {
        // A lane under each, so the feet read against the ground.
        const strip = must(world.spawn({ name: `${w.name}-lane`, position: { x: x - 1, y: 0.005, z: w.z } }));
        world.set(strip, og.Transform, { scale: { x: 12, y: 0.01, z: 0.5 } });
        world.insert(strip, og.Mesh, { geometry: k.box, material: lane, receiveShadow: true });
        const model = w.model === "models/walker.glb" ? walkerModel : must(og.load.gltf(world, w.model));
        const anim = must(og.load.animation(world, w.model));
        const humanoid = og.anim.humanoid(world, w.name === "rt-strider" ? { skeleton: anim.skeleton, rig: striderRig } : { skeleton: anim.skeleton });
        bones.push(must(humanoid).bones.length);
        // The strider's walk was retargeted by the cook; the others' here.
        const clip =
          w.name === "rt-strider"
            ? anim.clips.find((c) => c.name === "walk")!.clip
            : must(og.anim.retarget(world, { clip: libWalk.clip, source: library.skeleton, target: anim.skeleton })).clip;
        const e = must(world.instantiate(model, { name: w.name, position: { x: x - 5, y: 0, z: w.z }, quaternion: axisAngle({ x: 0, y: 1, z: 0 }, Math.PI / 2) }));
        world.insert(e, og.Skeleton, { asset: anim.skeleton, rootMotion: "planar" });
        world.insert(e, og.AnimationPlayer, { clip, speed: 1, loop: true, fade: 0 });
      }
      world.set(world.find("anim-physics")!, AP, { walkerBones: bones[1]!, striderBones: bones[0]!, stoutBones: bones[2]! });
    },
    // ragdoll2d: a puppet on a ledge above a step and the floor.
    ragdoll2d: (world: World) => {
      const x = AP_ORIGIN.ragdoll2d;
      const stone = k.mat({ color: { r: 0.42, g: 0.44, b: 0.48 }, roughness: 0.9 });
      for (const [name, cx, cy, w, h] of [
        ["rd-floor", x, -0.25, 12, 0.5],
        ["rd-ledge", x - 1.5, 0.5, 2.4, 1],
        ["rd-step", x + 0.7, 0.2, 1.2, 0.4],
      ] as const) {
        const e = must(world.spawn({ name, position: { x: cx, y: cy, z: -0.2 } }));
        world.set(e, og.Transform, { scale: { x: w, y: h, z: 0.4 } });
        world.insert(e, og.Mesh, { geometry: k.box, material: stone, receiveShadow: true });
        world.insert(e, og.Collider, { shape: SHAPE_BOX });
      }
      const model = must(og.load.gltf(world, "models/puppet2d.glb"));
      const anim = must(og.load.animation(world, "models/puppet2d.glb"));
      const puppet = must(world.instantiate(model, { name: "puppet", position: { x: x - 1.6, y: 1, z: 0 } }));
      world.insert(puppet, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" });
      world.insert(puppet, og.AnimationPlayer, { clip: anim.clips[0]!.clip, loop: true, fade: 0 });
      world.insert(puppet, og.Ragdoll, { weight: 0, blendTime: 0.3, impulse: { x: 110, y: 40, z: 0 } });
      must(og.physics.setRagdollRig(world, puppet, must(og.physics.ragdollRigFromSkeleton(world, anim.skeleton))));
      world.set(world.find("anim-physics")!, AP, { puppet });
    },
    // cloth: a sheet hung from its top row, laid on the floor in folds.
    cloth: (world: World) => {
      const x = AP_ORIGIN.cloth;
      k.ground(x, { x: 0.36, y: 0.37, z: 0.4 });
      // The floor the sheet lands on (the ground plane is only drawn).
      const floor = must(world.spawn({ name: "cloth-floor", position: { x, y: -0.5, z: 0 } }));
      world.insert(floor, og.Collider, { shape: SHAPE_BOX, size: { x: 6, y: 1, z: 6 } });
      const red = k.mat({ color: { r: 0.75, g: 0.12, b: 0.1 }, roughness: 0.85, side: "double" });
      const bar = must(world.spawn({ name: "sheet-bar", position: { x: x, y: 1.62, z: 0 } }));
      world.set(bar, og.Transform, { scale: { x: 1.4, y: 0.04, z: 0.04 } });
      world.insert(bar, og.Mesh, { geometry: k.box, material: k.mat({ color: { r: 0.3, g: 0.22, b: 0.14 }, roughness: 0.6 }) });
      const sheet = must(world.spawn({ name: "sheet", position: sheetAt(0) }));
      world.insert(sheet, og.Cloth, {
        columns: SHEET.n,
        rows: SHEET.n,
        spacing: SHEET.spacing,
        pins: PIN_TOP,
        thickness: SHEET.spacing,
        substeps: 2,
        iterations: 6,
        friction: 0.9,
        radius: 0.03,
        gust: 0,
        material: red,
      });
      world.set(world.find("anim-physics")!, AP, { sheet });
    },
    // fluids: a layering tank, a blending tank and four cups.
    fluids: (world: World) => {
      const x = AP_ORIGIN.fluids;
      const stone = k.mat({ color: { r: 0.4, g: 0.38, b: 0.36 }, roughness: 0.9 });
      // A wall: its box drawn behind the fluids' plane, its collider deep
      // around it (a collider whose face lay on the plane would leave the
      // particles' contacts to rounding).
      const wall = (name: string, cx: number, cy: number, w: number, h: number) => {
        const e = must(world.spawn({ name, position: { x: x + cx, y: cy, z: -0.5 } }));
        world.set(e, og.Transform, { scale: { x: w, y: h, z: 1 } });
        world.insert(e, og.Mesh, { geometry: k.box, material: stone });
        const c = must(world.spawn({ name: `${name}-collider`, position: { x: x + cx, y: cy, z: 0 } }));
        world.insert(c, og.Collider, { shape: SHAPE_BOX, size: { x: w, y: h, z: 4 } });
      };
      // Two tanks 3.6 m wide on the floor, and four cups above.
      for (const [t, cx] of [["a", -2.6], ["b", 2.6]] as const) {
        wall(`tank-${t}-floor`, cx, -2.25, 4.4, 0.5);
        wall(`tank-${t}-left`, cx - 2, -0.6, 0.4, 3.4);
        wall(`tank-${t}-right`, cx + 2, -0.6, 0.4, 3.4);
      }
      const cups = [-2.7, -0.9, 0.9, 2.7];
      cups.forEach((cx, i) => {
        wall(`cup-${i}-floor`, cx, 2.3, 1.3, 0.2);
        wall(`cup-${i}-left`, cx - 0.6, 2.7, 0.1, 0.8);
        wall(`cup-${i}-right`, cx + 0.6, 2.7, 0.1, 0.8);
      });
      const fluid = (name: string, at: Vec3, d: Partial<Value<typeof og.Fluid2D>>) => {
        const e = must(world.spawn({ name, position: at }));
        world.insert(e, og.Fluid2D, { radius: 0.16, layer: 3, ...d });
        return e;
      };
      // Tank a: heavy blue (density 3) released beside light orange, one
      // group, mixing 0: the blue slides under the orange and they layer.
      // (Poured on top, the light fluid squeezed out between them jets
      // over the walls.)
      const heavy = fluid("heavy", { x: x - 3.5, y: -0.85, z: 0 }, { count: 260, size: { x: 1.7, y: 2.2 }, group: 1, density: 3, color: { x: 0.12, y: 0.4, z: 0.95 }, seed: 1 });
      const light = fluid("light", { x: x - 1.7, y: -0.85, z: 0 }, { count: 260, size: { x: 1.7, y: 2.2 }, group: 1, density: 1, color: { x: 1.0, y: 0.55, z: 0.1 }, seed: 2 });
      // Tank b: red poured on yellow, one group, mixing 1: they blend.
      fluid("red", { x: x + 2.6, y: 0.2, z: 0 }, { count: 200, size: { x: 3.4, y: 1.0 }, group: 2, mixing: 1, color: { x: 0.95, y: 0.08, z: 0.06 }, seed: 3 });
      fluid("yellow", { x: x + 2.6, y: -1.3, z: 0 }, { count: 200, size: { x: 3.4, y: 1.0 }, group: 2, mixing: 1, color: { x: 1.0, y: 0.9, z: 0.1 }, seed: 4 });
      // The cups: four lone fluids (eight fluids drawn this frame).
      const colors = [{ x: 0.2, y: 0.85, z: 0.3 }, { x: 0.2, y: 0.85, z: 0.9 }, { x: 0.95, y: 0.35, z: 0.75 }, { x: 0.9, y: 0.9, z: 0.92 }];
      cups.forEach((cx, i) => fluid(`cup-${i}`, { x: x + cx, y: 2.75, z: 0 }, { count: 45, size: { x: 0.9, y: 0.5 }, color: colors[i]!, seed: 10 + i }));
      world.set(world.find("anim-physics")!, AP, { heavy, light });
    },
  };
}
