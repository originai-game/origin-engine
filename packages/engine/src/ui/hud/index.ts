// The HUD components (docs/ui.md §11 "HUD", Appendix C), merged into og.ui
// by the package entry. Each is declared in crates/og-api/api.ogt (module
// `ui`, `host false`) and built only from og.ui's public members.

import { abilityBar, abilitySlot, statusGrid } from "./abilities.ts";
import { orbMeter, radialMeter, statBar } from "./meters.ts";
import { compass, minimapView, objectiveTracker } from "./nav.ts";
import { comboCounter, judgement, scorebug } from "./scores.ts";
import { hudTimer } from "./timer.ts";
import { flightHud } from "./vehicles.ts";
import { goalTimeline, matchStats, playerPlate, substitution } from "./sports.ts";
import { laneKeys } from "./genres.ts";

export const hud = Object.freeze({
  statBar,
  orbMeter,
  radialMeter,
  abilitySlot,
  abilityBar,
  statusGrid,
  minimapView,
  compass,
  objectiveTracker,
  hudTimer,
  comboCounter,
  judgement,
  scorebug,
  flightHud,
  matchStats,
  goalTimeline,
  playerPlate,
  substitution,
  laneKeys,
});
