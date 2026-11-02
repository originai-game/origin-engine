// The genre patterns (docs/ui.md §11, U6d), merged into og.ui with the rest
// of the component library (../index.ts). Each is declared in
// crates/og-api/api/ui.ogt (`host false`) and written here over public
// og.ui calls only.

import { cardHand } from "./cards.ts";
import { inventoryGrid, paperdoll } from "./inventory.ts";
import { rewardChoice } from "./rewards.ts";
import { gachaBanner, pullSequence } from "./gacha.ts";
import { heroButton } from "./core.ts";

export const patterns = Object.freeze({
  cardHand,
  heroButton,
  inventoryGrid,
  paperdoll,
  rewardChoice,
  gachaBanner,
  pullSequence,
});
