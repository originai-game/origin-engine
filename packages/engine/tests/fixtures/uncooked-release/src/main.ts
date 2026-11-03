// See project.ogt: a small image and grid loaded at tick 1 and drawn for
// good (the warm-up), then the measured ones loaded at tick 20, drawn once
// ready, released at tick 50.
import { og, defineResource, defineSystem, f64, u32, type Asset, type World } from "@origingame/engine";
import { encodePng } from "../../../../src/node/png.ts";
import { gridGlb } from "../../uncooked/glb.ts";

const Loads = defineResource("Loads", {
  warmPicture: f64(),
  warmGrid: f64(),
  warmShown: u32(),
  picture: f64(),
  grid: f64(),
  shown: u32(),
  released: u32(),
});

const size = 1024;
const picture = encodePng({ width: size, height: size, rgba: new Uint8Array(size * size * 4).map((_, i) => (i * 7) & 0xff) });
const warmPicture = encodePng({ width: 4, height: 4, rgba: new Uint8Array(4 * 4 * 4).map((_, i) => (i * 7) & 0xff) });

/** A sprite of `picture` and a model of `grid`, once both are ready; the tick they were shown, or 0. */
function show(world: World, picture: Asset, grid: Asset, x: number, scale: number): number {
  if (og.load.status(world, picture) !== "ready" || og.load.status(world, grid) !== "ready") return 0;
  const info = og.load.imageInfo(world, picture);
  const s = world.spawn({ name: "wall", position: { x: x - 1.2, y: 1, z: 0 } });
  if (info.ok && s.ok) world.insert(s.value, og.Sprite, { texture: info.value.texture, width: 1.6 * scale, height: 1.6 * scale });
  const m = world.spawn({ name: "field", position: { x: x + 1.2, y: 0, z: 0 }, scale: { x: scale, y: scale, z: scale } });
  if (m.ok) world.insert(m.value, og.Model, { asset: grid });
  return world.tick;
}

const Cycle = defineSystem("Cycle", { writes: [Loads, og.Transform, og.Sprite, og.Model] }, (_q, _t, world) => {
  const l = world.resource(Loads);
  if (!l.ok) return;
  // The warm-up: the same kinds of load, drawn before the first dump and
  // never released, so what the renderer makes once on first use (the
  // pipelines a sprite and a lit model draw with, made when a pass first
  // needs them) is in every dump and the dumps differ by the loads alone.
  if (world.tick === 1) {
    const p = og.load.imageData(world, "drawings/warm.png", warmPicture, {});
    const g = og.load.gltfData(world, "creations/warm.glb", gridGlb(2), {});
    if (!p.ok || !g.ok) throw new Error(p.ok ? (g.ok ? "" : g.reason) : p.reason);
    l.value.warmPicture = p.value;
    l.value.warmGrid = g.value;
  }
  if (world.tick > 1 && !l.value.warmShown) {
    l.value.warmShown = show(world, l.value.warmPicture as Asset, l.value.warmGrid as Asset, 0, 0.25);
  }
  // The first dump is at tick 15 (uncooked-release.test.ts): drawn by then.
  if (world.tick === 13 && !l.value.warmShown) throw new Error("the warm-up loads were not drawn before the first dump");
  if (world.tick === 20) {
    const p = og.load.imageData(world, "drawings/wall.png", picture, {});
    const g = og.load.gltfData(world, "creations/field.glb", gridGlb(256), {});
    if (!p.ok || !g.ok) throw new Error(p.ok ? (g.ok ? "" : g.reason) : p.reason);
    l.value.picture = p.value;
    l.value.grid = g.value;
  }
  if (world.tick > 20 && !l.value.shown) l.value.shown = show(world, l.value.picture as Asset, l.value.grid as Asset, 0, 1);
  if (world.tick === 50) {
    if (!l.value.shown) throw new Error("the measured loads were not drawn before their release");
    for (const a of [l.value.picture as Asset, l.value.grid as Asset]) {
      const r = og.load.release(world, a);
      if (!r.ok) throw new Error(r.reason);
    }
    l.value.released = world.tick;
  }
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Cycle],
  resources: [Loads],
  start(world: World) {
    const cam = world.resource(og.Camera);
    if (cam.ok) {
      cam.value.position = { x: 0, y: 2, z: 4 };
      cam.value.quaternion = og.transform.lookAt({ x: 0, y: 2, z: 4 }, { x: 0, y: 0.5, z: 0 });
    }
  },
});
