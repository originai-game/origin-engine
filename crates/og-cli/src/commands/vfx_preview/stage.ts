// The stage `og vfx preview` plays an effect on (vfx_preview.rs writes it to
// .og/vfx-preview/main.ts with its __NAMES__ filled in): a dark ground at
// dusk (or a grey one under an overcast day, `--stage day`), the effect at
// the origin, a camera the command places.
//
//   stage:measure   an unlit black ground and sky, no bloom or ambient light:
//                   only what the effect draws differs from the empty frame
//   stage:marks     a grey ground lit from straight above, a black sky, no
//                   bloom: what an effect of decals alone leaves on it
//                   differs from the empty frame
//   stage:show      the ground, a moon (or the day's sun) and bloom, as a
//                   game would show it
//   eye:px:py:pz:tx:ty:tz:fov    the camera, looking from p at t
//   lift:<metres>   how far above the origin the next effects play (one whose
//                   origin is its centre is raised to stand on the ground)
//   play:<asset id> the effect at the origin, replacing the last
//   add:x:z:<asset id>  another effect at (x, 0, z), keeping the others
//   clear           despawns them
import { og, defineSystem } from "@origingame/engine";
import type { Entity, GraphicsQuality, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const DIM: string = "__DIM__";
const VFX2D: string[] = __VFX2D__;
const QUALITY: string = "__QUALITY__";
const CONTEXT: { stage?: string; enabled?: boolean; look?: string; scene?: string; position?: number[]; camera?: number[] } = __CONTEXT__;
const VIEW_HEIGHT = __H__;
const VIEW_WIDTH = __W__;
const shown: Entity[] = [];
const log: { name: string; tick: number }[] = [];
const starts: { path: string; tick: number; duration: number; loop: boolean; elements: string[] }[] = [];
let lift = 0;
let ground: Entity | null = null;
let moon: Entity | null = null;
let contextReady = false;

function stage(world: World, mode: string): void {
  const cam = must(world.resource(og.Camera));
  const rs = must(world.resource(og.RenderSettings));
  const show = mode === "show";
  if (show && contextReady) return;
  // Grain changes every tick: over black it would measure the whole frame
  // as the effect. The show bundle keeps the project's authored grain.
  if (!show) must(world.resource(og.PostEffects)).filmGrain = 0;
  // A dusk shows what glows; a day shows what is lit (smoke, dust, debris,
  // marks), which a moon leaves near black.
  const day = show && CONTEXT.stage === "day";
  const marks = mode === "marks";
  // Shown, the sky is the far ground's tone: no bright band above the horizon.
  cam.background = day ? { r: 0.1, g: 0.115, b: 0.135 } : show ? { r: 0.014, g: 0.02, b: 0.038 } : { r: 0, g: 0, b: 0 };
  rs.bloom = show;
  if (DIM === "2d") {
    cam.orthographic = true;
    cam.height = 10;
    cam.position = { x: 0, y: 0, z: 10 };
    return;
  }
  rs.environmentIntensity = day ? 0.35 : show ? 0.15 : 0;
  if (ground !== null) world.despawn(ground);
  if (moon !== null) world.despawn(moon);
  const box = must(og.render.boxGeometry(world, {}));
  const tone = day ? 0.16 : show ? 0.12 : marks ? 0.5 : 0;
  // Measured, the ground is a sheet: the front view, level with it, sees it
  // edge on and so sees what the effect draws under it.
  const thick = show ? 0.5 : 0.02;
  ground = must(world.spawn({ position: { x: 0, y: -thick / 2, z: 0 } }));
  must(world.set(ground, og.Transform, { scale: { x: 200, y: thick, z: 200 } }));
  // Measured, the ground is unlit black: it stops what falls on it and shows
  // no light. A decal lands on lit surfaces alone, so marks are measured on
  // a lit one.
  must(world.insert(ground, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color: { r: tone, g: tone, b: tone + (show && !day ? 0.01 : 0) }, roughness: 0.85, unlit: !show && !marks })) }));
  moon = null;
  if (day) {
    moon = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.35, y: -0.8, z: -0.5 }) }));
    must(world.insert(moon, og.DirectionalLight, { color: { r: 1, g: 0.95, b: 0.88 }, intensity: 1.5, castShadow: true }));
  } else if (show) {
    moon = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.3, y: -0.6, z: -0.6 }) }));
    must(world.insert(moon, og.DirectionalLight, { color: { r: 0.55, g: 0.6, b: 0.9 }, intensity: 0.25, castShadow: true }));
  } else if (marks) {
    moon = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0, y: -1, z: -0.001 }) }));
    must(world.insert(moon, og.DirectionalLight, { color: { r: 1, g: 1, b: 1 }, intensity: 1, castShadow: false }));
  }
  if (show && CONTEXT.enabled) {
    if (CONTEXT.look) must(og.render.applyLook(world, must(og.load.look(world, CONTEXT.look))));
    if (CONTEXT.scene) {
      if (ground !== null) world.despawn(ground);
      if (moon !== null) world.despawn(moon);
      ground = moon = null;
      must(world.instantiate(must(og.load.scene(world, CONTEXT.scene))));
    }
    if (QUALITY !== "") must(og.render.applyQuality(world, QUALITY as GraphicsQuality));
    contextReady = true;
  }
}

