// The pixel side-on family's sample scene (docs/art.md §2.6): a hero, a
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
    // The ground line is y = 0, a fifth of the view up from its bottom.
    const view = { h: VIEW_HEIGHT, w: (VIEW_HEIGHT * 16) / 9 };
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = view.h;
    camera.position = { x: 0, y: view.h * 0.3, z: 50 };
    camera.background = { r: 0.02, g: 0.02, b: 0.05 };

    // The far layer fills the view behind everything.
    const back = CELLS["art/backdrop.png"]!;
    sprite(world, "art/backdrop.png", 0, view.h * 0.3, -20, 0.5, Math.max(view.w / back.w, view.h / back.h));
    // The ground: tiles whose top edge (the grass) is y = 0, down past the view's bottom.
    const tile = CELLS["art/ground.png"]!;
    const across = Math.ceil(view.w / tile.w / 2) + 1;
    for (let row = 0; row * tile.h < view.h * 0.25; row++) {
      for (let i = -across; i <= across; i++) sprite(world, "art/ground.png", i * tile.w, -row * tile.h, -5, 1);
    }
    // The beats, spread across the view (x in view widths).
    sprite(world, "art/tree.png", -0.27 * view.w, 0, 0);
    sprite(world, "art/lantern.png", 0.2 * view.w, 0, 0);
    sprite(world, "art/crate.png", 0.08 * view.w, 0, 1);
    sprite(world, "art/hero.png", -0.06 * view.w, 0, 2);
  },
});
