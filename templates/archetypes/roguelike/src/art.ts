// The sprite sheet's clips (assets/sprites/dive.ogt, written by
// tools/make-assets.py), by name; systems pick clips by index, so the
// indices are fixed here and checked against the sheet in `start`.

import { og } from "@origingame/engine";
import type { FlipbookInfo } from "@origingame/engine";

export const CLIPS = ["hero", "rat", "skeleton", "guardian", "potion", "blade", "ward", "oil", "coin", "relic"] as const;

export function clipOf(name: string): number {
  const i = (CLIPS as readonly string[]).indexOf(name);
  if (i < 0) throw new Error(`the sheet has no clip '${name}'`);
  return i;
}

export function checkClips(info: FlipbookInfo): void {
  CLIPS.forEach((name, i) => {
    if (og.anim.clip(info, name) !== i) throw new Error(`assets/sprites/dive.ogt: clip '${name}' must be clip ${i} (src/art.ts reads clips by position)`);
  });
}

/** Tiles of tiles/tomb.ogt the floor is carved with. */
export const TILE = { floor: 16, stairs: 19, pedestal: 20, unseen: 21, remembered: 22 } as const;
