// The foundations (docs/ui.md §2): viewport, particle emitter and
// layer, merged into og.ui by the package entry. Each is declared in
// crates/og-api/api/ui.ogt (module `ui`, `host false`).

import { effectLayer, emitter } from "./effects.ts";
import { viewFrame } from "./media.ts";

export const foundations = Object.freeze({
  viewFrame,
  emitter,
  effectLayer,
});
