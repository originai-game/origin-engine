// The puzzle kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { puzzle } from "@origingame/engine/extensions/puzzle/index.ts";   // bundled
//   import { puzzle } from "../assets/ui-kits/puzzle/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { goalChips } from "./src/goals.ts";
import { sagaMap } from "./src/saga.ts";

export type * from "./types.ts";

export const puzzle = registerUiExtension("puzzle", {
  sagaMap,
  goalChips,
});
