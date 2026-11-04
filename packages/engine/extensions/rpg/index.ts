// The rpg kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { rpg } from "@origingame/engine/extensions/rpg/index.ts";   // bundled
//   import { rpg } from "../assets/ui-kits/rpg/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { conversation } from "./src/dialogue.ts";
import { castBar, partyFrames, raidFrames, unitFrame } from "./src/frames.ts";
import { inventoryGrid, paperdoll } from "./src/inventory.ts";
import { questLog, questTracker } from "./src/quests.ts";
import { turnOrder } from "./src/turns.ts";

export type * from "./types.ts";

export const rpg = registerUiExtension("rpg", {
  inventoryGrid,
  paperdoll,
  questLog,
  questTracker,
  conversation,
  unitFrame,
  partyFrames,
  raidFrames,
  castBar,
  turnOrder,
});
