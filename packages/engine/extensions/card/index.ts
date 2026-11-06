import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { cardHand } from "./src/cards.ts";
import { deckBuilder } from "./src/deck.ts";
import { rewardChoice } from "./src/rewards.ts";
export type * from "./types.ts";
export const card = registerUiExtension("card", { cardHand, rewardChoice, deckBuilder });
