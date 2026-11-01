// The component library (docs/ui.md §11): containers, navigation and data
// components, merged into og.ui by the package entry beside the UI kit.
// Every member is declared in crates/og-api/api/ui.ogt (module `ui`) as a
// `host false` function written here over public og.ui calls only.

import { accordion, card, scrollArea, splitView } from "./containers/panels.ts";
import { confirm, contextMenu, dialog, menu, popover, sheet } from "./containers/layers.ts";
import { actionHints, breadcrumbs, carousel, pagination, radialMenu, tabBar } from "./containers/navigation.ts";
import { dataGrid, dataList, table } from "./data/table.ts";
import { nodeGraph, tree } from "./data/tree.ts";
import { keyValueRows, statCompare } from "./data/rows.ts";
import { calendar } from "./data/calendar.ts";

export const components = Object.freeze({
  card,
  scrollArea,
  accordion,
  splitView,
  dialog,
  confirm,
  popover,
  menu,
  contextMenu,
  sheet,
  tabBar,
  carousel,
  breadcrumbs,
  pagination,
  actionHints,
  radialMenu,
  dataList,
  dataGrid,
  table,
  tree,
  nodeGraph,
  keyValueRows,
  statCompare,
  calendar,
});
