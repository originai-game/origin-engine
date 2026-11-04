// The racing kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { racing } from "@origingame/engine/extensions/racing/index.ts";   // bundled
//   import { racing } from "../assets/ui-kits/racing/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { driftScore, itemSlot } from "./src/arcade.ts";
import { garage } from "./src/garage.ts";
import { lapTimer, raceBanner, rearView, standingsTower } from "./src/racing.ts";
import { speedometer } from "./src/speedometer.ts";

export type * from "./types.ts";

export const racing = registerUiExtension("racing", {
  speedometer,
  standingsTower,
  lapTimer,
  rearView,
  raceBanner,
  itemSlot,
  driftScore,
  garage,
});
