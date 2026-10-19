// The sprite sheet's clips (assets/sprites/garden.ogt, written by
// tools/make-assets.py), by name. Systems pick clips by index every tick,
// so the indices are fixed here and checked against the sheet in `start`.

import { og } from "@origingame/engine";
import type { FlipbookInfo } from "@origingame/engine";

/** The sheet's clips in the order the .ogt declares them. */
export const CLIPS = ["beetle", "moth", "snail", "lantern", "thorn", "thorn_shoot", "frost", "frost_shoot", "hedge", "seed", "flake", "cursor", "refused"] as const;
export type ClipName = (typeof CLIPS)[number];

/** A clip's index by name. */
export function clipOf(name: string): number {
  const i = (CLIPS as readonly string[]).indexOf(name);
  if (i < 0) throw new Error(`the sheet has no clip '${name}'`);
  return i;
}

/** Checks the loaded sheet against CLIPS once, in `start`. */
export function checkClips(info: FlipbookInfo): void {
  CLIPS.forEach((name, i) => {
    if (og.anim.clip(info, name) !== i) throw new Error(`assets/sprites/garden.ogt: clip '${name}' must be clip ${i} (src/art.ts reads clips by position)`);
  });
}
