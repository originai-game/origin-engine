// The HUD components (docs/ui.md §2 "HUD"), merged into og.ui
// by the package entry. Each is declared in crates/og-api/api.ogt (module
// `ui`, `host false`) and built only from og.ui's public members.

import { objectiveTracker } from "./nav.ts";

export const hud = Object.freeze({
  objectiveTracker,
});
