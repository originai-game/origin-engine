import { registerUiExtension } from "@origingame/engine/ui-extensions";
import { songSelect } from "./src/songs.ts";
import { laneKeys } from "./src/genres.ts";
import { comboCounter, judgement } from "./src/scores.ts";
export type * from "./types.ts";
export const rhythm = registerUiExtension("rhythm", { songSelect, laneKeys, comboCounter, judgement });