function view(world: World, a: number[]): void {
  if (DIM === "2d") return;
  const offset = CONTEXT.position ?? [0, 0, 0];
  a = CONTEXT.camera ?? a.map((value, i) => i < 6 ? value + offset[i % 3]! : value);
  const cam = must(world.resource(og.Camera));
  cam.position = { x: a[0]!, y: a[1]!, z: a[2]! };
  cam.quaternion = og.transform.lookAt(cam.position, { x: a[3]!, y: a[4]!, z: a[5]! });
  cam.fov = a[6]!;
  const reach = Math.hypot(a[0]! - a[3]!, a[1]! - a[4]!, a[2]! - a[5]!);
  cam.near = Math.max(0.05, reach * 0.02);
  cam.far = reach * 4 + 100;
}

function clear(world: World): void {
  for (const e of shown.splice(0)) world.despawn(e);
}

function play(world: World, path: string): void {
  clear(world);
  if (VFX2D.includes(path)) {
    const info = must(og.load.flipbook(world, path));
    const clip = info.clips[0]!;
    // Whole screen pixels per art pixel, the frame filling at most 80% of
    // the view (Camera.height 10 m over the view's height in pixels).
    const k = Math.max(1, Math.floor(Math.min((0.8 * VIEW_HEIGHT) / info.frameHeight, (0.8 * VIEW_WIDTH) / info.frameWidth)));
    const m = 10 / VIEW_HEIGHT;
    const e = must(world.spawn({ position: { x: 0, y: 0, z: 0 } }));
    must(world.insert(e, og.Sprite, { texture: info.texture, width: info.frameWidth * k * m, height: info.frameHeight * k * m }));
    must(world.insert(e, og.Flipbook, { sheet: info.sheet, clip: clip.clip }));
    shown.push(e);
    starts.push({ path, tick: world.tick, duration: clip.duration, loop: clip.mode !== "once", elements: ["flipbook"] });
    return;
  }
  const asset = must(og.load.vfx(world, path));
  const info = must(og.vfx.info(world, asset));
  const at = CONTEXT.position ?? [0, 0, 0];
  shown.push(must(og.vfx.spawn(world, { asset, position: { x: at[0]!, y: at[1]! + lift, z: at[2]! } })));
  starts.push({ path, tick: world.tick, duration: info.duration, loop: info.loop, elements: info.elements });
}

export default og.game({
  tick: 60,
  seed: 1,
  systems: [
    defineSystem("Watch", { reads: [og.VfxEvent, og.FlipbookEvent], writes: [] }, (_q, _t, world) => {
      for (const e of world.events(og.VfxEvent)) log.push({ name: e.name, tick: world.tick });
      for (const e of world.events(og.FlipbookEvent)) log.push({ name: e.name, tick: world.tick });
    }),
  ],
  start: (world) => {
    if (QUALITY !== "") must(og.render.applyQuality(world, QUALITY as GraphicsQuality));
    stage(world, "show");
  },
  // Pixel art keeps its pixels, and a measured frame its edges: no temporal
  // antialiasing to soften either or to carry one view into the next.
  render: { antialias: DIM === "2d" ? "off" : "__ANTIALIAS__" },
  commands: {
    stage: (world, args) => stage(world, String(args[0])),
    eye: (world, args) => view(world, args.map(Number)),
    lift: (_world, args) => {
      lift = Number(args[0]);
    },
    play: (world, args) => play(world, args.map(String).join(":")),
    add: (world, args) => {
      const asset = must(og.load.vfx(world, args.slice(2).map(String).join(":")));
      const at = CONTEXT.position ?? [0, 0, 0];
      shown.push(must(og.vfx.spawn(world, { asset, position: { x: Number(args[0]) + at[0]!, y: at[1]!, z: Number(args[1]) + at[2]! } })));
    },
    clear: (world) => clear(world),
  },
  facts: () => ({ starts, log }),
});
