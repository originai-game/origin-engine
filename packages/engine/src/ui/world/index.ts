// The world components (docs/ui.md §1, U6c), merged into og.ui by the
// package entry. Each is declared in crates/og-api/api.ogt (module `ui`,
// `host false`) and built only from og.ui's public members: world anchors
// for tags, mass elements for what comes by the thousand, and the ground
// projected with og.Camera for shapes that lie on it.

import { pathLine, placementGhost, selectionRing, telegraph } from "./ground.ts";
import { damageNumbers, overheadBars } from "./mass.ts";
import { interactionPrompt, lootLabel, nameplate, speechBubble, waypoint } from "./tags.ts";

export const worldUi = Object.freeze({
  nameplate,
  overheadBars,
  damageNumbers,
  interactionPrompt,
  waypoint,
  selectionRing,
  telegraph,
  pathLine,
  placementGhost,
  lootLabel,
  speechBubble,
});
