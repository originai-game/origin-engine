// Written by tools/art-samples/prepare.ts for the look "16bit-pixel": each sprite's size in metres
// (its brief's cell over ArtPixels.unit) and the view's height. Do not edit.
export const VIEW_HEIGHT = 14;
export const CELLS: Record<string, { w: number; h: number }> = {
  "art/hero.png": { w: 1.0625, h: 2 },
  "art/tree.png": { w: 3, h: 3 },
  "art/crate.png": { w: 2, h: 2 },
  "art/lantern.png": { w: 1.875, h: 3 },
  "art/ground.png": { w: 2, h: 1.625 },
  "art/backdrop.png": { w: 24.875, h: 14 },
};
