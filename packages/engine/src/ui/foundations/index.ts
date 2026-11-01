// The foundations (docs/ui.md §11): text, rich text, icon, image and art
// slot, shape, divider, spacer, viewport, particle emitter and layer,
// merged into og.ui by the package entry. Each is declared in
// crates/og-api/api/ui.ogt (module `ui`, `host false`).

import { effectLayer, emitter } from "./effects.ts";
import { artSlot, iconMark, shapeBox, viewFrame } from "./media.ts";
import { prose, separator, space, text } from "./text.ts";

export const foundations = Object.freeze({
  text,
  prose,
  iconMark,
  artSlot,
  shapeBox,
  separator,
  space,
  viewFrame,
  emitter,
  effectLayer,
});
