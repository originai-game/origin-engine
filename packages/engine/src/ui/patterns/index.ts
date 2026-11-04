// The genre patterns (docs/ui.md §11, U6d), merged into og.ui with the rest
// of the component library (../index.ts). Each is declared in
// crates/og-api/api/ui.ogt (`host false`) and written here over public
// og.ui calls only.

import { cardHand } from "./cards.ts";
import { rewardChoice } from "./rewards.ts";
import { heroButton } from "./core.ts";
import { starBar } from "./stars.ts";
import { idleGenerator, prestigePanel } from "./idle.ts";
import { storeFront } from "./store.ts";
import { battlePass } from "./pass.ts";
import { backlog, barks } from "./story.ts";
import { characterSelect, draft, lobby, versus } from "./roster.ts";
import { worldMap } from "./maps.ts";
import { songSelect } from "./songs.ts";
import { ledger, overlayBar } from "./ledger.ts";
import { chat } from "./chat.ts";
import { photoMode } from "./photo.ts";
import { deckBuilder } from "./deck.ts";
import { cgGallery, routeChart } from "./gallery.ts";
import { settingsTree } from "./settings.ts";

export const patterns = Object.freeze({
  cardHand,
  heroButton,
  rewardChoice,
  starBar,
  idleGenerator,
  prestigePanel,
  storeFront,
  battlePass,
  backlog,
  barks,
  lobby,
  characterSelect,
  versus,
  draft,
  worldMap,
  songSelect,
  ledger,
  overlayBar,
  chat,
  photoMode,
  deckBuilder,
  cgGallery,
  routeChart,
  settingsTree,
});
