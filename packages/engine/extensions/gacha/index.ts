// The gacha kit (docs/ui.md §17): its components, registered as one typed
// namespace beside og.ui. A game imports it explicitly:
//
//   import { gacha } from "@origingame/engine/extensions/gacha/index.ts";   // bundled
//   import { gacha } from "../assets/ui-kits/gacha/index.ts";              // installed copy

import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { gachaBanner, pullSequence } from "./src/gacha.ts";

export type * from "./types.ts";

export const gacha = registerUiExtension("gacha", {
  gachaBanner,
  pullSequence,
});
