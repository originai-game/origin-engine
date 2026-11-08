// The component library (docs/ui.md §2): containers, navigation and data
// components, merged into og.ui by the package entry beside the UI kit.
// Every member is declared in crates/og-api/api/ui.ogt (module `ui`) as a
// `host false` function written here over public og.ui calls only.

import { accordion } from "./containers/panels.ts";
import { contextMenu, menu, popover, sheet } from "./containers/layers.ts";
import { actionHints, tabBar } from "./containers/navigation.ts";
import { nodeGraph } from "./data/tree.ts";
import { foundations } from "./foundations/index.ts";
import { controls } from "./controls/index.ts";

export const components = Object.freeze({
  accordion,
  popover,
  menu,
  contextMenu,
  sheet,
  tabBar,
  actionHints,
  nodeGraph,
  ...foundations,
  ...controls,
});
