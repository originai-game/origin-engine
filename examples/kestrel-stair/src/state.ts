// The run's state (the Game resource) as a system sees it: read once into a
// plain copy at the start of the system, changed there, and written back at
// its end field by field, only the fields that changed.
//
// A resource handle's write is one engine call that costs in proportion to
// the whole resource (about 0.11 ms for Game's 130 fields, 0.01 ms for
// Prefs' 11), and the next read after a write fetches the whole record
// again (world.resource(Game) about 0.23 ms), so a game that keeps its run
// in one resource and writes it the way og-core shows spends most of a
// tick there (FINDINGS #14). Commands and setup keep using the live handle.

import type { Value, World } from "@origingame/engine";
import { Game } from "./components.ts";
import { must } from "./world.ts";

type GameV = Value<typeof Game>;

const open = new WeakMap<object, { handle: GameV; before: Record<string, unknown> }>();

/** The Game resource as a plain copy for this system call. */
export function begin(world: World): GameV {
  const handle = must(world.resource(Game));
  const copy = { ...handle } as GameV;
  open.set(copy, { handle, before: { ...copy } });
  return copy;
}

/** Writes back what changed in `g` since `begin` (or the last flush); a live handle passes through. */
export function flush(g: GameV): void {
  const t = open.get(g);
  if (!t) return;
  const now = g as unknown as Record<string, unknown>;
  const handle = t.handle as unknown as Record<string, unknown>;
  for (const k of Object.keys(t.before)) {
    if (now[k] !== t.before[k]) {
      handle[k] = now[k];
      t.before[k] = now[k];
    }
  }
}
