// The 3D sample scenes' shared layout (docs/art.md §2.6): a hero, a stepped
// tree, a crate, a lantern on the ground, and a far ridge with a village,
// every one a cooked model (assets/models/), so the cook conforms each to
// the project's look (ArtShape, ArtTexels, ArtPalette). Copied from
// tools/art-samples/scene3d.ts (a game owns its own code); src/main.ts and passes its
// camera. Nothing moves: the scene is a still for the style's frame.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { DRESSING, MODELS } from "./models.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** Where the camera stands and what it looks at. */
export interface View {
  eye: { x: number; y: number; z: number };
  at: { x: number; y: number; z: number };
  fov: number;
  /** Turn of the sun about +Y, in radians (0: from the front left). */
  sun?: number;
  /** The way the sunlight travels, in place of `sun` (a set's own light: low and behind the scene). */
  sunDirection?: { x: number; y: number; z: number };
  /** The sun's strength (default 3) and the lantern's (default 6). */
  sunIntensity?: number;
  lanternIntensity?: number;
  /** Where the lantern's light hangs (default its glass on the family's lantern), and more warm lights (a set's hand lantern). */
  lanternAt?: { x: number; y: number; z: number };
  lights?: { x: number; y: number; z: number; intensity: number; distance: number }[];
  /** A procedural sky of these colours with its sun on sunDirection, in place of the built-in one (a set's dusk). */
  sky?: { zenith: { r: number; g: number; b: number }; horizon: { r: number; g: number; b: number }; sunColor: { r: number; g: number; b: number }; brightness?: number };
  /** The ground model, when the family briefs its own (default the textured glb). */
  ground?: string;
}

function model(world: World, path: string, x: number, z: number, turn = 0, y = 0, name = path.slice(7, -4)) {
  const asset = must(og.load.gltf(world, path));
  must(world.instantiate(asset, { name, position: { x, y, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) }));
}

export function build(world: World, view: View) {
  model(world, view.ground ?? MODELS.ground!, 0, 0);
  model(world, MODELS.far!, 0, -22);
  model(world, MODELS.tree!, -3.2, -1.6, 0.4);
  model(world, MODELS.lantern!, 2.9, -1.2, -0.3);
  model(world, MODELS.crate!, 1.3, 0.2, 0.5);
  model(world, MODELS.hero!, -0.6, 0.9, 0.35);
  DRESSING.forEach((d, i) => model(world, d.path, d.x, d.z, d.turn ?? 0, d.y ?? 0, `dressing-${i}`));

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, view.sunDirection ?? { x: 0.5 * Math.cos(view.sun ?? 0), y: -0.65, z: -0.55 + 0.3 * Math.sin(view.sun ?? 0) }) }));
  world.insert(sun, og.DirectionalLight, { intensity: view.sunIntensity ?? 3, castShadow: true });
  const glow = must(world.spawn({ name: "lantern-light", position: view.lanternAt ?? { x: 3.32, y: 2.3, z: -1.2 } }));
  world.insert(glow, og.PointLight, { color: { r: 1, g: 0.62, b: 0.3 }, intensity: view.lanternIntensity ?? 6, distance: 6 });
  for (const l of view.lights ?? []) {
    const e = must(world.spawn({ name: "warm-light", position: { x: l.x, y: l.y, z: l.z } }));
    world.insert(e, og.PointLight, { color: { r: 1, g: 0.6, b: 0.28 }, intensity: l.intensity, distance: l.distance });
  }

  const cam = must(world.resource(og.Camera));
  cam.position = view.eye;
  cam.quaternion = og.transform.lookAt(view.eye, view.at);
  cam.fov = view.fov;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
  if (view.sky && view.sunDirection) {
    const d = view.sunDirection;
    const sky = must(world.resource(og.Sky));
    sky.enabled = true;
    sky.zenith = view.sky.zenith;
    sky.horizon = view.sky.horizon;
    sky.sunColor = view.sky.sunColor;
    sky.sunDirection = { x: -d.x, y: -d.y, z: -d.z };
    sky.brightness = view.sky.brightness ?? 1.1;
  }
}
