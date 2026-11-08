// The genre patterns (docs/ui.md §2, U6d), merged into og.ui with the rest
// of the component library (../index.ts). Each is declared in
// crates/og-api/api/ui.ogt (`host false`) and written here over public
// og.ui calls only.

import { overlayBar } from "./ledger.ts";
import { settingsTree } from "./settings.ts";

export const patterns = Object.freeze({
  overlayBar,
  settingsTree,
});
