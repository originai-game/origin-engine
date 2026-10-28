// The painted 2D family's sample scene (docs/art.md §2.6): a hero, a
// stepped tree, a crate, a lantern, the ground and a far layer, each a
// sprite whose texture the cook conforms to project.ogt's look. Sizes come
// from src/cells.ts (the art brief's cells, tools/art-samples/prepare.ts),
// so every sprite shares the look's pixel size. No game logic: the scene is
// a still for the style's frame (README.md).
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { CELLS, VIEW_HEIGHT } from "./cells.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** A sprite of `path` at its cell's size, its anchor at (x, y): the foot by default. */
let count = 0;
function sprite(world: World, path: string, x: number, y: number, layer: number, anchorY = 0, scale = 1) {
  const cell = CELLS[path]!;
  const texture = must(og.load.texture(world, path));
  const e = must(world.spawn({ name: `${path.slice(4, -4)}-${count++}`, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { texture, width: cell.w * scale, height: cell.h * scale, anchor: { x: 0.5, y: anchorY }, layer }));
  return cell;
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    // The ground is the far layer's own floor band, cut from it
    // (tools/art-samples/slice.py): one sprite across the view whose top
    // edge is the ground line y = 0, the far layer standing on its bottom.
    const view = { h: VIEW_HEIGHT, w: (VIEW_HEIGHT * 16) / 9 };
    const ground = CELLS["art/ground.jpg"]!;
    const back = CELLS["art/backdrop.jpg"]!;
    const floor = (ground.h * view.w) / ground.w;
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = view.h;
    camera.position = { x: 0, y: view.h / 2 - floor, z: 50 };
    camera.background = { r: 0.02, g: 0.02, b: 0.05 };

    sprite(world, "art/backdrop.jpg", 0, -floor, -20, 0, Math.max(view.w / back.w, view.h / back.h));
    sprite(world, "art/ground.jpg", 0, 0, -5, 1, view.w / ground.w);
    sprite(world, "art/tree.png", -0.28 * view.w, 0, 0);
    sprite(world, "art/lantern.png", 0.22 * view.w, 0, 0);
    sprite(world, "art/crate.png", 0.08 * view.w, 0, 1);
    sprite(world, "art/hero.png", -0.08 * view.w, 0, 2);
  },
});
