// The 3D looks' preview scene (templates/looks/preview/README.md): a
// traveller with a lamp on a path, a pine, an oak, boulders and a bush, a
// crate and a street lantern on a meadow that rolls away to hills and a
// village. Every model is cooked, so the cook conforms each to the look's
// art contract (ArtShape, ArtTexels, ArtPalette); the look's frame does the
// rest. Round forms and rolling land on purpose: a look's facets, bands,
// strokes or material must show on them. A look that stands pixel sprites in
// its set (preview.json's sprites) draws those beats as sprites; a look whose
// models bring their own ground (content/models/field.ts) stands the scene on it.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { GROUND, SPRITES } from "./sprites.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function model(world: World, name: string, x: number, z: number, turn = 0) {
  const at = { name, position: { x, y: 0, z }, quaternion: og.transform.fromAxisAngle({ x: 0, y: 1, z: 0 }, turn) };
  const sprite = SPRITES[name];
  if (sprite) {
    const texture = must(og.load.texture(world, sprite.path));
    const e = must(world.spawn(at));
    must(world.insert(e, og.Sprite, { texture, width: sprite.width, height: sprite.height, anchor: { x: 0.5, y: 0 }, facing: "cameraY", dropShadow: 0.5 }));
    return;
  }
  const path = name === "ground" ? GROUND : name === "crate" ? "base/models/crate" : `base/models/${name}`;
  must(world.instantiate(must(og.load.gltf(world, path)), at));
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/main.ogt",
  start(world) {
    model(world, "ground", 0, 0);
    model(world, "hills", 0, -24);
    model(world, "tree", -3.4, -2.2, 0.4);
    model(world, "oak", 4.6, -4.2, 1.1);
    model(world, "rocks", -2.3, 0.9, 0.3);
    model(world, "lantern", 2.9, -1.4, -0.3);
    model(world, "crate", 1.2, 0.1, 0.5);
    model(world, "hero", -0.5, 0.9, 0.35);

    const sun = must(world.spawn({ name: "sun", quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: 0.55, y: -0.6, z: -0.5 }) }));
    world.insert(sun, og.DirectionalLight, { intensity: 3, castShadow: true });
    const glow = must(world.spawn({ name: "lantern-light", position: { x: 3.32, y: 2.3, z: -1.4 } }));
    world.insert(glow, og.PointLight, { color: { r: 1, g: 0.62, b: 0.3 }, intensity: 6, distance: 6 });

    const cam = must(world.resource(og.Camera));
    const eye = { x: 3.4, y: 4.6, z: 9.2 };
    cam.position = eye;
    cam.quaternion = og.transform.lookAt(eye, { x: 0, y: 0.9, z: -1.2 });
    cam.fov = 40;
    must(world.resource(og.RenderSettings)).sky = true;
  },
});
