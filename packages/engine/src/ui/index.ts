// The component library (docs/ui.md §2): containers, navigation and data
// components, merged into og.ui by the package entry beside the UI kit.
// Every member is declared in crates/og-api/api/ui.ogt (module `ui`) as a
// `host false` function written here over public og.ui calls only.

import { contextMenu, menu, popover } from "./containers/layers.ts";

export const components = Object.freeze({
  popover,
  menu,
  contextMenu,
});
