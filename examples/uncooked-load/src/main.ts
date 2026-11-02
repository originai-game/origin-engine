// Content made at run time and loaded uncooked: a vase written as a .glb
// and a drawing written as a PNG (src/make.ts), loaded from their bytes and
// drawn once ready.
import { og, defineResource, defineSystem, f64, u32, type Asset, type World } from "@origingame/engine";
import { image, smooth, vase } from "./make.ts";

const Made = defineResource("Made", { vase: f64(), drawing: f64(), shown: u32() });

/** The glaze: bands of blue and cream with a darker rim, u around the vase and v down it. */
const glaze = image(256, 256, (u, v) => {
  const band = 0.5 + 0.5 * Math.sin(v * 38 + Math.sin(u * Math.PI * 12) * 0.9);
  const rim = v < 0.08 ? 0.45 : 1;
  return [(0.16 + 0.78 * band) * rim, (0.3 + 0.62 * band) * rim, (0.62 + 0.3 * band) * rim, 1];
});

/** A drawing: a sun with rays, on a clear background. */
const drawing = image(128, 128, (u, v) => {
  const x = u - 0.5;
  const y = v - 0.5;
  const r = Math.hypot(x, y);
  const ray = 0.5 + 0.5 * Math.cos(Math.atan2(y, x) * 12);
  if (r < 0.22) return [1, 0.78 - r, 0.2, 1];
  if (r < 0.3 + 0.16 * ray && ray > 0.55) return [1, 0.6, 0.15, 1];
  return [0, 0, 0, 0];
});

// The vase's profile (radius, height), bottom to top, smoothed through a spline.
const profile = smooth([
  [0, 0], [0.32, 0], [0.4, 0.08], [0.52, 0.32], [0.55, 0.5], [0.46, 0.78], [0.26, 1.02], [0.2, 1.2], [0.25, 1.32], [0.3, 1.38],
], 6);

// Spawned once both are in place: from a system, a model is spawned with og.Model (world.instantiate is for outside systems).
const Show = defineSystem("Show", { writes: [Made, og.Transform, og.Model, og.Sprite] }, (_q, _t, world) => {
  const m = world.resource(Made);
  if (!m.ok || m.value.shown) return;
  const [v, d] = [m.value.vase as Asset, m.value.drawing as Asset];
  for (const a of [v, d]) {
    if (og.load.status(world, a) === "failed") throw new Error(og.load.error(world, a) ?? "failed");
  }
  if (og.load.status(world, v) !== "ready" || og.load.status(world, d) !== "ready") return;
  m.value.shown = world.tick + 1;
  const e = world.spawn({ name: "vase", position: { x: -0.7, y: 0, z: 0 } });
  if (e.ok) world.insert(e.value, og.Model, { asset: v });
  const info = og.load.imageInfo(world, d);
  const s = world.spawn({ name: "drawing", position: { x: 0.9, y: 0.85, z: 0 } });
  if (info.ok && s.ok) world.insert(s.value, og.Sprite, { texture: info.value.texture, width: 1.1, height: 1.1 });
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Show],
  resources: [Made],
  start(world: World) {
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      const eye = { x: 0, y: 1.3, z: 3.6 };
      cam.value.position = eye;
      cam.value.quaternion = og.transform.lookAt(eye, { x: 0, y: 0.6, z: 0 });
      cam.value.fov = 40;
      cam.value.background = { r: 0.03, g: 0.035, b: 0.05 };
    }
    // Bytes from code here; from a player they would arrive by upload, the platform store or a message.
    const budget = { bytes: 4 * 1024 * 1024, triangles: 20_000, textureSize: 1024 };
    const v = og.load.gltfData(world, "made/vase.glb", vase(profile, 64, glaze), { budget });
    const d = og.load.imageData(world, "made/sun.png", drawing, { budget });
    if (!v.ok) throw new Error(v.reason);
    if (!d.ok) throw new Error(d.reason);
    const m = world.resource(Made);
    if (m.ok) [m.value.vase, m.value.drawing] = [v.value, d.value];
  },
});
