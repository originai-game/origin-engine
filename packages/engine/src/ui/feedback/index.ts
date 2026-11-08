// The feedback components (docs/ui.md §2 "Feedback"), merged into og.ui
// by the package entry. Each is declared in crates/og-api/api.ogt (module
// `ui`, `host false`) and built only from og.ui's public members.

import { celebration } from "./celebrations.ts";
import { formatNumber } from "./core.ts";
import { tooltipCard } from "./tooltips.ts";

export const feedback = Object.freeze({
  tooltipCard,
  formatNumber,
  celebration,
});
