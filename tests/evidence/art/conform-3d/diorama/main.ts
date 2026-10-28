// The fx-showcase looks diorama (templates/fx-showcase/src/looks.ts) with
// its tree, crate and five spheres as cooked models, so the project's look
// (project.ogt's `look`) conforms them; the ground, wall, post and lantern
// are the diorama's runtime geometry. Same layout, camera and sun.
import { og } from "@origingame/engine";
import type { Material, World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world: World) {
    const box = must(og.render.boxGeometry(world, {}));
    const sphere = must(og.render.sphereGeometry(world, { radius: 0.5 }));
    const mat = (d: Parameters<typeof og.render.standardMaterial>[1]): Material => must(og.render.standardMaterial(world, d));
    const put = (name: string, geometry: typeof box, material: Material, x: number, y: number, z: number, sx: number, sy: number, sz: number) => {
      const e = must(world.spawn({ name, position: { x, y, z } }));
      world.set(e, og.Transform, { scale: { x: sx, y: sy, z: sz } });
      world.insert(e, og.Mesh, { geometry, material, castShadow: true, receiveShadow: true });
    };
    const model = (name: string, path: string, x: number, y: number, z: number, turn = 0) => {
      const asset = must(og.load.gltf(world, path));
      must(world.instantiate(asset, { name, position: { x, y, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) }));
    };
    put("ground", box, mat({ map: must(og.load.texture(world, "textures/tile.png")), roughness: 0.9 }), 0, -0.1, 0, 40, 0.2, 40);
    ["red", "gold", "blue", "chrome", "green"].forEach((c, i) => model(`ball-${c}`, `models/ball-${c}.ogt`, -3.2 + i * 1.6, 0.7, 0.6));
    model("crate", "models/crate.glb", 2.8, 0.6, -2.2, 0.6);
    model("tree", "models/tree.ogt", -3.2, 0, -3);
    put("lantern", sphere, mat({ color: { r: 1, g: 0.6, b: 0.2 }, emissive: { r: 1, g: 0.55, b: 0.15 }, emissiveIntensity: 6, roughness: 0.6 }), 1.2, 2.2, -1.4, 0.5, 0.5, 0.5);
    put("post", box, mat({ color: { r: 0.2, g: 0.2, b: 0.22 }, roughness: 0.6 }), 1.2, 1, -1.4, 0.12, 2, 0.12);
    put("wall", box, mat({ color: { r: 0.6, g: 0.62, b: 0.7 }, roughness: 0.9 }), 0, 2, -14, 30, 4, 0.5);
    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.45, y: -0.7, z: -0.5 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3 });
    const cam = must(world.resource(og.Camera));
    const eye = { x: 0.4, y: 3.2, z: 8.5 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 1.1, z: -1 });
    cam.exposure = 0.8;
    const settings = must(world.resource(og.RenderSettings));
    settings.sky = true;
    settings.environmentIntensity = 0.8;
  },
});
