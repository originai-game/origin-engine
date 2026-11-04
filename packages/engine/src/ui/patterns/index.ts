// The genre patterns (docs/ui.md §11, U6d), merged into og.ui with the rest
// of the component library (../index.ts). Each is declared in
// crates/og-api/api/ui.ogt (`host false`) and written here over public
// og.ui calls only.

import { cardHand } from "./cards.ts";
import { inventoryGrid, paperdoll } from "./inventory.ts";
import { rewardChoice } from "./rewards.ts";
import { gachaBanner, pullSequence } from "./gacha.ts";
import { heroButton } from "./core.ts";
import { sagaMap } from "./saga.ts";
import { goalChips } from "./goals.ts";
import { starBar } from "./stars.ts";
import { idleGenerator, prestigePanel } from "./idle.ts";
import { storeFront } from "./store.ts";
import { battlePass } from "./pass.ts";
import { backlog, barks, conversation } from "./story.ts";
import { questLog, questTracker } from "./quests.ts";
import { characterSelect, draft, lobby, versus } from "./roster.ts";
import { worldMap } from "./maps.ts";
import { songSelect } from "./songs.ts";
import { garage } from "./garage.ts";
import { ledger, overlayBar } from "./ledger.ts";
import { chat } from "./chat.ts";
import { rtsCommands } from "./commands.ts";
import { photoMode } from "./photo.ts";
import { loadout } from "./loadout.ts";
import { deckBuilder } from "./deck.ts";
import { cgGallery, routeChart } from "./gallery.ts";
import { settingsTree } from "./settings.ts";

export const patterns = Object.freeze({
  cardHand,
  heroButton,
  inventoryGrid,
  paperdoll,
  rewardChoice,
  gachaBanner,
  pullSequence,
  sagaMap,
  starBar,
  goalChips,
  idleGenerator,
  prestigePanel,
  storeFront,
  battlePass,
  conversation,
  backlog,
  barks,
  questLog,
  questTracker,
  lobby,
  characterSelect,
  versus,
  draft,
  worldMap,
  songSelect,
  garage,
  ledger,
  overlayBar,
  chat,
  rtsCommands,
  photoMode,
  loadout,
  deckBuilder,
  cgGallery,
  routeChart,
  settingsTree,
});
