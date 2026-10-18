// The UI kit: og.ui's game widget set (host calls and their callback sugar)
// and the screen templates, merged into og.ui by the package entry. Every
// member is declared in crates/og-api/api.ogt (module `ui`); the templates
// are `host false` functions written here over the calls.

import { widgets } from "./bind.ts";
import { platformerHud, shooterHud, puzzleHud, idleHud, racingHud } from "./huds.ts";
import { alertDialog, confirmDialog, loadingScreen, mainMenu, pauseMenu, resultsScreen } from "./menus.ts";
import { settingsScreen } from "./settings.ts";
import { inventoryScreen, leaderboardScreen, levelSelect, shopScreen } from "./store.ts";
import { cutscene, tutorialOverlay } from "./story.ts";

export { settingsWithDefaults } from "./settings.ts";
export { fmt, clock } from "./util.ts";

export const uiKit = Object.freeze({
  ...widgets,
  mainMenu,
  pauseMenu,
  settingsScreen,
  platformerHud,
  shooterHud,
  puzzleHud,
  idleHud,
  racingHud,
  cutscene,
  shopScreen,
  inventoryScreen,
  levelSelect,
  resultsScreen,
  leaderboardScreen,
  tutorialOverlay,
  loadingScreen,
  confirmDialog,
  alertDialog,
});
