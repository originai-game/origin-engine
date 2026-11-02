// See project.ogt: load at tick 10, draw once ready, release at tick 40.
import { og, defineResource, defineSystem, f64, u32, type Asset, type World } from "@origingame/engine";
import { encodePng } from "../../../../src/node/png.ts";
import { gridGlb } from "../../uncooked/glb.ts";

const Loads = defineResource("Loads", { picture: f64(), grid: f64(), shown: u32(), released: u32() });

const size = 1024;
const picture = encodePng({ width: size, height: size, rgba: new Uint8Array(size * size * 4).map((_, i) => (i * 7) & 0xff) });

const Cycle = defineSystem("Cycle", { writes: [Loads, og.Transform, og.Sprite, og.Model] }, (_q, _t, world) => {
  const l = world.resource(Loads);
  if (!l.ok) return;
  if (world.tick === 10) {
    const p = og.load.imageData(world, "drawings/wall.png", picture, {});
    const g = og.load.gltfData(world, "creations/field.glb", gridGlb(256), {});
    if (!p.ok || !g.ok) throw new Error(p.ok ? (g.ok ? "" : g.reason) : p.reason);
    l.value.picture = p.value;
    l.value.grid = g.value;
  }
  const [p, g] = [l.value.picture as Asset, l.value.grid as Asset];
  if (world.tick > 10 && !l.value.shown && og.load.status(world, p) === "ready" && og.load.status(world, g) === "ready") {
    l.value.shown = world.tick;
    const info = og.load.imageInfo(world, p);
    const s = world.spawn({ name: "wall", position: { x: -1.2, y: 1, z: 0 } });
    if (info.ok && s.ok) world.insert(s.value, og.Sprite, { texture: info.value.texture, width: 1.6, height: 1.6 });
    const m = world.spawn({ name: "field", position: { x: 1.2, y: 0, z: 0 } });
    if (m.ok) world.insert(m.value, og.Model, { asset: g });
  }
  if (world.tick === 40) {
    for (const a of [p, g]) {
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
