// The run's state (the Game resource) as a system sees it.
//
// FINDINGS #19 is fixed: a resource handle's field read and write are
// typed-array accesses of the record (no JSON, whatever its size), so the
// systems use the live handle directly. These two helpers were the
// workaround (a plain copy, written back field by field); they stay as the
// handle itself and a no-op so the systems' call sites read as before.

import type { Value, World } from "@origingame/engine";
import { Game } from "./components.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;

/** The Game resource's live handle. */
export function begin(world: World): GameV {
  return must(world.resource(Game)) as GameV;
}

/** Nothing to write back: the handle's writes reach the engine by themselves (FINDINGS #19, fixed). */
export function flush(_g: GameV): void {}
