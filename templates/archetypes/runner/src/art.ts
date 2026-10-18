// The art: pixel-art flipbook sheets and the city's tilemaps, all made by
// tools/make-assets.py and loaded once in `start` from the cooked bundle.
//
// - sprites/courier.ogt: the courier's clips (COURIER below, in sheet order),
// - sprites/props.ogt: crate, fence, moon and cloud, one frame each,
// - sprites/coin.ogt: the coin's turn,
// - sprites/sky.png: the sky's colour bands, stretched over the view,
// - maps/far.ogt, near.ogt, roofs.ogt: the three parallax strips (tilemaps
//   with `repeat x` over tiles/city.ogt).

import { og } from "@origingame/engine";
import type { FlipbookInfo, Texture, TilemapInfo, World } from "@origingame/engine";

/** The courier's clips, by index in sprites/courier.ogt (checked against the sheet in `loadArt`). */
export const COURIER = { idle: 0, run: 1, jump: 2, fall: 3, crash: 4 } as const;
/** The props' clips, by index in sprites/props.ogt. */
export const PROPS = { crate: 0, fence: 1, moon: 2, cloud: 3 } as const;

export interface Art {
  readonly courier: FlipbookInfo;
  readonly props: FlipbookInfo;
  readonly coin: FlipbookInfo;
  readonly sky: Texture;
  readonly far: TilemapInfo;
  readonly near: TilemapInfo;
  readonly roofs: TilemapInfo;
}

function ok<T>(r: { ok: true; value: T } | { ok: false; reason: string }, what: string): T {
  if (!r.ok) throw new Error(`${what}: ${r.reason} (run python3 tools/make-assets.py and og cook .)`);
  return r.value;
}

function expectClips(info: FlipbookInfo, clips: Readonly<Record<string, number>>, path: string): void {
  for (const [name, index] of Object.entries(clips)) {
    const found = og.anim.clip(info, name);
    if (found !== index) throw new Error(`${path}: clip ${name} is ${found}, the code expects ${index}`);
  }
}

/** Loads every sheet, texture and map; the cooked bundle has them ready at once. */
export function loadArt(world: World): Art {
  const courier = ok(og.load.flipbook(world, "sprites/courier.ogt"), "sprites/courier.ogt");
  const props = ok(og.load.flipbook(world, "sprites/props.ogt"), "sprites/props.ogt");
  const coin = ok(og.load.flipbook(world, "sprites/coin.ogt"), "sprites/coin.ogt");
  expectClips(courier, COURIER, "sprites/courier.ogt");
  expectClips(props, PROPS, "sprites/props.ogt");
  return {
    courier,
    props,
    coin,
    sky: ok(og.load.texture(world, "sprites/sky.png"), "sprites/sky.png"),
    far: ok(og.load.tilemap(world, "maps/far.ogt"), "maps/far.ogt"),
    near: ok(og.load.tilemap(world, "maps/near.ogt"), "maps/near.ogt"),
    roofs: ok(og.load.tilemap(world, "maps/roofs.ogt"), "maps/roofs.ogt"),
  };
}
