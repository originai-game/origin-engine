// The shooter kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { shooter } from "@origingame/engine/extensions/shooter/index.ts";   // bundled
//   import { shooter } from "../assets/ui-kits/shooter/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { ammoCounter, crosshair, damageDirection, killfeed, lowHealthVignette } from "./src/combat.ts";
import { loadout } from "./src/loadout.ts";
import { scoreboard } from "./src/scoreboard.ts";

export type * from "./types.ts";

export const shooter = registerUiExtension("shooter", {
  crosshair,
  ammoCounter,
  killfeed,
  damageDirection,
  lowHealthVignette,
  scoreboard,
  loadout,
});
