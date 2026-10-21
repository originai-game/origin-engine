// Every word a player reads comes from assets/strings/<locale>.ogt.

import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";

export function say(world: World, key: string, args?: Record<string, string | number>): string {
  return og.i18n.t(world, key, args);
}
