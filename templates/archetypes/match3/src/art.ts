// The art: assets/sprites, written by tools/make-assets.py and loaded once in
// `start`. The gems are one flipbook sheet (gems.ogt over gems.png): clip 2c
// is colour c's idle loop (a long hold, then a glint), clip 2c + 1 its clear
// burst (once). The board is a nine-sliced panel with a hand-written 7 x 7
// tilemap of checker cells on it; the selection frame and the backdrop are
// plain textures.

import { og } from "@origingame/engine";
import type { FlipbookSheet, Texture, TilemapAsset, World } from "@origingame/engine";

/** Gem colours, by Piece.colour. Six cuts, so colour is never the only cue. */
export const GEMS = ["ruby", "sky", "leaf", "sun", "plum", "coral"] as const;

/** The idle clip of a colour (an index into the sheet's clips). */
export const idleClip = (colour: number): number => colour * 2;
/** The clear burst of a colour. */
export const clearClip = (colour: number): number => colour * 2 + 1;

/** Seconds of one idle cycle (gems.ogt: 2.4 s of hold, then 0.16 s of glint). */
export const IDLE_CYCLE = 2.56;

/** The panel's nine-slice insets in texture pixels (tools/make-assets.py PANEL_INSET). */
export const PANEL_INSET = 32;

export interface Art {
  readonly gems: FlipbookSheet;
  readonly gemTexture: Texture;
  readonly panel: Texture;
  readonly backdrop: Texture;
  readonly ring: Texture;
  readonly board: TilemapAsset;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason} (run tools/make-assets.py and og cook)`);
  return r.value;
}

export function loadArt(world: World): Art {
  const sheet = must(og.load.flipbook(world, "sprites/gems.ogt"), "sprites/gems.ogt");
  GEMS.forEach((name, c) => {
    if (sheet.clips[idleClip(c)]?.name !== `idle_${name}` || sheet.clips[clearClip(c)]?.name !== `clear_${name}`) {
      throw new Error(`sprites/gems.ogt does not list idle_${name} and clear_${name} as clips ${idleClip(c)} and ${clearClip(c)}`);
    }
  });
  return {
    gems: sheet.sheet,
    gemTexture: sheet.texture,
    panel: must(og.load.texture(world, "sprites/panel.png"), "sprites/panel.png"),
    backdrop: must(og.load.texture(world, "sprites/backdrop.png"), "sprites/backdrop.png"),
    ring: must(og.load.texture(world, "sprites/ring.png"), "sprites/ring.png"),
    board: must(og.load.tilemap(world, "sprites/board.ogt"), "sprites/board.ogt").map,
  };
}
