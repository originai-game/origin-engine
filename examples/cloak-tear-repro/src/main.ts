import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { readonly ok: true; readonly value: T } | { readonly ok: false; readonly reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const VIEWS = [0, 90, 180, 270];
let view = 0;
function start(world: World): void {
  const sun = must(world.spawn({ name: "sun" }));
  must(world.insert(sun, og.DirectionalLight, { intensity: 3 }));
  must(world.resource(og.RenderSettings)).sky = true;
  const anim = must(og.load.animation(world, "models/traveller.glb"));
  const body = must(world.instantiate(must(og.load.gltf(world, "models/traveller.glb")), { name: "t", position: { x: 0, y: 0, z: 0 } }));
  must(world.insert(body, og.Skeleton, { asset: anim.skeleton, rootMotion: "off" }));
  must(world.insert(body, og.Garment, { mesh: 1, maxDistance: 0.25, backstop: 0.0, bending: 0.3, iterations: 6 }));
  Object.assign(must(world.resource(og.Wind)), { strength: 0.5, direction: { x: 0.3, y: 0, z: 1 }, speed: 6, gust: 0.8 });
  place(world);
}
function place(world: World): void {
  const cam = must(world.resource(og.Camera));
  const a = (VIEWS[view] ?? 0) * Math.PI / 180;
  cam.position = { x: Math.sin(a) * 3.2, y: 1.0, z: Math.cos(a) * 3.2 };
  cam.quaternion = { x: 0, y: Math.sin(a / 2), z: 0, w: Math.cos(a / 2) };
  cam.fov = 40;
}
export default og.game({ tick: 60, seed: 1, start, commands: { orbit: (world: World, args: readonly string[]) => { view = Number(args[0]); place(world); } } });
