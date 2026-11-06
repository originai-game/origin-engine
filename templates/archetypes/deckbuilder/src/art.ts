// The sprite sheet's clips (assets/sprites/units.ogt, written by
// tools/make-assets.py), by name; systems pick clips by index, so the
// indices are fixed here and checked against the sheet in `start`.

import { og } from "@origingame/engine";
import type { FlipbookInfo } from "@origingame/engine";

const UNITS = ["warden", "hunter", "imp", "hound", "slime", "knight", "king"] as const;
export const CLIPS = [...UNITS, ...UNITS.map((u) => `${u}-token`)] as const;

export function clipOf(name: string): number {
  const i = (CLIPS as readonly string[]).indexOf(name);
  if (i < 0) throw new Error(`the sheet has no clip '${name}'`);
  return i;
}

export function checkClips(info: FlipbookInfo): void {
  CLIPS.forEach((name, i) => {
    if (og.anim.clip(info, name) !== i) throw new Error(`assets/sprites/units.ogt: clip '${name}' must be clip ${i} (src/art.ts reads clips by position)`);
  });
}
