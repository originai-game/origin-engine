// The 3D sample scenes' shared layout (docs/art.md §2.6): a hero, a stepped
// tree, a crate, a lantern on the ground, and a far ridge with a village,
// every one a cooked model (assets/models/), so the cook conforms each to
// the project's look (ArtShape, ArtTexels, ArtPalette). Each family's
// src/main.ts copies this file (a game owns its own code) and passes its
// camera. Nothing moves: the scene is a still for the style's frame.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

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
}

function model(world: World, path: string, x: number, z: number, turn = 0) {
  const asset = must(og.load.gltf(world, path));
  must(world.instantiate(asset, { name: path.slice(7, -4), position: { x, y: 0, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) }));
}

export function build(world: World, view: View) {
  model(world, "models/ground.glb", 0, 0);
  model(world, "models/hills.ogt", 0, -22);
  model(world, "models/tree.ogt", -3.2, -1.6, 0.4);
  model(world, "models/lantern.ogt", 2.9, -1.2, -0.3);
  model(world, "models/crate.glb", 1.3, 0.2, 0.5);
  model(world, "models/hero.ogt", -0.6, 0.9, 0.35);

  const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.5 * Math.cos(view.sun ?? 0), y: -0.65, z: -0.55 + 0.3 * Math.sin(view.sun ?? 0) }) }));
  world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true });
  const glow = must(world.spawn({ name: "lantern-light", position: { x: 3.32, y: 2.3, z: -1.2 } }));
  world.insert(glow, og.PointLight, { color: { r: 1, g: 0.62, b: 0.3 }, intensity: 6, distance: 6 });

  const cam = must(world.resource(og.Camera));
  cam.position = view.eye;
  cam.quaternion = og.transform.lookAt(view.eye, view.at);
  cam.fov = view.fov;
  const settings = must(world.resource(og.RenderSettings));
  settings.sky = true;
}
