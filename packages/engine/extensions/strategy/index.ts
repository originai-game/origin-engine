// The strategy kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { strategy } from "@origingame/engine/extensions/strategy/index.ts";   // bundled
//   import { strategy } from "../assets/ui-kits/strategy/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { rtsCommands } from "./src/commands.ts";
import { resourceBar } from "./src/resources.ts";
import { commandCard, dragSelect, productionQueue, rtsConsole, selectionPanel } from "./src/rts.ts";
import { wavePreview } from "./src/waves.ts";

export type * from "./types.ts";

export const strategy = registerUiExtension("strategy", {
  productionQueue,
  selectionPanel,
  commandCard,
  rtsConsole,
  dragSelect,
  rtsCommands,
  wavePreview,
  resourceBar,
});
