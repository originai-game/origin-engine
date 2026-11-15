// The 2D looks' preview scene (templates/looks/preview/README.md): a far
// layer, a strip of ground, a tree, the hero, a crate and a lantern, each a
// sprite of assets/art/ (the look's authored art, copied in by render.ts) at
// the size src/sprites.ts gives it. The cook conforms each texture to the
// look's art contract; the look's frame does the rest.
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { ART, VIEW_HEIGHT } from "./sprites.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function sprite(world: World, beat: string, x: number, y: number, layer: number, w = ART[beat]!.width, h = ART[beat]!.height) {
  const texture = must(og.load.texture(world, `art/${ART[beat]!.file}`));
  const e = must(world.spawn({ name: `${beat}@${x},${y}`, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { texture, width: w, height: h, anchor: { x: 0.5, y: 0 }, layer }));
}

export default og.game({
  tick: 60,
  seed: 1,
  look: "looks/main.ogt",
  start(world) {
    const view = { h: VIEW_HEIGHT, w: (VIEW_HEIGHT * 16) / 9 };
    const cam = must(world.resource(og.Camera));
    cam.orthographic = true;
    cam.height = view.h;
    // The ground line is y = 0, a fifth of the view up from its bottom.
    cam.position = { x: 0, y: view.h * 0.3, z: 50 };
    cam.background = { r: 0.02, g: 0.02, b: 0.05 };
    const back = ART.backdrop!;
    const fill = Math.max(view.w / back.width, view.h / back.height);
    sprite(world, "backdrop", 0, view.h * 0.8 - back.height * fill, -20, back.width * fill, back.height * fill);
    const tile = ART.ground!;
    const across = Math.ceil(view.w / tile.width / 2) + 1;
    // Tiles whose top edge (the grass) is y = 0, down past the view's bottom.
    for (let row = 1; (row - 1) * tile.height < view.h * 0.2; row++) {
      for (let i = -across; i <= across; i++) sprite(world, "ground", i * tile.width, -row * tile.height, -5);
    }
    sprite(world, "tree", -0.27 * view.w, 0, 0);
    sprite(world, "lantern", 0.2 * view.w, 0, 0);
    sprite(world, "crate", 0.08 * view.w, 0, 1);
    sprite(world, "hero", -0.06 * view.w, 0, 2);
  },
});
