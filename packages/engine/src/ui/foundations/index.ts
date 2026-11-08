// The foundations (docs/ui.md §2): the effect layer, merged into og.ui
// by the package entry. Each is declared in
// crates/og-api/api/ui.ogt (module `ui`, `host false`).

import { effectLayer } from "./effects.ts";

export const foundations = Object.freeze({
  effectLayer,
});
