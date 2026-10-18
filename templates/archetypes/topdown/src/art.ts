// The flipbook sheets (assets/sprites/*.ogt, written by tools/make-assets.py)
// and their clips, looked up by name once in `start`.

import { og } from "@origingame/engine";
import type { FlipbookInfo, World } from "@origingame/engine";
import { Facing } from "./components.ts";

export interface Sheets {
  readonly hero: FlipbookInfo;
  readonly slime: FlipbookInfo;
  readonly items: FlipbookInfo;
}

export function loadSheets(world: World): Sheets {
  const load = (path: string): FlipbookInfo => {
    const r = og.load.flipbook(world, path);
    if (!r.ok) throw new Error(`${path}: ${r.reason}`);
    return r.value;
  };
  return { hero: load("sprites/hero.ogt"), slime: load("sprites/slime.ogt"), items: load("sprites/items.ogt") };
}

/** A clip's index by name; a missing clip is an authoring error. */
export function clip(info: FlipbookInfo, name: string): number {
  const i = og.anim.clip(info, name);
  if (i < 0) throw new Error(`the sheet has no clip '${name}' (its clips: ${info.clips.map((c) => c.name).join(", ")})`);
  return i;
}

/**
 * The hero's clips are authored in this order (assets/sprites/hero.ogt):
 * idle, walk and attack, each down, up, side. So a clip is its action's
 * base plus the facing's row; left is the side row drawn with flipX.
 */
export const HeroClip = { Idle: 0, Walk: 3, Attack: 6 } as const;

export function heroClip(action: number, facing: number): number {
  const row = facing === Facing.Down ? 0 : facing === Facing.Up ? 1 : 2;
  return action + row;
}

/** Checks the hero sheet's order against HeroClip once, in `start`. */
export function checkHeroClips(info: FlipbookInfo): void {
  const names = ["idle_down", "idle_up", "idle_side", "walk_down", "walk_up", "walk_side", "attack_down", "attack_up", "attack_side"];
  names.forEach((n, i) => {
    if (clip(info, n) !== i) throw new Error(`assets/sprites/hero.ogt: clip '${n}' must be clip ${i} (src/art.ts reads clips by position)`);
  });
}
