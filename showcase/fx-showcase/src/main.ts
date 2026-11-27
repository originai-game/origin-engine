import { og, math, quat, defineResource, defineSystem, Entity, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { GROUPS } from "./effects.ts";
import type { Effect } from "./effects.ts";

// FX Showcase: every effect of the installed packs on one stage, one at a
// time. It plays them in order on its own; ArrowRight / ArrowLeft step,
// ArrowDown / ArrowUp jump a group, Space holds the current one. Each effect
// stands where its pack's review placed it, with the camera at its reach.

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

interface Entry { readonly pack: string; readonly group: number; readonly zh: string; readonly en: string; readonly e: Effect }

// The geometric pack's mechanisms, placed as its review placed them.
function geo(name: string, reach: number, look: number, loops: boolean, seconds: number, y = 0.15): Effect {
  return { name, reach, look, y, rot: [0, 0, 0], fly: 0, params: [], loops, seconds };
}
const GEOMETRIC = [
  geo("explosion", 0.6, 0.6, false, 3), geo("impact", 0.28, 0.35, false, 2), geo("muzzle", 0.16, 0.3, false, 1.5),
  geo("pickup", 0.18, 0.35, false, 2), geo("magic", 0.4, 0.45, false, 2.5), geo("dust", 0.35, 0.3, false, 3),
  geo("splash", 0.28, 0.35, false, 2.5), geo("trail", 0.3, 0.5, true, 3, 0.5), geo("celebration", 0.45, 0.7, false, 3),
];

const ENTRIES: readonly Entry[] = [
  ...GROUPS.flatMap((g, i) => g.effects.map((e) => ({ pack: "cinematic-realism", group: i, zh: g.zh, en: g.en, e }))),
  ...GEOMETRIC.map((e) => ({ pack: "geometric", group: GROUPS.length, zh: "几何风格", en: "Geometric", e })),
];
const GROUP_COUNT = GROUPS.length + 1;

/** How long an effect stays on stage before the next: a loop shows for 3 s, a one-shot for its own length (marks cut short). */
function hold(e: Effect): number {
  return e.loops ? 3 : Math.min(e.seconds, 5) + 0.6;
}

const Show = defineResource("Show", {
  shown: Entity(), active: u32(), index: u32(), started: u32(), held: u32(),
});

function play(world: World, index: number): void {
  const s = must(world.resource(Show));
  if (s.active) world.despawn(s.shown);
  const n = ENTRIES.length;
  const entry = ENTRIES[((index % n) + n) % n]!;
  const e = entry.e;
  const rad = 0.017453292519943295;
  const asset = must(og.load.vfx(world, `base/vfx/${entry.pack}/${e.name}`));
  s.shown = must(og.vfx.spawn(world, {
    asset, position: { x: e.fly ? -e.fly / 20 : 0, y: e.y, z: 0 }, seed: 37,
    quaternion: quat.fromEuler({ x: e.rot[0] * rad, y: e.rot[1] * rad, z: e.rot[2] * rad }),
    ...(e.params.length ? { params: e.params } : {}),
  }));
  s.active = 1;
  s.index = ((index % n) + n) % n;
  s.started = world.tick;
  const cam = must(world.resource(og.Camera));
  cam.position = { x: 3.4 * e.reach, y: e.look + 2 * e.reach, z: 6.2 * e.reach };
  cam.quaternion = og.transform.lookAt(cam.position, { x: 0, y: e.look, z: 0 });
  cam.near = Math.max(0.02, e.reach * 0.05);
}

/** The first entry of the group `step` groups from the current one. */
function groupStart(index: number, step: number): number {
  const g = (((ENTRIES[index]!.group + step) % GROUP_COUNT) + GROUP_COUNT) % GROUP_COUNT;
  return ENTRIES.findIndex((x) => x.group === g);
}

const Showcase = defineSystem("Showcase", { reads: [og.Input], writes: [Show, og.Transform, og.Camera, og.Vfx] }, (_q, _t, world) => {
  const s = must(world.resource(Show));
  const input = must(world.resource(og.Input));
  if (input.justPressed("Space")) s.held = s.held ? 0 : 1;
  const t = (world.tick - s.started) / 60;
  const entry = ENTRIES[s.index]!;
  const step = input.justPressed("ArrowRight") ? 1 : input.justPressed("ArrowLeft") ? -1 : 0;
  if (step) {
    og.assert.reachable("an arrow key stepped to another effect", {});
    return play(world, s.index + step);
  }
  const group = input.justPressed("ArrowDown") ? 1 : input.justPressed("ArrowUp") ? -1 : 0;
  if (group) {
    og.assert.reachable("an arrow key jumped to another group", {});
    return play(world, groupStart(s.index, group));
  }
  if (t >= hold(entry.e)) return play(world, s.held ? s.index : s.index + 1);
  if (entry.e.fly) {
    // A projectile's flight along +x through the origin at 0.5 s.
    must(world.set(s.shown, og.Transform, { position: { x: (t - 0.5) * entry.e.fly / 10, y: entry.e.y, z: 0 } }));
  } else if (entry.pack === "geometric" && entry.e.name === "trail") {
    must(world.set(s.shown, og.Transform, { position: { x: math.sin(t * 2) * 0.8, y: 0.5, z: 0 } }));
  }
  const font = must(og.load.font(world, "@fonts/latin/NotoSans-Latin"));
  const white = { r: 0.92, g: 0.93, b: 0.95 };
  must(og.render.text(world, { font, screen: true, position: { x: 32, y: 52, z: 0 }, size: 28, color: white, outline: 2,
    text: `${entry.en}  ·  ${entry.e.name}` }));
  must(og.render.text(world, { font, screen: true, position: { x: 32, y: 84, z: 0 }, size: 16, color: white, opacity: 0.7, outline: 1,
    text: `${entry.pack}  ${s.index + 1} / ${ENTRIES.length}${s.held ? "  (held)" : ""}\n← → effect   ↑ ↓ group   Space hold` }));
});

export default og.game({
  tick: 60,
  seed: 1,
  resources: [Show],
  systems: [Showcase],
  start: (world) => {
    const cam = must(world.resource(og.Camera));
    cam.fov = 42;
    cam.far = 120;
    cam.exposure = 0.65;
    cam.background = { r: 0.11, g: 0.14, b: 0.18 };
    const box = must(og.render.boxGeometry(world, {}));
    const block = (p: { x: number; y: number; z: number }, sc: { x: number; y: number; z: number }, c: { r: number; g: number; b: number }, roughness = 0.8): void => {
      const e = must(world.spawn({ position: p }));
      must(world.set(e, og.Transform, { scale: sc }));
      must(world.insert(e, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color: c, roughness })) }));
    };
    // A ground, a pad under the effect, one-metre and ten-centimetre cubes for scale, and depth behind.
    // Dark asphalt and weathered concrete (albedo about 0.05 to 0.2), so light, smoke and sparks read against them.
    block({ x: 0, y: -0.15, z: 0 }, { x: 40, y: 0.3, z: 40 }, { r: 0.045, g: 0.048, b: 0.052 }, 0.9);
    block({ x: 0, y: 0.04, z: 0 }, { x: 2, y: 0.08, z: 2 }, { r: 0.09, g: 0.095, b: 0.1 }, 0.6);
    block({ x: -2, y: 0.5, z: -1 }, { x: 1, y: 1, z: 1 }, { r: 0.18, g: 0.18, b: 0.17 });
    block({ x: -0.45, y: 0.13, z: 0 }, { x: 0.1, y: 0.1, z: 0.1 }, { r: 0.18, g: 0.18, b: 0.17 });
    block({ x: 3, y: 1.5, z: -4 }, { x: 1, y: 3, z: 1 }, { r: 0.1, g: 0.11, b: 0.12 });
    block({ x: -4, y: 1, z: -7 }, { x: 4, y: 2, z: 1 }, { r: 0.13, g: 0.14, b: 0.15 });
    const sun = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.4, y: -0.8, z: -0.3 }) }));
    must(world.insert(sun, og.DirectionalLight, { color: { r: 1, g: 0.9, b: 0.78 }, intensity: 2, castShadow: true }));
    play(world, 0);
  },
  commands: {
    // goto:<pack>:<effect>, for captures.
    goto: (world, args) => {
      const i = ENTRIES.findIndex((x) => x.pack === String(args[0]) && x.e.name === String(args[1]));
      if (i >= 0) play(world, i);
    },
    hold: (world) => { must(world.resource(Show)).held = 1; },
  },
});
