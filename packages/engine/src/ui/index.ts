// The component library (docs/ui.md §2): containers, navigation and data
// components, merged into og.ui by the package entry beside the UI kit.
// Every member is declared in crates/og-api/api/ui.ogt (module `ui`) as a
// `host false` function written here over public og.ui calls only.

import { accordion, card, splitView } from "./containers/panels.ts";
import { confirm, contextMenu, dialog, menu, popover, sheet } from "./containers/layers.ts";
import { actionHints, breadcrumbs, carousel, pagination, radialMenu, tabBar } from "./containers/navigation.ts";
import { dataGrid, dataList, table } from "./data/table.ts";
import { nodeGraph, tree } from "./data/tree.ts";
import { chart } from "./data/charts.ts";
import { keyValueRows, statCompare } from "./data/rows.ts";
import { calendar } from "./data/calendar.ts";
import { foundations } from "./foundations/index.ts";
import { controls } from "./controls/index.ts";
import { patterns } from "./patterns/index.ts";

export const components = Object.freeze({
  card,
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
  chart,
  keyValueRows,
  statCompare,
  calendar,
  ...foundations,
  ...controls,
  ...patterns,
});
