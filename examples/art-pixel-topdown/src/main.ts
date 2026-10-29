// The pixel top-down family's sample scene (docs/art.md §2.6): a tile map
// (assets/maps/meadow.ogt on the tile set art/terrain.ogt: grass, a raised
// ledge with its cliff face and steps, an auto-tiled path and pond) with a
// hero, a stepped tree, a crate and a lantern standing on it, each a sprite
// whose texture the cook conforms to project.ogt's look, as the tile set's
// tiles are. Sprite sizes come from src/cells.ts (the art brief's cells,
// tools/art-samples/prepare.ts), so every sprite shares the look's pixel
// size; one tile is one metre. No game logic: the scene is a still for the
// style's frame (README.md).
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { CELLS, VIEW_HEIGHT } from "./cells.ts";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

/** An upright sprite of `path` at its cell's size, standing on its feet at (x, y); y-sorted with the others. */
let count = 0;
function sprite(world: World, path: string, x: number, y: number) {
  const cell = CELLS[path]!;
  const texture = must(og.load.texture(world, path));
  const e = must(world.spawn({ name: `${path.slice(4, -4)}-${count++}`, position: { x, y, z: 0 } }));
  must(world.insert(e, og.Sprite, { texture, width: cell.w, height: cell.h, anchor: { x: 0.5, y: 0 }, layer: 0, sortY: true }));
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = must(world.resource(og.Camera));
    camera.orthographic = true;
    camera.height = VIEW_HEIGHT;
    camera.position = { x: 0, y: 0, z: 50 };
    camera.background = { r: 0.02, g: 0.02, b: 0.05 };

    // The map's top-left corner, so its 20 x 12 tiles are centred on the view.
    const info = must(og.load.tilemap(world, "maps/meadow.ogt"));
    const left = -info.width / 2;
    const top = info.height / 2;
    const map = must(world.spawn({ name: "meadow", position: { x: left, y: top, z: 0 } }));
    must(world.insert(map, og.Tilemap, { map: info.map, collision: false }));
    /** Where a sprite stands in cell (cx, cy): x across the cell, `foot` down it (0 its top edge, 1 its bottom). */
    const at = (cx: number, cy: number, foot = 0.8) => [left + cx, top - cy - foot] as const;

    sprite(world, "art/tree.png", ...at(2, 5));
    sprite(world, "art/crate.png", ...at(9.5, 5));
    sprite(world, "art/lantern.png", ...at(12, 5, 0.6));
    sprite(world, "art/hero.png", ...at(7.5, 6.5));
  },
});
