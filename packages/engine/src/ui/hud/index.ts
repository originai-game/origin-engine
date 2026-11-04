// The HUD components (docs/ui.md §11 "HUD", Appendix C), merged into og.ui
// by the package entry. Each is declared in crates/og-api/api.ogt (module
// `ui`, `host false`) and built only from og.ui's public members.

import { abilityBar, abilitySlot, statusGrid } from "./abilities.ts";
import { ammoCounter, crosshair, damageDirection, killfeed, lowHealthVignette } from "./combat.ts";
import { castBar, partyFrames, raidFrames, unitFrame } from "./frames.ts";
import { orbMeter, radialMeter, statBar } from "./meters.ts";
import { compass, minimapView, objectiveTracker } from "./nav.ts";
import { comboCounter, judgement, resourceBar, scoreboard, scorebug, turnOrder } from "./scores.ts";
import { hudTimer } from "./timer.ts";
import { flightHud, speedometer } from "./vehicles.ts";
import { lapTimer, raceBanner, rearView, standingsTower } from "./racing.ts";
import { driftScore, itemSlot } from "./arcade.ts";
import { commandCard, dragSelect, productionQueue, rtsConsole, selectionPanel } from "./rts.ts";
import { goalTimeline, matchStats, playerPlate, substitution } from "./sports.ts";
import { laneKeys, wavePreview } from "./genres.ts";

export const hud = Object.freeze({
  statBar,
  orbMeter,
  radialMeter,
  abilitySlot,
  abilityBar,
  statusGrid,
  unitFrame,
  partyFrames,
  raidFrames,
  castBar,
  crosshair,
  ammoCounter,
  minimapView,
  compass,
  objectiveTracker,
  killfeed,
  scoreboard,
  hudTimer,
  comboCounter,
  judgement,
  resourceBar,
  turnOrder,
  speedometer,
  scorebug,
  flightHud,
  standingsTower,
  selectionPanel,
  commandCard,
  productionQueue,
  rtsConsole,
  dragSelect,
  matchStats,
  goalTimeline,
  playerPlate,
  substitution,
  wavePreview,
  laneKeys,
  lapTimer,
  raceBanner,
  itemSlot,
  driftScore,
  rearView,
  damageDirection,
  lowHealthVignette,
});
